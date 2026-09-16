# replacement-architecture.md — landed phases, evicted

**A record of finished work, not a plan.** Every section here sat under a ✅
heading in `plans/replacement-architecture.md` and was moved out on 2026-09-11 once the phase had shipped,
leaving the heading, a stub and a pointer in the live doc. Nothing here is
owed and nothing here should be acted on; what each section is *for* is the
reasoning — the design as sized, what the building changed, the measurement.
`check_state_of_play.py` reads the ✅ headings that stay, and fails when a
landed section keeps more than 40 lines in the live doc
(`engineering-practices.md` §4). Later phases are appended by the PR that
lands them.

#### RA-3 — payloads and structure (tickets 9, 6, 7, 8) — ✅ landed 2026-08-25

*Evicted 2026-09-11 from `plans/replacement-architecture.md`, where the heading and a stub remain.*


In that order: 9 first because 6's batch id has nowhere to live without it, and
7 after 6 because the bypass sites are where the LKI frame is captured.

9. `execute_actions` batch form; `apply_combat_damage` and the SBA sweep use it.
6. Payload upgrades: layer-computed LKI frame on battlefield-leaving zone
   changes (CR 603.10a), `cause`, batch id, resolution context.
7. Close the three `// REPLACEMENT-BYPASS:` sites with the pop-aware dispatch.
   (Shipped as written. The *pop* itself turns out to be unjustified — see the
   divergence note below — and its removal is RC's, not this ticket's.)
8. Demote `CreatureDied` / `PlaneswalkerDied` / `LegendRuleSacrificed` to
   display sugar. There is no matcher in RA, so the deliverable is a test proving
   the `ZoneChange` + LKI frame carries everything the three events carried, plus
   doc comments marking them display-only. `fuzz_games`' `creatures_died` stat
   and `ui/display.rs` keep reading them; nothing else may.

   **Executed as deletion, not demotion** (2026-08-26, review). "Display-only"
   is a policy in a doc comment, and the type system can enforce the same thing
   for free by removing the variant. `AuraDied` and `SpellResolved` went with
   them for the same reason, and `PermanentLeftBattlefield` had never had an
   emitter. The deciding argument was measured rather than aesthetic: the
   redundancy was hiding an undercount. `CreatureDied` was emitted only from the
   SBA sweep, so a creature killed by a spell produced none, and `creatures_died`
   read 5.3 where the zone changes say 6.2. That is the one fuzz number RA-3
   moves.

**What executing it changed in this document.** Four things, recorded here
because §9 is where the next phase reads:

- **The untap sweep is a third batch caller.** §4.2 named `apply_combat_damage`,
  the SBA sweep and "any 'each player …' effect"; CR 502.1 says the untap step's
  permanents untap *simultaneously*, which is the same rule and the ticket did
  not name it. Batched.
- **`execute_actions` returns `Result<(), String>`, not `Result<Vec<GameAction>, String>`.**
  §4.2 specified the performed-action vector; in RA nothing consumes it, and the
  project does not add a return value speculatively. RB adds it when
  `apply_replacements` gives it a customer — a one-line change.
- **`play_land` was a fourth chokepoint bypass** that §11's derivation missed,
  because that derivation counted `change_zone` callers and `play_land` wrote
  straight to `move_object`. It is the most frequent zone change in the game, and
  it is why `ZoneChangeCause::PlayedAsLand` had no call site. Fixed in ticket 6.
  Running correction: **11** production movers, not the 9 §11 derived or the 10
  RA-2 corrected it to.
- **The early stack pop has no surviving justification, and this document
  endorsed it.** Ticket 7 is written as a "pop-aware dispatch", which takes the
  pop as given. Audited in review (2026-08-26): `resolve_top_of_stack` removes
  the object from the `stack` `Vec` before resolving, documented as keeping an
  in-flight Counterspell from seeing it. Nothing can see it. **CR 608.2g** says
  no spell may normally be cast and no ability activated during a resolution, so
  nothing can *acquire* the resolving object as a target mid-resolution; and a
  spell cannot choose itself at CR 601.2c because `enumerate_legal_selections`
  and `has_any_legal_choice` already exclude it by `exclude_id`. Meanwhile
  **CR 608.2 keeps a resolving spell on the stack** until 608.2n or 608.3a moves
  it, so the pop is an engine artifact the rules do not have.

  RA-3 shipped the pop-aware dispatch as specified and documented the artifact
  rather than removing it mid-ticket. **The removal is sized in
  `codebase-state.md` (Deferred Migrations 7) and slotted for RC**, because RC
  turns `place_on_battlefield` into `EnterBattlefield`'s performer and therefore
  rewrites `init_zone_state` — the other reader of `GameState::resolving` —
  anyway. Doing both at once leaves `resolving` deleted or reduced to one field,
  and deletes a `remove_from_zone_collection` leniency branch that can currently
  mask a genuinely missing stack object.

- **The CR 601.2 rewind has two halves and RA-3 fixed one.** The rollback is now
  silent, which is what `// CAST-ROLLBACK:` had claimed since RA-1. The *forward*
  hand→stack move is still announced at CR 601.2a, before it is knowable whether
  the cast rewinds, so a replay still contains a move the rules say never
  happened. Deferring the announcement to 601.2i without deferring the move is a
  two-phase cast; recorded as Deferred Migrations item 5 and wanted before
  Phase 6, since the trigger matcher reads that log.

**One design point the ticket list did not settle, decided during execution.**
CR 704.3 says the game checks every condition and *then* performs all applicable
state-based actions "simultaneously as a single event". The old sweep performed
704.5f's moves before it evaluated 704.5g's conditions, so it converged to the
right board through `check_state_based_actions_loop` but could never produce the
simultaneity CR 704.7 and CR 616.1 are written against. The sweep now gathers
against one game state and performs one batch, deduped per object with the first
condition in CR order naming the cause. That is a real behavior change with a
visible consequence: **a player controlling two Isamarus, one dead to lethal
damage, is now asked which to keep** — the old sweep skipped the prompt by
having already removed the dead one. Both then die, which is what the rule says.
`fuzz_games --games 50 --seed 12345` did not move, so the case is rare in the
current pool, but it is a live `DecisionProvider` call and RB's §11 item 7
blast-radius watch applies to it.

**Exit criterion (all of RA) — met 2026-08-25:** every state mutation observable
by CR 614 or CR 603 is emitted from exactly one place, and an event log replay
can distinguish drawn from tutored, destroyed from sacrificed, and countered from
resolved. "Every" includes the life mutations: after RA the only `life_total`
writers are `perform_action`'s own arms, and the only emitter of
`GameEvent::ZoneChange` is its `ZoneChange` arm.

Two mutations are outside it by construction rather than by oversight, and both
are recorded in `codebase-state.md`: counter annihilation and attachment SBAs
have no `GameAction` variant to propose through (RB item 5 gives counters one),
and the CR 601.2a announcement above.

### Phase RB — the pipeline, with counters and regeneration as consumers — ✅ landed 2026-08-26

*Evicted 2026-09-11 from `plans/replacement-architecture.md`, where the heading and a stub remain.*


1. `ReplacementDef`, `EventPattern`, `Rewrite`, `ReplacementOutcome`,
   `ReplacementClass`, `Uses`; `Effect::Replacement` (and its `then: Option<Effect>`
   half, which reuses `resolve_effect`);
   `register_static_effects` skips replacement bodies.
2. `ReplacementEffectRegistry` with duration expiry.
3. `apply_replacements` — the §4.1 loop, including 616.1a–f, 614.5, 616.1g
   recursion, 614.17c's blocked-event path, and APNAP.
4. `GameAction::Destroy`; `Primitive::Destroy` lowers to it; indestructible
   moves to the "can't" check (CR 702.12b/614.17).
5. **Consumer 1 — counters.** `CounterType::{Shield, Stun, Finality}` and their
   CR 122.1c/d/h effects. No card text; 164 cards.
6. **Consumer 2 — regeneration.** CR 701.19a shield (`Uses::Once`), 701.19b
   static, 701.19c "can't be regenerated" blocking application not creation.
7. **Consumer 3 — Kalitas, Traitor of Ghet**, added deliberately so
   `EventPattern` is not defined under trivial pressure (§8c, "should the grammar
   work move earlier?"). It is the only RB card with a two-sided filter and a
   `then` half, and it forces the first `ObjectFilter` leaf decision
   (`nontoken`) at the moment the "two customers before a variant" guard is
   cheapest to apply. Plus the hand-read of ~50 predicate clauses described
   there.
8. **CR 704.6d — commander in graveyard/exile → command zone.** See §11 item 1:
   this is an SBA, *not* a replacement, and `check_state_based_actions` already
   takes a DP. It can ship here or earlier.
9. CR 903.9b — commander to hand/library, with `exempt_from_614_5: true`.

**What executing it changed in this document — ✅ landed 2026-08-26.** Nine
items, one PR, and the corrections are recorded here because §9 is where the
next phase reads.

- **`Uses::CounterBacked` is gone; `Uses` ships as `{ Static, Once }`.** §3.2
  now carries the reasoning. The short form: CR 122.1c/d state their effects
  verbatim and the counter removal is the substituted event or the rider, never
  a spent use — and a use would have written `PermanentState.counters` from
  inside `consume_use`, off the chokepoint.

- **CR 122.1c is two effects, and its replacement half is narrower than it
  looks.** "One or more shield counters ... create a single replacement effect
  **and** a single prevention effect", and the replacement half reads "would be
  destroyed **as the result of an effect**" — CR 701.8b way 1 only. A shield
  counter never answers CR 704.5g through that path; its prevention half stops
  the damage first. Read loosely, one counter saves a creature twice. This is
  what gives `GameAction::Destroy`'s `source` field a customer on day one, as
  `DestructionSource { Effect(id), StateBasedAction }`.

- **`EventPattern` ships six arms, not one per `GameAction` variant.**
  `DrawCard`/`GainLife`/`LoseLife` affect a *player* and `AffectedSet` names
  only objects, so an arm for one would have no scoping mechanism and would be
  a card that silently does nothing. §9 schedules draw and life replacement for
  RE anyway; they land there with the player-scoping mechanism CR 614.1's
  "whatever they're affecting" needs for a player. The growth contract
  constrains the *axis*, and the axis is intact.

- **`Rewrite` ships `Prevent` and `Instead`.** `Amount`, `Retarget` and
  `EnterWith` have no RB customer and no application path, and an arm the
  pipeline cannot apply is the same silent card. The five-arm algebra and the
  574-clause census are recorded in the enum's own docs with the phase that
  gives each arm a customer, so the completeness claim survives as documentation
  rather than as three `todo!()`s. §3.2b is unchanged and still the spec.

- **`execute_actions` is three phases, and the split is CR 704.3.** Decide for
  every member against one board, then perform, then run riders. That is what
  "checks for any of the listed conditions ... then performs all applicable
  state-based actions simultaneously as a single event" asks for, and it is
  where §4.3's CR 101.4 APNAP ordering lives: choices in APNAP order of chooser,
  performance in batch order, riders last (CR 615.5). **CR 101.4d's restart is
  unreachable in this shape rather than unimplemented** — no event is performed
  during the decision phase, so the only state a choice can change is
  `consume_use`, which strictly *removes* candidates and can never widen an
  earlier player's options.

- **§4.1's loop needed a second set, and it hangs without one.** CR 903.9b is
  the only `exempt_from_614_5` effect *and* it is optional, so the decline path's
  "mark applied and continue" is ignored by a filter the exemption bypasses.
  Recorded in §4.1.

- **`gather` needs a fast path, and it is not an optimization.** Reading
  effective abilities is a full `compute_characteristics` walk, and an ungated
  sweep runs one per permanent per proposed action — measured against the untap
  step alone that is thousands of extra layer walks per fuzz game on boards
  where nothing has a replacement ability. The gate is exact rather than
  heuristic: an object can only *have* a static replacement ability if it
  printed one (`GameState::replacement_ability_sources`, recorded at ETB, a set
  so it cannot drift) or a Layer 6 row granted it one
  (`RegistryScopeSummary::any_granted_replacement`, narrowed to grants of
  replacement bodies so the flag is not permanently on). Counters are scanned
  rather than cached, so no test fixture can place one and be silently ignored.
  Measured: 13.01 → 13.04 ms/game at `--games 200 --seed 12345`, medians of
  three interleaved runs in one worktree.

- **`AffectedSet` and `ZoneChangeCause` moved into `types/`.** `src/types/` had
  zero `crate::engine` references and `ReplacementDef` needs both. Each moved
  with a `pub use` left behind, so no call site changed.

- **Items 6 and 7 needed engine vocabulary §9 did not budget**, and every piece
  of it is named by a rule: `Primitive::{Tap, RemoveFromCombat, RemoveAllDamage}`
  for CR 701.19a's rider, `Primitive::{AddCounters, RemoveCounters}` plus their
  `GameAction`s for CR 122.1, `Primitive::CreateToken` for Kalitas's rider, and
  `ObjectFilter::Token` for its nontoken clause. `CreateTokens` as a
  *replaceable* event is still RE's — until a CR 614.16 doubler exists there is
  nothing to replace.

- **CR 704.6d needed a fact nobody was recording.** "Put into that zone since
  the last time state-based actions were checked" is unanswerable a moment
  later, so `GameObject.zone_change_epoch` is stamped by `move_object`. The
  window is read at the **top** of `check_state_based_actions`, not the bottom:
  a commander that CR 704.5g puts into a graveyard moves *during* a check, and
  an end-of-check boundary would place the move before the boundary it is
  supposed to be after. This is the field `codebase-state.md` item 10 wants for
  CR 400.7; it does not implement 400.7, which also needs the 400.7a–c
  exceptions.

- **CR 704.7's dedupe stayed in the SBA sweep**, where RA-3 put it, rather than
  moving into `execute_actions` as §4.2 specifies. The sweep is what knows CR
  order for naming the cause, and a generic same-result dedupe would have to
  re-derive it. §4.2's sentence should be read as "upstream of
  `apply_replacements`", which it is.

- **§11 item 7's blast-radius watch held.** Every existing test now traverses
  the pipeline and **not one new `DecisionProvider` prompt appeared** on the
  current pool. §4.1's two-candidate rule was never relaxed. `fuzz_games --games
  50 --seed 12345` is identical to the pre-RB baseline on every line.

**Not done in RB, and named rather than discovered later:** CR 614.15's
self-replacement effects have a `ReplacementClass::SelfReplacement` bucket and
no producer, so `ResolutionContext` still has three fields (§11 item 3 — land
the fourth with the first card that needs it); CR 614.17c's blocked-event path
therefore always drops the event, which is right today and will need revisiting
when a self-replacement exists. §3.3's source 2 (static abilities functioning in
other zones) is untouched and still deferred past RE — §11 item 4 asked RB to
*size* it, and RB did not; the sweep it would change is now written, so the cost
is a zone parameter on one loop in `engine/replacement/gather.rs` plus a
timestamp on `GameObject` (Deferred Migrations item 9's, already owed).

#### RC-1 — delete the early stack pop — ✅ landed 2026-09-01

*Evicted 2026-09-11 from `plans/replacement-architecture.md`, where the heading and a stub remain.*


`codebase-state.md` Deferred Migrations item 7, on its own and first. `RC` was
going to carry it "along" with the performer migration; measured, it is its own
PR and it is the RA-1 of this phase — the safest shape the project writes.

Stop popping at the top of `resolve_top_of_stack`; keep taking the `StackEntry`
(the body needs to own it); let `move_object`'s `remove_from_zone_collection(Stack)`
do the removal it is already asked to do; remove the object from `stack`
explicitly on the ability path, which has no zone change.

**Why first rather than bundled.** It is a *deletion* that makes the tree
simpler before the complicated thing lands, it removes a leniency branch that
can currently mask a genuinely missing stack object, and it leaves
`GameState::resolving` with one reader instead of two — so RC-2 rewrites
`init_zone_state` against a smaller thing. The counter-argument, that
`init_zone_state` then churns twice, is real and is the same trade RA-1 made in
the other direction; here the churn is small because the pop touches
`resolve_top_of_stack` and `remove_from_zone_collection`, not
`init_zone_state`'s body.

**Audit first, and this is the ticket's actual content:** five production sites
read `stack.is_empty()` (`zones.rs`, `legality.rs`, `mana_helpers.rs`,
`cast.rs`, `priority.rs`) and would newly see the resolving object. None is
reachable during a resolution today, but CR 608.2g's "unless an effect
instructs" case makes `cast.rs`'s reachable once RC-era cards arrive.

**✅ Shipped 2026-09-01. The audit above undercounted, and the miss is the
finding.** There are **six** `stack.is_empty()` readers, not five: `zones.rs:169`
had drifted to `:177`, and the unlisted sixth is `ui/display.rs:287`,
`format_stack`. It turned out to have **no production caller** — `pub`, with
every use a test in its own file — so nothing rendered differently; had it been
called, the CR would still have been on the deletion's side, since CR 608.2 puts
the resolving object on the stack. `stack.rs:27`'s guard is a seventh occurrence
and is correctly outside both counts: it runs before the resolution. The
`GameState::resolving` count of six was exact, and the field is down to one
reader — CR 110.2b's default controller in `init_zone_state`. **The lesson is the
one §9 keeps re-learning:** a count written into a plan is a measurement with a
date on it, and RC-2's `place_on_battlefield` / `init_zone_state` figures should
be re-run before they are built on, not read off this table.

`resolve_popped` became `resolve_taken` — there is no pop left to name it after —
and `cast.rs`'s site carries a comment naming the CR 608.2g choice it will have
to make, unfixed here.

**Exit met.** Whole suite green, zero warnings, and `fuzz_games --games 200
--seed 12345` byte-identical to a same-day `main` binary on **both** pools
outside `=== Timing ===`, three runs each. A `--dump-events` diff at 40 games was
added on top of the summary — identical after canonicalizing the per-process v4
`ObjectId`s, same event kinds at the same counts. **`--dump-events` also caught
something the summary structurally cannot**, and it is not RC-1's: CR 704.5d's
token sweep emits in `HashMap` order, so two `TokenCeasedToExist` lines swap
between runs of the *same* binary on the `stress` pool. Recorded against
`codebase-state.md` Deferred Migrations item 6, whose routing fixes it.

**Original exit criterion, for the record:** whole suite green, zero warnings,
`fuzz_games` identical on every line. No new events, no new behavior. If the fuzz
numbers move, the deletion changed something it should not have.

#### RC-2 — `EnterBattlefield` as an event — ✅ landed 2026-09-01

*Evicted 2026-09-11 from `plans/replacement-architecture.md`, where the heading and a stub remain.*


`GameAction::EnterBattlefield { object, controller, mods }`; `place_on_battlefield`
becomes its performer; `EventPattern::EnterBattlefield`, `Rewrite::EnterWith`,
`EnterMods`.

**Its consumer is enters-tapped (CR 110.5b), `AffectedSet::SourceOnly` only** —
"this land enters tapped", which needs no frame at all. 773 cards say it, and it
is the RB-item-4 of this phase: the smallest thing that proves the event works.
Enters-with-counters (CR 122.6a, 580 cards) rides here too if `EnterMods` is
already carrying counters, and `init_etb_counters` is the site it replaces.

**Not in RC-2:** CR 614.12a's choice-before-entry. §9's original bullet grouped
it with enters-tapped; it is a *frame* question the moment the choice depends on
what the permanent would be, so it moves to RC-4.

**Watch:** the test-side churn is where the surprise is. `put_on_battlefield`
has **284** call sites and `place_bare` **73**. Neither should need to change —
both are `test_support` helpers over `place_on_battlefield` — but that is the
claim to verify in the first commit, not the last.

**✅ Shipped 2026-09-01.** Eight findings, in the order they cost something.

**1. The site count was a grep count, and the ratio it hid is the lesson.**
There are **two** production *calls* to `place_on_battlefield` — `zones.rs`'s
inside `init_zone_state`, and `resolve.rs`'s token path — plus the definition.
The row's "10" was doc-comment mentions, and the `game_state.rs` figure was six
of them, not five. What the audit should have counted is what actually drove
the diff: **92 direct callers in `src/`, of which 88 are inside `#[cfg(test)]`**.
A performer's signature is owned by its tests, not by its production callers,
and this is the second RC row in a row whose count was a measurement with a date
on it (RC-1's `stack.is_empty()` was the first).

**2. `init_zone_state` is gone, not rewritten.** Its whole body was the
battlefield branch, and the branch was the `PermanentState` creation — which
now belongs to the `EnterBattlefield` performer. CR 110.2b's default controller,
the question RC-1 deliberately left as `GameState::resolving`'s only reader,
moved out as `GameState::default_enter_controller` and is read at the *proposal*
instead. The field still has exactly one reader.

**3. The proposal is made after `move_object`, not inside it.** `init_zone_state`
runs before `move_object` writes `obj.zone`, so proposing from there would have
announced the entry *before* the `ZoneChange` — a reordering, and the criterion
for this phase was "the new events and nothing reordered". The proposal is
instead the statement after `perform_action`'s `ZoneChange` emit, which leaves a
window one `emit` wide in which the object is in the battlefield *zone* with no
`PermanentState`. Both facts are commented at the site; neither is
comfortable, and the alternative was worse.

**4. The only intended addition to the event stream is one `ETB` per land drop,
and it closes a hole rather than opening one.** `stack.rs` announced a resolving
permanent spell, `resolve.rs` announced a token, and `play_land` announced
nothing at all — so `GameEvent::PermanentEnteredBattlefield` was missing for the
most frequent entry in the game. The performer is the single emitter now.
Measured at 40 games / seed 12345 / `--threads 1`, canonicalizing the
per-process v4 `ObjectId`s to first-seen order and diffing against a same-day
`main` binary with the two new cards unregistered so the pools match: **708
added lines on `performance`, 699 on `stress`, every one an `ETB` for a land,
zero deletions and zero reorderings.** The single `TokenCeasedToExist` swap on
`stress` is `codebase-state.md` Deferred Migrations item 6 and was reproduced
against `main` alone.

**5. `EnterWith` had to merge, exactly as §3.2 predicted, and `EnterMods::merge`
is where CR 616.1f's accumulation lives.** Status is `|=` (CR 110.5b gives a
permanent one tapped value) and counters are `+` per kind (CR 122.6a is about
counters being put on it). CR 614.5's applied set is the whole termination
argument: an `EnterWith` produces an event its own pattern still watches.

**6. `gather` needed a source and `chooser_for` needed a sibling.** The
battlefield sweep cannot see an entering permanent, and
`replacement_ability_sources` is written by `register_static_effects` *inside*
the performer — so without an explicit entering-object source, gated ahead of
the fast path for `commander_zone_replacement`'s reason, every "this permanent
enters tapped" is dead text. That is the gate leg `CLAUDE.md` requires of a new
gather source. Separately, `chooser_for` answers CR 616.1 from the board, and an
entering permanent has no controller — so it fell through to the *owner*, which
is the wrong player the moment someone casts a permanent spell they do not own.
`chooser_for_event` reads the answer off the proposal.

**7. ❌ WRONG — struck 2026-09-02 by RC-3, and the retraction is worth more
than the finding was.** As written, this finding said: "CR 616.1's
multi-candidate branch is *not* reachable on an entry, and no printed card can
make it so at `AffectedSet::SourceOnly` … an `AffectedSet::Filter` effect
cannot match an object that is not on the battlefield. So the branch is RC-3's
unlock, not RC-2's."

**The second sentence is false and was never true, so the conclusion is too.**
`AffectedSet` is matched by *two* functions on two paths, and RC-2 checked one
of them:

| Path | Matcher | Governs | Battlefield gate |
|---|---|---|---|
| layer registry | `compute.rs::effect_applies_to` (`:629`) | `ContinuousEffect.affected` | **yes** — RC-3's line |
| replacement pipeline | `gather::set_affects` → `GameState::object_matches_filter` | `ReplacementDef.affected`, `RestrictionDef.affected` | **none** |

Probed on `main` at 4f9eb94: a Root-Maze-shaped `ReplacementDef`
(`Filter { ByType(Land) }`, `EnterWith(tapped)`) taps an entering Forest, and
with Idyllic Beachfront entering under it `ask_choose_replacement` fires — two
candidates, CR 616.1's question asked. **The branch has been reachable since RB
merged, one registered card away.** Root Maze, Kismet, Loxodon Gatekeeper and
Frozen Aether were never blocked on RC-3, and `root_maze` ships in RC-3 to
close it rather than waiting for a card PR (§3.3, and the Kalitas precedent
below).

**What survives.** The *first* route is still genuinely shut: two
entry-modifying abilities on one card is Slumbering Trudge, Chocobo Camp, Steel
Dromedary, Rotating Fireplace and Arixmethes, every one needing {X}, a
condition or a trigger. RC-2's two cards — Idyllic Beachfront (CR 110.5b,
status) and Chainbreaker (CR 122.6a, counters), one per half of `EnterMods` —
are the right two for CR 614.1c's own axis and two genuinely different
performer paths. Both join `PERFORMANCE_POOL` (57 → 59); Adaptive Shimmerer is
registered into the stress pool alone for the ordering claim (see the review
pass below). The accumulation behaviour is covered by a two-ability fixture,
labelled as one.

**The transferable error**, since this is the second unreachable-branch gap in
three phases: the finding named a mechanism by its **type** (`AffectedSet::Filter`)
when the property it asserted belongs to a **call path**. One type, two matchers,
and a reachability claim is only ever true of a path. A claim of the form "no X
can reach Y" now owes the list of functions that match X — which is a thing
`grep` answers in one line, and nobody ran it.

**8. Blood Moon does not strip an entering tapland's ability, and RC-3 is
exactly one line away from fixing it.** The real ruling is that a tapland under
Blood Moon enters untapped; `gather` reading the *effective* ability list was
supposed to deliver that for free. It does not, because Blood Moon's row is a
`ContinuousEffect` whose `AffectedSet` is a `Filter`, and
`effect_applies_to`'s battlefield gate returns `false` for a filter effect
against an object that is not on the battlefield. **No filter-scoped row in the
layer registry reaches an entry** — which is the same statement as RC-3's "an
entering Clone matches no filter, Dress Down included", now with a registered
card behind it and a test asserting the wrong answer so RC-3 has to flip it.
(The original wording here was "no `Filter` effect reaches an entry at all",
which overreached into the replacement pipeline's ungated matcher — see the
strike on finding 7. The *layer* half was right, and it is what RC-3 fixes.)

**Exit met.** Whole suite green (810 tests, 17 of them new), zero warnings,
`check_module_layout.py` and `check_claude_md.py` both pass. `specdb owed` is
unchanged over the shipped phases, and RC-2 claims five atoms in full —
ATOM-110.5b-001/-002, ATOM-122.6a-001, and ATOM-209.1-001 / ATOM-306.5b-001,
the last two being Phase 5 Pre-Work atoms that had had no test at all until the
loyalty rewrite gave them one. Two partials: BOUNDARY-DEF-614.1c-001 (the
out-of-set member is a triggered ability, which item 6 owes) and ATOM-614.12-002
(its own scenario is a token copy of Voice of All, which needs CV and RC-4).

**Determinism holds and the pipeline is free.** `fuzz_games --games 200 --seed
12345 --threads 1`, three runs per pool on the shipped binary, byte-identical
outside `=== Timing ===` on **both** pools. Interleaved A/B in one sitting
against a same-day `main`, on identical card pools so the delta is the engine
alone: **104.5 → 103.8 ms/game on `performance`, 108.2 → 106.9 on `stress`**,
medians of three alternating runs each — flat, and both deltas are smaller than
the spread within either arm. That is the expected shape: the new gather source
costs one `compute_characteristics` walk per *entry*, and entries are rare
against untap steps and SBA sweeps. **RC-3's line is the one on the hot path,
and this measurement is the control it will be read against.**

**Nine changes from the review pass (PR #81), and the last one is the phase's
most consequential number.**

- **The A/B measured the wrong build, and the reviewer's question about clone
  pressure is what surfaced it.** `gather`'s fast path answered only "is
  *anything* on this board a static replacement source" — and RC-2 is the first
  phase to put replacement sources in `PERFORMANCE_POOL`, so from the first
  tapland onward that gate is true for the rest of the game and the sweep walked
  **every permanent** with a full `compute_characteristics` per proposed action.
  The exit-criterion A/B above ran against a build with the two cards
  unregistered, so it measured the gate *closed* and reported flat. **A
  per-permanent gate — the same predicate one object at a time — is worth
  10.3% of total game time on `performance` and 9.2% on `stress`**, medians of
  three and five interleaved rounds at 200 games, with every `performance` run
  separated. Event streams are byte-identical on both pools at 40 games, which
  is what "exact, not a heuristic" has to mean.
- **The dominant clone was not the one named.** `def: (**def).clone()` is real
  but small beside `get_effective_abilities`, which is a whole
  `EffectiveCharacteristics` construction — two `Vec`s and three `HashSet`s —
  per call. That is why the answer to "should we sweep the codebase for clones"
  is no: **the lever was a gate, not a clone**, and the sweep would have found
  the small one and missed the large one. The clone that matters for the AI use
  case is `GameState::clone` for search, which is `codebase-state.md` item 42's
  territory and wants a search harness to profile against before anyone touches
  it.
- **CR 613.7m is unimplemented and was found by asking whether attachment
  breaks the entering-permanent ordering.** It does not — CR 613.7e re-timestamps
  the *Aura or Equipment*, never its host, so an entering Aura only ends up newer
  still. But 613.7m says objects entering *simultaneously* are ordered by APNAP
  rather than by allocation, and `allocate_timestamp` can only produce allocation
  order. Exact today because every entry is its own singleton batch; reachable
  the moment `CreateTokens` (RE) or CR 614.13's auxiliary zone changes (RC-4)
  land. Recorded under `codebase-state.md` item 4, including that the fix is a
  decision point rather than a sort — 613.7m orders the active player's objects
  "in the order of that player's choice".

**Six more from the same pass, and two of those are findings too.**

- **The two-card rule wanted a third card, and §3.3 says why.** The ordering
  claim — CR 122.6a's counters go on before anything can observe the permanent —
  is falsifiable only by a **0/0**, and it was being made against a hand-built
  fixture. That is §3.3's own sharpest finding ("a bespoke fixture can cover an
  atom while the registered pool cannot build the same scenario") landing on the
  phase that quoted it. **Adaptive Shimmerer** ({5}, 0/0, Flash, three +1/+1
  counters, colorless) is registered — stress pool only, since RC-2's engine path
  is already measured by the two cards in `PERFORMANCE_POOL`. The rejected
  reasoning is recorded because it was wrong on the facts: the note claimed a
  0/0 with +1/+1 counters is `{G}{W}`-shaped and rare, and Adaptive Shimmerer and
  Ivy Elemental are both castable in any deck.
- **`chooser_for_event` was a one-arm wrapper and is gone.** `chooser_for` takes
  the whole proposal now. The wrapper existed because `chooser_for` took an
  `EventSubject`, and the real content of the finding is that *an entry's chooser
  is not a property of the board* — so the subject-shaped signature was the
  thing that could not answer, not a caller that needed special-casing.
- **The entering permanent's candidates are spliced in after the sweep, not
  before it.** They are gathered before the fast-path gate for cost and appended
  after the sweep for order: `next_timestamp` is monotonic and nothing gives
  another object a new timestamp when something enters, so the entering permanent
  is the newest object on the board and CR 613.7's oldest-first puts it last.
- **`apply_rewrite` takes the event by value**, which removes the `EnterMods`
  clone `Rewrite::EnterWith` was paying to own something the loop was about to
  discard. Clone pressure is a real axis here — a tree search clones
  `GameState`, and this loop runs inside every proposal.
- **`EnterMods::merge` uses plain addition**, matching
  `PermanentState::add_counters`, which is where the number ends up. The
  `saturating_add` it replaced picked a clamp width the type has no business
  choosing and would have been the only place in the engine with a different
  overflow story.
- **`EnterMods` now documents what a third status would cost**, because "face
  down" looks like another `bool` and is not one. The plumbing really is one
  field — [`Rewrite`] and [`EventPattern`] do not grow, which is the growth
  contract working — but CR 707.2 makes a face-down permanent a 2/2 colorless
  creature with no name or abilities, which is a **Layer 1a copiable-values**
  change and wants Phase CV underneath it. The printed population agrees: nothing
  prints "permanents enter face down" as an effect over another player's
  permanents (Scryfall, 2026-09-01), and morph/manifest are instructions the
  *mover* carries — CR 701.34a's "put it onto the battlefield face down as a 2/2
  creature card" would seed `default_enter_mods` exactly the way CR 306.5b's
  loyalty does, not register a `ReplacementDef`.

**Not changed, and the ticket asked:** `put_on_battlefield_this_turn` (10 call
sites, absent from the row above) routes through the performer like
`put_on_battlefield` and `place_bare`, and like them needed no re-pointing. All
three did need **one line each**, for a reason the "neither should need to
change" claim did not anticipate: `init_etb_counters` moved off the performer,
so the two helpers documented as firing ETB counters now pass
`default_enter_mods` explicitly, and `place_bare`'s promise not to fire them is
what makes it the right fixture for a test that counts events.

#### RC-3 — the membership gate and the frame's ability list — ✅ landed 2026-09-02

*Evicted 2026-09-11 from `plans/replacement-architecture.md`, where the heading and a stub remain.*


§5c's finding, and the reason this is a PR rather than a paragraph inside RC-2.
`effect_applies_to`'s `game.battlefield.contains_key(&id)` gate at
`compute.rs:629` returned `false` for any filter-scoped `ContinuousEffect`
against an object not on the battlefield — so an entering Clone matched no
filter, **Dress Down included**, and kept the ability Dress Down should have
taken away.

**One line of code, and it is in the hottest path in the engine.** That is the
whole reason it is separated: `compute_characteristics` is what every layer
query runs, `layers-architecture.md` §12 measured the ungated CR 604.2
existence check at 5.2×–8.0×, and a gate that starts admitting non-battlefield
objects changes what that walk does on every board. RC-3's deliverable is as much
the measurement as the behavior.

**Five findings.**

**1. The predicate is the battlefield *zone*, and that is what makes it free.**
`move_object` writes `obj.zone` before the `EnterBattlefield` performer builds
the `PermanentState` — RC-2's one-`emit`-wide window, documented in the
`ZoneChange` arm — so an entering permanent is already *in* the zone. Swapping
`game.battlefield.contains_key` for `obj.zone == Battlefield` admits exactly the
entering object and nothing else: hidden zones keep their own zone tag, so no
library or graveyard card becomes filter-matchable. The feared 5.2× never
arrives because the newly-admitted set is one object wide. **Measured over seven
interleaved runs: `Frames/walk` 1.24 → 1.24 on `performance` and 1.16 → 1.17 on
`stress`; ms per 1,000 layer walks 0.807 → 0.808 and 0.679 → 0.694.** §12's
warning was about admitting a *population*, and the fix admits a singleton.

**2. CR 614.12 is two membership rules, not one, and the second was missing.**
The clause everyone quotes is (3) — effects that already exist and would apply.
The clause in the *first sentence's* parenthesis is the mirror: an effect "may
come from the permanent itself if [it affects] only that permanent (as opposed
to a general subset of permanents that includes it)". `gather`'s source 1a
pushed every static replacement ability the entering permanent had, and
`set_affects` matches a `Filter` against any object in any zone — so an
entering Orb of Dreams found its own "Permanents enter tapped" and tapped
itself. Source 1a now takes a `SelfScope` and admits `AffectedSet::SourceOnly`
alone. **It belongs in RC-3 and not RC-4** because it is a question about
membership in the applicable set, which needs no frame to answer.

**3. Its consumers needed no new cards, and that had to be checked rather than
assumed.** §9's plan said "Dress Down + a Clone-shaped probe", which is stale
twice: Dress Down needs an ETB trigger and a delayed one (item 6), and Clone
needs Phase CV. What was already registered answers the same question — Blood
Moon + Idyllic Beachfront (the tapland enters untapped, CR 305.7) and Humility +
Chainbreaker (the Scarecrow enters with no -1/-1 counters and lives). Both pairs
were in `PERFORMANCE_POOL` before this phase, which is why the phase widens an
engine path rather than opening one — §3.3's axis, argued rather than waived.

**4. The card RC-3 does ship is for RB's and RC-2's gap, not its own.** Root
Maze (`{G}`, "Artifacts and lands enter tapped") makes CR 616.1's
multi-candidate branch reachable in a fuzz game, which finding 7's retraction
above shows was never blocked on this phase. Chosen over Kismet, Loxodon
Gatekeeper and Frozen Aether because those scope to "your opponents", which
reads `chars.controller` — finding 5. `ObjectFilter::Or` is new, for
"Artifacts and lands"; its `targeting.rs` arm short-circuits where `And` does
not, because a leaf can answer `Err` and `set_affects` collapses `Err` to
`false`.

**5. `base_controller` answered *owner* for a resolving object, and RC-3 made
that askable.** `resolve_top_of_stack` takes the `StackEntry` before it resolves
anything, so both of the first two probes miss for the whole resolution. Right
for a land drop, wrong for a spell cast by a non-owner (CR 110.2b), and
previously unreachable for an entering permanent because the gate stopped every
filter. `GameState::resolving` carries the default across exactly that window
and the leg goes above the fallback. **It fixes a wrong answer the current pool
cannot produce** — `check_cast_legality` refuses "another player's spell" — so
the test builds the disagreement after an ordinary cast, and the trap it removes
is for whoever relaxes that check.

**Exit met.** Whole suite green (823 tests, 7 of them new), zero warnings, both `check_*.py`.
`specdb owed` unchanged and clean for RC; ATOM-614.12-003 claimed in full,
ATOM-614.12-001 partially and deliberately — its board is Yixlid Jailer, and
`ObjectFilter` has no zone leaf, so the scenario is inexpressible rather than
unimplemented. Determinism: three 200-game runs per pool byte-identical outside
`=== Timing ===`, and `--threads 1` vs `--threads 8` identical on the
engine-work block. Event streams at 40 games: **36/40 `performance` and 34/40
`stress` games byte-identical**, and every one of the 10 divergences has a
Humility or Blood Moon on the battlefield ahead of it and a permanent entering
modified under it. A behaviour change cascades where RC-1's deletion did not, so
the claim that can be checked is the *first* divergence per game — not the
whole-stream diff, which is noise past that point.

#### RC-4 — the overlay — ✅ landed 2026-09-02

*Evicted 2026-09-11 from `plans/replacement-architecture.md`, where the heading and a stub remain.*


`compute_as_entering`, the read-side accessor pair, CR 614.12 clauses (1)–(3),
CR 614.17d in both its printed shapes, CR 616.1b's producer, the first
`AmountExpr::CountOf` in the layer walk, and §11 item 19. **Not shipped, and
sized out in the doc before code**: CR 614.13/13a/13b and the batch-scoped
frame, which are RC-5 below; CR 616.1c, which is CV-2's.

**The shape.** `engine/layers/lookahead.rs` — a `Lookahead` built from the
proposal (object, proposed controller, pending `EnterMods`), threaded through
`FrameCache`, and read by the two accessors `compute.rs` now makes its
concrete-state reads through: `entity` (controller, CR 302.6's clock, counters)
and `rows_in_layer` (the registry slice, then the entering object's would-be
rows). Both answer for the would-be permanent when the object being computed
is the entering one and off the real board otherwise, which is how §5b's
asymmetry falls out of the structure rather than being enforced: the entering
permanent's anthem is in its own frame and reaches no other object's, and a
count over `battlefield_ids_ordered` does not see it. `EntryFrame`
(`engine/replacement/lookahead.rs`) computes the frame at most once per
pipeline iteration and only when a filter-scoped `affected` asks; both
`set_affects` and `is_prohibited` read it, for an `EnterBattlefield` — which
since RC-4b is also the zone change onto the battlefield, so the frame has one
basis and no pending one.

**Six findings.**

1. **The frame lives on the stack, and the decision-site invariant says it
   may.** `codebase-state.md` item 40's test is "drop it and re-derive": the
   frame is a pure function of `GameState` and the proposal being decided, so
   it is bookkeeping. The proposal (`apply_replacements`' `event`) is the
   outcome-bearing thing, and it was already item 40's first violator; RC-4
   added one decision site to that entry — the N-player "opponent of your
   choice" — and no new state.

2. **The re-count was still wrong, and the count was the wrong instrument.**
   Five sites became eight became nine; what a hypothetical perturbs is four
   *kinds* of read, and only those four moved (§5's second note).

3. **The frame falsified item 19's theorem, and the suppression shipped
   narrower.** Once `set_affects` reads the pending `EnterMods`, a `PowerLE`
   filter can match before a counter lands and not after, so the CR 616.1
   choice between "creatures with power 1 or less enter tapped" and Adaptive
   Shimmerer's three counters decides whether it enters tapped. The rule that
   shipped admits only members whose applicability no `EnterMods` field can
   move, and carries a debug-build check and three expiry conditions
   (`codebase-state.md` item 47).

4. **CR 614.17d is two events, not one.** "Can't enter the battlefield" watches
   the `ZoneChange`, because a refused entry would strand the card in the
   zone; "can't have counters put on it" is asked as the `AddCounters` it is
   (CR 122.6) when an `EnterWith` would add them, and refuses the counters
   while the entry goes on. `pattern.object` reads the source zone (Grafdigger's
   Cage) and `affected` reads the frame (Worms of the Earth) — one rule, and
   `cant-effects-architecture.md` §5.3 carries it.

5. **The consumers were not the ones the plan named.** Grist and Thassa are
   `Effect::Conditional` statics (item 7f, closed 2026-09-06 — but each still
   wants a `Condition` leaf nothing has needed yet) and Master Biomancer needs a dynamic
   counter amount (RC-5). What was buildable: an anthem creature for clause
   (2), Keldon Warlord for the count boundary, Containment Priest as the first
   replacement whose *filter* reads the frame — a Sol Ring returned under March
   of the Machines is a creature to it — and Dryad Arbor as the only road a
   fuzz game has to a creature that "wasn't cast". Dryad Arbor also reached a
   CR 205.1a bug in `apply_set_subtypes` (Blood Moon made it a Mountain with no
   creature type), fixed in its own commit.

6. **Two engine-cost changes rode along, both attributable.**
   `targeting::object_matches_filter` now takes one layer walk per filter
   instead of one per leaf, and only when a leaf reads a characteristic — a
   walk-count *drop* on every targeting sweep. `CountOf` is one frame per
   permanent per query, a `Frames/walk` *rise* on every board with a Warlord.
   The measurement separates them (three binaries: `main`, the engine with
   the pools unchanged, and shipped).

**Measurement.** Three binaries, interleaved in one sitting, 50 games / seed
12345 / `--threads 1`, medians of seven rounds — `main` (A), the engine with the
pools unchanged (B), shipped (C); `codebase-state.md`'s RC-4 block carries the
tables. **The frame is close to free**: B against A is flat on every fixture
row (walks within 0.1%, frames within 0.6%, both game-content deltas). On
time it read +3.2% / +1.6% at 50 games and, re-run at 200 games over three
interleaved rounds, **−2.2% / +1.1%** for `performance` / `stress` — inside
the spread, so the indirection §11 item 5 said to watch for did not leak into
the hot walk by any measurement made. **The count is the quadratic §12 predicted**:
`Frames/walk` 1.25 → 1.45 on `performance` and 1.20 → 1.31 on `stress`, all of
it Keldon Warlord's 1 + N frames per walk, which is why he is in
`PERFORMANCE_POOL` and why item 7's cross-call memoization is still the lever.

**Exit met.** Whole suite green (853 tests, 30 of them new), zero warnings,
both `check_*.py`, `specdb owed` unchanged. Event streams at 40 games against
`main` with the pools unchanged: 33/40 and 32/40 byte-identical, and every one
of the 15 divergences attributed to a CR 616.1 prompt that no longer fires —
Chainbreaker or Idyllic Beachfront under one Root Maze, or any land under two.
Determinism: three 200-game runs per pool byte-identical outside `=== Timing
===`; `--threads 1` and `--threads 8` identical in the results and engine-work
blocks. Reachability counted in 40 `stress` games: the Priest entered 13 times
and exiled a Dryad Arbor twice.

#### RC-4b — entering is one event — ✅ landed 2026-09-02

*Evicted 2026-09-11 from `plans/replacement-architecture.md`, where the heading and a stub remain.*


**The defect** is `codebase-state.md`'s "Before Triggered abilities" item 4,
found in RC-4's review. An entry is two proposals: the `ZoneChange` that moves
the card, and the `EnterBattlefield` proposed from *inside* its performer
(RC-2's one-`emit`-wide window). A replacement that substitutes the entry —
Containment Priest — therefore runs after the move, and its substitute is a
second move out of the zone. The log then holds a zone change into the
battlefield, a zone change out of it with `from: Battlefield` and a CR 603.10a
LKI frame, and two `zone_change_epoch` bumps, for a card the CR says was exiled
from its graveyard and never entered. Nothing reads the log for triggers yet,
so it is unreachable today and a bug-in-waiting for critical-path item 6. The
review's trace artifact (pinned to 8ac4ad7) is the before-picture.

**The design: the entry is the only proposal for entering.**

1. `change_zone` with a battlefield destination routes to `propose_entry`,
   which proposes `GameAction::EnterBattlefield { object, from: Option<Zone>,
   controller, mods, cause }` as an ordinary batch member. Callers do not
   change — `play_land`, `resolve_top_of_stack`, the `Returned` road, every
   test. No `ZoneChange { to: Battlefield }` proposal exists any more;
   constructing one is a debug assertion.
2. The `EnterBattlefield` performer moves the card, emits the `ZoneChange`
   event, builds the entity and emits `PermanentEnteredBattlefield`. Both
   arms call one private move-and-emit function, and the cast path (item 7)
   calls its emitter half at 601.2i for a move it performed silently at
   601.2a. CLAUDE.md's one-emitter bullet is reworded to what is then true:
   one emitter function, three callers, each of which performed the move it
   announces; and its `CAST-ROLLBACK` exemption widens to both directions of
   CR 601.2's move, which are not events until 601.2i.
3. `Rewrite::Instead(ZoneChangeTo)` on an entry yields `ZoneChange { from,
   to, cause }` with the entry's own `from`, performed as one move by the
   ordinary arm. No hop, no LKI walk, one event, one epoch.
4. A dropped entry (CR 614.6) leaves the card where it was, so
   `propose_entry`'s "replaced away" error is deleted, and a CR 614.17d
   "can't enter" may watch the entry directly.
5. `pattern_watches` gains one arm: `EventPattern::ZoneChange { to:
   Some(Battlefield), .. }` also matches an `EnterBattlefield { from, .. }`
   proposal, with `from` and `cause` compared as today. That keeps Worms of
   the Earth's restriction and any zone-change-shaped replacement working,
   and it puts them in the same CR 616.1 bucket as the `EnterWith`s — one
   event, which is what the CR says entering is.
6. `EntryFrame`'s `Pending` basis is deleted. Every entry proposal carries
   its controller and mods, so there is one basis, and `is_prohibited`'s
   derived-frame block shrinks to one arm.
7. **The cast rewind, bundled here because it is the same bug from the
   other side** (`codebase-state.md` item 51). `cast_spell`'s CR 601.2a move
   goes through `change_zone` and emits; the rewind sites move the card back
   with the silent `CAST-ROLLBACK` mover, so a cast that fails at 601.2b,
   601.2c or 601.2h leaves a `ZoneChange { Hand → Stack, Cast }` with no
   counterpart. Nothing in the CR replaces a card being put onto the stack —
   RS-2's "can't cast" is a CR 601.3 question asked of the *player*, ahead
   of 601.2a (`cant-effects-architecture.md` §4.3), and its Tier 1b/1e exits
   are more rewind sites, so the phantom gets more reachable with RS-2, not
   less. The 601.2a move therefore uses the silent mover in both directions,
   tagged, and the zone change is emitted at 601.2i beside `SpellCast` — the
   moment CR 601.2i says the spell becomes cast. Mana abilities activated in
   601.2g stay performed and stay in the log (CR 732.1). The RA test that
   asserts the cast's zone change precedes its resolution's still passes,
   because it still does.

**Two consequences, decided before code (2026-09-02).**

- **Tokens: the cheap answer, and where the honest one lives.** A token is
  created in `Zone::Battlefield` with no entity and in no collection — the
  state the walk's membership gate already reads as "entering" and CR 704.5d
  reads as "on the battlefield" — and its entry carries `from: None`. An
  `Instead(ZoneChangeTo)` on it is performed as `ZoneChange { from:
  Battlefield, to }` with no LKI (no permanent ever existed to look back at),
  which is the shape tokens have today: the log says `from: Battlefield` for
  a token CR 111 says was created in exile, and CR 704.5d removes it. A
  dropped token entry is CR 111.5's "the token is not created", so
  `CreateToken` un-creates the object it made — `add_object` was not an event
  and neither is its reversal. **Why cheap:** the honest state is a token in
  no zone, and the honest *event* is a creation whose destination the entry's
  decision sets — which is Phase RE's `CreateTokens` proposal (CR 614.16's
  doublers need it anyway), not an `Option<Zone>` threaded through
  `GameObject.zone`'s 38 readers for a shape no registered card reaches:
  Containment Priest excludes tokens and Hallowed Moonlight is not registered.
  Recorded as `codebase-state.md` item 52 — and **not optional before Phase
  8** (owner, review): Dour Port-Mage's and Aang, Airbending Master's "leave
  the battlefield without dying" is a trigger keyed on `ZoneChange { from:
  Battlefield }`, and it would fire for a token Hallowed Moonlight created in
  exile. The residual is the same wrong `from` this phase fixed for cards,
  and RE's `CreateTokens` proposal is where it closes, back-stopped under
  "Before card breadth" item 8.
- **CR 608.3e.** A resolved permanent spell whose entry does not happen — a
  "can't enter" refused it, or a replacement dropped it — is still on the
  stack afterward, and `resolve_top_of_stack` moves it `Stack → Graveyard`
  with cause `Resolved`: the spell resolved, which is the rule's own wording.
  An entry *substituted* by a zone change (exile instead) leaves the card
  elsewhere and takes no leg. The Aura attachment that follows the entry is
  guarded the same way, since a host must not list an Aura that never
  arrived. Phase RE's list loses the item.

**The rule the audit produced** is §11 item 20: a performer may propose a
contained event only when the outer event is real whether or not the
contained one survives replacement. Entering fails it because the entry *is*
the zone change; casting fails it for a different reason, and its fix is
design item 7 above.

**What changes in the review's traces.** Trace A loses its outer zone-change
decision (steps 3–5), keeps steps 7–18 verbatim — the frame, both filters,
the bucket and the prompt — and ends in one move from the hand with no LKI.
Trace B is unchanged: the membership gate's "or entering" arm already admits
an object that has not moved. Trace C asks the same restriction at the entry
proposal, with the same controller and mods, and loses only its closing
caveat. The after-picture of A is the first page worth checking in under
`plans/traces/`.

**Verification.** `fuzz_games --dump-events` against main: every Containment
Priest exile shrinks from two zone changes to one, and nothing else moves.
Determinism unchanged. The RC-4 Priest tests' log assertions flip to one
event; the "replaced away" test goes; a token exiled instead ceases to exist;
a resolved spell whose entry is refused is in the graveyard. A cast rewound
at 601.2b, 601.2c or 601.2h leaves no `ZoneChange` in the log, and the
mana-ability taps it made stay.

**Sized:** ~400–600 additions and a similar deletion count across
`actions.rs` (the two arms, `propose_entry`, the shared mover),
`pipeline.rs` (the `Instead` arm), `gather.rs` (the `pattern_watches` arm),
`replacement/lookahead.rs` and `restriction/predicate.rs` (deletions),
`resolve.rs` (tokens, CR 608.3e), `stack.rs`, `cast.rs` (the silent 601.2a
move and the 601.2i emission, ~30), and tests. One PR, ahead of
RC-5, because RC-5 part 2 needs entries to be first-class batch members and
this is that half of it; and ahead of the trace sink, whose emit points sit
in the same performer.

**Landed 2026-09-02.** Shipped +378 / −250 engine across 12 files and +568 /
−28 tests across 4 — inside the band on the engine, over it on tests, as RC-4
was. Three commits: the tests first, failing on the pre-fix tree (10 of 12;
the two that pass are the regression guards the brief named), then the
entry, then the cast.

**Four findings.**

1. **Source 1a read the entering permanent through a plain walk, and it was
   right by accident.** `gather` asked `get_effective_abilities` of the
   entering object for its own `SourceOnly` replacements. The plain walk
   admits an object to filter-scoped Layer 6 effects by its *zone*, and the
   card was in the battlefield zone only because RC-2's performer had already
   moved it. Propose the entry before the move and the leg goes silent:
   Humility no longer strips an entering Chainbreaker's "enters with two
   -1/-1 counters" and Blood Moon no longer strips an entering tapland's
   "enters tapped" — measured, two RC-2 tests fail with the plain walk kept.
   The correct read is CR 614.12's frame, which `gather` already held for
   `set_affects`; source 1a now reads `frame.frame_of(object)`, computed once
   per iteration and shared. This is the one reach into `gather` beyond the
   `pattern_watches` arm the plan sanctioned, and it is a correction rather
   than a cost: one walk per gather either way, and one fewer where a filter
   also asks.
2. **The cast's move is announced after its 601.2g taps, and the plan's
   expected-divergence list did not say so.** Design item 7 puts the
   `ZoneChange` at 601.2i beside `SpellCast`; the mana abilities activated at
   601.2g are performed and logged before that. So every completed cast in a
   fuzz game reorders — 703 in 40 `performance` games, 693 in `stress` —
   which "nothing else moves" above did not anticipate. It is the designed
   shape, `test_a_cast_is_announced_at_601_2i_after_its_mana_abilities`
   asserts it, and a trigger keyed on the cast's zone change now fires after
   the taps, which is the order CR 601.2 gives the events. (The rule the
   taps survive a rewind under is CR 732.1 in the `tmnt` baseline; 727.1,
   cited here and in `codebase-state.md` item 51 until today, is rad
   counters.)
3. **The frame's `Pending` basis was the entry hop's shadow.** It existed to
   answer a CR 614.17d "can't enter" at the zone change ahead of the entry,
   deriving the controller and `EnterMods` the entry *would* carry. With the
   entry as the only proposal every frame is built from a proposal that
   carries both, and `FrameBasis` is a struct.
4. **CR 608.3e wanted a guard the plan did not name.** `resolve_top_of_stack`
   attaches a resolved Aura to its target after the entry; with an entry that
   can now be refused, that code would have pushed an Aura that never arrived
   onto its host's `attached_by`. Guarded on the Aura being on the
   battlefield.

**The token decision, as shipped.** The cheap answer above: created in the
zone with no entity, `from: None`, exiled-instead as `ZoneChange { from:
Battlefield, to: Exile }` with no LKI, un-created when dropped (CR 111.5).
`codebase-state.md` item 52 carries the residual and its RE home.

**Measurement.** Two release binaries — `main` (1fb9a80) and this branch — at
40 games / seed 12345 / `--threads 1` per pool with `--dump-events`, ObjectIds
masked, every hunk classified. **Exactly three classes of divergence, all
designed, and nothing else**: (1) a completed cast's `Hand → Stack [Cast]`
moves to just before `SpellCast` — 703 casts in `performance`, 693 in
`stress`; (2) a rewound cast's `Hand → Stack [Cast]` disappears — 303 and
289; (3) Containment Priest's exile collapses from two zone changes to one —
2, both in `stress`, the Dryad Arbor pair RC-4 counted. With main's stream
canonicalized by (1) and (3), every remaining hunk is (2), and the one
`stress` game with no rewind is byte-identical. Determinism: three 200-game
`--threads 1` runs per pool byte-identical outside `=== Timing ===`, and
`--threads 8` identical to `--threads 1` but for its `Threads:` line.
**Fixtures**, 200 games / seed 12345 / `--threads 1`, medians of three
interleaved rounds:

| Fixture | `performance` main | branch | Δ | `stress` main | branch | Δ |
|---|---|---|---|---|---|---|
| Layer walks | 100,641 | 100,496 | −0.14% | 92,331 | 92,139 | −0.21% |
| Layer frames | 136,595 | 136,402 | −0.14% | 121,161 | 120,917 | −0.20% |
| Frames/walk | 1.36 | 1.36 | 0 | 1.31 | 1.31 | 0 |
| Replacement gathers | 633 | 567 | **−10.4%** | 602 | 538 | **−10.6%** |
| Restriction queries | 636 | 569 | −10.5% | 605 | 542 | −10.4% |
| Time/game (ms) | 112.6 | 111.9 | −0.6% | 92.9 | 95.2 | +2.5% |
| Avg turns | 30.7 | 30.7 | 0 | 29.4 | 29.4 | 0 |

The gathers row is the phase's own claim, measured: an entry was two pipeline
passes — the zone change, then the nested entry — and is one, so roughly one
gather per entering permanent per game is gone, and the restriction query
that rode on each of them with it. The walks row is the predicted small drop:
the Priest's LKI walk (two in 40 `stress` games) and source 1a's frame
shared with `set_affects` wherever a filter also asks. Turn counts are
identical, which is the prompt count not moving. The `stress` time delta is
inside the spread RC-4's 200-game rounds showed for a change measured as
free (−2.2% / +1.1%) and was not re-run.

**Exit met.** 865 tests (12 new), zero warnings, both `check_*.py`, `specdb
owed` unchanged for RC. Trace A's after-picture is the first two tests in
`phase_rc4b_integration_test`, and the phase's trace page — the first under
`plans/traces/` — is `plans/traces/rc-4b-entering-is-one-event.html`: Traces A
and C from RC-4's review as they run now, B unchanged, and a fourth for the
cast, with every read labelled board or frame.

#### RC-5 — auxiliary zone changes and a dynamic entry amount — ✅ landed 2026-09-03

*Evicted 2026-09-11 from `plans/replacement-architecture.md`, where the heading and a stub remain.*


What RC-4 sized out, in the doc rather than in the moment. **Re-sized against
the tree 2026-09-03, before a line of RC-5 was written, and the re-size moved
two of the four pieces** — the subsection this replaces predates RC-4b and
described RC-2's nested `propose_entry`, which RC-4b deleted. The corrected
picture was already in `codebase-state.md` item 46; this section had not been
reconciled against it. What follows is the tree as it is.

##### Piece 2 — the batch-scoped frame — **closed by RC-4b. This is why.**

The claim was: "today an entry is proposed *inside* the zone change's
performer, so the second member of a `[ZoneChange, ZoneChange]` batch is
decided after the first was performed and sees it on the battlefield." **Every
clause of that is false of the tree.** RC-4b routes a battlefield destination
in `change_zone` to `propose_entry`, which proposes `EnterBattlefield` as an
ordinary batch member; `execute_batch_inner` phase 1 runs `apply_replacements`
for *every* member before phase 2 performs *any*; and a `ZoneChange { to:
Battlefield }` proposal is a debug assertion. `EntryFrame::new` is built from
the proposal inside phase 1, so it reads the pre-batch board by construction.
**§5b's "two Master Biomancers entering as one event give each other nothing"
is already true**, and the restructuring this piece was sized for (~400 in
`execute_batch_inner` and `perform_action`'s `ZoneChange` arm) was paid by
RC-4b's +378/−250.

What is genuinely left of item 46 is **not a frame question**, and it is two
things:

1. **Nothing produces a multi-entry batch.** `propose_entry` is called once per
   entry; `Primitive::CreateToken` loops it; `Primitive::ReturnToBattlefield`
   is a stub. So the batch-scoped frame is *right and unreachable from a
   registered card*, which is the RB-Kalitas shape this section keeps warning
   about. RC-5 does what can honestly be done about it: it **tests it at the
   `execute_actions` boundary** — two Master Biomancers as one two-member batch
   — which proves the engine and does not pretend to prove the pool. The line
   in the ledger says exactly that.
2. **CR 613.7m.** Unchanged, and see below.

##### Does CR 614.13 make CR 613.7m reachable? **No — and 613.7m stays in RE.**

Asked because "Before card breadth" item 4 named "CR 614.13's auxiliary zone
changes (RC-4)" as one of the two things that make APNAP timestamps reachable.
Measured: **it is not one of them.** 613.7m is about objects that "receive a
timestamp simultaneously, such as by entering a zone simultaneously or becoming
attached simultaneously". In this engine an object receives a timestamp in
exactly one production place — `place_on_battlefield`, one `PermanentState`
per entry (`game_state.rs:671`; the other production caller, `:789`, is
CR 613.7c's per-counter-kind stack, which is not an object and is unchanged
here). **CR 614.13's auxiliary zone changes allocate none of them**: devour's
are battlefield → graveyard and Sutured Ghoul's are graveyard → exile. Neither
destination has a timestamp, and neither is an entry.

So 613.7m needs *simultaneous entries*, which 614.13 does not produce, and it
stays where item 4's other half put it: reachable first from
`GameAction::CreateTokens` (Phase RE), which is also where item 52's token
residual lands. **Item 4's parenthesis is corrected to name RE alone.** The two
pieces were scheduled together on the belief that they were one restructuring;
they were one restructuring, RC-4b did it, and what is left of each has nothing
to do with the other.

##### Piece 4 — choice-carrying mods (CR 614.12a's full form) — unchanged

"As this enters, choose a color" (Voice of All, Painter's Servant) needs the
choice recorded on the permanent for a linked ability to read — CR 607, which
is `backlog.md` §2.2's. RC-4 claims 614.12a partially through the CR 616.1b prompt, which is
made before the entry and whose result the announced entry carries; the general
field waits on linked abilities. Listed here so it stays findable. **Sutured
Ghoul's third sentence is in this piece, not in piece 1** — its power and
toughness read "the exiled cards", which is CR 614.14's linked pair.

##### Piece 1 — CR 614.13/13a/13b, devour and its kin

An entry replacement whose *application* (a) prompts for a set of objects,
(b) moves them while applying, and (c) sets the `EnterMods` from the count.
None of the three exists: a `Rewrite` is a pure function of the event, riders
run *after* it (§4.1a), and `EnterMods.counters` carries literals.

**The arm, and the rule that permits it.** `Rewrite` is a closed algebra and a
new arm needs the CR rule that permits the operation; this one is CR 614.13's
own sentence — "an effect that modifies how a permanent enters the battlefield
**may cause other objects to change zones**". `Rewrite::EnterAfterMoving`
carries an `AuxiliaryMove`: where the choosable objects are, what they must be,
where they go and why, and what the entering permanent gets per object chosen.
Per-mechanic variety stays in the payload rather than in new arms — devour N is
`per_chosen: (PlusOnePlusOne, N)`, Sutured Ghoul is `per_chosen: None`.

**The moves are a nested batch with a *fresh batch id*, and that is a change to
§4.2.** A nested `execute_actions` joins the enclosing batch, on CR 120.3f's
grounds: lifelink's life gain is a *result of* the damage, part of the same
event. Devour's sacrifices are not part of the entry — they are performed in
phase 1, before the entry event exists — so joining would tell a CR 603.2c
"whenever one or more creatures die" trigger that two devour creatures'
sacrifices were one event. `execute_actions_new_batch` is the escape, one
caller, tagged; §4.2 gains the exception and CLAUDE.md's bullet gains four
words. Fresh applied sets fall out of it being a separate `execute_actions`
(CR 614.5), and Kalitas replacing a devoured creature's death is the test.

**The two exclusion sets live on `GameState`, and item 40 is why.** They are
per *batch*: 614.13a excludes the entering object and anything entering
simultaneously with it; 614.13b excludes anything already chosen by an entry
replacement applying to the same simultaneous entries. Both are consulted
across the CR 616.1 prompt and both change the outcome if lost — drop 614.13b's
set and Thunder-Thrash Elder sacrifices one Runeclaw Bear to devour 3 *and* to
devour 5, which is the CR's own example of the wrong answer. So they are
`GameState::entry_selection`, saved and restored around every batch the way
`open_batch`/`close_batch` handle the stamp, and **item 40's table gains no
third violator**.

**614.13a's first clause is reachable and its second is not.** A Sutured Ghoul
entering *from the graveyard* is in the graveyard when its own replacement
applies — RC-4b decides the entry before the move — so without the clause it
exiles itself; `change_zone(.., Battlefield, Returned)` is a production path
and a test drives it. The second clause needs two entries in one batch, which
only `execute_actions` can build today (see piece 2).

**Printed, counted 2026-09-03** (`keyword:devour`, `o:/as .* enters, exile/`):
devour is **23 cards**, not the "~30" this section carried; the graveyard-exile
variant is **5** (Sutured Ghoul, Living Lore, Dermotaxi, Mimeoplasm Revered One,
Frankenstein's Monster). CR 702.82c's **devour [quality]** — artifacts, lands,
Foods — is four of the 23 and costs nothing here, because the payload's `filter`
is what says "creatures".

**Two of the 23 the payload does not reach.** *Thromok the Insatiable* is
"devour X, where X is the number of creatures devoured this way": its multiplier
**is** the count, so X creatures give X² counters, and `per_chosen` is a
constant per object (item 63). *Frankenstein's Monster* exiles exactly X and puts
itself into the graveyard "if you can't", which is a cast-time X and a failure
branch. Both are payload shapes rather than new arms.

##### Piece 3 — a dynamic counter amount in `EnterWith`

Master Biomancer's "equal to this creature's power". **The type split is the
finding**: `EnterMods` is the payload of both `Rewrite::EnterWith` and
`GameAction::EnterBattlefield`, and §3.2 recorded that as a virtue — "the same
type describes what one effect *adds* and what the permanent will *end up
with*". The moment one side needs an *unevaluated* expression they part
company, because the event's mods must be numbers the performer can put on.
So `Rewrite::EnterWith` takes an `EnterModsTemplate` whose counters are
`(CounterType, AmountExpr)`, `apply_rewrite` evaluates it into an `EnterMods`,
and `EnterMods::merge` is untouched. ~14 construction sites change constructor
and nothing else.

**Evaluated against the source's frame, which is `EntryFrame::frame_of(source)`
and answers §5b for free.** The frame is `Some` exactly when the source *is*
the entering object, so an entering permanent's own "enters with a counter for
each ..." reads its hypothetical self and Master Biomancer — a real permanent —
is read off the real board. Elvish Archdruid enters under Biomancer with **2**
counters, not 3, with no clause anywhere saying so: it falls out of RC-4's
asymmetry, exactly as §5b predicted. `AmountExpr` gains `SourcePower`; the two
existing evaluators match exhaustively, so each grows an arm rather than
defaulting.

**It fires item 47's expiry conditions, and the predicate is revisited in the
same commit.** `ordering_cannot_change_outcome`'s theorem has two halves — every
member still applies, and the applications commute — and the second was free
while `merge` was `|=` and `+` over literals. An amount read off the frame is
not free: "enters with X counters where X is its own power" applied before and
after a `-1/-1` counter gives different answers. The premise added is exact
rather than conservative — an amount is order-invariant if it is `Fixed`, **or**
its instance's source is not the entering object, since only then can
`frame_of` return `Some`. Master Biomancer therefore keeps the suppressed
prompt and the fuzz pool keeps its zero-prompt property.

##### Sized

| Piece | Was | Now |
|---|---|---|
| 1 — CR 614.13/13a/13b | ~1,200 | ~1,200: the arm and its payload, the selection prompt and its `ChoiceKind`, the fresh-batch escape, `GameState::entry_selection`, two cards |
| 2 — batch-scoped frame + 613.7m | ~400 | **0** — RC-4b paid it; RC-5 adds the two-entry test and the ledger line |
| 3 — dynamic `EnterWith` amount | ~150 | ~250: the template split touches ~14 construction sites the estimate did not count |
| 4 — choice-carrying mods | not RC-5 | not RC-5 (CR 607, `backlog.md` §2.2) |

Together at the middle of the band rather than the top, because piece 2 is
gone. Cards: Thunder-Thrash Elder and Sutured Ghoul (piece 1), Master
Biomancer (piece 3). Painter's Servant is piece 4's and stays blocked.

##### Shipped 2026-09-03

**+2,239 / −121 across 18 files** — 748 engine and cards, 1,111 tests, 380
plans. The engine is inside the band and the tests are over it, which is the
third RC PR in a row with that shape. Two commits: the re-size above, then the
phase.

**Five findings.**

1. **CR 614.13b was redundant on every board the first draft of the tests could
   build, and a mutation pass is what found out.** Delete the `chosen` set and
   the CR's own example — one Runeclaw Bear, devour 3 and devour 5 — still gives
   three counters, because the Bear is in the graveyard by the time the second
   effect enumerates the battlefield. The rule is not bookkeeping; it needs two
   effects whose *zones chain*, and devour into a graveyard-reading exile is
   that board. Recorded as trace B on the phase's page, and the test is
   `test_a_devoured_creature_cannot_then_be_exiled_by_the_next_effect`.
   **The general lesson is §10's, sharpened:** "mutation-check every assertion"
   found not a weak assertion but a *weak board* — the test asserted the right
   thing about a scenario in which the rule could not fire.
2. **Sigarda does not stop your own devour, and the card doc said she did.**
   Her sentence is "spells and abilities **your opponents control** can't cause
   you to sacrifice permanents" — `SourceFilter::ControlledBy(Opponent)` — and
   devour is your own creature's ability, so your own Sigarda is a legal
   candidate for it. Written into Thunder-Thrash Elder's doc as a claim, caught
   by the test that asserted it. What the pair does measure is that the `cause`
   the candidate filter asks CR 101.2 with is the *effect's* controller.
3. **The CR 616.1 bucket puts the entering permanent's own effect last.**
   `gather` splices source 1a in after the battlefield sweep, on CR 613.7's
   oldest-first — the entering object is the newest there is. Two tests were
   written with the indices the other way round and asserted the wrong number
   for a real reason, which is the cheapest kind of test failure.
4. **The `EnterMods` split was not in the ~150 the piece was sized at.**
   `Rewrite::EnterWith` and `GameAction::EnterBattlefield` shared one type, and
   §3.2 recorded that as a virtue. It held exactly as long as every amount was a
   literal: the event's mods must be numbers the performer can put on, so the
   definition needed its own type the moment one amount was an expression.
   ~14 construction sites, mechanical, and the estimate is corrected to ~250
   above rather than left as evidence of good sizing.
5. **A prevented auxiliary move is unreachable in printed Magic, and the count
   still has to be right.** Nothing printed stops a sacrifice's *move* — it is
   not damage, so prevention does not apply, and not a destruction, so
   regeneration and indestructible do not either. `performed.len()` rather than
   `picked.len()` costs nothing and is the reading CR 701.21a gives; the test
   that pins it uses a `Rewrite::Prevent` fixture and says in its doc comment
   that it is engine-shaped rather than card-shaped.

**Measurement.** `plans/fuzz_ab.py`, three arms in one sitting: `main` (A),
this branch with `PERFORMANCE_POOL` as `main` had it (B), and as shipped (C).
**B is byte-identical to A on `performance` outside `=== Timing ===`** at 200
games / seed 12345 — same games, same counters, same event stream — so the arm,
the template evaluation and the two exclusion sets cost the unchanged pool
nothing. **B is identical to C on `stress`**, because `stress` is the whole
registry and a registered card is in it whether or not it joined the measured
pool. Between them the two columns partition the change: `performance`'s
movement is the two cards joining that pool, `stress`'s is the three cards being
registered. Time +0.7% (B) and +1.0% (C) on CPU/game, inside the ~2–6% spread a
sitting shows, not chased. The §3 fixture table is re-recorded with the A/B
beside it.

**Determinism.** Three shell `fuzz_games` runs at seed 12345, 200 games,
`--threads 1`, on both pools: identical outside `=== Timing ===`. The A/B's own
three timing rounds per arm agree with its threaded counter run, which is the
thread-independence half.

**Debug-assertion fuzz, added on review** — the release A/B measures cost and
says nothing about the assertions, and this phase's sharpest internal check
(`check_order_invariance`, a second gather per suppressed prompt) is
`cfg!(debug_assertions)`-only. A **debug** build at 400 games on each pool and
120 games with each new card forced into every deck: **0 errors, 0 panics, 0
uncast resolutions** throughout, with 134 devour resolutions in the forced run.
That is the evidence that the new paths hold up on boards nobody wrote a test
for; it is not evidence that the *rules* are right, which is what the review's
own findings are for.

**Exit met.** 937 tests (23 new), zero warnings, both `check_*.py`, `specdb
owed` unchanged. All three of CR 614.13's atoms are covered rather than
partial — ATOM-614.13-001, -614.13a-001 and -614.13b-001. **Trace page:**
[`plans/traces/rc-5-applying-an-entry-can-move-the-board.html`](traces/rc-5-applying-an-entry-can-move-the-board.html)
— devour's selection and its nested batch (A), the zone chain that makes
CR 614.13b bite (B), `frame_of(source)` and §5b's asymmetry (C), and two
entries decided against one board (D), which is the re-size's evidence.

##### What RC-5 leaves owed, named rather than implied

- **A production multi-entry batch.** `Primitive::ReturnToBattlefield` is the
  natural one and it is *not* a small job: `SelectionFilter` enumerates the
  battlefield, the stack and players and has no graveyard leaf, and a mass
  return is `default_enter_controller`'s known-wrong fourth road
  (`codebase-state.md` item 48). Sized at a graveyard leaf plus item 48's
  `controller` field plus the primitive — call it ~350 and a card — and it is
  what makes 614.13a's second clause and §5b's batch-scoped frame reachable
  from a game rather than from `execute_actions`. Recorded against item 46.
- **CR 613.7m**, in RE with `CreateTokens`, per the answer above.
- **Sutured Ghoul's P/T**, in piece 4 with CR 607 — `backlog.md` §2.2 is the
  live home for that work.
- **One batch for every auxiliary move of one entry event** — item 61, above.
- **Whose choice a granted devour is** — item 62. `AuxiliaryMove` has no chooser
  field, so "you" is the *effect's* controller. Right for Master Biomancer's
  "each other creature you control"; wrong for a devour granted by someone
  else's permanent, where CR 614.13a's "you" is the entering creature's
  controller. The two coincide on every registered board.
- **Thromok's `devour X`** — item 63.

#### RD-1 — the damage event's two subjects and its results — ✅ landed 2026-09-08

*Evicted 2026-09-11 from `plans/replacement-architecture.md`, where the heading and a stub remain.*


**Builds:** decisions 0, 4, 5, decision 1's `Multiplier`, `Halve` and
`PreventHalf` with `Rounding` (`Plus` waits for Torbran in RD-3, since an arm
with no consumer is the thing this doc refuses), the rider's two amount
leaves, and `Primitive::Mill` — a stub in `resolve.rs`
today, and N `ZoneChange { Library → Graveyard, cause: Milled }` proposals off
the top, which the RB-registered Leyline of the Void already watches, so the
first mill in the pool meets a replacement for free. **Consumers**, each with
its rulings pass:

- **Furnace of Rath** — "If a source would deal damage to a permanent or
  player, it deals double that damage to that permanent or player instead."
  `EventPattern::DealDamage`, `Filter { All }` + `Everyone`,
  `Amount(Multiplier(2))`, `Uses::Static`. Rulings, five: *two Furnaces multiply by 4* → test
  (CR 614.5's own example; `ATOM-614.5-001` moves from partial to full);
  *the damage counts as if from the original source, Furnace is not the
  source* → test on `DamageDealt.source`; *divide before doubling* and *trample
  divides before doubling* → structurally true, since `assign_combat_damage`
  divides before anything is proposed — asserted with War Mammoth, which is in
  the pool; *prevent-4-then-double or double-then-prevent-4* → **RD-2's**
  test, when Mending Hands exists. Two Furnaces in one deck is what the pool
  can build (it is not legendary), and two Furnaces on one damage event is
  CR 616.1's multi-candidate prompt from two *printed* cards
  (`COMP-614-616-DOUBLE-REPLACEMENT-001`), which the pool has never had.
- **Ghosts of the Innocent** — "If a source would deal damage to a permanent
  or player, it deals half that damage, rounded down, to that permanent or
  player instead." `Amount(Halve(Down))`, the same scope as Furnace. Its six
  rulings are listed under decision 1; four are RD-1's tests, and the one
  that matters most is Furnace beside Ghosts on 3 damage — 1 then 2, or 2
  then 1 — the first **non-commuting** CR 616.1 choice a fuzz game can reach
  (`COMP-614-DAMAGE-ORDERING-001`). Dictate of the Twin Gods was the first
  draft's second doubler and is dropped: same shape as Furnace, and two
  Furnaces already give the commuting pair.
- **Gisela, Blade of Goldnight** — "Flying, first strike. If a source would
  deal damage to an opponent or a permanent an opponent controls, that source
  deals double that damage to that player or permanent instead. If a source
  would deal damage to you or a permanent you control, prevent half that
  damage, rounded up." Two statics on one card, each a `Filter` + `PlayerSet`
  row: `Multiplier(2)` over `ByController(Opponent)` + `Opponents`, and
  `PreventHalf(Up)` over `ByController(You)` + `You` — decision 0's two player
  sets and both halves of decision 1 on one consumer. Rulings, three: *doubles
  from any source, including the opponent's own* → test; *the affected player
  orders* → the Furnace/Ghosts test generalized to a prevention (Gisela's
  half beside an opponent's Furnace on 5: prevent 3 then double 2, or double
  to 10 then prevent 5); *divide, then double* → the War Mammoth test.
- **Angel of Suffering** — "Flying. If damage would be dealt to you, prevent
  that damage and mill twice that many cards." The static, whole-event
  `Prevent` on a player subject — `Fixed(vec![])` + `You`, `Uses::Static` —
  with a rider reading `Multiply(ReplacedAmount, 2)`. RD-1's only prevention
  consumer, and the one that exercises `Rider`'s player subject (item 27) and
  the event-amount leaf. Rulings, three: *you mill all cards if fewer remain*
  → test (CR 701.13's "as much as it can"); *the damage is prevented even if
  you can't mill twice that many* → test; *if the damage can't be prevented
  you still mill twice that many* → **RD-4's** dovetail test, beside Reverse
  Damage (decision 4).
- **Loyalty Probe** — the fixture, decision 5. Tests: 3 damage to a 5-loyalty
  planeswalker leaves 2 (`ATOM-120.3c-001`), lethal damage kills it through
  CR 704.5i, damage to a creature planeswalker both marks and removes.
- The decomposition's own tests: a player's combat damage proposes a contained
  `LoseLife` in the damage's batch (the lifelink test's twin); a prevented
  damage proposes nothing; `LifeChanged.source` survives.

**`PERFORMANCE_POOL` +1, Furnace of Rath**, predicted: it is the first static
`DealDamage` source in the pool, so it opens the gather sweep on every damage
event while it is on the battlefield — a new engine path, and the one this PR
should measure. Loyalty Probe is registered and not pooled; `--require
"Loyalty Probe"` beside `copies/deck` is its reachability row.

**Atoms:** `ATOM-701.10g-001`, `ATOM-614.5-001`,
`COMP-614-616-DOUBLE-REPLACEMENT-001`, `COMP-614-DAMAGE-ORDERING-001` (Phase
6), `ATOM-120.3c-001` (filed Phase 8 — covered where it is, not re-filed).

##### As landed

Six commits, ~1,950 lines, inside the band and near the prediction (~1,500–1,700
was low by the card file's doc comments, which carry the rulings pass). Every
decision above shipped as designed; three things are worth recording because
they were decided in the writing rather than in the design check.

- **`COMP-614-DAMAGE-ORDERING-001` is `COVERS-PARTIAL`, not `COVERS`.** The
  atom's non-commuting pair is written as "plus 1" and "double", and
  `AmountRewrite::Plus` has no printed consumer until Torbran in RD-3 — so the
  registered board is halve-and-double instead. Everything the atom asserts
  about the *choice* is proved (it is presented, both orderings are correct and
  different, each effect applies once); the specific numbers 3→4→8 and 3→6→7
  are not reachable, and the test says so. `ATOM-614.5-001` did move from
  partial to full, on two printed Furnaces.
- **CR 120.3e is now gated on the target being a creature.** Not in the design
  check, and it falls straight out of writing 120.3 as a list of results: the
  performer marked damage on any battlefield object, which is bookkeeping the
  rule does not have. Unreachable from the pool — `SelectionFilter::Any` offers
  only creatures, planeswalkers and players — and pinned by a test, because the
  wither and infect arms land right beside it.
- **`DamageResults` is a struct of flags, not an `if`/`else` chain**, because
  "one or more of the following results" is CR 120.3's own phrase and a
  creature planeswalker takes 120.3c *and* 120.3e. One
  `compute_characteristics` call answers both questions.

**Measured** (`plans/fuzz_ab.py`, three arms against a same-day `main`
worktree, 200 games at seed 12345, both pools; `engineering-practices.md` §3's
table re-recorded at 50). The middle arm's prediction held exactly:

| | prediction | measured (`performance`) |
|---|---|---|
| `Replacement gathers` | +1 per player-target damage event, nothing else | **516 → 528** (+12.0/game), and `Restriction queries` +12.0 with it — one of each per contained `LoseLife` |
| `Layer walks` | flat | **377 → 377**. `DamageResults`' type read is a memo hit every time: `Memo hits` +59/game and nothing else |
| gameplay counters | identical | identical — turns, spells, damage events, total damage, life changes all unchanged |
| CPU/game | flat | +0.8%, inside the ~2–6% spread |

The whole middle-arm diff against `main` is six lines: the three timing lines,
`Memo hits`, `Replacement gathers` and `Restriction queries`. No fourth binary
was needed.

**The shipped arm is the pool change and reads as one**: CPU/game **−6.4%**,
avg turns 31.0 → 29.7, total damage up — a doubler in every red deck ends games
sooner. Determinism: `tests/determinism_test.rs` green, and three shell
`fuzz_games` runs at one seed identical line for line outside `=== Timing ===`.

**Reachability.** Loyalty Probe, forced: cast 206, resolved 204, in 133 of 200
`stress` games (66%), 1.49 copies/deck. Unforced, **CR 704.5i fires 4 times in
400 `stress` games** — three Probes bolted to zero, and one Merfolk
Thaumaturgist that Cytoshape turned into a copy of a Probe and which died on
the spot, because CR 707.2 does not copy counters. That SBA had measured 0 at
every game count since it was written, and the second route was not predicted.

#### RD-2 — CR 615.7 prevention shields, and the loop's unit — ✅ landed 2026-09-09

*Evicted 2026-09-11 from `plans/replacement-architecture.md`, where the heading and a stub remain.*


**Builds:** decisions 2, 3, 7, `PreventUpTo`/`PreventRemaining`, the rider's
prevented amount (`Rider.prevented: u64`, read by `AmountExpr::DamagePrevented`
in `evaluate_amount`; a 0 makes the rider's `GainLife`/`DealDamage` a
`never_happens` non-event, so no `Effect::Conditional` is needed for "if damage
is prevented this way"). "Shield" here is the glossary's sense (2), never the
counter. **Consumers:**

- **Mending Hands** — "Prevent the next 4 damage that would be dealt to any
  target this turn." The plain shield: one row, `NextDamage(4)`, target
  filled at resolution as an object or a player, `UntilEndOfTurn`. No rulings.
  Tests: depletes per point across two events (`ATOM-615.7-001`); a 4-shield
  against 5 lets 1 through; the row is gone at 0 and at cleanup (CR 615.3's
  "until they're used up or their duration has expired"); two attackers into a shielded player prompt one
  allocation and any allocation leaves the same total (`ATOM-615.7-002`); a
  shield cast after the damage prevents nothing (`ATOM-615.4-001`). Healing
  Salve was the canonical printing and is modal — `Effect::Modal` errors at
  `resolve.rs:146` — so its plain sibling ships instead.
- **Samite Healer** — "{T}: Prevent the next 1 damage that would be dealt to
  any target this turn." The activated shape, and a creature the random agent
  will activate. Second shape by §3.3's axes: a repeatable source of rows.
- **Safe Passage** — "Prevent all damage that would be dealt to you and
  creatures you control this turn." A `Filter` + `You` row with no amount and
  `Uses::Static` — the shape whose set is evaluated at the event. Rulings,
  four, all tests: *all damage, not just combat*; *creatures that entered
  after it resolved are covered* (the load-bearing one — `Filter`, not
  `Fixed`); *not planeswalkers you control* (Loyalty Probe under Safe Passage
  takes the damage); *no effect on damage already dealt*.
- **Samite Censer-Bearer** — "{W}, Sacrifice this creature: Prevent the next
  1 damage that would be dealt to each creature you control this turn."
  CR 615.11's consumer: N rows of `NextDamage(1)` from `EffectRecipient::
  FilteredPermanents`, one per creature at resolution. Ruling: *a separate
  1-point shield on each creature you control at the time the ability
  resolves* → test with a creature entering afterwards (`ATOM-615.11-001`).
  Kitsune Palliator's "each creature and each player" is the same card plus an
  each-player recipient `EffectRecipient` lacks; one customer, so it waits.
- **Reverse Damage** is RD-3's (it chooses a source), and its rider is the
  prevented-amount consumer there; RD-2 tests the amount channel through a
  fixture rider until then, and says so in the test's name.

**The pipeline change** is the group form of `apply_replacements`: phase 1
groups the batch by `subject_of`, runs one loop per group, and a chosen
instance is applied to each member — with the 615.7 allocation deciding how a
`PreventRemaining` splits over the members it applies to. `execute_batch_inner`'s
phases 2 and 3 are unchanged. Two tests guard it, and they are one board: a
creature with two shield counters blocked by two attackers loses **one**
counter and takes no damage (§11 item 15's board); give one of the two
blockers first strike — Knight of Meadowgrain is in the pool — and it loses
**two**, because CR 510.4 makes two combat damage steps and the per-subject
rule is per batch. The second is what shows the key is the batch and not the
turn.

**`PERFORMANCE_POOL` +1, Mending Hands**, predicted: the first registry row a
damage event meets, and the first `allocate` prompt reachable in a fuzz game.

**Atoms:** `ATOM-615.7-001`, `ATOM-615.7-002`, `ATOM-615.4-001`,
`ATOM-615.5-001`, `ATOM-615.11-001`, `BOUNDARY-DEF-615.1a-001`;
`ATOM-615.6-001` as `COVERS-PARTIAL` (filed
Phase 7 — its "the trigger does not fire" half is item 6's).

##### As landed

Six code commits and the docs, +2,393 / −237 across 30 files — roughly 1,180
engine, 440 cards, 770 tests — at the top of the band and a little over the
~1,800–2,000 prediction; the difference is `next_damage_shares`, which the
sizing counted as "the group form" and which turned out to be its own
function once the allocation had to reach across groups. Every decision above
shipped as designed. What follows was decided in the writing:

- **The allocation's buckets reach into groups not yet decided.**
  `apply_replacements` takes the rest of the batch as `later`, read for one
  thing: a multi-subject count's prompt spans every member the instance
  applies to, at their proposed amounts, and the answer is kept on
  `GameState::prevention_allocations` for the groups after, which read their
  shares rather than asking (item 40's shape, `entry_selection`'s
  precedent). One chooser is asserted across the buckets; a def that names
  two sides is an error rather than a guess (`codebase-state.md` item 92).
- **`PreventRemaining` uncapped prevents everything, and `capped` fills the
  cap.** The type's arithmetic stays total; `apply_rewrite` refuses
  `PreventRemaining` without a `NextDamage` count and a count without
  `PreventRemaining`, so the uncapped arm is reachable from no def.
- **A rider's count is not observable in the log, and the test says so.**
  Angel of Suffering under two attackers: one rider milling 10 and two
  milling 4 and 6 leave the same ten records under one batch id, because a
  rider's moves join the batch the damage was in. The test pins the *sum*;
  the shield-counter board is where the count itself shows.
- **A row an ability makes names the permanent** (CR 113.7a):
  `ctx.ability_source.unwrap_or(ctx.source)`, so Samite Healer's row
  outlives the stack object CR 608.2n deletes and is what the CR 616.1
  prompt offers a UI. `Regenerate` keeps `ctx.source` (item 95).
- **The row keeps its targets and nothing reads them** (item 90) — the fact
  half of Divine Deflection's note, done where the row type was written; the
  reader is the card's PR.
- **§11 item 29 was decided yes and built here**, as its own engine commit
  ahead of registration so the A/B could carry it as an arm; the composite
  atom drops to `COVERS-PARTIAL`, since it says the player chooses and the
  engine now declines to ask. **Item 30's re-ask came back "no move."** The
  trace page is `plans/traces/rd-2-a-decision-is-per-subject.html`.

**Measured** (`plans/fuzz_ab.py`, four arms against a same-day `main`
worktree, 200 games at seed 12345, both pools; `engineering-practices.md`
§3's table re-recorded at 50): `main`; `loop` — the fix commit, the group
form and consume-after-apply with pools unchanged; `suppress` — the item-29
commit, pools unchanged; `new` — shipped.

| | prediction | measured (`performance`, 200 games) |
|---|---|---|
| `Replacement gathers`, middle arms | flat — grouping changes decisions, not proposals | **498 → 498 → 498** (`stress` 526 → 526 → 525) |
| `Layer walks`, middle arms | flat | **364 → 364 → 364** (`stress` 459 → 459 → 459) |
| byte-identical outside `=== Timing ===` | **no**: one CR 616.1 prompt where a two-member batch under two Furnaces had two, and none where the suppression removes one, so the random agent's draws shift in those games | differs — **3 of 200** games between `main` and `loop`, **4 of 200** between `loop` and `suppress`, and every one of them has two Furnaces of Rath on the battlefield at the divergence (200-game event dumps, ids masked; the one other "difference" was a fizzle line printing an unmasked id) |
| what a changed answer moves | the small rows, inside the spread | Memo hits 56,988 → 56,971 → 56,994; Total damage 56.7 → 56.4 → 56.0 |
| CPU/game median | flat | 12.65 → 12.55 (−0.8%) → 12.53 (−0.9%) → 12.92 ms (+2.1%, the pool); ms / 1,000 walks −0.8%, −0.9%, +1.0% |

The shipped arm is the pool change and reads as one — a {W} instant in every
white deck, avg turns 29.7 → 30.1, one 95-turn game lifting p99. Zero errors
and zero panics on every arm and pool; `deterministic: yes` on all four.

**Reachability.** `Prevention allocations` — the row this section named — is
**0.00** per game on `performance` and **0.02** on `stress`, unforced. Forced:
Mending Hands in every performance deck is cast 229 / resolved 229 in 135 of
200 games (68%), 1.60 copies per deck, with **0.01** allocations per game; the
three unpooled cards in every stress deck give **0.05** per game — Samite
Healer cast 197 / resolved 194 in 126 games (63%), Safe Passage 208 / 191 in
131 (66%), Samite Censer-Bearer 194 / 191 in 136 (68%). So CR 615.7's choice
is reachable from the pool and rare: it needs the shielded player to be the
one two creatures attack in the same step, while the random agent aims "any
target" with no preference for itself. That is the honest number; RD-3's
Circle of Protection and RD-4's Pariah put a count on the player being
attacked, which is where it will move.

**Determinism.** `tests/determinism_test.rs` green; each arm's three timing
rounds identical to its threaded run; three shell `fuzz_games` runs at one
seed identical line for line outside `=== Timing ===`.

#### RD-3 — sources — ✅ landed 2026-09-09

*Evicted 2026-09-11 from `plans/replacement-architecture.md`, where the heading and a stub remain.*


**Builds:** `EventPattern::DealDamage { source: Option<SourcePattern>, combat:
Option<bool> }`, where `SourcePattern { object: Option<ObjectId>, filter:
Option<ObjectFilter> }` is CR 609.7a's chosen object and CR 609.7b's rechecked
property in one field — Circle of Protection: Red is `object: Some(chosen),
filter: Some(ByColor(Red))`, Guardian Seraph is `filter: Some(ByController(
Opponent))`, Fog is `combat: Some(true)`. The chosen source is a
`SelectionFilter::DamageSource` the resolution asks for through one
`ChoiceKind`, enumerated by `enumerate_legal_selections` as permanents and
spells on the stack — CR 609.7a's other categories (objects a stack object
refers to, objects a waiting replacement or delayed trigger refers to,
face-up command-zone objects) are unreachable until item 6 and Commander
designation, so `ATOM-609.7a-001` is `COVERS-PARTIAL` and says which
categories. "A source doesn't need to be capable of dealing damage" is free:
the enumeration does not ask. **Consumers:**

- **Circle of Protection: Red** — "{1}: The next time a red source of your
  choice would deal damage to you this turn, prevent that damage." Rulings:
  *source categories* → the partial above; *can be used even when there is no
  damage to prevent; it prevents the next damage (if any) this turn* → test:
  the row sits unused and expires at cleanup. Plus 615.8 (`Uses::Once`, whole
  event regardless of amount, `ATOM-615.8-001`) and 615.9/609.7b: the chosen
  creature loses red before it deals damage, the shield does not apply and is
  not spent (`ATOM-615.9-001`, `ATOM-609.7b-001`).
- **Reverse Damage** — "The next time a source of your choice would deal
  damage to you this turn, prevent that damage. You gain life equal to the
  damage prevented this way." The rider that reads the amount. Ruling: *only
  the first instance from that source; a second is not reversed* → test.
- **Guardian Seraph** — "If a source an opponent controls would deal damage to
  you, prevent 1 of that damage." CR 615.10's static partial with a source-side
  controller predicate. Rulings, three: *1 from each source each time* → test
  with two simultaneous sources, both reduced (`ATOM-615.10-001`'s shape on a
  player); *a source without a controller* → not expressible (cycling; and
  609.7c's non-battlefield source is covered instead by Lightning Bolt, a
  spell on the stack, `ATOM-609.7c-001`); *multiple Seraphs are cumulative* →
  test, two instances each once.
- **Daunting Defender** with **Pyroclasm** — CR 615.10's own example, verbatim
  (`ATOM-615.10-001`): each Cleric takes 1, the non-Cleric 2. Pyroclasm is
  `FilteredPermanents(ByType(Creature))`, already a recipient; it is the pool's
  first "each creature" damage and a board wipe the SBA batch has not seen.
- **Fog** — "Prevent all combat damage that would be dealt this turn." The
  `combat` flag, `Filter { All }` + `Everyone`, `Uses::Static`. Non-combat
  damage under Fog goes through.
- **Torbran, Thane of Red Fell** — "If a red source you control would deal
  damage to an opponent or a permanent an opponent controls, it deals that
  much damage plus 2 instead." `Amount(Plus(2))` with a source-side `ByColor(
  Red) ∧ ByController(You)` — the arm's consumer, which is why `Plus` lands
  here and not in RD-1. Rulings, three: *dealt by the same source* → test on
  `DamageDealt.source`; *if all of the damage is prevented, Torbran's effect
  no longer applies* → test: a shield empties the event first and
  `never_happens` drops it before Torbran is gathered (CR 614.7a re-asked per
  iteration); *divide before adding 2* → the trample test.
- **Dark Sphere** — "{T}, Sacrifice this artifact: The next time a source of
  your choice would deal damage to you this turn, prevent half that damage,
  rounded down." A resolution-created `PreventHalf(Down)` with a chosen
  source and `Uses::Once` — the prevention half of decision 1's rounding from
  a registry row rather than a static. Ruling: *two of these apply
  sequentially: 5 becomes 3 becomes 2* → test, two rows, each once.
- **Sokrates, Athenian Teacher** is recorded as a shape and not registered:
  its granted "If this creature would deal combat damage to a player, prevent
  that damage. This creature's controller and that player each draw half that
  many cards, rounded down" is a Layer 6 grant of a `Prevent` whose pattern
  names its own host as the source (`SourcePattern`'s `Self`), with a rider
  reading `Half(ReplacedAmount, Down)` to two players — every piece of which
  RD-1 and RD-3 build — but "hexproof as long as it's untapped" is RS-2's,
  and a registered card with a dead ability under a real name is what
  `engineering-practices.md` §3 forbids. Its ruling — unpreventable damage
  still draws — is decision 4's dovetail from the other side.

**`PERFORMANCE_POOL` +1, Guardian Seraph**, predicted: a static prevention with
a two-sided predicate is the first source the sweep evaluates a filter on per
damage event; Circle of Protection's activation competes for mana the random
agent rarely has, so it is registered and its `--require` count read.

**Atoms:** `ATOM-615.8-001`, `ATOM-615.9-001`, `ATOM-615.10-001`,
`ATOM-609.7b-001`, `ATOM-609.7c-001`; `ATOM-609.7a-001` and
`BOUNDARY-DEF-609.7a-001` as `COVERS-PARTIAL`.

##### As landed

Five code commits and the docs, +2,035 / −57 across 16 files — roughly 640
engine, 660 cards, 760 tests — over the ~1,400–1,600 prediction on every
axis. The cards are most of it: eight printings with a rulings pass each,
against a predicted "~400 cards". The engine came in at ~640 against ~370,
which is `SelectionFilter::DamageSource`'s three sites, `PatternFill` and the
two fixes below — none of them in the sizing, because the sizing counted
`pattern_watches` and the construction sites and stopped there. Every decision in §9's RD-3 section shipped as designed. What
follows is what the build decided, and the four the close owed.

**The four decisions this PR owed, each answered.**

1. **`SourcePattern` carries a general `Option<ObjectFilter>`, and the "two
   customers before a leaf" guard is what decided it.** The leaves the field
   actually reaches across all eight cards are `ByColor` (Circle of
   Protection: Red, Torbran), `ByController` (Guardian Seraph, Torbran) and
   `And` (Torbran) — **all three already in `ObjectFilter` with customers of
   their own**, so the general filter added *zero* new leaves where narrower
   per-card leaves would have added three. The guard also fired in the other
   direction, which is the more useful half: the one leaf a consumer wanted
   and did not get is "the effect's own host as the source" — Sokrates,
   Athenian Teacher's granted "if **this creature** would deal combat damage
   to a player" — with exactly one customer, recorded on `SourcePattern`'s
   own doc and in `phase_rd_cards.rs`'s module doc rather than written. §8c's
   axis 2 took its first real weight and grew by nothing.
2. **Two of CR 609.7a's four source categories stay unreachable, and each has
   a named blocker.** Reachable: *a permanent* and *a spell on the stack
   (including a permanent spell)*, both enumerated by
   `SelectionFilter::DamageSource`. Unreachable: (a) *any object referred to
   by an object on the stack, by a replacement or prevention effect that's
   waiting to apply, or by a delayed triggered ability that's waiting to
   trigger* — there is no referred-to relation to read. `StackEntry` carries
   `chosen_targets`, which is a *target* and not a reference (CR 115's word,
   and the atom's own example is an emblem naming a card in exile), a waiting
   replacement's `RegisteredReplacementEffect.targets` is the same thing, and
   CR 603.7's delayed triggered abilities do not exist until item 6. (b) *a
   face-up object in the command zone* — `GameState::command` is never
   populated; the Commander track fills it. `ATOM-609.7a-001` and
   `BOUNDARY-DEF-609.7a-001` are `COVERS-PARTIAL` naming exactly these two,
   and the boundary's out-of-set member — a card in hand referred to by
   nothing — is built whole.
3. **Guardian Seraph went into `PERFORMANCE_POOL` as predicted** (75 → 76),
   and the A/B says the prediction was right about *what* it measures and
   wrong about the size: `performance` layer walks 368 → 361 and replacement
   gathers 506 → 508. A source-side filter on every damage event is free at
   this board size, which is the honest reading of a per-permanent gate that
   only opens for `replacement_ability_sources`.
4. **Circle of Protection: Red's `--require` count contradicts the sentence
   that asked for it.** Forced into every stress deck alongside the other six,
   it is cast 181 / resolved 180 in 123 of 200 games; forced alone, 190 / 189
   in 130 of 200. And the activation — the thing "competes for mana the
   random agent rarely has" was about — happens **12,660 times across 200
   games, in 129 of them**, about 98 per game it reaches the battlefield. A
   `{1}` with no other use for the mana is something a random agent does
   until it runs out. CR 609.7a's chosen source is one of the best-exercised
   paths in Phase RD, not a rare one, and the prediction is struck rather
   than defended.

**Two things the cards found, shown failing against the pre-fix tree.**

- **`Primitive::DealDamage` never read `EffectRecipient::FilteredPermanents`,
  and looped `execute_action` besides.** Pyroclasm dealt no damage at all, and
  once it did it would have been one batch per creature. CR 704.3's
  simultaneity, CR 615.7's "two or more applicable sources at the same time"
  and CR 603.2c's "one or more" all read the batch, so a loop is unreachable
  from all three. The recipient is resolved inside the primitive rather than
  filled into `ctx.targets`, for `Primitive::CreateReplacement`'s reason: "every
  permanent matching this **now**" is a question only the primitive acting on
  them can ask without changing what the recipient means to a static ability.
- **`apply_rewrite`'s whole-event `Rewrite::Prevent` reported `prevented: 0`.**
  Reverse Damage's "life equal to the damage prevented this way" was zero.
  RD-1's Angel of Suffering rides on `ReplacedAmount` and RD-2's counts reach
  the number through an `Amount` arm, so this is the first def to ask a
  `Prevent` what it prevented. The arm reports it rather than the caller
  deriving it from a dropped event, because only the arm knows the event was
  damage — a `Prevent` on a destruction is regeneration and prevents nothing,
  the line `ReplacementDef::is_prevention` already draws.

**`Primitive::CreateReplacement` grew a third argument and nothing else did.**
`PatternFill { Authored, ChosenDamageSource }` — the handoff's design (a),
built as described: the card authors `SourcePattern { object: None, .. }`, the
resolution asks through `ChoiceKind::ChooseDamageSource` and overwrites, with
`Primitive::Restrict`'s `debug_assert` that the field was empty. Nothing on
`RegisteredReplacementEffect` changed, as the note predicted. Zero candidates
is CR 101.3 and the effect does nothing; one is forced and asks nobody.

**Item 47's condition (d), re-derived because this PR added the fields it names.**
`ordering_cannot_change_outcome`'s multiplier bucket is written against
`matches!(pattern, EventPattern::DealDamage { .. })`. Neither new field reads
the *amount*: `source` is about the object dealing the damage and `combat` is
CR 510.2's flag, so no member can fall out of applicability as another member
changes the number, and the suppression stays sound. Recorded in the
predicate's own doc and in `codebase-state.md` item 47.

**Measured** (`plans/fuzz_ab.py`, three arms against a same-day `main`
worktree at `e3b469d`, 200 games at seed 12345, both pools): `main`; `engine` —
this PR's engine with `registry.rs` and `PERFORMANCE_POOL` unchanged; `new` —
shipped.

| | prediction | measured |
|---|---|---|
| middle arm | "flat" | **byte-identical to `main` outside `=== Timing ===` on both pools** — not one counter moves |
| middle-arm CPU | flat | 14.10 → 14.78 ms median (+4.8%), which with identical counters is spread and nothing else (round 1 was 14.08 → 14.09) |
| shipped arm, `performance` | a move is the card | Layer walks 368 → 361, gathers 506 → 508, avg turns 30.1 → 30.3, CPU 14.10 → 14.15 ms (+0.4%) |
| shipped arm, `stress` | the card | Layer walks 437 → 496, gathers 513 → 563, memo hits 59,392 → 78,972 — eight cards in every deck |
| `Prevention allocations` | unchanged (nothing here carries a count) | 0.00 → 0.01 `performance`, 0.02 → 0.01 `stress`; deck-mix noise either way |

The middle arm being byte-identical is the strongest form §9's prediction
could take, and it says something worth keeping: **a new `EventPattern` field
costs nothing until a def uses it.** `pattern_watches` asks `source` and
`combat` only through an `Option`, and every pre-RD-3 def writes `None`.

Zero errors and zero panics on every arm and pool; `deterministic: yes` on all
three, and three shell runs at one seed identical line for line outside the
timing block.

**Reachability**, forced (`--require`, 200 `stress` games, the seven
comma-free names in one run and Torbran in its own because `--require` splits
on commas): Guardian Seraph 145 / 145 in 102 games, Circle of Protection: Red
181 / 180 in 123, Daunting Defender 144 / 143 in 106, Pyroclasm 169 / 149 in
111, Fog 188 / 163 in 118, Reverse Damage 185 / 169 in 119, Dark Sphere 221 /
221 in 144, Torbran 161 / 161 in 111. Every card in the PR is reachable from
a random game.

**No trace page.** §7's rule is "a phase that changes *how* a read is answered
rather than what the answer is", and RD-3 changes what is asked, not how: the
pipeline's shape is RD-2's, and the byte-identical middle arm is the direct
evidence that no existing read moved.

#### RD-4 — redirection and unpreventable damage — ✅ landed 2026-09-09

*Evicted 2026-09-11 from `plans/replacement-architecture.md`, where the heading and a stub remain.*


**Builds:** `Rewrite::Retarget(RetargetSpec { ToSource, ToHost,
ToSourceController, ToFixed(DamageTarget) })`, rewriting the proposal's
`target` and nothing else — `source`, `is_combat` and `unpreventable` travel
with the damage (Pariah's ruling: redirected combat damage is still combat
damage; Kor Chant's: it is dealt by the original source). CR 614.9's re-check
happens at application: a destination no longer on the battlefield, or no
longer a creature, planeswalker or battle, or a player who has left, makes the
rewrite return the event unchanged with nothing spent (decision 7). Whole-event
only — finding 23 has the partial case. And decision 6's flag and consult.
**Consumers:**

- **Pariah** — "Enchant creature. All damage that would be dealt to you is
  dealt to enchanted creature instead." `Retarget(ToHost)`, `Fixed(vec![])` +
  `You`, an Aura (Holy Strength opened the Aura path in LH-1). Rulings:
  *combat damage stays combat damage* → test on `is_combat`; *two Pariahs on
  two creatures: you choose which applies and cannot divide* → test: CR 616.1
  asks, the whole 6 lands on one host. Plus the host leaving: damage to you
  (`ATOM-614.9-001`'s destination half; its "shield not used up" half needs a
  `Once` redirect, which is Reflect Damage's shape — a player destination and
  a chosen source, registered here if RD-3's source choice is in, else recorded
  as the atom's remaining half).
- **Palisade Giant** — "All damage that would be dealt to you and other
  permanents you control is dealt to this creature instead." `Retarget(
  ToSource)`, `Filter { ByController(You) ∧ EachOther }` + `You`. The same two
  rulings, and the first consumer of an object filter and a player set on one
  row.
- **Pinpoint Avalanche** — "Pinpoint Avalanche deals 4 damage to target
  creature. The damage can't be prevented." The per-event flag, no other half.
  Tests: a shield counter's prevention is applied, prevents nothing, its rider
  removes the counter anyway, and a `NextDamage(4)` row is untouched
  (`ATOM-615.12-001`, `-002`, `COMP-615-UNPREVENTABLE-SHIELD-001`); the
  instance is offered once (`ATOM-615.12a-001`). Combust is the same shape
  behind "this spell can't be countered", which is §8a's missing counter event,
  so it is not registered.
- **The two restriction routes have no registrable consumer yet, and the PR
  says so.** Every "damage can't be prevented [this turn]" card carries a half
  the engine lacks: Skullcrack and Leyline of Punishment "players can't gain
  life" (RE's `GainLife` pattern arm), Unstable Footing kicker, Stomp an
  adventure, Flaring Pain flashback, Wild Slash a `Conditional`, Everlasting
  Torment wither. The consult at application is built and tested against both
  shapes as fixtures — a `Primitive::Restrict(ApplyReplacement { Prevention })`
  row for the turn-scoped form and a static `Effect::Restriction` on a fixture
  permanent for Leyline's — and Skullcrack and Leyline are named as the cards
  that land each in a game, after RE.
- **The dovetail test** (decision 4): under the fixture restriction, Lightning
  Bolt to the face with Angel of Suffering and Reverse Damage both watching —
  3 damage dealt, both riders run, 6 cards milled (`ReplacedAmount`), 0 life
  gained (`DamagePrevented`), no row spent. The same board against Pinpoint
  Avalanche on a creature with a shield counter: damage dealt, counter removed
  (Disciplined Duelist's ruling, verbatim).

**`PERFORMANCE_POOL` +1, Pariah**, predicted: the first `Retarget` and the
first `AffectedSet::Host` read on a damage event.

**Atoms:** `ATOM-614.9-001`, `ATOM-615.12-001`, `ATOM-615.12-002`,
`ATOM-615.12a-001`, `COMP-615-UNPREVENTABLE-SHIELD-001`.

**On filing.** Every atom above is in Phase 6 except `ATOM-120.3c-001` (Phase
8) and `ATOM-615.6-001` (Phase 7), and none is in `Backlog`, so nothing is
re-filed and `owed` — 9 today, none of them a replacement phase's — cannot move
by construction. What gated RC was the `// COVERS:` discipline, not the number
(`engineering-practices.md` §5.1), and that is RD's gate too.

##### As landed

Six code commits and the docs, against two notes RD-3's build left for this
one, neither of them contradicted. Every decision in §9's RD-4 section shipped
as designed except `RetargetSpec`'s arm list, below. The two features stayed as
independent as the sizing said: they share `is_unpreventable`'s caller and
nothing else.

**Three re-derived counts, because the sizing counted a tree RD-3 then rewrote.**

- `Primitive::DealDamage` is **17** sites, not 16 — 16 constructions plus the
  performer arm, which RD-3 had just moved to `FilteredPermanents` and one
  batch. The struct-variant change landed on that fresh code without touching
  it.
- `GameAction::DealDamage` is **44** sites, not 25. The prediction counted
  `src/`; the tests construct the event too, and RD-1 through RD-3 added 28 of
  them.
- `is_prohibited` callers **+1**, exactly as predicted — and it is the only
  thing the middle arm measures.

**`RetargetSpec` ships three arms, not four, and two are renamed.** §9 wrote
`{ ToSource, ToHost, ToSourceController, ToFixed(DamageTarget) }`.

- `ToSource` → **`ToEffectSource`** and `ToSourceController` →
  **`ToDamageSourceController`**. The two arms name two different objects and
  both contain the word "source": CR 609.7's is the source of the *damage*, and
  a `ReplacementInstance`'s `source` is the object whose ability the effect is.
  Palisade Giant's "this creature" is the second, Reflect Damage's "that
  source's controller" the first, and the adjacent pair as written read as one
  question with a `.controller` on the end. `naming-use-the-cr-vocabulary`.
- **`ToFixed(DamageTarget)` is not built.** No card can author a damage target
  it has not chosen yet, so the only thing that could fill it is
  `RegisteredReplacementEffect.targets` — `codebase-state.md` item 90's work,
  which arrives with Divine Deflection and needs `AmountExpr::Variable` besides.
  An arm with no possible customer is what `AmountRewrite`'s own rule forbids,
  and this is that rule applied to the arm that would have broken it.

**The four decisions this PR owed, each answered.**

1. **Reflect Damage is registered**, and §9's condition is met: RD-3 landed
   `SelectionFilter::DamageSource` and `PatternFill::ChosenDamageSource`, which
   is the whole of what the card needed. It closes `ATOM-614.9-001`'s "the
   shield is NOT used up" half, which Pariah structurally could not — Pariah is
   `Uses::Static`, so "not used up" is vacuous for it. Reflect Damage is the
   phase's only `Uses::Once` redirect and its only redirect to a *player*,
   which is also CR 614.9's last sentence made testable.
2. **Pariah went into `PERFORMANCE_POOL` as predicted** (76 → 77). The A/B's
   middle arm — the whole engine with `registry.rs` and `PERFORMANCE_POOL`
   unchanged — moved by exactly one thing on each pool and by nothing else:
   restriction queries 565 → 566 on `stress` (536 flat on `performance`) and
   memo hits +7 / +1. That is CR 615.12's consult, one `is_prohibited` per
   *prevention application*, with a few of those queries taking a real sweep on
   a stress board that has static restriction sources. RD-3's middle arm was
   byte-identical; this one is not, and the difference is a named branch rather
   than spread. Shipped-arm timing is +0.1% CPU/game and −0.5% ms/1,000 walks,
   medians of three interleaved rounds — both inside the ~2–6% spread and in
   opposite directions.
3. **CR 614.9's re-check reused the per-member *shape* and needed no new
   mechanism**, which is what RD-3's note 2 predicted. It is a free function
   beside `apply_rewrite`'s `Retarget` arm, asked per member at application,
   reporting through `Applied` — and deliberately **not** routed through
   `applies_to`, so a redirect whose destination is gone is still gathered,
   still offered to CR 616.1 and still chosen, and then does nothing. It is an
   existence-and-type check and not `validate_selection`: a redirect is not
   targeting (CR 115.1), so a hexproof creature is a perfectly good
   destination.
4. **RD-5's gate: closed, and Harm's Way goes to `backlog.md` §2.25.** Below.

**One thing the cards found, shown failing against the pre-fix tree.**

**An `AffectedSet::Filter` containing `ObjectFilter::EachOther` matched
nothing.** Palisade Giant's "other permanents you control" is the first card in
the crate whose affected set needs that leaf, and `set_affects` refused it: the
filter reached `object_matches_filter`, which takes `you` and no source id,
answered `Err`, and `set_affects` collapsed that to `false`. So the Giant
redirected the damage aimed at *you* — a player subject never reaches the object
filter — and none of the damage aimed at your other permanents. A card silently
doing half of what it says.

`codebase-state.md` item 103 had measured that `Err` path at **zero** across 600
fuzz games and left it, on the argument that a mid-game panic is worse than a
card doing nothing. That argument was about the *other* two causes, which are
card-authoring errors. This one is a leaf the type offers and the layer walk has
always answered. The fix hands `set_affects`'s existing `source` down to the
filter through one new entry point; `object_matches_filter` and
`object_matches_filter_in_frame` keep their signatures and keep refusing the
leaf, which is right for a *selection*.

**Two things the build decided that §9 did not ask about.**

- **The group's subject stopped standing in for its members'.** RD-2 captured
  one `EventSubject` per group because nothing could move it; CR 614.9 can. So
  the CR 616.1 chooser, the prompt's object and
  `Instead(RemoveCountersFromAffected)` now read the *event* rather than the
  group key. Without it, a redirect onto a creature carrying a shield counter is
  a hard error mid-batch — the counter's rewrite asks the group key for an
  object and gets a player. `§11` item 35.
- **CR 615.12's two application sites both use one predicate**,
  `is_unpreventable`, which is where the per-event flag and
  `Restriction::ApplyReplacement`'s two sources meet. RD-3's note 1 asked for
  exactly that and it needed no more.

#### RE-1 — skips, and the turn queue (CR 614.1b, 614.10, 614.10a, 500.7, 500.11) — ✅ landed 2026-09-11

*Evicted 2026-09-11 from `plans/replacement-architecture.md`, where the heading and a stub remain.*


**Builds:** decision 6 — the three variants, three arms, three performers
emitting the three begin events, `advance_turn` as a queue drainer with the
proceed-past, turn-number and duration rules, `Primitive::ExtraTurn` pushing
CR 500.7's most-recent-first, and 800.4k's refusal at the turn site (a rule,
ahead of the pipeline, reading `player_lost` — which RE-6 later makes true for
a reason). **Consumers:**

- **Yawgmoth's Bargain** — "Skip your draw step. Pay 1 life: Draw a card."
  `BeginStep { step: Some(Draw) }`, `Fixed(vec![])` + `You`, `Prevent`,
  `Uses::Static`; and an activated ability with `Cost::PayLife(1)` and
  `DrawCards(1)`, both of which exist. No rulings. Tests: the draw step's
  *contents* are not proposed — no `CardDrawn`, no `StepBegin { Draw }`,
  priority goes straight to the precombat main (`ATOM-614.10-001`). A random
  agent with one use for its life total will empty its library, which is what
  makes RE-6's Laboratory Maniac path reachable and is why this card is
  registered and **not pooled**.
- **Eon Hub** — "Players skip their upkeep steps." `BeginStep { step:
  Some(Upkeep) }`, `Everyone`, `Prevent`, `Uses::Static`. Rulings, three, all
  tests: *skipped entirely, untap to draw* → the event log; *"activate only
  during your upkeep" can't be activated* → no such ability exists to assert
  against, recorded; *untap-step triggers go on the stack at the draw step* →
  item 6's. The four-player form: every player's upkeep, one static, no
  prompt.
- **Meditate** — "Draw four cards. You skip your next turn." `DrawCards(4)`
  then `CreateReplacement` of a `BeginTurn` row, `You`, `Uses::Once`,
  `Duration::Indefinite` — a row that ends by use and never by time, the
  first of its kind, and `Duration::Indefinite` has waited for it. Ruling:
  *you skip one turn* → test; and 614.10a's own sentence: **two Meditates
  skip two turns** (`ATOM-614.10a-001`), the second row surviving the first
  proposal. The "until your next turn" test rides on it: a Cerulean Wisps-class
  effect on your creature lasts across the skipped turn to the one that
  begins.
- **Time Walk** — "Take an extra turn after this one." `Primitive::ExtraTurn`,
  the queue's producer. Ruling: *multiple extra-turn effects in one turn are
  taken in reverse order* → two Time Walks, the second resolved is the first
  taken (CR 500.7's "most recently created turn will be taken first"). And
  the board that puts skips and the queue in one PR: **Meditate then Time
  Walk** — the extra turn is the "next occurrence" the skip consumes, and the
  natural turn after it begins. The two `until_your_next_turn … extra_turn`
  tests already in `continuous_effects.rs` and `duration_registry.rs`, which
  simulate an extra turn by calling `begin_turn` twice, are rewritten against
  the queue so they prove the engine rather than the harness.
- **Moment of Silence** — "Target player skips their next combat phase this
  turn." `BeginPhase { phase: Some(Combat) }`, `Fixed(vec![])` filled with the
  target at resolution (Mending Hands' player fill), `Uses::Once`,
  `UntilEndOfTurn`. Rulings, three, all tests: *only their next combat
  phase, if any* → one row, spent once; *cast during combat: no effect* →
  `ATOM-614.10-002`, the row meets no proposal and expires at cleanup; *cast
  on a player when it is not their turn: no effect* → the subject is the
  active player, so a row on another player watches nothing this turn. The
  first targeted skip, and a four-player target.
- **Chronatog** is the natural activated skip and is out: "activate only once
  each turn" is an activation limit the ability model does not have, one
  customer here, recorded on the card file's module doc. **Relentless
  Assault** is the extra-phase shape and waits for the queue's second level.

**`PERFORMANCE_POOL` +1, Eon Hub**, predicted: a static skip on every
player's upkeep, every turn, in every game it reaches the battlefield — the
first card whose effect is a *dropped* turn-structure proposal in a measured
game. Time Walk is registered and not pooled: an extra turn in every blue deck
moves avg turns by design. The middle arm's own move is decision 6's
proposals, predicted below.

**Atoms:** `ATOM-614.10-001`, `ATOM-614.10-002`, `ATOM-614.10a-001`,
`BOUNDARY-DEF-614.1b-001`; `ATOM-614.10b-001` stays uncovered, decision 6's
zero-card reason in the test file. CR 500.7 has no atom in the corpus
(`backlog.md` §2.17 said it was thin); the two-Time-Walk test carries none and
says so. `ATOM-502.3-002` and `ATOM-703.4c-002` ("doesn't untap") are
`backlog.md` §2.14's and are not skips; this PR touches neither.

---

## As landed (2026-09-11)

Everything above shipped as sized, with two additions the sizing did not
predict and one condition it set that was not met.

**`turn_rotation`.** The section said the queue holds "the player only or the
`(player, turn)` pair", and the answer is the player — a stored turn number
means "the number this extra turn will have", which stops being true the first
time a skip sits between the queue entry and the turn. What the section did not
ask is how the *natural* rotation survives an extra turn, and
`(active_player + 1) % n` does not: CR 500.7 inserts an extra turn after a
turn, so the rotation has to resume from the player whose natural turn it was.
With two players and "you take an extra turn" that is the same answer by
accident; with four and Final Fortune at instant speed on someone else's turn
it is not. `GameState.turn_rotation` is the second field, advanced when a
natural turn is **proposed** rather than when one begins, because CR 614.10a
proceeds past a skipped turn rather than re-offering it. → §11 item 47.

**The first turn had never begun.** `GameState::new` parked the board on turn
1's untap step and nothing called `on_step_begin` for it, so turn 1's untap
sweep and land-drop reset had never run in any game the engine has played. The
section asked for "the first turn's untap step is proposed like any other" as a
*rule*; the tree made it a bug fix. `Game::setup` now calls `start_first_turn`,
which proposes the turn, its beginning phase and its untap step through the
chokepoint. The turn itself is unskippable there by construction rather than by
exemption — CR 614.4 needs the effect to exist before the event, and before the
first turn nothing has resolved. → §11 item 48.

**The schedule is consumed at the proposal site, and that is not a chokepoint
violation.** Popping `turn_queue` and advancing `turn_rotation` happen in
`advance_turn`, outside `perform_action`. Neither is state a CR 614 replacement
effect or a CR 603 trigger can see — they are what *builds* the proposal — and
both have to be spent whether or not the turn begins: a skipped extra turn is
gone (CR 614.10a), and the turn after a skipped P2 is P3's rather than P2's
again. Doing either in the performer gives the wrong answer in the one board
the phase exists for, Meditate against Time Walk.

**CR 508.8 stopped being a priority suppressor.** It lived in `Game::run_turn`
as "this step happens and grants nobody priority", which is not what the rule
says. At the proposal site the step does not happen, and the three
`attacks_declared` guards in `process_turn_based_actions` became a second
reading of one rule and are gone. The visible consequence is that a turn with
no attackers is **ten positions rather than thirteen**, which re-counted six
test loops that had been counting positions by hand.

**"Until your next turn" moved to the turn's begin hook** from the untap
step's. CR 611.2b says the turn, and the difference is now reachable in two
directions: eight printed cards skip the untap step, and CR 614.10a's "a
skipped turn expires nothing" is the same sentence from the other side.

**A fixture that hand-writes `active_player` now writes `turn_rotation` too**
(`test_support::set_active_player`). Three existing fixtures crossed a turn
boundary after moving the active player and got that player's turn twice.

## Measured (2026-09-11)

Three arms through `plans/fuzz_ab.py` against a same-day `main` worktree,
200 games seed 12345; the numbers and the §3 table are in
`engineering-practices.md` §3. The prediction was written before any arm ran.

| | predicted | measured |
|---|---|---|
| `Replacement gathers`, `performance` | +~16/turn, ~+450/game | **+14.8/turn, +447/game** (507 → 954) |
| `Restriction queries` | not predicted | +448/game (509 → 957) — one `is_prohibited` per proposal |
| `Layer walks`, `Board walks`, `Layer frames`, `Dependency checks` | flat | **flat** (363, 241, 4,262, 36 — unchanged) |
| `Memo hits` | flat | **58,262 → 59,191, +1.6%** — the prediction's one miss |
| every gameplay row | identical to `main` | **identical** |
| CPU/game | inside the spread | **+5.0%** (12.92 → 13.57 ms, five rounds, cleanly separated) |

The `Memo hits` miss is worth keeping: `gather`'s fast path stops the *walk*,
not the *query*, and `is_prohibited` asks one per proposal that the memo
answers. A flat `Layer walks` beside a moved `Memo hits` is what the gate
holding looks like.

### §8's event-kind gate: measured, and not built — the decision, with its number

§9's RE-1 bullet said "if CPU moves beyond [the spread], the fix is §8's
answer-preserving event-kind gate and it is **this PR's to build**". +5.0% is
inside the 2–6% spread `fuzz_ab` states, but the rounds separate cleanly, so
the number is real rather than noise and the condition deserved an answer
rather than a reading of the word "inside".

**A fourth arm settled it.** A hard-coded event-kind gate in `gather` — return
empty for the three begin actions, which is exactly what the maintained bitmask
would compute on a pool with no card watching one — measured **13.24 ms,
+2.5%**. So the sweep is **half** the cost and the chokepoint is the other
half: the batch open/close, the APNAP sort, `is_prohibited`, the grouping, the
frame and the emitted event. No gate can remove that half, and it is what the
three begin events *are* — 2,656 triggers read them.

**Not built, and the trigger named instead.** Buying 2.5 points costs a
per-`GameState` bitmask recomputed on every registry, battlefield and counter
change — and `any_replacement_counter`'s own doc already refused a maintained
set on exactly that ground: "counters have more than two chokepoints … a set
maintained at a chokepoint that does not exist is exactly the drift, and it
reads as a card that silently does nothing". A silently missing replacement
effect is the worst failure shape this engine has, and §8's ordering says do
not pre-optimize. **RE-9 is the PR that reads this number**: a proposal on
every land tap is several per turn on top of RE-1's sixteen, its own bullet
already calls it "the one whose A/B could say no", and 2.5 points is what the
gate would return it. The attribution arm is reproducible — the probe is four
lines in `gather` above `record_replacement_gather`.

### The two atoms that stayed open, and why

`ATOM-614.10b-001` — "skip …, then take another action" — has **no card**.
The corpus's own audit note says `o:/skip.*then/` is empty and RE's census
re-confirmed it at zero, so there is nothing to write the follow-up action onto
and a fixture wearing the rule would assert the engine's guess. Recorded in
`tests/phase_re1_integration_test.rs`'s module doc.

CR 500.7 has **no atom in the corpus** (`backlog.md` §2.17 said it was thin;
the sizing confirmed it), so `two_extra_turns_are_taken_most_recently_created_first`
claims none and says so.

`ATOM-614.10a-001` is `COVERS-PARTIAL`. Its board is two "skip your next draw
step" effects, and **every printed draw-step skip is a static** — Yawgmoth's
Bargain, Necropotence — so `Uses::Once`'s "one effect will be satisfied in
skipping the first occurrence, while the other will remain" cannot be built on
the draw step from the printed pool at all. Two Meditates are the same sentence
on the unit that does print consumably.

---

#### RE-2 — draw (CR 614.11, 614.11a, 121.2, 121.2a, 121.6a/b, 616.1g) — ✅ landed 2026-09-11

*Evicted 2026-09-11 from `plans/replacement-architecture.md`, where the heading and a stub remain.*

**Builds:** decision 1 whole — `DrawCards { player, n, cause }`, `DrawCard {
player, cause }`, `DrawCause`, `EventPattern::DrawCards { at_least }` and
`DrawCard { cause }`, `GameActionTemplate::DrawCards { n, player }`, the outer
performer's one-at-a-time decomposition handing the inherited applied set to
each inner (the first producer of `apply_replacements`' `inherited`), and the
two producers rewritten: `Primitive::DrawCards` proposes one outer; the draw
step proposes `DrawCards { n: 1, cause: TurnBased }`. `Game::setup`'s opening
hands keep calling `draw_card` (CR 103.4; the comment there is right — no
replacement can exist yet). **Consumers**, each with its rulings pass:

- **Thought Reflection** — "If you would draw a card, draw two cards instead."
  `EventPattern::DrawCard { cause: None }`, `Fixed(vec![])` + `You`,
  `Instead(DrawCards { n: 2, player: None })`, `Uses::Static`. Rulings, three:
  *Harmonize draws six* → test (`ATOM-121.2a-001`'s shape from the inner side);
  *two Thought Reflections draw four, three draw eight* → **the acid test**,
  `test_two_teferis_draw_four_not_infinity` on two of these, since it is not
  legendary and the pool can build two (§10; §3.2d); *the drawing player orders
  them* → the 4-player form with Alms Collector below.
- **Teferi's Ageless Insight** — "… except the first one you draw in each of
  your draw steps, draw two cards instead." `DrawCard { cause: Some(Effect) }`.
  Rulings, three: *a card put into hand without "draw" is not drawn* →
  structurally true (`ZoneChangeCause::PutIntoHand` proposes no draw), asserted;
  *ordering* → as above; *two copies draw four* → the legend rule makes this
  Thought Reflection's test. Its own test is the one decision 1 wrote:
  Teferi beside Thought Reflection during the draw step draws **three**.
- **Alms Collector** — "Flash. If an opponent would draw two or more cards,
  instead you and that player each draw a card." `EventPattern::DrawCards {
  at_least: Some(2) }`, `Fixed(vec![])` + `Opponents`. **Sized here as `Prevent`
  with a rider of two `DrawCards(1)`, and that is wrong** — §11 item 53: the
  affected player's half has to be the rewrite (`Instead(DrawCards { n: 1 })`)
  or CR 614.5 does not cover it and the card loops against an opponent's Thought
  Reflection. One rider, the controller's draw. Rulings, six, and
  four are tests: *applies to the instruction before any per-card effect* →
  `ATOM-616.1g-001`, with Thought Reflection on the other side; *Thought
  Reflection can double the resulting draws without Alms applying again* →
  test; *count the word "draw"* → Ancestral Recall (in the pool) meets it,
  two `Primitive::DrawCards(1)` in one resolution do not; *two players each
  control one and a third player would draw two or more: the third chooses
  which applies* → the 4-player test, `setup_game(4)`, and the only
  three-player CR 616.1 prompt reachable from two printed cards.
- **Notion Thief** — "Flash. If an opponent would draw a card except the
  first one they draw in each of their draw steps, instead that player skips
  that draw and you draw a card." `DrawCard { cause: Some(Effect) }`,
  `Opponents`, `Instead(DrawCards { n: 1, player: Some(You) })` — decision 1's
  correction. Rulings, three, all tests: *the opponent still discards* →
  Night's Whisper is not the shape (draw-then-lose-life), so a fixture
  draw-then-discard resolution is until RE-8's Mind Rot; *two Thieves: the
  drawing player picks one, then that Thief's controller picks among the
  rest, each once* → the three-player board, and the two-player one where "it
  really will be that player who draws"; both are the lineage rule observed
  from outside.

**`PERFORMANCE_POOL` +1, Thought Reflection**, predicted: the first static
draw source in the pool, so it opens the gather sweep on every draw step while
it is on the battlefield — and the first `Instead` whose output is decomposed.
It is seven mana, so the `--require` count beside `copies/deck` is read and
recorded.

**Atoms:** `ATOM-121.2a-001`, `ATOM-614.11-001`, `ATOM-121.6a-001`,
`ATOM-614.11a-001`, `ATOM-616.1g-001`, `BOUNDARY-DEF-614.1a-001` (uncovered
since RB and claimable by any `Instead` consumer — this one takes it);
`ATOM-614.11b-001` stays uncovered, decision 1's reason in the test file.
`ATOM-121.2-001` (ALREADY-IMPL) gains a `COVERS:` from the decomposition test.

**The three the section left open, decided 2026-09-11 before a line of code.**

1. **`DrawCause` rides on both variants, and a substituted outer keeps the cause
   it replaced.** The outer needs the field because the outer's performer is
   what stamps its inners, and the stamp is `if i == 0 { outer.cause } else {
   Effect }` — which is decision 1's "first inner `TurnBased`, every later one
   `Effect`" when the outer is `TurnBased` and "all `Effect`" when it is not.
   CR 614.6 makes a substituted event the *same* event in modified form, so
   Thought Reflection's `DrawCards { n: 2 }` inherits the draw step's
   `TurnBased` — and that is the whole of "the first one you draw in each of
   your draw steps": the recursion answers it, and no counter of cards-drawn-
   this-step is needed. Teferi beside Thought Reflection in the draw step draws
   three because the outer's first inner is the excepted card and its second is
   `Effect`; Teferi's own doubling is then an `Effect` outer whose *first* inner
   is `Effect` too, so it stops at three rather than running to four. **A rider's
   draw is `Effect`** — §4.1a gives a rider a fresh lineage, and Alms Collector's
   two draws are new instructions rather than the draw step's turn-based action,
   so the affected player's Teferi doubles them, which is right: they are not the
   first card that player drew. `Primitive::DrawCards` is `Effect` at every call
   site; the draw step is the one `TurnBased` producer in the engine (CR 121.1).
2. **The stamp lives in the outer's performer, off the loop index.** A field
   threaded through the decomposition would have to be set by whoever built the
   outer *and* by every rewrite that produces one, which is a field a card can
   forget — the argument `ReplacementClass::from_rewrite` and
   `ReplacementDef::is_prevention` already make. The index is already in hand
   where the decomposition happens and nowhere else needs it.
3. **An outer `DrawCards { n: 0 }` reaches the loop.** `never_happens` is
   CR 614.7a, and the two rules filed under it — 120.8 and 119.10 — each say in
   so many words that the event does not occur. CR 121.2 says only that the
   player "performs that many individual card draws", which is zero of them, and
   no rule says the *instruction* is not an event. Alms Collector's
   `at_least: Some(2)` does not match it and nothing prints `at_least: None`, so
   reaching the loop costs one gather and answers nothing wrongly, while dropping
   it upstream would be a rule the CR does not have. The performer's loop runs
   zero times, which is the no-op with no guard — `LoseLife`'s 0 is the same
   shape and the same reason it is a local convenience in `perform_action` rather
   than CR 614.7a.

**Re-counted against the tree (2026-09-11, after RE-1 landed), and three of the
row's numbers were wrong.** The three exhaustive matches are still three
(`subject_of`, `event_amount`, `perform_action`) and `pattern_watches` still
falls through to `false` at one `_` arm; `DrawCard`'s producers are two
(`resolve.rs`'s `Primitive::DrawCards` loop, `turns.rs`'s draw step) and
`Primitive::DrawCards` is one. What moved:

- **Test constructions: 1 → 0.** Nothing in `mtgsim/tests` or any `#[cfg(test)]`
  module constructs a `GameAction::DrawCard`; every draw test goes through
  `Primitive::DrawCards` and asserts a hand size. The `cause` field costs the
  test suite nothing.
- **`inherited`'s "1 call site" is the parameter, not the plumbing.** Feeding it
  is four signatures: `apply_replacements` has to *return* the group's applied
  set, `execute_batch_inner` has to carry it per member into phase 2,
  `perform_action` has to take it, and a third entry point beside
  `execute_actions` / `execute_actions_new_batch` has to hand it back down.
  §4.2's note that "a third caller needs the same argument made again" is
  answered here on the other axis: the lineage variant joins the enclosing batch
  exactly as `execute_actions` does, and differs only in what it seeds the
  applied set with.
- **`GameState::draw_cards` has no callers and is a third producer waiting to
  happen** (§11 item 50). Deleted by this PR.
- **`Game::setup`'s comment is wrong and RE-2 makes it wronger** (§11 item 51).
  The behaviour is right — CR 103.4's opening hands cannot meet a replacement —
  but the comment claims a route the code does not take.

#### RE-3 — life (CR 119.10, 119.7's "can't gain", the CR 120.3a loss as a replaceable event) — ✅ landed 2026-09-12

*Evicted 2026-09-12 from `plans/replacement-architecture.md`, where the heading and a stub remain.*

**Builds:** decision 2 — the two pattern arms, `AmountRewrite::LifeFloor`,
the two life templates with `TemplateAmount`, and `Restriction::Event.affected_players`
with its `is_prohibited` union (§11 item 26 closes). **Consumers:**

- **Rhox Faithmender** — "Lifelink. If you would gain life, you gain twice
  that much life instead." `GainLife`, `Fixed(vec![])` + `You`,
  `Amount(Multiplier(2))`. Rulings, three: *two Faithmenders quadruple* →
  test (`Amount` composing on a player subject, RD-1's Furnace pair on the
  other kind); *"becomes 10" from 3 becomes 17* → RE-6's `SetLifeTotal` test,
  when it exists; *2HG* → n/a. **Reachable from the pool today**: lifelink's
  contained `GainLife` (Vampire Nighthawk, Knight of Meadowgrain) is the first
  proposal it meets.
- **Tainted Remedy** — "If an opponent would gain life, that player loses that
  much life instead." `GainLife`, `Opponents`, `Instead(LoseLife {
  ReplacedAmount })`. Rulings, two, both tests: *Alhammarret's Archive beside
  it: the gaining player picks double-then-lose-6 or lose-3-then-nothing* →
  the ordering test, and Archive's second application not existing is
  `never_happens` on a proposal that is no longer a gain; *two Remedies: the
  second has no gain to apply to* → CR 614.7 by the same road. Four-player:
  three opponents, three rows' worth of one static.
- **Words of Worship** — "{1}: The next time you would draw a card this turn,
  you gain 5 life instead." A `Primitive::CreateReplacement` row, `DrawCard`,
  `Uses::Once`, `UntilEndOfTurn`, `Instead(GainLife { Fixed(5) })` — the
  kind-changing substitution, from a draw (RE-2's pattern) to life (this PR's
  template), and the first `Once` draw row. Ruling: *several Words used before
  a draw: choose which applies each time* → CR 616.1 among rows of one player.
  Leyline of Punishment's ruling about it — the gain refused, the draw
  replaced with nothing — is the "can't" test below.
- **Ali from Cairo** — "Damage that would reduce your life total to less than
  1 reduces it to 1 instead." `LoseLife { cause: Some(Damage) }`, `You`,
  `Amount(LifeFloor(1))`. Rulings, three, all tests: *not effects that reduce
  life without damage* → `Primitive::LoseLife` goes through; *works if Ali
  dies in the same event* → the SBA batch decides against one board, so a
  lethal Earthquake to Ali and to you is decided while Ali is there;
  *damage is still dealt, triggers on damage still see it* → `DamageDealt`
  carries the full amount and only the contained loss is clamped.
- **Alhammarret's Archive** — both halves: `GainLife` ×2 and `DrawCard {
  cause: Some(Effect) }` ×2 on one legendary artifact, RE-2's arm and this
  one's on one consumer, Gisela's shape. Its rulings are Rhox Faithmender's
  and Teferi's, already tests.
- **Skullcrack** — "Players can't gain life this turn. Damage can't be
  prevented this turn. Skullcrack deals 3 damage to target player or
  planeswalker." Two `Primitive::Restrict` rows, `UntilEndOfTurn`: the RD-4
  fixture's `ApplyReplacement { Prevention }` row, now printed, and the new
  `Event { GainLife, affected_players: Everyone }`. Rulings, two, both tests:
  *replacements that turn gain into something else won't apply because
  gaining is impossible* → Tainted Remedy under Skullcrack does nothing;
  *"becomes N" higher than current does nothing* → RE-6's. **Leyline of
  Punishment** is the static form (`Effect::Restriction` on the permanent,
  RS-1's sweep) and its opening-hand clause is §3.3 source 2's, which would be
  dead under a real name — so Leyline is *not* registered, and the static form
  is proved by the RD-4 fixture extended with the life arm. Recorded so the
  omission reads as the rule and not as an oversight.
- **Bloodletter of Aclazotz** is recorded as a shape and not registered: "if
  an opponent would lose life during your turn" is a conditional static whose
  condition — it is your turn — the `Condition` AST has no leaf for, with one
  customer. `LoseLife`'s pattern and `Opponents` are built here for it.

**`PERFORMANCE_POOL` +1, Rhox Faithmender**, predicted: the first static
`GainLife` source, opening the sweep on every lifelink gain, and a four-drop
with lifelink of its own.

**Atoms:** `ATOM-119.10-001` (the 0-gain non-event, already `never_happens`;
the test now has a watcher to prove nothing was offered). Nothing else in the
corpus is filed under life-gain replacement; the ordering board is
`COMP-614-DAMAGE-ORDERING-001`'s sibling and is a test with no atom, which
`specdb suspicious` will not mind.

##### As landed (2026-09-12)

**Built as sized, with four corrections.**

- **`EventPattern::LoseLife` carries a `LifeLossCausePattern`, not a
  `LifeLossCause`.** The cause's `Damage` arm holds the damage's *source*, so a
  pattern holding it verbatim could only name one object, and Ali from Cairo is
  about damage from anything. `DestructionSourcePattern` is the same projection
  of `DestructionSource` and is the precedent the section did not reach for.
- **`AmountRewrite::LifeFloor` is the first arm `apply` cannot answer for.** The
  clamp reads the affected player's life total, which that signature has no way
  to see, so `apply_rewrite`'s `LoseLife` leg is its only evaluator and the
  damage and gain legs refuse the pairing — a card-authoring error reported the
  way every other half-disagreeing `ReplacementDef` is. `apply` asserts rather
  than returning a plausible number. The floor is `i64` because a life total is:
  CR 119.6 loses the game at 0 *or less*, so below zero is a real state between
  SBA checks, and a `u64` would be a claim about the scale rather than about the
  cards. All three printed floors are 1.
- **`Primitive::Restrict` could not build a player-scoped row** (§11 item 57),
  and Skullcrack — its first printed consumer — needed one. The target-filling
  loop's demand became a marker: an empty `Fixed` beside `PlayerSet::Nobody` is
  the restriction waiting for the resolution's subject, everything else is
  complete as authored.
- **The CR 616.1 suppression premise was written about damage** (item 58), so
  `test_two_rhox_faithmenders_quadruple` failed as an unexpected prompt before
  it could fail as a number. `EventPattern::reads_the_amount` replaces the list
  of kinds, on the type whose arms it classifies.

**Counted against the tree before writing, and §9's row had three numbers
wrong.** `Restriction::Event` constructions: the row said ~6, the tree had 12
literal constructions (3 in `src`, 9 in tests) plus one exhaustive
destructuring — the "~6" was a grep of the type name, comments and `..`
patterns included. `GainLife` producers 2 / `LoseLife` 3 was right in spirit:
`keywords.rs:75` and `resolve.rs:319` for the gain; `actions.rs:948`,
`resolve.rs:330` and `costs.rs:414` for the loss, and none needed an edit.
`TemplateAmount` did not exist and is this PR's; `AmountRewrite` had six arms
and `LifeFloor` is the seventh.

**Sized 1,400–1,600 and shipped +1,889 / −50**, so the row was ~18% low — inside
`engineering-practices.md` §4's 1,500–2,500 band, over its own prediction, and
the overrun is in one column. Engine **496** against ~350, cards **486** against
~450, tests **907** against ~600. The card column is right because §9's table
was written with the rulings pass in; the test column is not, and the reason is
the phase's shape rather than a miscount: six cards over three event kinds means
six rulings passes' worth of boards, and the two that produced the most tests
(Ali from Cairo's three rulings, Skullcrack's four) are cards whose whole
interest is in what they do *not* watch. **A phase whose cards are mostly about
exclusions costs more test lines than one whose cards are about arithmetic**,
and that is the calibration RE-4 should carry forward: RD-1's +1,950 against
~1,500–1,700 was the same overrun and was read as the rulings pass alone.

**Decided here, because the section left them open.**

- **Ali from Cairo does not clamp a `LifeLossCause::Cost`**, and the answer is
  structural twice over: CR 119.4 refuses a payment larger than the life total
  *before* any replacement is asked, and the pattern names `Damage`. Paying 3 at
  3 life leaves you at 0 with him on the battlefield. The card says "damage that
  would reduce" and its first ruling says "this effect does not apply to effects
  which reduce your life without doing damage". `Cost` is the arm nothing had
  watched; it is watched by a test now.
- **A floor only ever reduces a loss, never reverses one.** From a total already
  at or below the floor the clamp takes the whole amount and the loss becomes 0.
  "Reduces it to N" is a bound on how far the loss may carry the total, not an
  instruction to raise it, and the arm's type agrees — a `LoseLife`'s amount is
  a `u64`.
- **`Restriction::Event.affected_players` unions, and `PlayerSet::Nobody` on an
  object-only restriction keeps meaning what it always meant.** `set_affects`
  asks the object set or the player set by what the event is *about*, never
  both, so the union costs no existing restriction an answer and "this is not
  about players" is the honest reading of every row written before RE-3.
- **`never_happens` gains no `LoseLife` arm.** CR 119.10 is written about gain
  and the CR has no counterpart for loss; RE-2 kept `DrawCards { n: 0 }` out on
  the same reasoning. So a clamp from the floor performs a loss of 0, which
  `perform_action`'s local guard makes silent, and a watcher would see the
  proposal — which is what the rules say and what no printed card yet asks
  about.
- **Two identical substitutions ~~are not~~ *are* a fourth suppression shape**,
  and the first answer was wrong. Two Tainted Remedies were left prompting on a
  cost argument — no pooled card reaches the board — where the project has a
  rule, RC-4's "never prompt for a choice with one outcome". Review pushed back,
  and checking it turned up a premise **shorter** than the other three shapes'
  and one that does not mention the pattern at all: if every member carries the
  same `Rewrite` and that rewrite is a pure, instance-invariant function of the
  event, the event after one application is the same whichever member applied
  it, so the whole trace is. The clause that is not free is instance-invariance
  — `GameActionTemplate::GainLife` embeds CR 609.6's source and
  `DrawCards { player: Some(You) }` the controller — and it is
  `template_is_instance_invariant`. §11 item 60.

**`codebase-state.md` item 53's open question, answered.** "Will the next
`apply_rewrite` arm need `&mut GameState`?" — the review's answer was "mostly
no", and it held. `LifeFloor` is the second arm to consult the board and it is
a **read**: the affected player's life total, taken through `get_player`. The
count of arms needing `&mut` stands at one.

**Atoms.** `ATOM-119.10-001` covered — the atom needed a life-gain replacement
to prove was *not* offered, and until this phase there was none.
`ATOM-119.7-004` covered, which §9 said the corpus did not have (item 59).
`ATOM-616.2-001` gains a second `COVERS-PARTIAL`. `specdb owed` still 9.

##### Measured (2026-09-12)

Three arms, `plans/fuzz_ab.py`, 200 games / seed 12345, `--rounds 7`
(§11 item 54), against a same-day `main` worktree.

**The middle arm is identical to `main` outside `=== Timing ===` on both
pools.** §9 predicted "flat — patterns over events that already flow. A move is
the card", and that is the byte-level result: two `EventPattern` arms, an
`AmountRewrite` variant, two `GameActionTemplate` arms and a field on
`Restriction::Event` change no counter in a game with no card that uses them.
CPU median 15.49 vs 15.73 ms (−1.5%), rounds straddling.

**The shipped arm's CPU/game is up, and it is the card.** Two sittings — the
phase's and the review's — gave **+4.0%** and **+2.1%**, which is the honest
width of that number here: both are inside §8's 2–6% spread and the arms' rounds
straddle in both. The per-unit rows are the stable ones and they say the same
thing twice: `ms / 1,000 walks` **+0.2%** then **−1.6%**, `CPU/turn p50`
**+2.3%** then **0.0%**. The walk did not get slower; there are more of them,
because Rhox Faithmender makes the games longer.

| | main | new | Δ |
|---|---:|---:|---|
| Avg turns | 29.6 | 29.8 | +0.7% |
| Spells cast | 22.8 | 23.4 | +2.6% |
| Damage events | 19.6 | 21.3 | +8.7% |
| Total damage | 56.1 | 61.1 | +8.9% |
| Life changes | 12.9 | 13.9 | +7.8% |
| **Layer walks** | **371** | **385** | **+3.8%** |
| **Replacement gathers** | **974** | **999** | **+2.6%** |
| **Restriction queries** | **977** | **1001** | **+2.5%** |
| CPU/game median | 15.73 ms | 16.36 ms | **+4.0%** (re-run: +2.1%) |
| ms / 1,000 walks | 42.40 | 42.49 | **+0.2%** (re-run: −1.6%) |

`Replacement gathers` +25/game is one per doubled gain, not a sweep the phase
added — the sweep was already running on every lifelink gain, finding nothing.
`--require Rhox Faithmender`: **cast 176, resolved 174, in 114 of 200 games
(57%), copies/deck 1.52** — between Eon Hub's 64% at five mana and Thought
Reflection's 46% at seven, which is where a four-drop belongs. Reach was never
this card's claim: it is the only RE consumer that needs no second card to do
anything.

`stress` moves much further (gathers 987 → 1143, avg turns 29.0 → 32.3, memo
hits +44%) with all six cards in the deck; `stress` milliseconds are a
threshold and never a comparison (§3.1).

**Determinism.** `fuzz_ab` reports `deterministic: yes` on all three arms, and
three shell `fuzz_games` runs at one seed are line-for-line identical outside
`=== Timing ===`.

**Re-measured at the review (2026-09-12)**, because the fourth suppression shape
removes a prompt. `performance` is **identical to the digit** — no pooled card
carries two identical `Instead` statics — and `stress` moves by less than one
game (`Avg turns` 30.9 → 30.8 at 50 games, gathers 1062 → 1058), which is
§11 item 55's *answer-preserving is not stream-preserving* showing up a second
time and the first time it was predicted before the run. Determinism green
again.

#### RE-6 — the game's end (CR 104.2b, 104.3e, 104.4a, 704.5a–c, 704.7, 119.5, 800.4j–k) — ✅ landed 2026-09-12

*Evicted 2026-09-12 from `plans/replacement-architecture.md`, where the heading and a stub remain.*


**Builds:** decision 5 — the two variants and arms, the SBA batch members,
704.7's per-player leg, the flag reset, `GameResult` on `GameState`,
`Primitive::LoseGame`, `Primitive::WinGame`, `Primitive::SetLifeTotal` (CR
119.5 — "the player gains or loses the necessary amount", proposed through
`GainLife`/`LoseLife` so Rhox Faithmender's "3 becomes 10 becomes 17" ruling
is a test here), `GameActionTemplate::PlayerWins`, the two rotation sites, and
`fuzz_games --players N`. **Consumers:**

- **Laboratory Maniac** — "If you would draw a card while your library has no
  cards in it, you win the game instead." `DrawCard { cause: None }`, `You`,
  `Instead(PlayerWins)`, gated on a new `Condition::LibraryEmpty` evaluated
  at gather — the leaf's second customer is Jace, Wielder of Mysteries, a
  planeswalker, so the leaf is written for one card and says so; CR 121.6a
  is what makes the proposal reach the pipeline at all. Rulings, two, both
  tests: *if you can't win (Angel's Grace), you don't lose for the attempted
  draw either — the draw was still replaced* → Platinum Angel's row refuses
  the `PlayerWins`, the draw never performs, no flag is set; *two Maniacs and
  an each-player draw: APNAP, and the game ends at the first win* → needs an
  each-player draw producer, which `EffectRecipient` lacks (RD-2's Kitsune
  Palliator note); recorded on the card, not tested.
- **Exquisite Archangel** — "Flying. If you would lose the game, instead exile
  this creature and your life total becomes equal to your starting life
  total." `PlayerLoses`, `You`, `Prevent` with a rider of `Exile(self)` and
  `SetLifeTotal(starting)`. Rulings, nine, five are tests: *lethal damage to
  it and to you at once: its effect applies, and you choose exile or
  graveyard* → the SBA batch decides against one board, and the rider's exile
  meets the death's `ZoneChange` as two proposals on one object — the CR 704.7
  same-object collapse from RA-3, now with a player loss beside it; *drew
  from an empty library: you won't lose again until you try again* → the
  flag reset; *two Archangels: you choose which* → CR 616.1 among two
  printed statics; *an effect saying you can't lose: doesn't apply* → Platinum
  Angel's row ahead of the pipeline; *life -4 becomes 20 is a 24-life gain,
  and cards that interact with gain see it* → Rhox Faithmender makes it 44,
  which is the `SetLifeTotal` decomposition observed. **`ATOM-704.7-001`** —
  Lich's Mirror's board, 0 life and an empty library in one check, one
  replacement replacing both — is built with the Archangel; `COVERS` if the
  atom's board is generic, `COVERS-PARTIAL` naming Lich's Mirror if it is not,
  read at write time. Lich's Mirror itself waits on `ShuffleIntoLibrary`.
- **Stunning Reversal** — "The next time you would lose the game this turn,
  instead draw seven cards and your life total becomes 1. Exile Stunning
  Reversal." A `CreateReplacement` row, `PlayerLoses`, `Uses::Once`,
  `UntilEndOfTurn`, `Prevent` with a rider of `DrawCards(7)` and
  `SetLifeTotal(1)`, then `Exile(self)` as the resolution's second instruction
  (CR 608.2c, not part of the row). Rulings, ten, four are tests: *fewer than
  seven cards: you lose immediately after* → the rider's draws flag the empty
  library and the next check proposes a loss the spent row cannot see;
  *everyone would lose at once but this applies to you: you win as soon as
  everyone else has lost* → the four-player board, CR 104.2a from a batch
  with four `PlayerLoses` members and one replaced; *can't lose: can't
  apply*; *does nothing if you concede* → recorded, no harness.
- **Platinum Angel** — "Flying. You can't lose the game and your opponents
  can't win the game." Two `Effect::Restriction` statics, `Event {
  PlayerLoses, affected_players: You }` and `Event { PlayerWins, Opponents }`.
  Rulings, three: *no game effect can cause you to lose — 0 life, empty
  library, ten poison, Phage — you keep playing* → the SBA proposal refused
  every check, and the game continues through it (the first fuzz-reachable
  game that runs past a lethal board); *concession still loses* → recorded;
  *effects saying the game is a draw are unaffected* → CR 104.4c has no
  producer, recorded.

**`PERFORMANCE_POOL` +1, Laboratory Maniac**, predicted: a three-drop static
draw watcher, and the first card that makes decking a *win* in a measured game
— fuzz games deck out rarely, so the `--require` count and "games ended by a
win" are the rows to read. Platinum Angel is registered and not pooled: a
seven-drop that turns lethal boards into stalls would move avg turns by design
and not by engine.

**The four-player run is this PR's second measurement.** `fuzz_games
--players 4` at 200 games on `performance`: zero panics is the bar, and every
row it moves is written down as the *starting point* for RE-7 — item 108's
permanents that stay — and for B3's 802.

**Atoms:** `ATOM-704.7-001`, `ATOM-614.11-002` (Laboratory Maniac, the
corpus's own example), `ATOM-104.2b-001` and `ATOM-104.3e-001` (Phase 8, the
two primitives — covered where they are, not re-filed), `ATOM-119.5-001` and
`-002` (Phase 8, `SetLifeTotal`, same), `ATOM-104.4a-001` (ALREADY-IMPL,
gains its `COVERS:` from the four-loss batch), `ATOM-800.4j-001` as
`COVERS-PARTIAL` (the priority half; "the turn continues without an active
player" is RE-7's).

##### As landed (2026-09-12)

**Built as sized, with five corrections.**

- **CR 104.1 is a line at the top of `execute_batch_inner`.** The row listed
  the performers and the settlement and nothing about what happens *after* a
  game ends. Stunning Reversal's four-player ruling answered it: the survivor
  "wins the game as soon as everyone else has lost", and the seven cards the
  rider would then draw are the game continuing to be over. One early return at
  the chokepoint stops a decomposition's later inners, a resolution's later
  instructions and the ending batch's riders at the same line; the state-based
  check and the priority loop read the same field. It is also the one stream
  change on the two-player pools — see Measured.
- **CR 704.3's "performed" is read off the event log, not off the performed
  set.** A loss Exquisite Archangel replaced performs no member, but its rider
  performs — CR 614.6's modified event, CR 615.5's "the rest of the effect" —
  and Stunning Reversal's eighth ruling ("you'll lose the game immediately
  after") needs that to count as an action performed, or a priority window
  opens between the draw that re-arms CR 704.5b and the check that reads it.
  A refused proposal emits nothing, so the indestructible creature still does
  not re-check forever. §11 item 61 says what this admits, and the CR 104.4b
  cap on the loop (`MANDATORY_LOOP_CHECKS`, settling a draw) is its answer.
- **`Primitive::Exile` and CR 608.2m at the stack were nowhere in the row.**
  The Archangel's rider and Stunning Reversal's second instruction both exile
  the effect's own source, so `Exile` stopped being a stub with `Implicit` as
  the source and battlefield targets as the rest, and the stack's CR 608.2n
  graveyard trip now checks the card is still there to make it.
- **The flag reset is the check's, not the performer's**, which the sizing
  left as "either". A replaced loss and a refused one both perform nothing,
  and a flag the performer cleared would have re-proposed the same loss at
  every later check — Platinum Angel's "you keep playing" would have been
  "you are proposed for losing every time anyone gets priority".
- **CR 506.2 at the attack-target list**, found by the harness this PR built:
  a departed seat was still offered as a target, and the four-player table's
  first numbers were a measurement of the random agent hitting empty chairs
  (item 66).

**Counted against the tree before writing, and the row was light where the
prompt said it would be.** `player_lost` had 27 mentions in `src` against the
row's "~4 `Game.result` readers": four direct writes in the sweep, two reads in
`turns.rs`, two in `pipeline.rs`, one in `game.rs`, plus the tests. `fuzz_games`
hard-coded two decks at three sites — the deck loop, the copies count and the
outcome key — so `--players N` was ~90 harness lines, not ~50. Six tests
already ran three or four players, and none of them a fuzz game.

**Sized 1,800–2,100, read as 2,100–2,600 at the prompt, and shipped +2,211 /
−272 in code** (docs excluded): engine **664** against ~550, cards **423**
against ~400, harness **94** against ~80, tests **1,030** against ~800. Inside
`engineering-practices.md` §4's band and inside the corrected read; over the
row by the two columns the row could not have seen — the priority loop's
rotation, the settlement and the two primitives in the engine column, and the
rulings pass in the test column, for RE-3's reason: three of the four cards are
about what does *not* happen.

**Decided here, because the section left them open.**

- **`Condition::LibraryEmpty` is evaluated at gather, not at application, and
  it holds because of two rules that were already there.** CR 604.2 makes a
  conditional static's effect exist while its condition holds, and CR 614.4
  asks whether the effect exists *before the event* — so a Laboratory Maniac
  whose library still has cards is not a candidate at all, and a Thought
  Reflection beside it is never a prompt with one live option. CR 121.6a is
  what puts the proposal in front of that gather: the draw reaches the pipeline
  with nothing to draw, and the condition is true at exactly that moment. The
  doubled draw is the board that separates the two readings — the first inner
  takes the last card and the second is the win — and it is a test. The
  evaluator is `settled_holds`, the one the layer pass and CR 613.11's cost
  effects already share, so the leaf is one question wherever it is asked
  (§11 item 64).
- **`GameResult` on `GameState` moves "the game is over" onto the
  chokepoint, and the stateful part is CR 104.1's "immediately".**
  `Game::check_game_over` is deleted; `Game::is_over` and `Game::result` are
  reads. The `PlayerWins` performer records a win; a batch that performed one
  or more `PlayerLoses` settles CR 104.2a/104.4a once every member has
  performed; nothing outside `engine::actions` writes the field. That is the
  materialize-not-derive rule: a fact recorded once, at the batch that made it
  true, and read thereafter.
- **CR 104.2a is checked per batch, never per member.** Two players losing in
  one check is a draw (104.4a); a performer that asked "is anyone left" after
  the first of them would have crowned the second. The four-loss Stunning
  Reversal board is the same rule from the other side — three perform, the
  batch settles the survivor's win, and the rider that follows performs
  nothing (§11 item 62).
- **`never_happens` gains no arm.** RE-2 and RE-3 declined because the CR
  names no rule making a 0-draw or a 0-loss a non-event; here there is no
  amount at all, so there is nothing a zero could mean. The one "never
  happens" the CR does state for these kinds — a player who has left cannot
  lose again — is CR 800.4k's shape, a rule at the *proposal* gate, which is
  where RE-1 put 800.4k and where the sweep's `in_game` guard is.

**Glossary triage** (`check_glossary.py --suggest`, 418 doc-comment lines,
13 candidates): *ruling*, *tested*, *refused*, *reasons*, *concession*,
*pooled*, *rulings*, *sentence*, *expressible*, *flag(s)*, *pattern*, *wrapper*.
No coinage — the CR's words, the practice's, and the types' own names.
RE-3's outcome again, and the second time "nothing to add" was the answer.

##### Measured (2026-09-12)

**The middle arm is the prediction to the digit: `Replacement gathers` +1 per
game and `Restriction queries` +1 per game on `performance` (999 → 1000, 1001
→ 1002), every gameplay row identical to `main`, `Layer walks` identical.** The
one row that moves besides is `Memo hits`, 61,652 → 61,444 (−0.3%), and it is
the post-mortem tail: until this PR a player who had just lost kept receiving
priority — and, with a random agent, casting spells — until the phase ended
and `Game` noticed, and CR 104.1 at the priority loop removed those questions.
So `registered vs main outside Timing: differ`, and the check is the gameplay
aggregates, which are identical to the printed digit (§11 item 65). CPU/game
across two sittings **−1.0% and −1.7%**, `ms / 1,000 walks` −1.0% and −1.7%,
`CPU/turn p50` 0.440 → 0.440 and 0.490 → 0.480 — flat, inside the spread, and
the sittings' absolute medians (15.7 and 17.5 ms) are §8's reminder that a
stored ms number is machine state.

**The pooled arm is a re-record, not a reading.** Laboratory Maniac, 80 → 81:
`Layer walks` 385 → 373, `Layer frames` 4,629 → 4,487, `Dependency checks`
41 → 34, CPU/game −4.7% and −5.0% across the two sittings. A 2/2 for three
displacing a slot's share of costlier cards, and §3.1's rule stands — never
A/B a number across a pool change. **`--require "Laboratory Maniac"`: cast
203, resolved 201, in 117 of 200 games (58%), copies/deck 1.57**, board
diversity 100%. **Games ended by a win: zero**, on `performance` and `stress`
at two seats and at four. Fuzz games deck out rarely, as the section said; the
number that says the path was walked is the 58%, and the one that says decking
is a win a measured game can reach came from the pre-fix four-player run,
whose ghost-attack-inflated 87-turn games decked out five times in 400
(item 66). The two-player pool at 30 turns does not.

**The four-player table is `engineering-practices.md` §3's**, recorded as
RE-7's starting point with the two rows it is measured against: turns after a
departure **21.0** per game and departed-owned permanents **32.2** at the end
on `performance` (22.8 and 34.2 on `stress`). Zero errors, zero panics on both
pools; one `stress` game ran to its 200th turn and ended there with a win, and
one ended in CR 104.4a's draw — two survivors trading lethal combat damage in
one step, both losses in one check, settled by the batch.
Three shell runs at one seed line-for-line outside `=== Timing ===`, and
`fuzz_ab.py`'s own check `deterministic: yes`.

#### RE-7 — leaving the game (CR 800.4a–e, 800.4c, 800.4m) — ✅ landed 2026-09-13

*Evicted 2026-09-13 from `plans/replacement-architecture.md`, where the heading and a stub remain.*

**Builds:** the rest of decision 5's N-player half, inside `PlayerLoses`'
performer as CR 800.4a says — "this is not a state-based action. It happens as
soon as the player leaves the game." In the rule's own order: every object the
player owns leaves the game (removed from hand, library, graveyard, exile,
command zone, battlefield and stack, one `GameEvent::LeftTheGame { object }`
each — not a zone change, CR 400.11: outside the game is not a zone); every
control-changing row in that player's favor ends (a Layer 2 row keyed by the
departed controller, `ContinuousEffect.controller`); their stack objects not
represented by cards cease to exist; and anything they still control is exiled
through `change_zone` with a new `ZoneChangeCause::ControllerLeftTheGame`,
which no catchall may absorb. 800.4b and 800.4d are refusals at
`propose_entry`, `CreateTokens`' performer and Layer 2's control change for a
departed player; 800.4e is a refusal at combat damage assignment; 800.4m sets
each "until that player's next turn" duration to expire when that turn would
have begun, on all three duration registries. 800.4c — a control effect ending
with no default controller left — is main item 9's revert and lands here as
its own arm. **Consumer:** Act of Treason, which is in the pool, both ways
round — the thief loses and the stolen creature goes home (800.4a's second
clause); the owner loses and the creature the thief still controls leaves the
game (first clause) — plus `setup_game(4)` tests for each clause and the
four-player fuzz run RE-6 opened, whose "permanents that stay" row this PR
zeroes. No cards.

**`PERFORMANCE_POOL`: no change.** The measurement is the four-player table:
RE-6's starting point against this PR's, and CPU on the two-player pools
byte-identical, since nothing here runs before a third player exists.

**Atoms:** `ATOM-800.4a-001`, `ATOM-800.4a-002`, `ATOM-800.4b-001`,
`ATOM-800.4c-001`, `ATOM-800.4d-001`, `ATOM-800.4e-001` (all Phase 9, covered
where they are); `COMP-800-PLAYER-LEAVES-COMMANDER-001` as `COVERS-PARTIAL`
until commander damage exists; `ATOM-800.4j-001` completes. **800.4m and
800.4k have no atom** — session-10 deferred both as "edge-case resolution
policies" whose prerequisites had not landed — so this PR files
`ATOM-800.4m-001` in `session-10.md` and covers it, the way RE-10's exit
criterion files CR 500.8's.

##### Decided before writing, because the section left four open

**1. "Leaves the game" is a performer, not a proposal, and its emitter is a
second function.** CR 400.11 — outside the game is not a zone — so an owned
object leaving is not a `ZoneChange`, and there is no destination to give one.
It is also not a `GameAction`: §3.2b's growth contract asks which CR rule
permits a new arm, and no printed card says "would leave the game", so an arm
here is one `EventPattern` nothing could ever match — "worse than a missing
one". The removal therefore lives **inside the `PlayerLoses` performer**, which
is one of `perform_action`'s own arms and so satisfies the chokepoint invariant
as written; the precedent is CR 704.5d's token sweep and CR 608.2n's ability,
both of which remove an object with no zone change and no proposal.
`GameState::remove_from_game` is the performer — `cleanup_zone_state`, the
zone collection, `remove_object` — and `GameEvent::LeftTheGame { object, owner,
from }` is the emitter, kept apart for the reason `move_object` and
`announce_zone_change` are. Clause 4's exile *is* a zone change and goes
through `change_zone` with a new `ZoneChangeCause::ControllerLeftTheGame`; it
is a result of the departure, so its nested batch joins the enclosing one
(§4.2's default).

**No LKI frame on `LeftTheGame`, and the question is recorded rather than
answered.** Whether a "leaves the battlefield" trigger fires when a permanent
leaves the *game* is item 6's, and the CR does not say in one sentence; the
frame costs a layer walk per permanent at the moment a game ends, which at two
seats is the loser's whole board. The field is two lines the day a trigger
wants it. → `codebase-state.md`, "Before Triggered abilities".

**2. The batch-order hazard: the losses go last, and the rule generalizes.**
CR 704.3 decides every member against one board and then performs them in batch
order; the performers are loud about the board they find. A `PlayerLoses`
member is the only one that *removes other members' subjects* — a creature
owned by the departing player that is dying in the same check is both a
`Destroy` member and, a moment later, an object that has left the game. So the
losses are gathered into their own vector and appended after the 704.5f–m
sweeps and the 704.6d moves: **a member that removes objects performs after the
members that were decided against them.** The subject-keyed dedupe is
indifferent (a loss's subject is a player and a death's is an object), CR
order among the losses themselves is preserved because they are gathered in it,
and the simultaneity the rule asserts is untouched — only the log's order
moves. The two alternatives were rejected on the invariants: a removal
"tolerant" of members already decided makes `perform_zone_change` quiet, which
is the opposite of "performers are loud"; and deferring the leave past phase 3
contradicts 800.4a's own "as soon as the player leaves the game" and would run
the riders against a board the departure should already have emptied.

**3. 800.4m expires at the turn boundary that skips the seat, and the site is
`next_turn_taker`.** "Until that player's next turn" is read by
`remove_expired_at_turn_start`, which `on_turn_begin` calls for the turn that
began — and a departed player's turn never begins, so the row would last
indefinitely, which 800.4m forbids in as many words. The moment the turn *would
have begun* is the moment the rotation passes the seat, and that moment already
has a function: `next_turn_taker`, which consumes what it reads and is where
RE-1 put 800.4k. Each seat it skips for `player_lost` expires that player's rows
at `turn_number + 1` — the number the turn would have carried — on all three
registries, and the queue half gets the same treatment because a queued extra
turn is a turn of theirs that would have begun. Idempotent, so the second
rotation past the same seat finds nothing. 800.4m's other half — "or until a
specific point in that turn" — has no rows: step- and phase-scoped durations are
`backlog.md` §2.12's and arrive with their first consumer.

**Two definitions, not three.** §9's row says `remove_expired_at_turn_start`
**3**; RS-0 made `ReplacementEffectRegistry` an alias of `DurationRegistry`, so
the three registries share two definitions and one of them is generic.

**4. A departed player's stack objects cease to exist through `remove_object`,
silently, and clause 3 is the residual clause 1 leaves.** CR 800.4a's third
sentence is scoped to objects the player *controlled* that are not represented
by cards — in this engine, an ability on the stack, whose `GameObject.owner` is
its activator, so clause 1 has already taken every one of them by the time
clause 3 looks. It is written anyway, keyed on `stack_entries`' controller, for
eight lines: it is the CR's own sentence, its customer is a copy of a spell
(CV-4, `is_copy`'s first writer, with an owner that is not its controller),
and the alternative
is a ledger entry longer than the code, which is what the ledger's head now
forbids. It emits **no event**: an ability ceasing to exist is announced nowhere
in this engine — CR 608.2n's is silent and CR 701.6b's is announced as the
*countering* — and nothing printed watches one. Clause 1's `LeftTheGame` is
what the log carries today, because clause 1 is what removes them.

##### As landed (2026-09-13)

**Built as designed, with one correction that changes the phase's scope
sentence.** CR 800.1 — "a multiplayer game is a game that begins with more than
two players" — is the gate on every rule here, and without it §9's "byte-identical
on both two-player pools, by construction" was a claim about the code rather
than about the rules. `GameState::is_multiplayer` reads the seat count the game
*began* with, so a four-player game down to two keeps CR 800.4; three RE-6
tests found the omission the first time the performer ran (§11 item 68).

**`engine/leaving.rs` is the new module**, and CR 800.4's own sentences are its
functions: `owned_objects_leave`, `end_control_given_to`,
`uncarded_stack_objects_cease`, `exile_objects_no_player_in_game_controls`.
CR 800.4f–i and CR 802 are named in its header as the neighbours that are not
here, because "Before Commander" item 4 is where they live and a reader arriving
at this file will look for them.

**Four things the section left open, decided before the code and unchanged by
it** — the chokepoint shape, the batch order, 800.4m's moment, and what a
departed player's stack objects go through. They are written out above, under
"Decided before writing"; §11 items 68–72 are what building it added.

**Three small wrong answers fixed here rather than recorded**, on the ledger's
own rule:

- **CR 608.2n's tail assumed its own object survives the resolution.** A spell
  whose owner leaves during its own resolution is gone from `objects`, and
  `resolve_taken` turned that into an error rather than into CR 608.2m's "it
  will continue to resolve fully". Six lines and a fixture (§11 item 71).
- **`fuzz_ab.py` could not read the row this PR exists to zero.** RE-6 added
  "Turns after a departure" and "Departed-owned permanents" to `fuzz_games` and
  not to the script that diffs two of its runs (§11 item 72).
- **`ATOM-800.4c-001`'s board could not reach its own rule**, which is a corpus
  defect rather than an engine one and is corrected in `session-10.md` with the
  reason (§11 item 70).

**Counted against the tree before writing, and the row was one short.** It
predicted "the five zone collections + the stack **6** sweeps"; there are
**seven** collections — `PlayerState::{library, hand, graveyard}` and
`GameState::{battlefield, stack, exile, command}` — and clause 1 is one loop
over all of them rather than six. `propose_entry` had the two callers the
prompt verified; `remove_expired_at_turn_start` had two definitions and not
three, RS-0 having made the replacement registry an alias.

**Sized 800–950, read as 900–1,200 at the prompt, and shipped +1,543 / −26**
(docs excluded): engine **540** against ~400 — of which ~300 is the new module
and 55 its own unit tests — and tests **1,003** against ~450. Over the
corrected read's top by about a quarter, and the column that ran over is the one
RE-3 and RE-6 both named: the rule has four clauses, three refusals, two moments
and a scope, and each wants a board plus the control board that says the rule is
doing the work. **The review added a third of the test column** — CR 603.6c's
frame, the no-duration control effect both ways round, and the two combat boards
that separate CR 800.4e from CR 510.1c — which is the sizing lesson rather than
an overrun: this phase's unit is a *sentence of one rule*, and the row costed it
by counting call sites. No cards, as the row said.

**Decided here, because building it asked.**

- **Clause 1 reads the zone collections, never `objects`.** The log records one
  `LeftTheGame` per object, so the order is observable, and `GameState::objects`
  is a `HashMap`. The collections are the same set in a deterministic order —
  `battlefield_ids_ordered` for the board, the `Vec`s for the rest — and the
  order among the seven is this engine's rather than the CR's, which names none.
- **Clause 4 and CR 800.4c are one predicate.** Both reduce to "the effective
  controller is not in the game": 800.4a asks it at the departure, 800.4c at the
  moment a control-changing effect ends. 800.4c's third condition — "there is no
  other effect giving control of that object to another player in the game" — is
  the layer walk's own answer rather than a second test, because the effective
  controller *is* the top Layer 2 row's. The sweep is free while everyone is
  still playing, which is every two-player game and every four-player one before
  its first departure.
- **Clause 2 does not consult a row's duration, and CR 110.2 is what lets it
  just delete one.** CR 800.4a ends the effect whether or not it had a duration
  — an Aethersnatch-shaped row, which nothing in CR 514.2's cleanup would touch,
  ends here and nowhere else — and control falls back with nothing to undo
  because CR 110.2's default is a value `PermanentState` *stores* (110.2b's
  caster, main item 9). Both halves of the judge answer that prompted the check
  are tests: the no-duration row ending when the player it favors leaves, and
  the creature *staying* with the thief when its default controller leaves
  instead, which is the board CR 800.4c can never fire on.
- **Clause 2's residual is the resolution's row, and that is why it is small.**
  A Layer 2 row from a static ability dies with its source, and clauses 1 and 4
  have just taken every source the departing player owned or controlled, so
  `cleanup_zone_state`'s `remove_by_source` ends those. What is left is Act of
  Treason's: a row whose source is a sorcery in a graveyard that nothing will
  disturb. `PlayerRef::Owner` and `Opponent` name no single beneficiary a row can
  be judged by and are left alone; they need nothing, because a creature the
  departing player owns left with clause 1 and one they merely control is exiled
  by clause 4.
- **`LeftTheGame` carries the CR 603.10a frame, and CR 603.6c is why.** The
  first cut left it off as an open question; the rule answers it in the sentence
  that names this event — "leaves-the-battlefield abilities trigger when a
  permanent moves from the battlefield to another zone, **or when a phased-in
  permanent leaves the game because its owner leaves the game**". So the frame
  is captured in the same one-statement window `perform_zone_change` uses, for
  the permanent and for nothing else. The residual is the qualifier: "phased-in"
  is a condition this engine cannot express, and `codebase-state.md`, "Before
  Triggered abilities" item 6 is what says so. → §11 item 73.

**Glossary triage** (`check_glossary.py --suggest`): no coinage. The words this
phase adds are the CR's own — *leaves the game*, *ceases to exist*, *default
controller*, *multiplayer* — and each already resolves to a rule number in the
source it appears in. RE-3's outcome and RE-6's, a third time.

##### Measured (2026-09-13)

**The row this PR exists to zero is zero: "Departed-owned permanents" 32.2 →
0.0 on `performance` and 34.2 → 0.0 on `stress`**, 200 games / seed 12345 at
four seats, and 32.7/32.8 → 0.0/0.0 on the 50-game fixture table. Zero errors
and zero panics on both pools, both arms.

**Both two-player pools are `IDENTICAL` outside `=== Timing ===`, which is the
first RE phase since RE-3 to manage it** and the first whose reason is a rule
rather than a gate: CR 800.1 puts the whole section out of scope at two seats
(`--rounds 0 --no-fixtures`, 200 games / seed 12345, both pools). Three shell
`fuzz_games` runs at one seed and `--players 4` are line-for-line identical
outside `=== Timing ===` on both pools, and `fuzz_ab.py` reads `deterministic:
yes` in both arms.

**Every four-player engine-work row moves, and one number says why:
`Frames/walk` 21.22 → 18.69.** A walk since LI-1 fills the whole working set,
and the working set is a board about 32 permanents smaller, so each one got
cheaper: `Memo hits` 212,676 → 187,526 (−11.8%), `Layer frames` 17,019 →
15,660 (−8.0%), `Dependency checks` 307 → 171. **CPU/game median −15.8% and
−14.0% across two sittings**, `ms / 1,000 walks` −19.4% and −17.7%,
`CPU/turn p50` 0.860 → 0.800 and 0.840 → 0.800 — which is not a
speed-up the engine earned but a board it stopped carrying, and it is the size
of item 108's cost. The proposal rows are flat as predicted, `Replacement
gathers` 2104 → 2102 and `Restriction queries` 2107 → 2107: this PR adds no
proposal.

**`Layer walks` is the one row that goes up — 802 → 838 — and it is CR 603.6c's
price, measured rather than assumed.** The frame a permanent leaves the game
with is an uncached walk each, so it costs about 33 per game across three
departures (+4.5%) and **zero at two seats**, where CR 800.1 puts the whole
section out of scope. On `stress` the two cancel: 1,239 → 1,240. It is recorded
as a rate rather than a total because the cancellation is a fact about *this*
board width, and a wider one would not repeat it.

**The gameplay rows move because the games are different**, and only one of them
is an engine reading: `Avg turns` 61.2 → 61.0 and `Total damage` 156.1 → 152.3
are a game whose departed players stopped blocking and attacking, and the seat
win shares move with them (87/63/36/14 → 95/58/34/13). Read them as "the same
harness on a different board", never as a delta.

**Both of `stress`'s outliers resolved, and reading them first was the check.**
`Hit turn limit` **1 → 0**: seed 12413 ran to its 200th turn with **99
departed-owned permanents** on the board, three seats' worth of creatures that
no player controlled still blocking; it now ends at turn 196 with a win.
`Draw` **1 → 0**: seed 12492's CR 104.4a draw at turn 46 is gone, and it is a
game that diverged rather than a rule that changed — with the departed players'
boards removed it ends at turn 43 with a win for P0, four combats earlier than
the double-lethal it used to reach. **And `Wins by effect` 0 → 1**, which is
RE-6's "games ended by a win: zero" losing its zero: seed 12410 is Thought
Reflection beside Laboratory Maniac in a draw step, the first inner taking the
last card and the second replaced by the win — RE-6's
`thought_reflection_beside_laboratory_maniac_asks_nothing_and_the_second_card_is_the_win`
reached in a measured game.

**`PERFORMANCE_POOL` unchanged at 81, as the section said**, and both arms print
`Card pool: performance (81 cards)`. No `--require` row: this PR registers no
card, and the path it opens is walked unforced in every four-player game with a
departure, which the "Turns after a departure" row of 20.8 says is most of them.

#### RE-4 — tokens (CR 614.16's token half, 111.5, 616.1g; items 46 and 52) — ✅ landed 2026-09-13

*Evicted 2026-09-13 from `plans/replacement-architecture.md`, where the heading and a stub remain.*

**Builds:** decision 3 — `CreateTokens { defs, controller }`,
`EventPattern::CreateTokens`, the performer that creates the objects and
proposes their entries as one batch, `CreateTokenIn { object, zone }` with
`TokenCreated`, and `Amount` over a `Vec` (a multiplier repeats each def).
`Primitive::CreateToken(def, amount)` becomes one proposal. **Consumers:**

- **Parallel Lives** — "If an effect would create one or more tokens under
  your control, it creates twice that many of those tokens instead."
  `CreateTokens`, `Fixed(vec![])` + `You` (the subject is the controller the
  tokens are created under), `Amount(Multiplier(2))`, `Uses::Static`.
  Rulings, two, both tests: *two Parallel Lives create four times* →
  `Amount` composing (the Furnace pair, third kind); *everything specified
  by the creating effect is true of the extra tokens* → structurally true of a
  repeated def, asserted on a token's characteristics. `ATOM-614.16-001` —
  "token replacement applies to tokens from other replacements" — is Kalitas's
  rider making a Zombie under Parallel Lives: two Zombies, and the atom's
  board is in the pool already.
- **Raise the Alarm** and **Hordeling Outburst** — "Create two 1/1 white
  Soldier creature tokens." / "Create three 1/1 red Goblin creature tokens."
  The first plural creations in the crate, and so **the first plural entry
  batch** (item 46): the test is RC-5's
  `test_two_biomancers_entering_together_give_each_other_nothing` reached from
  a printed card — two Soldiers under Master Biomancer each get Biomancer's
  counters and neither gets the other's — plus Root Maze asking once per token
  (CR 616.1g's per-entry fresh choice, §3.2d's contrast case).
- **Hallowed Moonlight** — "Until end of turn, if a creature would enter and
  it wasn't cast, exile it instead. Draw a card." A `Primitive::CreateReplacement`
  row, `EnterBattlefield { cast: Some(false) }`, `Filter { creatures }` +
  `Everyone`, `Instead(ZoneChangeTo { Exile })`, `UntilEndOfTurn` — every
  piece exists since RC-4b, and it is the card that reaches item 52. Rulings,
  two, both tests: *a creature token is put into exile instead and then
  ceases to exist* → the log holds `CreateTokens`, `TokenCreated { Exile }`
  and CR 704.5d's `TokenCeasedToExist`, and **no `ZoneChange { from:
  Battlefield }`** — the line Dour Port-Mage would have read, asserted absent;
  *a cast creature is unaffected, from any zone* → Grizzly Bears resolves
  normally under it.

**`PERFORMANCE_POOL` +2, Parallel Lives and Raise the Alarm**, predicted: the
first `CreateTokens` static source, and the producer that makes a plural entry
batch happen in a measured game — the engine path item 46 wanted measured, and
the first board on which CR 616.1 is asked once per token. Kalitas's rider
already makes single tokens in the stress pool, so the middle arm moves by one
gather per Zombie and nothing else.

**Atoms:** `ATOM-614.16-001`; `ATOM-111.5-002` (Phase 8 — a token not created
when a permanent with its characteristics can't enter: Worms of the Earth's
`ZoneChange` prohibition over a land token, covered where it is, not
re-filed); `ATOM-613.7m-001` stays on its `L03` ticket with decision 3's
"not asked" reason written beside it, since a homogeneous batch has no
observable order.

##### Decided before writing, because the section left four open (2026-09-13)

**1. The name is `CreateTokenIn { object, zone }` / `TokenCreated`, and the
event has two emitters.** CR 111's verb is *create* — 111.1 "some effects put
tokens onto the battlefield", 701.7a "create … put the specified number of
tokens … onto the battlefield" — and Hallowed Moonlight's ruling names the
substitute's *destination* ("put into exile instead") rather than a second
verb. So the action reads as the sentence it is: create the token *in* that
zone. `GameEvent::TokenCreated { object_id, controller, zone }` is its
announcement, and it is also announced for a token that enters: CR 111.2 is
two sentences — "the player who creates a token is its owner. The token
enters the battlefield under that player's control" — and CR 111.13 is the
line the event has to respect: a copy of a permanent spell becoming a token
"is not 'created' for the purposes of any replacement effects or triggered
abilities that refer to creating a token". That token enters from the stack
with a `from`, takes the card arm of the entry performer, and gets no
`TokenCreated`; a token made by an effect enters from nowhere, takes the
token arm, and does. One emitter function, `announce_token_created`, two
callers, each of which performed the placement it announces — the same shape
`announce_zone_change` has. The outer `CreateTokens` performer emits nothing
of its own, on `DrawCards`' and `Destroy`'s precedent: the contained events
announce themselves, and a "one or more tokens" trigger reads the batch the
way "one or more creatures die" does.

**2. The outer reports the event as decided; the log counts creations.**
Three tokens proposed and one dropped: `execute_actions` returns
`CreateTokens { defs: [3] }` as performed — the event as CR 616.1 left it
(and nothing reads a `CreateTokens` member of `performed` today; the
precedent is for the log and for whoever reads `performed` later) —
and the log holds two `TokenCreated`s. CR 111.5 says the third "is not
created", and un-creating it is no more an event than `add_object` was. The
precedent is `DrawCards { n }` against a library that runs out: the
instruction happened, fewer cards were drawn, `CardDrawn` is the count.
Nothing reads a token count off the outer — a rider reads `event_amount`,
which is the count *proposed*, and `settle_game_result` reads losses.

**3. `TokenDef` stays a description, and the rules pass moved two fields
the measurement had not.** `abilities: Vec<AbilityDef>`, `supertypes`,
`rules_text` and `enchant_filter` land as §2.27 sized them. `Arc<CardData>`
was rejected on CR 111.4's own word — a spell "sets" the token's
characteristics, it does not hand it a card — and on what it would put in
front of every token author: six fields CR 111.6 and 111.3 say a token has
none of (mana cost, the cast-time costs, a color indicator). CR 111.11's
by-name lookup is a `CardRegistry` read at resolution that *produces* a
`TokenDef`, and costs the same either way. Two corrections from reading CR
111 rather than the struct: **`name` is `Option<String>`**, because CR 111.4
gives an unnamed token the name "[subtypes] Token" and every def in the
tree had been naming Kalitas's Zombie "Zombie"; and **power and toughness
are `Option<i32>`**, because the lowering wrote `Some(0)/Some(0)` onto a
noncreature token and eighteen of CR 111.10's twenty are noncreature.
Construction sites counted before choosing: three in the tree (Kalitas, two
test fixtures).

**4. Trace page: no.** §7 named RE-4 for "how an entry's frame is answered
inside a plural batch", and RC-5's re-size found that read already answered
and already walked — `rc-5-applying-an-entry-can-move-the-board.html`'s
last trace is two entries decided against one board. What RE-4 changes is
what is *proposed* (one plural batch where there were N singletons; an
appearance where there was a move), which is RE-6's and RE-7's shape and
their answer. The three boards worth walking — Parallel Lives over a
plural creation, Master Biomancer beside Hallowed Moonlight asked once per
token, the exiled token's log — are each one test.

##### The shape, as built

- `GameAction::CreateTokens { defs: Vec<TokenDef>, controller }`. Subject
  `Player(controller)` — CR 614.16's "under your control" is who chooses.
  `event_amount` is `defs.len()`. Not in `never_happens`: CR 614.7a's
  rule is written about damage and life gain, and an empty creation is
  refused by the pattern instead — `EventPattern::CreateTokens` matches
  "one or more" (`!defs.is_empty()`), which is the rule's own phrase.
- `EventPattern::CreateTokens` carries no field. Printed constraints exist —
  *creature* tokens (Divine Visitation, Ojer Taq, Jinnie Fay), *Treasure*
  (Xorn), *Clue, Food or Treasure* (Academy Manufactor) — and none is
  registered here; `GainLife`'s rule applies (wait for the customer, size
  the retrofit): a `kind` field matched against each `TokenDef`'s types and
  subtypes, one `pattern_watches` clause, and `Amount` repeating only the
  matching defs. `reads_the_amount` is `false`: the one count it reads is
  "one or more", and no multiplier `ordering_cannot_change_outcome` admits
  (n ≥ 1) crosses that line.
- `Rewrite::Amount(Multiplier(n))` over the `Vec` repeats each def in place
  (`[A, B]` → `[A, A, B, B]`), which is Anointed Procession's "twice as many
  of each kind" and keeps the batch order — and so CR 613.7's timestamps —
  homogeneous per kind. Every other `AmountRewrite` is refused as an
  authoring error; `Plus` is the one with a printed customer (Xorn's "that
  many plus one"), and it waits because "plus one *of what*" is a question
  a heterogeneous `Vec` cannot answer without the kind field above.
- The performer creates each object in the battlefield zone with no entity,
  builds every entry with the same seed `propose_entry` uses, and proposes
  them as **one** `execute_actions` — contained, so fresh applied sets
  (CR 616.1g), joining the enclosing batch id. A member that did not perform
  is un-created (CR 111.5). CR 800.4b/d stays in the *producer*
  (`Primitive::CreateToken`), ahead of the proposal, where `propose_entry`
  keeps its own.
- `GameAction::CreateTokenIn { object, zone }` is what `substitute` returns
  for an `Instead(ZoneChangeTo)` on an entry with `from: None`. Its
  performer — `GameState::put_token_into` — adds the object to the zone's
  collection, writes its zone, stamps the zone-change epoch (CR 704.5d's
  sweep orders by it) and announces `TokenCreated`. Refuses the battlefield
  (that is an entry) and a non-token. No `EventPattern` arm: nothing prints
  "if a token would be created in exile", and it joins `Attach` as the
  second deliberate exemption from §3.2a's one-arm-per-variant.
- CR 613.7m is not asked (decision 3): a homogeneous batch has no observable
  order, and the batch order is the creation's. The first distinguishable
  creation — Academy Manufactor, Bestial Menace — is the prompt's customer.

##### As landed (2026-09-13)

**Built as designed, with the two corrections the rules pass forced on the
type (§11 item 74) and one on the event (item 75).** Twenty-two tests in
`phase_re4_integration_test`; two existing tests read the log's shape around a
token and were updated to the designed one — RB's Kalitas-rider order gains the
creation line between the exile and the entry, and RC-4b's token test, which
had asserted the cheap answer by name, asserts the honest one. A probe written
against the pre-fix tree (the item-52 assertion on the engine's own vocabulary,
no card) failed there on exactly the `from: Battlefield` line and passed with
the fix; it was deleted once the same assertion lived in the phase's file.

**Counted against the tree before writing, and the row held.** The three
exhaustive matches — `subject_of`, `event_amount`, `perform_action` — were
exactly as counted; `pattern_watches`, `reads_the_amount` and `display.rs`
each wanted an arm the compiler asked for; `game_state.rs`, on the seven-file
list, needed nothing for the variants. `propose_entry` was *split* rather than
restructured — `entry_proposal` builds, the two callers execute — which is what
let the plural batch share one seed with the single entry. `PERFORMANCE_POOL`
is a sized array, which the pool commit learned the loud way.

**Sized 1,650–1,850 and shipped +1,867 / −171** before the docs and the
review: engine 594 against ~600, cards 322 against ~350, tests 951 against
~700 — the test column over again, for RE-3's, RE-6's and RE-7's reason, and
this time a third of the overage is the two lineage findings (§11 items
77–78, ~90 lines with their fixtures). Zero warnings; 1,375 tests. The review
(`plans/handoffs/re-4-review.md`) then added what the owner asked in: a
rider carrying its lineage, the one-exit suppression shape, the token
pattern's kind, the creation template, and two more cards.

**Glossary triage** (`check_glossary.py --suggest`): nine candidates at
landing and twenty-four after the review, one coinage between them — *def*,
which the review asked about (R20) and which is defined now, beside
*instance*. *Ruling* and *rulings* are Scryfall's word and §3.4's; *pooled*,
*producer*, *doubler*, *multiplier*, *pattern* and *template* are this
project's vocabulary since RC and RD (three of them are type names);
*asserted*, *carry*, *module*, *offered*, *nesting*, *guard* and the rest are
English.

##### Measured (2026-09-13, at landing and again after the review)

Four arms. `main` (4271b1d, rebuilt); **engine**, the branch with the cards
*unregistered*, so `main`'s registry and decks in both pools — the arm §11
item 80 made the standing engine reading, after the middle arm turned out to
read the `stress` decks; **registered**, the branch with the old pool;
**pooled**, as shipped (83 and 139). Two seats and four, 200 games / seed
12345, `plans/fuzz_ab.py`, one timing sitting per seat count after the review
and two before it.

**Engine and registered are `IDENTICAL` to `main` outside `=== Timing ===`
on `performance` at both seat counts** — line for line, once the two rows the
new binary prints are set aside. No pooled card creates a token, meets a
rider across a Reflection, or puts an exit beside an enters-with, so nothing
RE-4 or its review built runs in a measured `performance` game until Raise the
Alarm is pooled. CPU/game −0.4% and +0.1% at two seats (13.58 → 13.53 and
13.60 ms), −1.0% and +0.2% at four (52.04 → 51.54 and 52.14); `CPU/turn p50`
0.390 → 0.390 and 0.720 → 0.720. Flat, both signs, inside the spread; the
two sittings at landing read +2.1%/−1.3% and +1.0%/−0.6% the same way.

**The engine arm's `stress` reading is exact, and it separates the three
themes.** Two seats: 188 of 200 games byte-identical to `main`; eleven games
differing by exactly one `TokenCreated` line per Zombie Kalitas makes (26
lines: the +1 gather per creation, visible as `Layer frames` 7,166 → 7,116 —
the gather's walk of Kalitas — and rounding away in `Replacement gathers`,
1114 → 1113); and **one game re-routed**, seed 12441, Containment Priest and
Root Maze beside a Dryad Arbor — the one-exit prompt theme C stopped asking,
after which the random agent's stream is a different game (`Layer walks`
521 → 519, wins 103/97 → 102/98). Theme A re-routed none: the boards holding
an Alms Collector or a Notion Thief across from a Thought Reflection (games
150, 153, 168, 182) differ only by their creation lines, which is CR 614.5
giving the same answer whether a rider's draw carries its lineage or not on
every board short of the symmetric one. Four seats: `Layer walks` 1,240 →
1,240, gathers 2517 → 2513, frames 23,502 → 23,476, seats 91/53/41/14 →
92/54/39/14, every outcome row otherwise equal.

**The pooled arm is a re-record and a bigger board.** Two seats,
`performance`: `Replacement gathers` 1002 → 1043 per game — a creation and
two entries per Alarm, plus Parallel Lives' gather on each — `Layer walks`
373 → 386, `Memo hits` 59,005 → 64,893, avg turns 29.9 → 30.7, combats with
attackers 10.2 → 10.7, creatures died 7.0 → 7.5, total damage 57.8 → 69.3;
max turns 72 → 95, and that game is the p99 (43 → 78 ms). CPU/game +11.0%
this sitting, +12.9% and +9.0% the two at landing; `ms / 1,000 queries` +1.0%,
+2.7% and −0.8%; `CPU/turn p50` 0.390 → 0.400. Read as RE-3's finding was: the
walk did not get slower, there is more game — two 1/1s a cast are two
permanents that attack, block, die and are walked. Four seats, `performance`:
gathers 2102 → 2145, walks 838 → 839, avg turns 61.0 → 61.4, `Dependency
checks` 171 → 111 (a different board), CPU/game −4.5%, `CPU/turn p50` 0.720 →
0.740.

**Two rows are new** (the review's R12 and R15): `Replacement prompts`, the
CR 616.1 questions actually put to a player per game — 0.49 on `performance`
at two seats and 2.38 at four on the engine arm, 0.74 and 1.83 pooled, 2.54
and 23.80 on `stress` — the row `ordering_cannot_change_outcome` moves and
nothing else should; and `Max batch depth`, the deepest nesting any game
reached — **7** on `stress` at both seat counts, 6 on `performance`, across
1,600 games — which is what `BATCH_NESTING_LIMIT`'s 32 is headroom over, and
the number that makes it measured rather than magic.

**Reachability**, `--require`, 200 games. `performance`: Parallel Lives cast
**168**, resolved **168**, in **116 of 200 games (58%)**, copies/deck 1.54;
Raise the Alarm cast **201**, resolved **201**, in **140 (70%)**, copies/deck
1.49; board diversity 100%. `stress`: Divine Visitation cast **131**, resolved
**131**, in **98 (49%)**, 1.30; Bard, King of Dale **135** / **135** in **94
(47%)**, 1.29; diversity 99%. So a doubled creation decided and then its
entries — CR 616.1g in a measured game — is a board most games reach, and the
kind-changing substitution is reached in every other `stress` game.

**Outliers.** At landing the four-seat `stress` run overflowed the stack at
seed 12523 (§11 item 77) and ran one game to the turn limit (seed 12538, a
Circle of Protection: Red and Words of Worship stall); after the review the
four-seat arms hit no turn limit and no draw, and the two-seat `stress`
registered arms hit the limit twice in 200 (seeds 12386 and 12538) on the
same stall, with no token in either game's last four hundred events — RE-3's
and RD-3's cards and the random agent, on decks the two new registrations
reshuffled. Zero errors and zero panics on every arm, both pools, both seat
counts. Three shell runs at one seed, `--threads 1`, both pools, two seats and
four: `IDENTICAL` outside `=== Timing ===`.

**`PERFORMANCE_POOL` 81 → 83, every arm's header read, and both §3 tables
re-recorded twice** — at landing and after the review, since the pool and
then the engine moved.

#### RE-5 — counters, on permanents and players (CR 614.16's counter half, 122.1, 122.6, 122.6a; item 43, `backlog.md` §2.16) — ✅ landed 2026-09-13

*Evicted 2026-09-13 from `plans/replacement-architecture.md`, where the heading and a stub remain.*

**Builds:** decision 4 — the second door on `CounterChange`, `Amount` on an
entry's mods, `AddCounters.by` and its `CounterSubject`, item 43's `EnterMods`
player half with the merge keyed on `(kind, player)`, `CounterChange.by`, and
`PlayerState`'s kind → count map with CR 704.5c reading it. **Consumers:**

- **Doubling Season** — both abilities, registered whole now that RE-4 built
  the first. Tokens: Parallel Lives' def. Counters: `CounterChange { counter:
  None, adding: true, by: None }`, `Filter { ByController(You) }`,
  `Amount(Multiplier(2))`. Rulings, five, four are tests: *planeswalkers
  enter with double loyalty* → Loyalty Probe enters with 6, through the entry
  door; *permanents that enter with counters* → Chainbreaker's rust counters,
  and Master Biomancer's grant on a creature entering (the CR 616.2 ordering
  board, decision 4); *loyalty paid as a cost is not doubled* → `Cost::
  AddCounters` does not propose, and no `CounterChange` sees it — asserted
  against the RD-1 fixture; *two Seasons quadruple* → §10's
  `test_two_doubling_seasons_quadruple`, on both halves.
- **Hardened Scales** — "If one or more +1/+1 counters would be put on a
  creature you control, that many plus one +1/+1 counters are put on it
  instead." `CounterChange { counter: Some(PlusOnePlusOne), .. }`, `Filter {
  Creature ∧ ByController(You) }`, `Amount(Plus(1))` — RD-3's `Plus`, second
  kind. Rulings, three, all tests: *enters with that many plus one* → the
  entry door on a +1/+1 entry (Master Biomancer's grant); *you choose the
  order no matter who controls the sources* → Scales beside an opponent's
  Doubling Season, the affected permanent's controller asked; *each extra
  Scales adds one* → two rows, each once.
- **Vorinclex, Monstrous Raider** — "Trample, haste. If you would put one or
  more counters on a permanent or player, put twice that many … instead. If
  an opponent would put one or more counters …, they put half that many …
  rounded down." Two rows: `CounterChange { by: Some(You) }` +
  `Multiplier(2)` and `{ by: Some(Opponent) }` + `Halve(Down)`, both over
  `Filter { All }` + `Everyone` — decision 4's putter predicate on both
  subjects, and RD-1's rounding on a new kind. Rulings, three: *cares who is
  putting* → an opponent's Battlegrowth on your creature halves to 0 (a 0
  count meets the `RemoveCounters`-style no-op guard, not `never_happens` —
  CR 614.7a is written about damage and life gain, and the test says which
  guard fired); *122.6a's default* → the entry door with `by` unset reads the
  controller; *ordering* → as Scales. Its "or player" half is **built**: your
  Live Fast under your Vorinclex gets four energy.
- **Winding Constrictor** — "If one or more counters would be put on an
  artifact or creature you control, that many plus one of each of those
  kinds …. If you would get one or more counters, you get that many plus one
  of each of those kinds of counters instead." The second player-subject
  watcher, on `Plus(1)`, and the object half's "each of those kinds" is
  `Amount` applied per matching kind, which the entry door already does.
  Rulings, six, four are tests: *enters with that many plus one* → entry
  door; *multiple instructions in one effect each get plus one* → two
  `Primitive::AddCounters` in one resolution, two proposals; *two
  Constrictors: plus two* → two rows; *can't apply to itself or to anything
  entering with it* → RC-3's membership rule and RE-4's batch, asserted.
- **Live Fast** — "You draw two cards, lose 2 life, and get {E}{E}." The
  producer: `AddCounters { subject: Player(you), counter: Energy, n: 2, by:
  you }`, and every other instruction it carries exists (RE-2's draw, RA's
  loss). No rulings beyond energy's definition, which is the test: a player
  has a map, the map has a kind, and nothing else changes.
- **Primal Vigor** — "If one or more tokens would be created, twice that many
  … If one or more +1/+1 counters would be put on a creature, twice that many
  …" — `Everyone` on both halves, and the ruling "it doesn't matter who
  controls the tokens or the creature" is the four-player test: an opponent's
  Raise the Alarm makes four.

**`PERFORMANCE_POOL` +1, Hardened Scales**, predicted: a one-mana static that
opens the sweep on every `AddCounters` (Battlegrowth is in the pool) and every
counter-bearing entry (Chainbreaker, Master Biomancer's grants, Loyalty Probe
in the stress pool). Doubling Season is registered and not pooled — a
five-drop, and its token half would double the pool's Zombies, which is a
gameplay change the A/B should not carry with the engine change.

**Atoms:** `ATOM-122.6a-001` moves from the default half to a named-player test
(item 43's field, read by Vorinclex); `ATOM-122.1f-001` and `ATOM-704.5c-001`
(Phase 5-Pre, poison — read off the map now, and covered where they are); the
rest of §2.16 is thin under its phrasings, and the doubling tests say so.

##### Decided before writing, because the section left four open (2026-09-13)

**1. The entry door is the entry's mods, not a proposal the performer
contains.** CR 122.6 does not describe a second event: "putting counters on
that object ... refers ... also to an object that's given counters as it
enters the battlefield" folds the entry's counters into the one event the
entry already is, the way CR 614.1c folds "enters with" into it and RC-4b's
zone-change door reads the same entry as the move. So `EventPattern::
CounterChange` watches an `EnterBattlefield` whose `mods.counters` carries a
matching kind with one or more counters, and `Rewrite::Amount` over an entry
rewrites each matching kind's count in the mods — decision 4's reading, and
three things decide it against the contained shape. The performer's ordering
guarantee stays: a creature entering with +1/+1 counters is never a 0/0 on
the battlefield while a nested pipeline sweeps the board (`place_on_
battlefield`'s doc). Hardened Scales' ruling — "you choose the order to apply
those effects, no matter who controls the sources" — is one CR 616.1 step
over `EnterWith` and `Amount` members together, which a contained proposal
would split into two steps with the order fixed. And no new event line is
written for an entry, so the engine arm is predicted byte-identical to `main`
on both pools (Chainbreaker is pooled and enters with counters in most games).
The tests either way: Doubling Season's own ruling (Loyalty Probe enters with
**6**), CR 616.1g on Raise the Alarm under Master Biomancer and the Season —
the token half applied once at the creation, the counter half once at each
of the four entries, neither offered at the other's step — and CR 122.6a's
default read off the entry: an opponent's Loyalty Probe under my Vorinclex
enters with **1**, because the opponent puts its counters on.

**2. `CounterSubject { Object, Player }` on both counter actions, in
`events/event.rs` beside `DamageTarget`.** Counted before choosing:
`AddCounters` has six producers and `RemoveCounters` two, and the readers
that branch on the field are `subject_of`, `pattern_watches` and
`perform_action` — two arms each — plus `strip_prohibited_counters`'
synthetic event, the `RemoveCountersFromAffected` leg of `substitute`, the
new `Amount` leg, `display.rs` and three tests. The pair is one kind through
`CounterChange.adding`, so a subject on one and an id on the other would
make `subject_of` answer them differently; `GameEvent::CountersChanged`
carries the subject too, since CR 122.1's "on an object or player" is what a
"whenever you get" trigger and a "whenever a permanent gets" trigger will
read off it. A player removal performs (CR 701.2's as-much-as-possible, the
object arm's mirror) and has no producer until energy is paid.

**3. `PlayerState.poison_counters` is replaced, not aliased.** A `BTreeMap<
CounterType, u32>` — `CounterType` gains `Ord` so the map iterates in enum
order, process-independent, which `CLAUDE.md`'s determinism rule asks of any
collection reaching a count — and `CounterType::{Poison, Energy}`. Eight
readers and four test writes, all mechanical; CR 704.5c reads
`counter_count(Poison)`. An alias would be a second name for one fact.

**4. `by` is a `PlayerId` on `AddCounters`, an `Option<PlayerSet>` on the
pattern, and absent from `RemoveCounters`.** The section wrote `PlayerRef`,
and `PlayerRef::Opponent`'s own doc is "a targeted or otherwise identified
opponent" — one player. Vorinclex's "if an opponent would put" is any
opponent, `PlayerSet::Opponents` is exactly that, and `PlayerSet::contains
(you, putter)` already exists. Nothing prints a *remover*, so the removal
arm matches only a pattern that asks nothing. The putter is the
resolution's controller for a primitive and a rider, and for entry counters
it is CR 122.6a's default: the entry's `controller` field, which CR 616.1b
settles ahead of every 616.1e effect that could ask.

**5. Item 43's named half is not built, and the item closes.** CR 122.6a's
first sentence — the effect "may specify which player puts those counters on
it" — has no printed customer: three Scryfall queries on 2026-09-13
(`enters with ... You/An opponent/That player put`, `"enters with" "put on it
by"`, `counters on it as it enters` outside an "enters with") return nothing,
and the seven printed "would put one or more counters" watchers all read the
default. The door reads the default off the entry; the `Option<PlayerId>`
per counters entry and the `(kind, player)` merge key that item 43 sized are
recorded on `EnterMods::counters` for the card that prints one. `ATOM-122.
6a-001` is covered on its default half, and its own expected result is
corrected in the test's annotation: Doubling Season's counter half reads
"a permanent you control" (Scryfall, 2026-09-13), not who put them on, so
the effect the atom names as caring about the putter does not, and
Vorinclex is the reader.

**6. `backlog.md` §2.29: RE-5 adds no shape, and records what it makes
reachable.** Season beside Season is the multiplier bucket (suppressed,
correct — the product); Season beside Scales does not commute (a real
prompt, Scales' own ruling); Scales beside Scales and Scales beside
Biomancer's `EnterWith` at its second iteration are additive on one kind,
one outcome either way, and are asked — the table's next two rows, needless
and not wrong. Item 47's condition (c) fires here in the form the multiplier
shape can see: a pattern that reads an entry's mods. The re-derivation is
that the door reads two things a multiplier of one or more leaves invariant
— which kinds the mods carry, and whether each carries one or more — and a
member's `affected` filter over an entering permanent reads the CR 614.12
frame, which +1/+1 counters feed; so the multiplier clause asks
`affected_is_mods_invariant` of an entry's members, the clause the
`EnterWith` shape already asked. `reads_the_amount` stays `false` for
`CounterChange`: `Halve` can carry a kind from one to zero and is admitted
to no suppressed bucket.

**7. A player's counters have their own primitive, `Primitive::
GetCounters`.** Oracle's verb for a player is "get" ("you get {E}{E}", "that
player gets a poison counter"), and `Primitive::AddCounters` resolves its
recipient as permanents; one primitive answering for both would make
`EffectRecipient::Controller` mean two things. Resolved through
`resolve_player_for_self`, as `GainLife` is.

**8. "One or more" is read by the pattern, and a zero meets the performer.**
`pattern_watches`' `AddCounters` arm asks `n >= 1` — CR 614.16's own phrase,
the line `CreateTokens` draws with `!defs.is_empty()` — and the door asks it
of each kind. A count Vorinclex halves to zero matches nothing further and
meets `perform_action`'s no-op guard, not `never_happens`, whose rule is
written about damage and life gain; the test names the guard.

**9. The cost half is a ledger line.** `Cost::AddCounters` is unimplemented,
so Doubling Season's "loyalty paid as a cost is not doubled" cannot be
asserted against anything; when `backlog.md` §2.11 builds loyalty costs the
payment needs a fact on the event that says it is a cost — `LifeLossCause::
Cost`'s shape — so that CR 614.16's "the effect of a resolving spell or
ability" is what the pattern matches. `codebase-state.md`, "Found by RE-5".

**10. Sized:** types ~160 (the subject, the pattern's `by`, two counter
kinds, the primitive), engine ~330 (two performer arms, the door, the two
`Amount` legs, the player map and its CR 704.5c reader, the premise clause),
cards ~380 with a rulings pass each, tests ~950, fixture updates ~40 —
**~1,850**, at the band's top for RE-3's, RE-4's and RE-7's reason: the
rulings become tests. `PERFORMANCE_POOL` +1 as predicted. Trace page:
decided at the close.

##### The shape, as built

- `CounterSubject { Object(ObjectId), Player(PlayerId) }` in
  `events/event.rs` beside `DamageTarget`, on `GameAction::AddCounters`,
  `RemoveCounters` and `GameEvent::CountersChanged`. `subject_of` answers
  the pair through the one field; CR 616.1's chooser is the permanent's
  controller or the player. `AddCounters::by: PlayerId` is the resolving
  effect's controller (`Primitive::AddCounters`, `GetCounters`) and, for the
  synthetic event `strip_prohibited_counters` asks CR 101.2 about, the
  controller the permanent enters under. `RemoveCounters` has no `by`.
- `EventPattern::CounterChange { counter, adding, by: Option<PlayerSet> }`.
  Three arms in `pattern_watches`: an `AddCounters` — `adding`, `n >= 1`
  (CR 614.16's "one or more"), the kind, and `by.contains(you, putter)`; a
  `RemoveCounters` — `!adding`, the kind, and `by.is_none()`, since nothing
  prints a remover; and **the door**: an `EnterBattlefield` whose
  `mods.counters` carries a kind with `n >= 1` the pattern's kind admits,
  with the putter read as CR 122.6a's default, the entry's `controller`.
  `reads_the_amount` stays `false`, and the arm's doc says why "one or more"
  is not an amount.
- `Rewrite::Amount` gained two legs. Over an `AddCounters`, `counter_
  arithmetic` — `Multiplier`, `Halve`, `Plus`; the prevention arms and the
  life floor refused as the pairing errors they are; a count no `u32` holds
  refused rather than wrapped. Over an `EnterBattlefield`, the same
  arithmetic per kind in the mods the pattern matched — its kind, and its
  putter against the entry's controller — with a kind at zero leaving the
  mods, so the performer spends no CR 613.7c timestamp on nothing.
- `ordering_cannot_change_outcome`'s multiplier clause asks
  `affected_is_mods_invariant` of an entry's members. Item 47's condition
  (c) — a pattern arm that reads `mods` — fired from the multiplier side, and
  the test that names it is the `PowerLE` doubler beside a plain one: a real
  two-outcome prompt at the entry door, none over a proposal.
- `PlayerState.counters: BTreeMap<CounterType, u32>` replaces
  `poison_counters` — `CounterType` gains `Ord`, and `Poison` and `Energy` —
  with `counter_count`, `add_counters` and `remove_counters` (CR 701.2's
  as-much-as-possible, the permanent's mirror). CR 704.5c reads
  `counter_count(Poison)`. `perform_action`'s two counter arms take both
  subjects; a player removal announces only a transition.
- `Primitive::GetCounters(CounterType, AmountExpr)`, resolved through
  `resolve_player_for_self` — Oracle's "you get", one primitive because
  `AddCounters` resolves its recipient as permanents.
- Not built, each with its reason on the type: `EnterMods`' named putter
  (no printed customer, §11 item 83); a cost's counters as a cause on the
  event (`codebase-state.md` "Found by RE-5" item 129); the additive
  suppression shapes (`backlog.md` §2.29, §11 item 85).

##### As landed (2026-09-13)

**Built as decided, and the two-seat prediction held to the byte.** Four
commits before the docs — the vocabulary, the door and the legs, the six
cards, the pool — **+1,911 / −101** across 18 files against the ~1,850
sized: engine and types 403 against ~490, cards 506 against ~380, tests
1,002 against ~950 (34 in `phase_re5_integration_test`, thirteen of them
engine-only fixtures ahead of any card, the rest one ruling each). Zero
warnings; 1,420 tests. Three probes were written red first against the
vocabulary commit — the `Amount` arm refusing a proposal, an entry getting 2
and 3 where a doubler should make 4 and 6 — and went green with the door.
Two of the fixture tests were wrong as first written, and both were the
predicate being right: a `PowerLE` doubler beside a plain one at an entry is
a real two-outcome order, and an `EnterWith` beside a halving on one entry
is too; each is asserted as the prompt it is.

**Counted against the tree before writing, and the row held.** Six
`AddCounters` producers and two `RemoveCounters` producers, the three
readers that branch on the field, the template leg, the synthetic event,
`display.rs` and three tests — exactly the sites the design record listed.
`event_amount` and `never_happens` needed nothing: a count of counters is
not CR 615.5's "that much", and "one or more" is the pattern's question.
Eight `poison_counters` readers and four test writes, all mechanical.

**Glossary triage** (`check_glossary.py --suggest`): twelve candidates, one
coinage — *putter*, defined and watched. *Pooled*, *producer*, *multiplier*,
*watcher* and *mods* are this project's vocabulary since RC and RD;
*rulings*, *tested*, *asserted*, *beside* and *carry* are English.

**Trace page: no**, decided at the close and argued under "Trace-page
decisions" below.

##### Measured (2026-09-13, at landing)

Four arms, `plans/fuzz_ab.py`, 200 games / seed 12345, two seats and four:
`main` (4343eef, rebuilt in its worktree); **engine**, this branch with the
six cards *unregistered* — `main`'s registry in both pools, every header
read; **registered**, the old pool; **pooled**, as shipped (84 and 145).
Three timing rounds at two seats, two at four.

**The prediction held to the byte on every arm it could be read on.**
Engine and registered are **`IDENTICAL` to `main` outside `=== Timing ===`
on `performance` at two seats and at four**, and **the engine arm is
`IDENTICAL` on `stress` at both seat counts too** — every line, including
the two turn-limit games at two seats, which the engine change did not
re-route. Nothing in the old pool or in `main`'s registry watches a counter
event, so the door is a `pattern_watches` arm no def reaches, the two
`Amount` legs are never entered, the multiplier clause's entry test is never
asked, and the subject enum changes no proposal's count. CPU/game median
15.08 → 15.09 (engine, +0.1%) and 15.25 (registered, +1.1%) at two seats,
47.80 → 47.54 (−0.5%) and 47.06 (−1.5%) at four; `ms / 1,000 walks` the
same numbers; `CPU/turn p50` 0.400 → 0.400 and 0.710 → 0.710 — flat, both
signs, inside the spread. The registered arm on `stress` differs because the
registry *is* the deck there (139 → 145 cards, item 80's reason), and it
reads as a different board: the two turn-limit stalls reshuffled away, avg
turns 33.0 → 30.3, walks 572 → 481 at two seats.

**The pooled arm is a re-record, and the row that moved is the one the
section said would.** `Replacement prompts` 0.74 → **1.14** per game on
`performance` at two seats and 1.83 → **2.44** at four: Hardened Scales
beside a second Scales, and beside Master Biomancer at an entry's second
iteration — the additive pairs `backlog.md` §2.29 now lists, each asked and
each with one outcome. `Replacement gathers` 1043 → 1060 at two seats — the
sweep Scales opens on every `AddCounters` and every counter-bearing entry
while it is on the battlefield — and 2145 → 2145 at four, where the board
is bigger and the Scales rarer per gather. The rest is a different board:
avg turns 30.7 → 31.4, total damage 69.3 → 59.0 (a one-mana enchantment in
a nonland slot, and creatures that grow rather than trade), `Dependency
checks` 40 → 28, max turns 95 → 84 with the p99 game gone (76.9 → 53.9 ms).
CPU/game −0.4% at two seats and +1.3% at four, `ms / 1,000 queries` −2.4%
and −0.7%, `CPU/turn p50` 0.400 → 0.390 and 0.710 → 0.685: flat. `Max batch
depth` 5 → 6 on `performance` at two seats, 7 → 8 on `stress` at four —
still what `BATCH_NESTING_LIMIT`'s 32 is headroom over.

**Reachability**, `--require`, 200 games. `performance`: Hardened Scales
cast **226**, resolved **226**, in **136 of 200 games (68%)**, copies/deck
1.54, board diversity 100%. `stress`: Doubling Season cast 119, resolved 119, in **97 of 200 (48%)**, 1.29; Winding Constrictor cast 150, resolved 148, in **112 of 200 (56%)**, 1.23; Live Fast cast 133, resolved 119, in **91 of 200 (46%)**, 1.26; Primal Vigor cast 136, resolved 136, in **101 of 200 (50%)**, 1.33. Vorinclex cannot be
required by name — `--require` splits on commas — and is read through its
tests and the registered arm's `stress` decks instead.

**Outliers.** Zero errors and zero panics on every arm, both pools, both
seat counts; no turn limit on any arm at four seats, and the two at two
seats are `main`'s own (seeds 12386 and 12538, RE-4's stall), which the
engine arm reproduces line for line and the registered arm's reshuffled
decks do not reach. Three shell runs at one seed, `--threads 1`, both
pools, two seats and four: `IDENTICAL` outside `=== Timing ===`.

**`PERFORMANCE_POOL` 83 → 84, every arm's header read, and both §3 tables
re-recorded** — `plans/fuzz-record.md`, at the top.

##### Reviewed (2026-09-14) — theme A of `plans/handoffs/re-5-review.md`

**The named putter is built, and the pattern pair is two arms.** The owner's
review rejected item 43's close on an empty Scryfall query — the CR is the
customer, a printed card is the test (`engineering-practices.md` §4, the
rule this review adopted) — and pointed at Bold Plagiarist, whose trigger has
the *opponent* put counters on a creature they do not control: the putter is
an authored fact on a proposal too, which RE-5 had not looked for. Built as
the review file sized it: `EntryCounters { counter, n, by: Option<PlayerId> }`
rows on `EnterMods` merged on `(kind, putter)`, `EntryCountersTemplate` with
an `Option<PlayerRef>` resolved by `pipeline::putter_of`, the door and the
CR 101.2 check reading each row's putter ahead of the entry's controller,
and `by: Option<PlayerRef>` on `Primitive::AddCounters` and `GetCounters`
resolved by `resolve_putter`. `EventPattern::CounterChange { adding }` became
`AddCounters { counter, by }` and `RemoveCounters { counter }` — one arm per
`GameAction` variant, bearing the action's name as every other arm does
(the review's second pass; the first pass had coined `CountersPut`), and the
"`by` must be `None` on a removal" caveat gone with the test that asserted
it. The second pass also made the primitives' putter a plain `PlayerRef`
written as `You` by every printed one-shot, after `None | Some(You)` was
read as two names for one player. Four tests in: the named putter at an entry
read ahead of the controller (2 where the default would halve to none), one
kind from two putters as two rows under Vorinclex, Bold Plagiarist's shape on
a proposal, and a `Prevent` over `RemoveCounters`. 1,423 tests, zero
warnings; §11 item 83 rewritten, item 43 reopened and closed as built.

**Re-measured, four arms at two seats and four, and nothing moved.** Every
counter row is identical to the landing's run: engine and registered
`IDENTICAL` to `main` outside `=== Timing ===` on `performance` at both seat
counts, the engine arm `IDENTICAL` on `stress` at both, and the pooled arm's
50-game fixture tables byte-identical to the ones recorded on 2026-09-13 —
every registered producer writes the default putter, and a row keyed on
`(kind, None)` merges as a kind did. CPU/game +0.7% (engine), +0.1%
(registered), +0.8% (pooled) at two seats; −0.9%, −0.7%, +0.9% at four;
`CPU/turn p50` 0.400 → 0.410 and 0.730 → 0.720 — flat, both signs.
`deterministic: yes` on every arm. Hardened Scales `--require` 226 / 226 in
136 of 200, as at landing.

##### Reviewed (2026-09-14) — theme B, and the second pass on A

**The suppression predicate is the commutation table `backlog.md` §2.29
designed.** The owner's R1 asked why two Scales, and Scales beside Biomancer,
were asked when they commute; the answer RE-5 had given — recorded on §2.29,
the table wants its own session — became the session. `ordering_cannot_
change_outcome` now classifies each candidate by what its application does
to the event another could read (`Commuting`: multiplier, additive,
mods-adding, draw doubler, idempotent substitute, absorbing exit), each
carrying the counter kinds it touches, and suppresses the prompt when every
pair commutes on the kinds both touch — one board read, which kinds the
entry's mods hold now, and `classify` exhaustive over `Rewrite` and
`AmountRewrite` so the expiry conditions are a compile error at the table.
New cells with one outcome: two additive members; a plus beside an
`EnterWith` writing to a kind the mods already hold; arithmetic on disjoint
kinds; an exit beside arithmetic; and the sixth shape §2.29 named, Divine
Visitation beside Parallel Lives, whose RE-4 test now asserts no prompt.
Still asked, and tested as real: a multiplier beside a plus on a shared kind,
a multiplier beside any `EnterWith` it touches, a plus on every kind beside
an `EnterWith` writing a new kind, a halving beside anything.
`check_order_invariance` dispatches on the chosen member's cell and ran
under every suppression in the suite. 1,425 tests, zero warnings; §2.29
graduated. The second pass on theme A renamed the arms to the action's
names and made the primitives' putter a written `PlayerRef::You`.

**Re-measured, and the cells are rare in the pool.** Four arms, two seats
and four. `Replacement prompts` on the pooled `performance` arm: 1.14 → 1.14
at 200 games and 2.14 → 2.12 at 50 at two seats; 2.44 → 2.42 and 3.12 → 3.08
at four; `stress` at four 5.04 → 4.94. So the pool's prompts were not the
counter pairs the section had assumed: with two Scales and a +1/+1 event, or
two Biomancers beside a Scales, needed in one game, the cells fire a few
times in 200, and what the pool asks is two Guardian Seraphs (`PreventUpTo`
beside `PreventUpTo`, one outcome and the table's next cell — RD's arm, not
this review's) and devour beside Master Biomancer (opaque by design, since
it prompts and moves the board). The engine arm is `IDENTICAL` to `main` on
`performance` at both seat counts and on `stress` at two; at four seats on
`stress` it differs by the one new cell `main`'s registry can reach —
Divine Visitation beside Parallel Lives — prompts 11.28 → 11.27, gathers
2385 → 2385, total damage 158.2 → 158.3: a game or two re-routed. CPU/game
−0.4% (engine), −0.4% (registered), +1.0% (pooled) at two seats; −0.1%,
−0.1%, +1.4% at four — flat. Both `fuzz-record.md` tables re-recorded from
this run.

#### RE-8 — the producers (CR 701.9, 701.9b, 701.22) — ✅ landed 2026-09-14

*Evicted 2026-09-14 from `plans/replacement-architecture.md`, where the heading and a stub remain.*

**Builds:** decision 8, with the design check below overturning three of its
pieces — `Primitive::Discard(n, DiscardChooser)` with `ChoiceKind::Discard` (the
cleanup discard's, widened) and "at random" from `GameState.rng`; `by:
Option<SourceFilter>` on `ReplacementDef` rather than a `caused_by` on the
zone-change pattern; `GameAction::Scry { player, n }`, its arm,
`GameEvent::Scried`, `Primitive::Scry` and two scry choice kinds. **The
to-battlefield leg is not built** — every printed customer functions from the
hand, which needs critical-path item 6a; decision 3 below counts it.
**Consumers:**

- **Mind Rot** — "Target player discards two cards." The default chooser;
  the target's choice through `EffectRecipient::Target` on a player. Its test
  is Notion Thief's ruling from RE-2, now against a printed card: a
  draw-then-discard the Thief modified still discards.
- **Hymn to Tourach** — "Target player discards two cards at random." The
  second chooser, from the owned `rng` (`CLAUDE.md`: randomness is never
  ambient), so `tests/determinism_test.rs` covers it by construction.
- **Nephalia Academy** — "If a spell or ability an opponent controls causes
  you to discard a card, you may reveal that card and put it on top of your
  library instead of putting it anywhere else." `by`'s printed customer, and
  the one in the family that does not need the hand: a Land, so the
  battlefield sweep finds it and its `Filter` reaches a card in hand the way
  every `Filter` already reaches any object in any zone. `optional`, so it is
  a real CR 616.1 prompt. Its "you may reveal" is §2.9's information model
  and a no-op in an omniscient engine, recorded on the card.
- **Opt** — "Scry 1. Draw a card." The scry producer, and CR 608.2c's "A,
  then B" in one resolution.
- **Eligeth, Crossroads Augur** — "Flying. If you would scry a number of
  cards, draw that many cards instead. Partner." `EventPattern::Scry`, `You`,
  `Instead(DrawCards { n: ReplacedAmount, player: None })`. Partner is
  CR 702.124's deck-construction ability with no in-game text, so nothing is
  dead under the name. Its test is §10's acid test with a different producer:
  `test_opt_with_eligeth_draws_two_and_never_scrys` — Opt under Eligeth draws
  two and the log holds no scry — proves §4.1a's instruction split and the
  kind-changing `Instead` reading the event's amount; the Goggles of Night
  form waits for critical-path item 6, since Goggles scries from a *trigger*.
- **Dodecapod, Wilt-Leaf Liege, Loxodon Smiter, Nullhide Ferox** and
  **Obstinate Baloth** stay out with the entry template, under critical-path item 6a: their
  clause is on a card in *hand*, and `gather` has no source that asks one.
  **Library of Leng** and **Guerrilla Tactics** stay out for their own second
  facility — a hand size (`backlog.md` §2.15) and critical-path item 6 — and the board they
  make together is `plans/atomic-tests/supplemental-docs/603-2f-complexity.md`,
  whose discriminator is §2.9 rather than either.

**`PERFORMANCE_POOL` +2, Mind Rot and Opt**, predicted: the pool's first
discard outside cleanup and its first scry, so `ZoneChange { Discarded }` and
`Scry` become rows critical-path item 6 can read from a measured game, and the random agent
finally reaches CR 701.9b's choice. Hymn to Tourach, Nephalia Academy and
Eligeth are registered and not pooled — a second discard spell would double the
first's measurement, and the other two are a land and a six-drop with nothing
in the pool to work on.

**Atoms:** `ATOM-701.9b-001` (the random discard, whole — Hymn to Tourach, with
the "not invoked" half asserted as zero prompts); `ATOM-701.22a-001` (whole, on
a scry-3 fixture, since the ordering is what the atom is about and no
registered card scries more than one); `ATOM-701.22b-001` as `COVERS-PARTIAL`
— scry 0 announces nothing and looks at nothing, and "the trigger does not
fire" waits for critical-path item 6; `ATOM-701.9b-002` stays **uncovered**, its third
chooser having no card.

##### Decided before writing, because the section left five open (2026-09-14)

**1. The cause-side predicate is `ReplacementDef.by: Option<SourceFilter>`, and
it is not a field on the pattern.** The section wrote `caused_by:
Option<PlayerRef>` on `EventPattern::ZoneChange`; both halves are overturned,
and the readers were counted first. On the pattern it costs a parameter on
`pattern_watches` (two callers, `gather::applies_to` and
`restriction::predicate`) and two arms of it, since a zone-change pattern
watches an entry through a second door; on the def it costs one conjunct in
`applies_to`, one builder, one line in `ReplacementDef::new` — the struct is
built by a literal in exactly one place — and `pattern_watches` is not touched,
so the restriction file is not either. Six sites against five, which decides
nothing. **What decides it is that `Restriction::Event` reuses `EventPattern`
verbatim and already has a `by`.** A cause predicate on the pattern would give
a "can't" two of them, with one meaning: Tamiyo, Collector of Tales' "spells and
abilities your opponents control can't cause you to discard cards" would be
writable as the restriction's `by` or as the pattern's, and nothing would
choose. One quality, two spellings is the defect `ObjectFilter::Token`'s own doc
refuses. The second half of the argument is the growth contract: `EventPattern`'s
arms constrain *the event's fields*, and who controls the proposing spell is not
one — it is `ActionContext::resolution`, provenance rather than payload, which
is why CR 101.2's `by` sits outside the pattern on a restriction already.

`SourceFilter` over `PlayerRef` follows from the same sentence. CR 101.2's "by"
*is* this type; `PlayerRef::Opponent`'s own doc is "a targeted or otherwise
identified opponent" — one player — where `SourceFilter::ControlledBy(Opponent)`
already means "any opponent of the effect's controller", evaluated by
`cause_matches`. So Dodecapod's "a spell or ability an opponent controls causes
you to discard" and Tamiyo's "can't cause you to discard" become literally one
predicate with one evaluator, which is `cant-effects-architecture.md` §3.1's
claim that a "can't" is the same predicate as the effect it withholds, said
about the cause axis for the first time. `cause_matches` moves out of
`restriction/predicate.rs` onto the type as `SourceFilter::matches`, so both
readers ask the enum rather than one file asking the other.

**2. A causeless event matches no `by`, and the rule is already written.** The
cleanup discard is CR 514.1's turn-based action, proposed under
`ActionContext::new` with `resolution: None`, so `cause` is `None` and
`cause_matches` returns `false` for any `Some(by)` — never `true`, never
"however caused". That is the right answer and not an omission, and it is the
answer the existing doc already argues for Sigarda and CR 704.5's sacrifices:
a turn-based or state-based action has no controller, so no `SourceFilter`
matches one. Read forward it says Nephalia Academy does not redirect a card
discarded to hand size, and read backward it says a hypothetical "spells and
abilities your opponents control can't cause you to discard cards" does not
exempt its controller from CR 514.1 — which is Tamiyo's printed behaviour, and
the reason Library of Leng has to print "you have no maximum hand size"
separately. RE-8 adds no rule here; it adds a second customer for one.

**3. The to-battlefield leg does not ship, and neither do five of the section's
six consumers — because they function from the hand and `gather` has no source
that asks a card off the battlefield.** Found by the §8 rules pass: CR 701.9's
neighbours are silent, and the rule that watches it from the ability side is
CR 113.6, whose 113.6m puts an ability whose effect moves the object it is on
out of a zone in *that* zone. `gather`'s five sources are the CR 903.9b rule,
the entering permanent (CR 614.12), the battlefield sweep, the counters and the
registry; a card in hand is in none of them, so Dodecapod's and Wilt-Leaf
Liege's clauses would be dead text, and `replacement_ability_sources` is
populated at ETB, so no gate would see them either. That facility is
**critical-path item 6a**, sized at §11 item 9, and item 9 says in as many words
that it is not this phase's to build.

**What that costs, counted rather than assumed.** Scryfall, 2026-09-14: eleven
cards say "onto the battlefield instead", six are a sorcery's own instruction,
and **every replacement effect among them is this one family** — Loxodon Smiter,
Nullhide Ferox, Obstinate Baloth, Wilt-Leaf Liege, with Dodecapod's variant
wording beside them. So `GameActionTemplate`'s entry arm would have no printed
customer at all, which is exactly the exception `engineering-practices.md` §4
carves out of its own ship-the-arm rule: *an arm whose customer needs a facility
the PR does not have is a ledger line pointing at that facility.* It is one,
under critical-path item 6a. **`by` keeps a printed customer without the facility**: Nephalia
Academy is a Land, so it is on the battlefield sweep, and its `AffectedSet::
Filter` reaches a card in hand the way every `Filter` already reaches any object
in any zone. Its one gap is "you may reveal that card", which is §2.9's
information model and a no-op in an engine whose every decision provider sees
the whole board — the same no-op CR 701.22a's "look at the top N cards" is in
this same PR.

**4. Scry is an event, it announces one, and CR 701.22b is a `never_happens`
arm.** `GameAction::Scry { player, n }`, subject the player, because Eligeth,
Crossroads Augur replaces it; its performer asks the choices and reorders the
library in the arm, proposing nothing, since CR 701.22 moves no card between
zones. What it announces is `GameEvent::Scried { player, n }`, and CR 701.22d is
why it must: "an ability that triggers whenever a player scries triggers after
the process described in rule 701.22a is complete, **even if some or all of
those actions were impossible**" — so a scry against a one-card library is still
a scry, and critical-path item 6 has a line to read that no zone change would have given it.
`event_amount` reports `n`, which is CR 615.5's "that much" about a scry and is
what `AmountExpr::ReplacedAmount` reads for Eligeth; nothing prints a rider over
a scry, and the arm is written because the match is exhaustive and a
fallthrough would report "no meaning outside a CR 615.5 rider" from inside one.

**`never_happens` gains its fourth arm and RE-2's `DrawCards { n: 0 }` still
does not, and the difference is the rules' own words.** CR 701.22b: "If a player
is instructed to scry 0, **no scry event occurs**. Abilities that trigger
whenever a player scries won't trigger." That is CR 614.7a's shape exactly —
CR 120.8's "does not deal damage at all" and CR 119.10's "no life gain event
would occur" — where CR 121.2 says only that the player performs that many
individual draws, which is RE-2's decision 3 and stands. It is load-bearing
twice: a scry 0 that reached the loop would let Eligeth turn it into a draw of
zero, and it would put a `Scried` line in the log that CR 701.22b says must fire
nothing.

**The choices are CR 701.22a read literally, and CR 102.2 keeps them off the
prompt when they are forced.** One `pick_n` over the cards looked at — which go
on the bottom, bounds `(0, k)`, "any number of them" — then one
`choose_ordering` per group that holds two or more, "in any order" twice. Opt is
scry 1, so it asks once and never orders; `ATOM-701.22a-001`'s scry 3 is a
fixture, and it is the atom that wanted the ordering built rather than deferred.
`k` is the cards actually there, not `n`: CR 701.22a looks at the top N and a
short library has fewer, which 701.22d's "even if some or all of those actions
were impossible" then covers.

**5. Discard is one batch of N members, and CR 701.9b's chooser is a field on
the primitive.** `Primitive::Discard(AmountExpr, DiscardChooser)` with
`DiscardChooser::{Affected, AtRandom}` — the rule's first two shapes, and its
third ("allow another player to choose") waits for its card, so the enum has two
arms and not three. One batch on `Primitive::Mill`'s argument: the cards are
chosen and then move together, and CR 603.2c wants "whenever one or more cards
are discarded" to fire once for Mind Rot. Each card is still its own member and
its own subject, which is what Library of Leng's ruling describes from the other
side — *"the Library allows you to decide whether or not to use it on each of
the cards"* — one CR 616.1 decision per member, in the batch's order, which is
also that ruling's "you get to decide the order the cards are placed on the
library". A hand shorter than the amount discards what is there (CR 101.3), the
way `Primitive::Mill` already mills what is there.

**The chooser is not a `ChoiceKind`.** `ChoiceKind::DiscardToHandSize` becomes
`ChoiceKind::Discard { source: Option<ObjectId> }` — one kind for one question,
with `None` naming CR 514.1's turn-based action and `Some` the spell or ability
that caused it, which is the "why am I being asked this" field
`ChooseAuxiliaryZoneChange` and `ApplyOptionalReplacement` already carry.
`AtRandom` asks nothing at all, which is `ATOM-701.9b-001`'s own expected
result.

**6. `GameActionTemplate::DrawCards.n` becomes a `TemplateAmount`.** Eligeth is
"draw **that many** cards instead", which is `ReplacedAmount` read off the scry
through `event_amount` — decision 2's amount on decision 1's template, as the
section wrote. Thought Reflection and Notion Thief become `Fixed(2)` and
`Fixed(1)`, and `pipeline::draw_doubler_commutes` reads the `Fixed` arm and
classifies a `ReplacedAmount` as no class at all: an identity substitution
commutes with anything, but nothing prints one, and a member with no class keeps
CR 616.1's question, which is the safe direction for a premise.

**7. Hymn to Tourach draws from `GameState.rng`, and the check that it does is
`tests/determinism_test.rs` by construction.** `GameState::rng` is seeded by
`Game::reseed`, and the determinism test builds its deck from
`default_registry`, so a registered Hymn is in every deck of every seeded game
it plays. Two runs in one process share a `RandomState` but **not** a
`rand::rng()` thread-local — that generator advances between them — so a random
discard drawn from ambient randomness diverges on the second run and the test
fails on the log comparison. The three shell `fuzz_games` runs at one seed are
the end-to-end half, with `--require "Hymn to Tourach"` forcing a copy into
every deck so the path is walked rather than hoped for.

**8. `--require` becomes repeatable, and that is the ten-line fix.** The flag
splits its argument on commas, so "Eligeth, Crossroads Augur" cannot be named —
Vorinclex, Monstrous Raider hit the same wall in RE-5 and was read through its
tests instead. A second phase reading a consumer's reachability off its tests is
the tell. The fix is not a new separator, which would have to be documented and
would leave two spellings: **the flag accumulates**, so `--require A --require
"B, C"` is the union of both, every existing invocation means what it meant, and
a name with a comma is said on its own.

**9. Sized:** types ~200 (the `Scry` action and pattern, the `Scried` event, the
chooser enum, `by` and its builder, two choice kinds, the template amount),
engine ~430 (two performers, `ask_discard`/`ask_scry`, the `by` conjunct with
its cause threaded through `gather`, the `Scry` substitute arm, the three
exhaustive matches, `cause_matches` moved), cards ~330 with a rulings pass each,
tests ~600, harness ~15 — **~1,575**, near the band's floor and the smallest RE
PR since RE-7. `PERFORMANCE_POOL` +2, Mind Rot and Opt. Trace page: decided at
the close.

##### As landed (2026-09-14)

**Built as decided, with the design check's three overturns intact.** Two
commits of code before the docs — the vocabulary with both performers, then
the five cards with their tests — **+1,737 / −115** across 23 files against
the ~1,575 sized: types **195** against ~200, engine **445** against ~430,
cards **371** against ~330, tests **710** against ~600, harness **16** against
~15. Zero warnings in **both** profiles, which the release build had not been
getting; 1,449 tests, 24 of them in `phase_re8_integration_test`.

**Counted against the tree before writing, and one number was wrong in the
cheap direction.** The three exhaustive matches over `GameAction` were exactly
three (`subject_of`, `event_amount`, `perform_action`) and `display.rs` was the
compiler-forced fourth, as RE-4 found. `pattern_watches` took one arm rather
than one arm plus a field, because the cause predicate moved off the pattern.
`ReplacementDef` is built by a **struct literal in exactly one place** — its
own `new` — so a field on it cost one line and a builder where the sizing had
allowed for more. What the sizing missed is that `substitute`'s two draw legs
had to become one arm with a nested match: `match (template, event)` moves the
event into a tuple, so an arm that destructures it cannot also hand it to
`template_amount`.

**Three things were decided in the writing rather than in the design check.**

- **CR 514.1's cleanup discard is one turn-based action over N cards, and it
  was N of them.** The rule says the active player "discards **enough cards**
  to reduce their hand size to that number"; the engine asked one card at a
  time in a `while` loop and moved each through its own `change_zone`, so a
  hand of ten made three batches where CR 603.2c's "whenever one or more cards
  are discarded" should see one. Under thirty lines with a fixture that proves
  it, which is `codebase-state.md`'s rule for fixing rather than recording —
  and it is the only thing that moved this PR's engine arm, which a fifth
  binary then attributed exactly.
- **`ChoiceKind::Scry`'s `source` is an `Option`, like the discard's.** Every
  scry today is a resolving effect's, but `ActionContext::resolution` is an
  `Option` and inventing an id to fill the field would be a worse answer than
  saying so.
- **A random discard returns its cards in hand order**, as the chosen discard
  does. CR 701.9b chooses *which* cards and no rule in CR 701.9 gives the
  order they reach the graveyard in, so the two choosers agreeing is worth
  more than an order no rule names.

**Glossary triage** (`check_glossary.py --suggest`): two candidates, *agreeing*
and *shorter*, both ordinary English and neither a term of art. Two British
spellings in the new card file were caught by the gate and fixed.

**Atoms.** `ATOM-701.9b-001` **whole** — Hymn to Tourach, with the atom's own
"P0's `choose_discard` is NOT invoked" asserted as a `ScriptedDecisionProvider`
with an empty queue. `ATOM-701.22a-001` **whole**, on a scry-3 fixture: the
atom's board is five cards, one to the bottom and two reordered on top, and no
registered card scries more than one, so it is built with a fixture and says
so. `ATOM-701.22b-001` **partial** — scry 0 looks at nothing, asks nobody and
writes no `Scried` line, which is the half CR 701.22b is written about; "the
trigger does not fire" is claimable against the same board the day critical-path item 6
lands. `ATOM-701.9b-002` stays uncovered: its third chooser has no card.
`specdb owed` is **9 before and 9 after** — none of these is in a shipped
phase, so the gate cannot move, and Phase 8 goes 9 full to 11 full plus 1
partial.

##### Measured (2026-09-14, at landing)

**Five arms, not four, and the fifth is what makes the reading exact.**
`plans/fuzz_ab.py`, 200 games / seed 12345, two seats and four: `main`
(71f28c1, rebuilt in its worktree); **engine**, this branch with the five
cards unregistered — `main`'s registry in both pools, every header read (84 /
145); **registered**, the old pool (84 / 150); **pooled**, as shipped (86 /
150); and **engine-oldcleanup**, the engine arm with CR 514.1's `while` loop
restored.

**`engine-oldcleanup` is `IDENTICAL` to `main` outside `=== Timing ===` on
both pools at two seats and at four.** So the prediction held on the part it
was about: two new producers, a third clause in `applies_to`, a new
`GameAction` variant with its three exhaustive arms and a `TemplateAmount` on
a template cost a board with nothing watching them **exactly nothing**, to the
byte. Every counter the engine arm moves is CR 514.1's reshape — one prompt of
N where there were N prompts of one, which spends the random agent's RNG
differently from that turn on. Counted game by game: **23 of 200 on
`performance` and 29 of 200 on `stress`** reach a cleanup discard of two or
more and re-route from there. The aggregates it moves are tenths: avg turns
31.4 → 31.3, gathers 1060 → 1052, walks 384 → 383, prompts 1.14 → 1.16 at two
seats; at four, gathers 2150 → 2142 and walks 828 → 825.

**The pooled arm is a re-record and a smaller board.** Two nonland cards in a
36-slot deck dilute what was there: `Replacement prompts` 1.14 → **0.57** per
game on `performance` at two seats — Hardened Scales meeting a second Scales
less often, not a prompt that stopped being asked — with gathers 1060 → 1018,
walks 384 → 373 and avg turns 31.4 → 30.4. At four seats the same shape,
prompts 2.42 → 1.95 and gathers 2150 → 2135. On `stress` the five cards are
in every deck and the board changes much further: gathers 1048 → 1058 at two
seats and 2345 → 2213 at four, prompts 1.40 → 1.83 and 5.64 → 6.79.

**CPU is flat or down on every arm, both seat counts.** Two seats, medians of
three interleaved rounds: `main` 16.65 ms, engine 16.26 (−2.3%), registered
16.40 (−1.5%), pooled 15.16 (−8.9%); `CPU/turn p50` 0.430 → 0.440 / 0.440 /
0.430. Four seats, medians of two: `main` 55.07 ms, engine-oldcleanup +0.7%,
engine +0.8%, registered −2.0%, pooled −2.0%; `CPU/turn p50` 0.775 → 0.785 /
0.770 / 0.770 / 0.805. The pooled arm's −8.9% at two seats is the shorter game,
not a faster walk: `ms / 1,000 queries` is −1.6%. `deterministic: yes` on every
arm at both seat counts.

**Reachability**, `--require`, 200 games. `performance`: Mind Rot cast **173**,
resolved **172**, in **129 of 200 games (64%)**, copies/deck 1.43; Opt cast
**210**, resolved **209**, in **139 of 200 (70%)**, 1.52 — the highest-reaching
pooled pair since Raise the Alarm, and both are cheap. `stress`: Hymn to
Tourach cast 149, resolved **146**, in **102 of 200 (51%)**, 1.28 — and the
gap between those two numbers is what the review caught: the first reading was
129, and the missing seventeen had resolved perfectly well under a Leyline of
the Void, which replaced CR 608.2m's move to the graveyard so the spell left
the stack as `Exiled` rather than `Resolved`. The harness counted the cause;
it counts "left the stack and was not countered" now (§11 item 91), and
reading the same log is what found §11 item 90 beside it. Nephalia Academy
cast 221, resolved 221, in **149 of 200 (74%)**, 1.31 — it is a land, so every
deck runs it; **Eligeth, Crossroads Augur** cast 121, resolved 121, in **99 of
200 (50%)**, 1.23, which is the first reachability row this project has for a
card whose name contains a comma. The harness change is what bought it.

**Outliers.** Zero errors, zero panics and **zero turn limits** on every arm,
both pools, both seat counts. The longest `stress` game at two seats is
`main`'s own 155 turns, which the engine arm reproduces exactly and the
registered decks reshuffle away (max turns 78). Three shell runs at one seed,
`--threads 1`, both pools, two seats and four: `IDENTICAL` outside
`=== Timing ===`.

**`PERFORMANCE_POOL` 84 → 86, every arm's header read, and both §3 tables
re-recorded** — `plans/fuzz-record.md`, at the top.

##### Trace page: no, decided at the close (2026-09-14)

§7 asks for a page when a phase changes *how* a read is answered. RE-8 changes
what is **proposed** (two producers where there were two `Err`s), adds one more
conjunct to a predicate that already ran on every gather, and changes the
*batching* of one turn-based action. None of those is a read taking a different
path: `ReplacementDef::by` is a field comparison against a value
`ActionContext` has carried since RA, and the scry performer reads no board at
all — it looks at a `Vec` and writes it back.

The one candidate the phase produced is Eligeth: an `Instead` that changes an
event's **kind** and sizes the substitute from the replaced event's own amount.
Both halves are already on a page — RE-2's walks a kind-changing `Instead`
whose output is then decomposed, and `TemplateAmount::ReplacedAmount` is
RE-3's Tainted Remedy. What is new is the producer, and a new producer is a
new *event*, not a new read. Recorded because the phase produced a candidate
and declined it; the argument shares the shape of the five before it, under
"Trace-page decisions" below.

#### RE-10 — extra phases, and the turn plan (CR 500.8, 500.11, 505.1) — ✅ landed 2026-09-14

*Evicted 2026-09-14 from `plans/replacement-architecture.md`, where the heading and a stub remain.*

**Added at RE-1's review (2026-09-11), on §11 item 49.** Decision 6 pulled
CR 500.7 into a skips PR so that `backlog.md` §2.17 "does not rewrite
`advance_turn` a second time"; that holds for extra *turns* and not for extra
*phases*, because RE-1's cursor names a phase **type** and a turn can hold two
combat phases. This PR is that cursor, its producer, and the card. It is a
tenth RE PR rather than a rewrite of RE-1 because **it adds no event kind** —
the proposal is still `BeginPhase { phase, player }`, unchanged.

**The design, in four decisions.** Sized after the other nine, so it carries
its own rather than pointing at "The design check".

**1. The cursor becomes an index into a per-turn plan, and `next_phase`'s chain
goes.** `GameState.turn_plan: TurnPlan` is CR 500.1's five phases, seeded by
`GameState::new` and rebuilt by `on_turn_begin` — so a **skipped turn builds
no plan**, which is CR 614.10a for free — and `drain`'s cursor becomes
`Option<usize>` into it. `TurnUnit::Phase` carries the index. This finally
spends `state::game_state::next_phase`'s pre-RE-1 TODO, which described this
type by name and which RE-1 rewrote into a pointer.

**2. "After this phase" is an insertion at the cursor, and CR 500.8's ordering
falls out of it.** `Primitive::ExtraPhases(Vec<PhaseType>)` splices at
`cursor + 1`. *"If multiple extra phases are created after the same phase, the
most recently created phase will occur first"* is then the splice's own
behaviour — a second insertion at the same index pushes the first later — with
no comparator anywhere, exactly as CR 500.7's "most recently created turn"
turned out to be `Vec::pop`.

**3. An inserted main phase is `PhaseType::Postcombat`, and the choice is
unobservable.** Checked rather than assumed: **every production reader of the
two main types matches them as one arm** — `cast.rs:659`, `zones.rs:190` and
`oracle/legality.rs:50`, each asking "is this a main phase". The CR does not
name an additional main phase either way; CR 500.8 says only "directly after
the specified phase". The axis a later card would grow is a field on the plan
entry, and its customer is the first card that prints a distinction between the
two main phases the engine must keep.

**4. CR 500.9 and 500.10 stay out, and the field they want is named.** An extra
*step* needs a plan entry that overrides its phase's natural step list —
500.10's "any other steps that phase would normally have are skipped" **is**
that override, and it is one `Option<Vec<StepType>>` on `PlannedPhase`. Its
only producer is Obeka, Splitter of Seconds, whose ability is **triggered**: it
cannot land before item 6 whatever RE does. So the field waits, on "an arm the
pipeline cannot apply is worse than a missing one", and this is the paragraph
that says where it goes.

**Builds:** `TurnPlan` and `PlannedPhase`; `GameState.turn_plan`; `drain`'s
index cursor and `next_turn_unit` reading the plan; `next_phase`'s chain
deleted; `Primitive::ExtraPhases` with its splice; `Primitive::Untap` gaining
the `EffectRecipient::FilteredPermanents` arm that `DealDamage` and
`CreateReplacement` already have. **Consumers:**

- **Aggravated Assault** — {2}{R} Enchantment. *"{3}{R}{R}: Untap all
  creatures you control. After this main phase, there is an additional combat
  phase followed by an additional main phase. Activate only as a sorcery."*
  The producer, and **the only one of the 46 in reach whole**:
  `ActivationRestriction::OnlyAsSorcery` exists and is enforced
  (`cast.rs:345`, where Bonesplitter's Equip is its current customer), and its
  untap is the new recipient arm. Seize the Day needs flashback, World at War
  needs rebound *and* "creatures that attacked this turn", Relentless Assault
  needs the second of those, and the rest are triggers. Rulings pass at
  registration, in the card file's doc comment.
- **Moment of Silence** — registered by RE-1, and this is what gives its first
  ruling a board the engine produces: *"if they manage to have two combat
  phases, then only their next one combat phase is skipped."* RE-1 tests it
  against a second `BeginPhase { Combat }` proposal the fixture makes by moving
  the cursor and says so; RE-10 deletes the cursor move. **✅ landed
  2026-09-14**: both of Moment of Silence's first two rulings moved to
  `phase_re10_integration_test`, against an Aggravated Assault that makes
  the second combat phase for real.

**Tests:** two phases inserted after the main phase the ability resolved in, in
that order; two activations, the second's phases taken first (CR 500.8); combat
state reset between two combat phases in one turn — `on_phase_end(Combat)`
already clears all five fields, and the test is what keeps that true; mana
emptying at each inserted phase's end (CR 500.5); a turn's position count with
and without; Moment of Silence's ruling without the fixture; four players.

**Atoms: none, and that is a gap this section names rather than absorbs.**
`session-4.md` files 500.8, 500.9 and 500.10 DEFERRED with no atom ids and
assigns them to *Phase 9*, whose stated content is formats and multiplayer —
which reads like the era's `TurnPlan` TODO rather than a judgement. `specdb
owed` therefore cannot move either way. **CR 500.8 should have an atom**, and
authoring one is the corpus's own work (`session-4.md`), not this PR's.

**`PERFORMANCE_POOL` +0 predicted, and the A/B decides.** Aggravated Assault is
{2}{R} plus a {3}{R}{R} activation — eight mana to use once — and every
activation makes the game *bigger*, which is the shape §3.1a rejected Altar's
Reap for at +20.2%; the reachability answer is a `--require` row, as it was for
Circle of Protection: Red. The engine's change needs no pooled card at all: the
plan replaces the chain on **every turn of every game**, so the middle arm
walks it unforced. Predicted `Replacement gathers`, `Layer walks` and every
gameplay row **flat**, and CPU flat or slightly down — an index beats a chained
`match` per unit. **If the middle arm is not flat, the cursor change did
something it should not have**, and that is the whole reading of this PR's A/B.

**Measured size:** ~700 engine and cards, ~400 tests ≈ **1,100–1,300** — under
the band's floor, like RE-7's 800–950, and for the same reason: one type, one
cursor, one producer, one card.

**One naming decision to make before the type is written**, found by RE-1's
glossary pass: **the crate already has a plan.** `plan_payment`, `pay_with_plan`
and `plan_and_pay` are the cost system's, and `planned_sacrifices` sits one
letter from `PlannedPhase`. That is the `ToSource` / `ToSourceController` shape
the glossary's polysemy gate exists for, caught this time *before* the build
rather than mid-PR. Keep `TurnPlan` — it is the name
`state::game_state::next_phase`'s own TODO used and the payment one is a local
idiom rather than a type — and **land the collision as `plan`'s two numbered
senses in `plans/glossary.md`** with this PR. It cannot be added earlier: the
gate asserts every defined term resolves in `mtgsim/src`, and `TurnPlan` does
not exist yet.

**Three things this section and the row say that the tree does not**, checked
against `6e5d76a` on 2026-09-14 before any of it was trusted — the §13c
convention, and all three change a number above.

1. **`next_phase` has one production caller, not "~8 readers".** The row counted
   eight *mentions*: the definition, one `use`, the one call
   (`turns.rs:531`) and **five asserts inside a single unit test**
   (`game_state.rs:2250–2254`). Deleting the chain is one call site and one
   test function, not eight migrations.
2. **`next_phase`'s wrap arm is already dead code.** `next_turn_unit` returns
   `TurnUnit::Turn` for the ending phase *before* it asks what follows, so
   `PhaseType::Ending => PhaseType::Beginning` — the arm whose comment says
   "wraps to next turn" — is reached only by the unit test in 1. The plan's
   "past the last entry is `TurnUnit::Turn`" replaces a branch production never
   took, which is why deleting the chain removes a turn-boundary rule from two
   places and leaves it in one.
3. **"It re-counts nobody's fixtures" is false, and the number is 46.** §9's
   ordering paragraph says RE-10 changes `advance_turn`'s *cursor* and not a
   turn's *shape*, "so unlike RE-1's CR 508.8 refusal it re-counts nobody's
   fixtures". That is true of turn shape and false of the cursor: **46 sites
   assign `GameState.phase` by hand** (28 through `Phase::new`, 18 as struct
   literals), and **19 of them then drain** — `test_support.rs` 1,
   `phase_lg_integration_test` 1, `phase_re1_integration_test` 10,
   `phase_re2_integration_test` 1, `phase_re7_integration_test` 6. A cursor
   those sites do not write is a cursor they desync from, and every one of
   those 19 would advance from wherever the cursor happened to be rather than
   from the phase the fixture set. Decision 1b is what that costs, and it is
   the difference between this PR's prediction and its size.

##### Decisions — what §9 left open, numbered by what is here

**1a. `TurnPlan` is a field on `GameState`, it carries its own cursor, and the
drainer maintains both — RE-1's `turn_queue` precedent, one level down.**

The plan does not outlive the turn *as a fact* — `on_turn_begin` rebuilds it,
so a skipped turn builds none and CR 614.10a is free, exactly as §9 says — but
the **field** outlives it, because `advance_turn` returns between every two
units and the cursor has to survive that return. Today it survives as
`self.phase.phase_type`, which is the thing CR 500.8 makes ambiguous.

The cursor is written by the drainer rather than by the `BeginPhase` performer,
and that is not a chokepoint exemption: it is the sentence RE-1 already wrote
about `turn_queue` and `turn_rotation` — *"the schedule is read and consumed
here, not in a performer … neither is state a CR 614 replacement effect or a
CR 603 trigger can see, and both have to be spent whether or not the turn
begins."* A cursor is schedule, not board. It also keeps §9's "**it adds no
event kind**" promise literally true: `GameAction::BeginPhase { phase, player }`
is untouched, and so is its pattern arm, its event and its display line. The
alternative — an index on `Phase`, filled by the performer — needs the index on
the action, which is the event change this PR exists not to make.

**1b. The 46 hand-written positions go through one seam,
`GameState::set_position(phase_type, step)`, and the seam is the whole of what
finding 3 costs.** It writes `self.phase` *and* seeks the cursor to the first
plan entry of that type. All 46 migrate, not the 19 that drain: two ways to
move the position by hand is how the twentieth fixture desyncs silently, and a
seam only some callers use is not a seam. "First entry of that type" is
well-defined for every fixture in the tree, because the only fixture that
wanted a *second* combat phase is the one this PR deletes
(`a_phase_skip_cast_during_combat_is_spent_on_the_next_combat_phase`,
RE-1's, which fakes it by moving the cursor) and Aggravated Assault replaces.

**2. `PlannedPhase` holds a phase and not its steps, and the step chain
survives untouched.** Decision 4 above already settled this without saying it
was a decision: CR 500.10's override is `Option<Vec<StepType>>` on
`PlannedPhase` and it waits for item 6, so a plan that held steps today would
hold them as a copy of `initial_step`/`next_step` with no producer able to make
it differ — a second spelling of the chain, which is the thing decision 1
deletes the first spelling of. So `PlannedPhase { phase_type }`, one field,
and `initial_step`/`next_step` keep answering. The type exists rather than the
plan being a bare `Vec<PhaseType>` because both this section and `backlog.md`
§2.17 name it as the home 500.10's field goes on, and a named home that does
not exist is where a field lands somewhere else.

**3. `Primitive::ExtraPhases(Vec<PhaseType>)` splices a list, and one
resolution is one splice.** Aggravated Assault creates two phases —
*"an additional combat phase followed by an additional main phase"* — in one
resolution, and CR 500.8's ordering only works if they go in together: the
list keeps the printed order inside one effect, and a *second* resolution
splicing at the same index pushes the first pair later, which is
*"the most recently created phase will occur first"* with no comparator, as
decision 2 says. A single-phase variant would make Aggravated Assault two
splices and the printed order an accident of which one ran second.

**4. The `Primitive::Untap` arm rides, and its card is Aggravated Assault.**
It is not speculative surface: the card's own first sentence is *"Untap all
creatures you control"*, and without the arm the card is not registerable, so
this PR would ship its producer and no consumer. The arm is
`EffectRecipient::FilteredPermanents` resolved the way `DealDamage` resolves it
(`resolve.rs:218`) — `battlefield_ids_ordered`, filtered, **one
`execute_actions` batch**, because CR 701.26b's untaps here are simultaneous
and CR 603.2c's "whenever one or more permanents untap" reads the batch. The
existing target path joins the same batch rather than keeping its
`execute_action` loop, which is the chokepoint rule's "a simultaneous rule
needs `execute_actions`, not a loop" and changes nothing for the two registered
consumers, both single-target (`alpha.rs:102`, `phase_lg_cards.rs:57`).
CR 608.2b's off-battlefield skip stays ahead of the batch.

**5. Only the phase half of `backlog.md` §2.17 graduates; the step half stays
and the entry says which.** §2.17's own text already anticipates this — *"This
entry stays open for that half"* — so the edit strikes CR 500.8 as graduated,
leaves CR 500.9/500.10 with their Obeka reason, and retitles nothing: the entry
keeps its name because the name is still what it is about.

**6. The corpus files CR 500.7 and 500.11 as DEFERRED to Phase 9, and RE-1
shipped both — a decision the tree posed and §9 did not.** `session-4.md:976` and `:982`,
plus the summary table at `:2489`. RE-10 authors CR 500.8's atom because exit
criterion 5 asks for it; the two stale DEFERRED lines beside it are RE-1's to
have corrected and are **flagged here rather than edited**, for the reason
§2.17 gives about the same file: the corpus is authored, and a correction to it
is the corpus's own work rather than a side effect of the PR that noticed.

#### RF — the gather's zone leg — ✅ landed 2026-09-16

*Evicted 2026-09-16 from `plans/replacement-architecture.md`, where the heading and a stub remain.*

**One PR, `replacement/gather-zone-leg`, off `main` after #155.** Critical-path
6a's remainder and `roadmap-v2.md` A5b: LK (2026-09-14) gave a static ability
a way to *register* off the battlefield and left this sweep visiting
`battlefield_ids_ordered` alone (§11 items 4 and 9's (c)). Lettered `RF` for
the reason LK was not called `A5` — a roadmap row is not a phase code, and
this is the replacement track's next letter after RE. Written at the close,
as A5b's row allowed ("the doc can be written while it lands"), so the
decisions below are recorded rather than proposed; the PR description carried
them at review.

##### The finding that sets the scope: the sweep is not a loop over zones

§11 item 9 once called this "one loop over a zone list instead of
`battlefield_ids_ordered`". LJ found that sentence wrong for the *affected*
side, where the work was the working set; it is wrong for the *source* side
for a different reason — **cost**. Four libraries are ~400 objects, a gather
runs 2,273 times a four-seat game (`fuzz-record.md`, A4e's block), and the
read is a `compute_characteristics` per object: an ungated zone walk is the
10.3% RC-2 paid, several times over. So the question the PR was asked to
decide — what the sweep visits for a zone that is not the battlefield — has
the same answer the battlefield has: **a set of the objects worth a walk**,
and the sweep visits the set.

##### Decisions

**1. A second candidate set beside `replacement_ability_sources`, not a
widened one and not a per-zone index.** `GameState::zone_replacement_ability_sources`
holds the objects off the battlefield whose *printed* static replacement
ability functions in the zone they are in — `zone_function::functions_in`
on printed types at the registration doors, which LK already built:
`arrive_in_zone` for every move, `place_on_battlefield` for the entry, and
a new `create_in_zone` for the one door LK did not have, a card created in a
zone without moving — `Game::new`'s library, and the test helpers, which
`put_in_graveyard` alone had been routing through the hook. Retired at
`cleanup_zone_state` on leaving any zone, refiled on arrival.

Why a second set: the two are read by different sweeps. The battlefield sweep
probes its set by id inside a walk of `battlefield_ids_ordered`; the zone leg
*iterates* its set, and on every board that plays no such card that set is
empty — a structural zero, LJ's `reachable_zones` argument again. One widened
set would cost the leg a store probe per battlefield source on every gather to
tell the two apart. Why not a per-zone index: the leg wants "which objects",
and an object's zone is one store read away; a second index over the same fact
would be maintained at the same doors and could only disagree.

The set is the gate's *printed* leg, and it over-approximates in the one
direction the battlefield set does — a Layer 6 strip costs a walk and never an
answer — because the sweep re-asks `functions_in` of the **effective** list
with the effective types. That is what makes "Cards in hands lose all
abilities" strip a Colossus in hand (the `hollow_hands` fixture): the gate
says candidate, the frame says nothing, and the discard goes to the graveyard.

**2. The granted and copied legs say *where* the way a row names it.**
`RegistryScopeSummary::any_granted_replacement` and `any_copied_replacement`
were bools meaning "some object on the battlefield may have one" — a
replacement ability on an object that never printed it, which this phase
calls **unattributed** because neither printed set can name the object. The
zone leg needs to know where such an object can be, and a row answers by its
affected set in one of two shapes: a `Filter` row names **zones**, unioned
into `unattributed_replacement_zones`, which the leg walks whole while the
row exists; a `SourceOnly`, `Fixed` or `Host` row names **objects**, sets
`any_named_unattributed_replacement`, and the leg reads `row.source` or the
`Fixed` ids off the rows themselves, wherever they are (`Host` is a
permanent and the battlefield sweep's). Granted and copied are one pair of
fields, since a copy row answers "where" the way a grant does; the
restriction gate's two bools stay bools and battlefield-only, its zone leg
being owed against Abrupt Decay (`codebase-state.md` main item 146).

**As landed this was wrong in one arm, and the review caught it.** The
first cut carried a `ZoneSet` per row shape — `Filter` its zones, `Fixed`
and `Host` the battlefield on CR 611.2c and 400.7, and `SourceOnly` **`ALL`**,
because the summary is computed from the rows alone and could not say the
source's zone. Sound, free with no producer, and recorded as a ~20-line
item. The review asked why a 20-line item was not done inline; the honest
answer was that the fix as sized (a `source_zone` on the row) was not the
right fix. Reading a named row's objects by name needs no field, is exact
for all three shapes, and turns the summary's question from "which zones"
into "which shape" — so the item is not recorded, because it does not exist.

**3. Order, and the entering object.** The leg runs after the battlefield
sweep and before source 1a's splice — source 1a being the gather's name for
the *entering* permanent's own abilities, read off CR 614.12's frame rather
than off the board, the leg that makes "this land enters tapped" work — in
**CR 613.7d timestamp order** — the
battlefield's own key, so CR 616.1's list is one order rather than two, and
process-independent for `CLAUDE.md`'s reason. It **skips the entering
object**: source 1a reads that object off CR 614.12's frame with the
parenthesis's narrower scope (`SourceOnly` rows only), and reading it again
from its source zone with the sweeps' scope would offer a filter-scoped row
to the entry it is about — "permanents enter tapped" tapping itself, which
is the Orb of Dreams bug one zone over. `gather.rs`'s unit test is that
board. So `EntryFrame` has exactly one thing to say off the battlefield, and
it is *no*: `is_entering(id)`, answered off the proposal with no frame
computed.

`SelfScope::OnBattlefield` became `SelfScope::Existing`: an object in a zone
is one of CR 614.12 clause (3)'s "continuous effects that already exist"
whether or not that zone is the battlefield.

**4. `functions_in` is asked of every ability, on the battlefield too.**
`push_static_ability_replacements` takes the object's frame and the zone it
is asked *as* in — its own for the two sweeps, the battlefield for the
entering permanent (CR 113.6h) — and skips an ability that does not function
there. The battlefield sweep's check is CR 113.6's default arm and always
true for a permanent's replacement ability, so this costs it a few branches
per ability; what it buys is one home for the rule rather than a leg-shaped
exception, and the `timid_golem` fixture proves the negative from three
zones.

**5. The affected side, and the `debug_assert`.** `set_affects`'s `Filter`
arm now asks the layer walk's `in_zones_or_entering` question ahead of the
filter: the entering object counts as on the battlefield (CR 614.12 asks
what it *would be* there), anything else must be in the row's zones. Three
registered rows were hiding behind the assert, all written battlefield-scoped
by LJ's mass rewrite and reaching everything by accident: **Rest in Peace**
and **Leyline of the Void** say "from anywhere" and are `ZoneSet::ALL`;
**Nephalia Academy** acts on a card in its controller's hand and is
`ZoneSet::HAND`. Shown to fail first — with the check in and the rows
unchanged, exactly those three cards' tests failed and nothing else — which
is also the audit: every other `Filter` row in the card files is about a
permanent, a player or an entering object. The `sealing_ward` fixture beside
Rest in Peace on a milled creature card is the CR 109.2 distinction the check
makes observable.

**6. The card, and what it needed that the sizing did not name.** Darksteel
Colossus rather than Blightsteel: the owner named Blightsteel, its clause is
the same plus infect, and infect (CR 702.90) is not a keyword the engine has
— a registered Blightsteel would be a card wearing a real name while
behaving differently (`engineering-practices.md` §3). Nexus of Fate is the
second shape §3.3 asks for: an instant, never a permanent, replaced from the
stack when it resolves (CR 608.2n) and when it is countered; registered and
not pooled, for Time Walk's reason. The clause is written the way Wonder
writes its graveyard — `Condition::SourceInZone(ZoneSet::ALL)` is CR 113.6b's
statement, read syntactically to file the card and again at each proposal —
and its "shuffle it into its owner's library instead" is a substitute plus a
rider. The rider needed a writer of a library's order the engine did not
have in play: **`GameAction::ShuffleLibrary`** through the chokepoint (a
library's order is game state and "whenever a player shuffles" is a trigger
CR 701.24b, e, f name), `GameEvent::LibraryShuffled`, no `EventPattern` arm,
and **`Primitive::ShuffleLibrary`** whose `Implicit` recipient is the
source's *owner* — "its owner's". The rider moves nothing, which is
CR 701.24c rather than a shortcut: a commander's CR 903.9b can send the
substitute to the command zone instead, and its owner's library is shuffled
all the same while the card stays there — the test that has CR 616.1 twice
on one card. `Primitive::ShuffleIntoLibrary` (`backlog.md` §2.5) stays
unbuilt for the same reason in reverse: a spell's "shuffle target card into
your library" must move, and a rider must not.

**7. The frame is read only for an event the printed clause could watch
(added at the review, 2026-09-16).** As landed, the zone leg computed the
Colossus's frame on every gather — a one-frame non-member walk, or a memo
hit — and found one function later that a damage event is not a zone
change. Flat per decision, and ~150 walks a game at two seats and ~550 at
four for nothing; the review's "there has to be a more efficient way" was
right. The map's value is now the object's **printed** replacement defs,
kept by `register_static_effects` — the one site that already reads printed
abilities, so the leg never reads `card_data` itself — and the leg asks
`def_applies` of them before computing a frame. **Exact rather than a
shortcut**, and the argument is the gate's own: an object's effective
replacement defs are its printed ones or fewer, because the two other ways
onto the effective list — a grant and a copy — reach the leg by name or by
zone through decision 2's legs, a strip only removes, and nothing rewrites a
printed def in place (Layer 3, text, is the route every gate leg leaves
open). So a printed def that does not apply has no effective def that
could. Asked through the same function the effective def is asked through,
so the pairing of patterns and events lives once. What the precheck cannot
see is the "as long as" clause, which the frame read then asks. The
battlefield sweep still computes a frame for every printed source on every
gather; the same argument would let it ask the printed def first, and that
is the lever left — answer-preserving, but it moves the `Layer walks` row,
so it wants an A/B of its own.

**What the PR refused.** `Board::seed` and `membership` are untouched — a
source off the battlefield is read as a non-member, its own CDA walk plus
whatever zone-reaching rows LJ admits, which is exactly right — so "the
working set is the real one" (item 4) was true of the affected side and not
of this one. A trace page was refused at the close on §7's test — what
changed is which sources are read, not how a read is answered — and
**written at the review the same day**: the owner could not see why the leg
walks no library the moment a Colossus is in one, which is §7's own trigger,
a question the diff could not answer. `plans/traces/rf-a-source-off-the-battlefield.html`.

##### The pieces, measured

| | Site | Shipped |
|---|---|---:|
| The shuffle — `GameAction::ShuffleLibrary`, its arm, `GameEvent::LibraryShuffled`, `Primitive::ShuffleLibrary`, the display and amount arms | `engine/actions.rs`, `events/event.rs`, `engine/resolve.rs`, `types/effects.rs`, `ui/display.rs`, `pipeline.rs`, `gather.rs` | +89 |
| The leg — the second set and its three doors, the summary's two `ZoneSet`s, the sweep, `functions_in` on every ability, `is_entering`, the affected-side check, three rows corrected, `create_in_zone` in the test helpers | `state/{game_state,continuous_effects,game}.rs`, `engine/{zones,replacement/gather,replacement/lookahead}.rs`, `test_support.rs`, two card files, one test | +324 / −112 |
| Darksteel Colossus, Nexus of Fate, three fixtures, the registrations | `cards/phase_rf_cards.rs`, `registry.rs`, `mod.rs` | +289 |
| Tests — eleven integration, one unit | `tests/phase_rf_integration_test.rs`, `gather.rs` | +397 |
| The pool entry | `registry.rs` | +11 |
| The review — the printed-def precheck and its map, named rows by name, `Proposal` and `Asked`, the timestamp sort, the shuffle batch, `Effect::replacement_body`, the one-event mill test | `gather.rs`, `continuous_effects.rs`, `game_state.rs`, `resolve.rs`, `types/effects.rs`, two tests | +262 / −126 |

**+1,110 / −113 across 20 files** before docs and the review — engine 436,
cards 324, tests 349 — inside `engineering-practices.md` §4's band. Commits
in that order, each building on its own: the shuffle, the leg, the cards,
the pool, then the review as one commit.

##### Measure

Three arms at two seats and four (`plans/fuzz_ab.py`, defaults; `--players
4`), `main` at 1fe9a14: the leg with both cards registered and unpooled, and
the pool entry on top. Measured twice — at landing and again at the review
commit (ca88adf), after decision 7 — and the difference is the review's
finding.

**At landing.** The engine arm was `IDENTICAL` to `main` on every
`performance` counter at both seat counts — the candidate set is empty on a
board with no such card, so the leg is one `is_empty` per gather — and read
+0.6% and −1.2% per decision, inside §3.1's 2.5 points and the sitting's
spread. The pooled arm was the card's price and the shape decision 1
predicted: `Layer walks` 367 → 521 at two seats and 809 → 1,365 at four, one
non-member frame per gather per Colossus in a hand or a library (so
`Frames/walk` fell, 20.3 → 11.7), per decision +3.9% and −0.3%. Flat, and
the review declined to accept it: a frame on every gather for a clause that
watches one kind of event is work the printed def can refuse.

**At the review.** With decision 7's precheck the pooled arm walks `Layer
walks` 363 at two seats and 787 at four — *fewer* than `main`'s 367 and 809,
because its games are different games — and reads −0.1% and −5.1% per
decision; the engine arm −2.9% and −1.6%. The engine arm is `IDENTICAL` at
two seats and differs at four on **one line, `Memo hits` 189,560 →
189,566**, which a fourth arm attributes: the review commit with the
`Effect::replacement_body` peel reverted in `puts_a_replacement_ability` is
byte-identical to `main`, so the six hits are a copied Laboratory Maniac —
a `Conditional` replacement body, pooled beside Cytoshape — lighting the
gate its wrapper had hidden it from. A silent gap closed in passing: the
copied ability was never gathered before, and the two old bools matched a
bare `Effect::Replacement` only, the shape `cost-architecture.md` §8 item 1
had named for the cost gate. The block is in `plans/fuzz-record.md`; its
reachability rows, unchanged by the review, show both cards cast and
resolved in a fifth to a half of the games, and one Nexus countered.

#### RE-9 — mana (CR 106.6a, 106.12; RA's unnamed debt) — ✅ landed 2026-09-15

*Evicted 2026-09-15 from `plans/replacement-architecture.md`, where the heading and a stub remain.*

**Builds:** decision 7 — `ProduceMana`, its arm, the one performer replacing
`resolve_mana_effect`'s and `Primitive::ProduceMana`'s direct writes,
`ManaAdded` emitted for the first time, and `tapped_for_mana` read off the
activation cost (CR 106.12). **Consumers:**

- **Mana Reflection** — "If you tap a permanent for mana, it produces twice as
  much of that mana instead." `ProduceMana { tapped_for_mana: Some(true) }`,
  `Fixed(vec![])` + `You`, `Amount(Multiplier(2))`. Rulings, four, three are
  tests: *only a mana ability with {T} in its cost* → Dark Ritual (in the
  pool) adds three, not six; *a triggered mana ability is unaffected* → item
  6's, recorded; *restrictions and riders apply to all the mana* →
  `ATOM-106.6a-001`, on a fixture ability with a `special` atom, since no
  registered land restricts its mana; *two Reflections quadruple* → the
  `Amount` pair, fourth kind.
- **Nyxbloom Ancient** — "Trample. If you tap a permanent for mana, it
  produces three times as much of that mana instead." `Multiplier(3)`, the
  second factor the arm has seen. Ruling: *two Ancients: nine times* → test.

**`PERFORMANCE_POOL` +1, Mana Reflection**, predicted: a six-drop static on
the hottest path in the engine. Its row is the one this PR exists to read.

**Atoms:** `ATOM-106.6a-001`; `ATOM-106.6-001` (Phase 5-Pre, the restriction
survives the type — covered by the same fixture, and it is not in `owed`'s nine
because its ticket is not `NEW`).

#### The design check — thirteen decisions (2026-09-15, before the first line)

Read against CR 106.5, 106.6, 106.6a, 106.12, 106.12a–b, 107.1b, 605.1a–b,
605.3a–b, 605.5b, 614.1a, 614.5, 614.6 and 616.1 (`MTG-Rules/versions/tmnt.txt`),
decision 7 above, §4.1's six rules, §11 items 43 and 54, the RE-1 archive's
"§8's event-kind gate" and `codebase-state.md` item 111. Every card named
below had its oracle text and rulings fetched from Scryfall on 2026-09-15
(`engineering-practices.md` §3.4). **The rules pass (§8 of the practices doc)
paid twice before a line was written**, and the two findings it produced are
decisions 2 and 5.

**Verified against the tree, 95d9e92 (2026-09-15).** The two silent writers
are `engine/mana.rs:91` / `:94` (`resolve_mana_effect`, a mana ability) and
`engine/resolve.rs:530` / `:533` (`Primitive::ProduceMana`, a spell) —
`resolve.rs` moved from 507 under RE-10's `ExtraPhases` arm; every other
`mana_pool.add` in `src/` is a test fixture (ten sites, all past a `mod tests`).
`GameEvent::ManaAdded { player_id, source_id, mana: HashMap<ManaType, u64> }`
is `events/event.rs:80`, rendered at `ui/display.rs:434`, emitted at **zero**
sites. `resolve_mana_effect` is `(&mut self, effect, player_id)` — no
`ActionContext` — and both of `activate_mana_ability`'s callers already hold
one (`cast.rs:549` in the CR 601.2g window, `priority.rs:162`), built by
`ActionContext::new(decisions)`, so neither carries a resolution. **No
registered mana ability is an `Effect::Sequence`** — all four
`AbilityType::Mana` defs in `src/cards/` and the builder's
`mana_ability_single` are one `Atom(ProduceMana)`, and `land_types.rs`
synthesizes the intrinsic ones in the same shape. **No registered ability
produces two types in one `ManaOutput`**, and **no card authors a `special`
atom** — `add_special` has exactly the two writers as callers — so
`ATOM-106.6a-001`'s restricted land is a fixture by necessity. The exhaustive
matches a new variant costs are `subject_of`, `event_amount` and
`perform_action` as the census counted, plus `EventPattern::reads_the_amount`
and `display.rs`, compiler-forced; `pattern_watches` and `never_happens` fall
through to `false`. `pipeline::kinds_of` answers `Kinds::All` for any pattern
without a counter kind, and `classify` admits `Multiplier(n ≥ 1)` on any
pattern that reads no amount, so the commuting argument for two Reflections
is already written. The mana denominator, measured on 95d9e92 with a
four-file probe before this section was written: **81** mana productions a
game on `performance` at two seats against 983 gathers, 114 / 1028 on
`stress`, 151 / 2012 at four seats (200 games / seed 12345).

**The census, re-run, and it disagrees with decision 7's row in one
direction.** `o:/tap.* for mana/ o:instead` and `o:/tapped for mana/ o:instead`
together are **fourteen** printed replacement effects, not the "2 + 3 + 1"
the RE table counted. Three multiply the amount — Mana Reflection, Nyxbloom
Ancient, and **Virtue of Strength** ("if you tap a *basic land* for mana … three
times", an Adventure card the engine cannot register). **Six change the
type**: Contamination ("it produces {B} instead of any other type *and
amount*"), Infernal Darkness and Deep Water ("{B}" / "{U} instead of any other
type" — the amount unchanged, both rulings say so in as many words), Hall of
Gemstone (a color chosen at upkeep), Naked Singularity (a per-basic-type map)
and Harvest Mage (a color of your choice). The one "would add … instead" is
False Dawn — "spells and abilities you control that would add *colored* mana
instead add that much white mana", the only printed watcher that does not say
"tapped", and unregistrable for its second sentence (spend white as any color
is a payment rule `ManaPool` does not have). That is CR 106.12b's own sentence
— a replacement "if a permanent is tapped for mana **of a specific type and/or
amount** modifies the mana production event" — and it prices two fields and one
template arm the row never counted. Decisions 6 and 7 take them.

**1. The event is decision 7's, and its subject is the player whose pool it
enters.** `GameAction::ProduceMana { player, source, mana: Vec<(ManaType,
u64)>, special: Vec<ManaAtom>, tapped_for_mana: bool }` — the field named by
the CR's phrase and not `tapped`, which at a call site reads as the
permanent's state (review, 2026-09-15). CR 106.6a's own noun is "the
amount of mana produced by a spell or ability", and CR 106.12b names the thing
a replacement modifies "the mana production event"; `ProduceMana` is that
event under the name `Primitive` already gave it. `source` is the permanent
whose mana ability resolved, or the spell or ability object for a
`Primitive::ProduceMana` (`ctx.source`). `mana` is a `Vec` and not the
`HashMap` the log's event carries, for the reason `CLAUDE.md`'s determinism
rule gives every collection reaching a log: `--dump-events` renders the
event, and a map's iteration order is the process's. **The subject is
`EventSubject::Player(player)`** — CR 616.1's chooser is "the affected player",
Mana Reflection's "if *you* tap" is `affected_players: You`, and the permanent
is not the subject because a spell's production (Dark Ritual) has no
permanent at all. `affected_objects` is `NO_OBJECTS` on every def; which
*permanent* was tapped is the pattern's question (decision 6), not the set's.
N-player by construction: a `PlayerSet`, nothing that says "the other player".

**2. `tapped_for_mana` is CR 106.12's, read off the activation cost, and it
is a rule rather than the ruling decision 7 leaned on.** CR 106.12: *"To 'tap
[a permanent] for mana' is to activate a mana ability of that permanent that
includes the {T} symbol in its activation cost."* So `tapped_for_mana` is
`ability.costs` containing `Cost::Tap`, computed in `activate_mana_ability`
where the effective ability is already in hand, and **never** read off
`PaymentPlan` — the plan is `plan_payment`'s ordering of the same list, its
`ordered` field is private, and CR 106.12 says "activation cost", which is the
ability's. The two mechanisms decision 7 named as "CM-3's lock-in already knows
what was paid" turn out to be one list read from the end the rule names. Three
consequences fall out and each is a test: `Primitive::ProduceMana` from a
spell proposes `tapped_for_mana: false` (CR 605.5b — a spell is never a mana
ability; Dark Ritual under Mana Reflection adds three, its first ruling); a
mana ability with no {T} — Krark-Clan Ironworks' sacrifice, in the pool —
proposes `false` and is not doubled; and a *triggered* mana ability (CR
605.1b, item 6's eight "whenever you tap … add an additional" cards) will
propose `false` the day it exists, which is what Mana Reflection's second
ruling says and what `ATOM-106.12a-001` will read. A `bool`, not an `Option`:
every production knows.

**The fact has two readers, and both need it.** A *replacement* reads it off
the **proposal**: CR 106.12b, "a replacement effect that applies if a
permanent 'is tapped for mana' … modifies the mana production event", is Mana
Reflection's pattern asking `tapped_for_mana` of the `GameAction` before
anything is added. A *trigger* reads it off the **performed event**: CR
106.12a, "an ability that triggers whenever a permanent 'is tapped for mana'
… triggers whenever such a mana ability resolves and produces mana", is item
6's matcher asking the same fact of `GameEvent::ManaAdded`. The flag rides
from the one to the other unchanged, because no rewrite touches it (a
type-changer retypes units; a multiplier repeats them). **Triggered mana
abilities then shake out from the definition alone**: Wild Growth's "whenever
enchanted land is tapped for mana, its controller adds an additional {G}" is a
CR 605.1b trigger that item 6 fires on the `ManaAdded` with `tapped_for_mana: true`,
resolving at once without the stack (CR 605.4a); its *own* production is a
second `ProduceMana` proposal with `tapped_for_mana: false`, since no permanent
was tapped to activate it, so Mana Reflection does not double the extra {G}
— and that is Mana Reflection's second ruling, "that triggered mana ability
won't be affected", verbatim. RE-9 builds the flag and the proposal; item 6
builds the trigger and finds the answer already fixed.

**3. Both writers become proposers of one event; `resolve_mana_effect` gains
the two things it has never had; a production of nothing performs and
announces nothing.** `resolve_mana_effect(&mut self, effect, player, source,
tapped, ctx)` — the signature change §9's row did not count, mechanical because
`activate_mana_ability` holds `permanent_id`, `ability.costs` and `ctx` at the
call. Its `Fixed`-only limitation stays exactly as decision 7 said; its
`Sequence` arm stays too and proposes one event per atom, which no registered
ability reaches. `Primitive::ProduceMana` evaluates its dynamic amounts as it
does now and proposes with `ctx.controller`, `ctx.source`, `tapped_for_mana:
false` through the `actx` its arm already has in scope. The one performer,
`perform_action`'s `ProduceMana` arm, is the only `mana_pool.add` and
`add_special` outside tests. **A production whose every amount is zero and
whose atom list is empty is the performer's no-op, not `never_happens`'**: no
rule says a zero production is no event — CR 106.5 is about an *undefined
type* and CR 107.1b's Viridian Joiner example says "adds no mana", which is a
statement about the pool — so RE-2 decision 3's line holds and the guard sits
beside `LoseLife`'s and `AddCounters`' in `perform_action`, announcing nothing
because nothing was added. **A "can't" has no printed customer here** — no card
says a permanent can't *produce* mana; the printed prohibitions are on
*activating* (Stony Silence's family), which is CR 602's and not this event's —
so `is_prohibited` is asked as it is of every proposal and refuses nothing.

**4. The production is its own batch, separate from the cost's, and joins an
enclosing one by default.** CR 605.3b: a mana ability "resolves immediately
after it is activated" — its resolution is a step after its cost, exactly as a
spell's is, and the two events are already two batches today: `pay_costs`'
`Cost::Tap` runs through `execute_action` and closes before
`resolve_mana_effect` runs. CR 106.12a confirms the seam from the trigger side —
"triggers whenever such a mana ability *resolves and produces mana*", not when
the permanent taps — so "whenever you tap a land for mana" reads the
production and not the `Tap`, and folding the two into one batch would hand
CR 603.2c one event where the rules have two. Inside CR 601.2g's window the
cast is not a proposal and opens no batch, so today the production nests in
nothing; the default `execute_actions` (join the enclosing batch) is right for
the one case CR 605.3a names and no code path builds — a mana ability
activated "in the middle of … resolving a spell" is a result of that
resolution's instruction, and a caller that ever nests it makes no new
argument. No `// AUXILIARY-MOVE:`, no new batch entry point.

**5. `Amount(Multiplier(n))` scales every plain entry and repeats every
restricted atom `n` times — decision 7's "`special` riding through unchanged"
is wrong for a multiplier, and CR 106.6a is why.** *"Any restrictions or
additional effects created by the spell or ability will apply to **all mana
produced**."* A `ManaAtom` is one unit of mana carrying its restrictions, so a
doubled production of one restricted {G} is two restricted atoms, not one
restricted and one free — which is `ATOM-106.6a-001`'s board verbatim and Mana
Reflection's third ruling ("that will apply to all the mana it produces this
way"). The repeat is in place, RE-4's `repeat_n` over token defs. CR 106.6a's
last two sentences then cost nothing: "a separate delayed triggered ability is
created for each mana produced" and "a separate effect is created once for
each mana produced" are the atom's `grants` and `persistence` riding on each
copy — the shape CR 106.6a describes is the shape `ManaAtom` already is.
**`Plus` and `Halve` are refused on this event** as the pairing errors
`CreateTokens` refuses: nothing prints "produces one more mana" as a
replacement — every "add an additional" is CR 605.1b's trigger, eight cards —
and nothing halves mana. `took_effect` is any entry or the atom count
changing, so `Multiplier(1)` reports untouched. Two Reflections quadruple and
two Ancients ninefold (both fourth rulings) with **no prompt**: `Multiplier`
beside `Multiplier` is the commuting cell RD-1 wrote, `kinds_of` reads
`Kinds::All` here, and the provider primed with nothing is the first assertion.

**6. `EventPattern::ProduceMana { tapped_for_mana: Option<bool>, source:
Option<ObjectFilter> }`, reading no amount.** `tapped_for_mana: Some(true)` is
all fourteen printed cards; `None` is False Dawn's "would add colored mana"
(unregistrable, but printed, so the `Option` is not a two-arm enum wearing a
`bool`); `Some(false)` has no card and is the field's honest third answer.
`source` is the constraint on *which permanent* — "a land" (Contamination,
Infernal Darkness), "a land you control" (Deep Water), "a basic land" (Virtue
of Strength) — asked of the proposal's `source` through `object_matches_filter`
exactly as `EventPattern::DealDamage`'s `SourcePattern.filter` is, and a plain
`ObjectFilter` rather than a `SourcePattern` because nothing prints a *chosen*
permanent tapped for mana. It is on the pattern and not in `affected_objects`
because the subject is the player (decision 1) and the set cannot see the
permanent. Deep Water's second ruling — "affects lands you control when it
resolves and any lands you gain control of this turn" — falls out of the
gather evaluating the filter at the proposal. **The axis this arm does not
grow along yet is CR 106.12b's "of a specific type"** — "tapped for {G}",
"would add colored mana" — and it waits for its first registrable card with a
`mana_type` field; False Dawn is named so the field has a customer on record.
`reads_the_amount` is `false`: neither field reads a count, a production of any
size matches, and two multipliers commute (decision 5).

**7. The type-changing substitution ships, on `engineering-practices.md` §4's
rule, with Deep Water as its printed card.** The rule adopted at RE-4's review:
an arm the PR's own type opens, with a printed customer, sized under about
eighty lines, ships in that PR. `GameActionTemplate::ProduceMana { mana_type:
ManaType, amount: TemplateAmount }` and `substitute`'s leg over
`GameAction::ProduceMana` are that arm — the engine half is ~50 lines — and
Deep Water registers **whole today**: `{U}: Until end of turn, if you tap a land
you control for mana, it produces {U} instead of any other type` is an
activated ability whose effect is `Primitive::CreateReplacement(def,
Duration::UntilEndOfTurn, PatternFill::Authored)`, Fog's own shape, with
`Instead(ProduceMana { mana_type: Blue, amount: ReplacedAmount })`. The leg
**retypes every unit in place** — each plain entry to `mana_type`, merged into
one, and each atom's `mana_type` with its restrictions kept (CR 106.6: a
restriction "doesn't affect the mana's type", and the converse holds — a type
change does not drop the restriction, which is the ability's) — so Deep
Water's first ruling ("the amount of mana produced is unchanged, but it will
all be {U}") and Infernal Darkness's ("would add {W}{W}, it adds {B}{B}
instead") are the two tests. `Fixed(n)` is Contamination's "instead of any
other type *and amount*": the production becomes `n` units of the type. **Which
restriction those `n` units carry is the one question this leg has to answer
without a rule**, because the old units are gone and only their metadata can
say. Every printed mana ability produces mana that is *uniformly* restricted
or uniformly free — a Forest's {G}, Boseiju's spend-only-on-instants — and for
those the answer is plain: the `n` units carry whatever the old units all
carried. A production mixing restricted units with free ones ("add {G}, and
add {G} that can be spent only on creature spells" in one ability) has no
printed instance and no CR sentence to decide it, so the leg **returns `Err`
on that board instead of guessing** — the same loud refusal every other
pairing the pipeline cannot answer gets, chosen over silently dropping a
restriction the ability printed. A fixture asserts the refusal, and the day a
card mixes, its ruling is the answer to write. Contamination and Infernal Darkness are fixtures
in the test file, not registered: each carries an upkeep half that is item 6's
(a trigger; cumulative upkeep). Hall of Gemstone, Naked Singularity and
Harvest Mage wait for a chosen color on a permanent, a per-basic-type read and
a choice in the leg — three facilities, recorded, none under eighty lines.
`event_amount(ProduceMana)` is the unit total (plain amounts plus atoms), which
is what `ReplacedAmount` reads and what a rider's "that much" would mean.
**The suppression proof gets three new answers, each by §4.1's question**:
`template_is_instance_invariant` is `true` (the type is def data, identical
across two Deep Waters); `template_is_idempotent` is `true` (retyping twice is
retyping once, and `Fixed(n)` twice is `Fixed(n)`); and `replaces_that_many`
admits the template with `ReplacedAmount` beside a multiplier — retype-then-
double and double-then-retype are one event — while `Fixed` beside a
multiplier stays CR 616.1's real question (`Fixed(1)` then ×2 is 2; ×2 then
`Fixed(1)` is 1). No card's ruling contradicts either cell; none speaks to it.

**8. `GameEvent::ManaAdded` is reshaped in the PR that first emits it:
`{ player_id, source_id, mana: Vec<(ManaType, u64)>, tapped_for_mana: bool }`.**
The `HashMap` goes for decision 1's reason — the event has never been
rendered, and the first render would have been the determinism regression's.
`tapped_for_mana` rides on the event because CR 106.12a's trigger is its
second reader (decision 2) — "triggers whenever such a mana ability resolves
and produces mana" — and `ATOM-106.12a-001`'s Mana Web wants *which*
permanent, which `source_id` already is. Restricted
atoms are **folded into the per-type counts** in proposal order: CR 106.6's
first sentence says a restriction "doesn't affect the mana's type", so the log
reports {G}{G} for a doubled restricted Forest and the pool keeps the
restriction where it lives. `display.rs`'s arm is rewritten to the `Vec`.

**9. The probe lands as a permanent diagnostic row.** `record_mana_production`
on the engine's diagnostics struct, called once by the performer, printed by
`fuzz_games` as `Mana productions` beside `Replacement gathers`, and a row in
`fuzz_ab.py`'s `ROWS` and the fixture table (decided at review, 2026-09-15).
It is the denominator every mana number after this is read against — gathers
per production — and it costs one `Cell` increment on the hottest path, which
the A/B measures with everything else. The `main` arm of this one sitting
reads `?` for it, by construction. **The struct's name collides with the game's
counters, and the review is where it was seen**: `GameState.counters` is
`EngineCounters`, the diagnostics, while `PermanentState.counters` and
`PlayerState.counters` one struct over are CR 122's — three fields, one
spelling, two meanings. The rename is a mechanical sweep (six `EngineCounters`
sites, ~75 `.counters.record_*` and accessor calls) and rides in **no rules
PR** (`refactor/object-set-rename`'s precedent, `codebase-state.md` main item
124): its own PR, proposed as `EngineMeters` / `game.meters`, a
word the CR never uses and one that reads as measurement at every call site.
This PR adds its one method under the existing name and does not touch the
collision.

**10. `PERFORMANCE_POOL` +1, Mana Reflection, decided by the A/B's third arm.**
A six-drop static on the hottest path in the engine, whose row is the one this
PR exists to read: from the turn it resolves every land tap is a gather with a
match. Its pooled arm will move the gameplay rows by design — twice the mana is
more spells and bigger boards — and the reading is "more game, not a slower
walk", `ms / 1,000 walks` beside `CPU/turn p50`. Nyxbloom Ancient stays out:
the same path at seven mana with a 5/5 trample body that changes combat. Deep
Water stays out: an activated `{U}` the random agent will spend on nothing, so
its reachability is a `--require` row.

**11. Lever 2 is declined on the prediction and decided on the A/B, with the
rule written before the number.** RE-1's shape is this PR's — one proposal per
unit that also gathers — and its cost was +447 gathers for +5.0%; RE-2's +34
for +0.5%. Linear in gathers, 81 predicts **+0.9%** and RE-2's rate puts the
upper end at ~1.2%, against the 2.5-point gate §11 item 54 named. **The rule:**
seven rounds (`--rounds 7`, never three — item 54's own lesson), and the
engine arm's CPU/game median on `performance` at two seats is the number. Under
2.5 points: ship without the gate, which is the prediction. At or above it:
build §8's event-kind bitmask as a fourth arm, measure it the same way, ship it
if the gate brings the delta under the line. If the gate does not close it,
§8's fallback stands — the two writers keep their direct write, the branch is
closed unmerged, and `codebase-state.md` item 111 gets a Deferred Migrations
line that names the number. Recorded either way, which is exit criterion 1.

**12. No trace page.** The read RE-9 changes is what is *proposed* — two
direct writes become one performer — and not how any read is answered, which
is the property §7 names and the argument RE-4's "no" made; every other RE
phase but RE-2 declined on the same ground.

**13. Atoms.** `ATOM-106.6a-001` `COVERS` — a fixture land with a restricted
{G} under Mana Reflection adds two restricted atoms. `ATOM-106.6-001` `COVERS`
by the same fixture — the restricted mana is still green and pays a creature
spell's {G}, its expected result verbatim. `ATOM-106.12a-001` `COVERS-PARTIAL`:
the event carries "tapped for mana" as a fact (a Forest's production is
`tapped_for_mana: true`, Dark Ritual's `false`) and the atom's *query* — "was this
permanent tapped for mana this turn" — is item 6's trigger, said in the test.
None of the three is in `owed`'s nine, by construction.

**Re-counted with decisions 6 and 7 in:** exhaustive matches 3 + `reads_the_
amount` + `display.rs`; `pattern_watches` 1; `substitute` 1 leg and two
predicate arms; `replaces_that_many` 1; `resolve_mana_effect` 1 rewritten +
signature; `Primitive::ProduceMana` 1; `activate_mana_ability` 1 call; the
diagnostic row 4 files. **~420 engine, ~260 cards (three registered, two fixtures
with their rulings), ~650 tests, ~20 harness ≈ 1,300–1,450** — inside §4's band
where the row's 950–1,150 was below its floor, and the difference is the
template and its card.

**Reviewed 2026-09-15, the same day.** Two scope questions were put to the
owner and both were taken as recommended: *(a)* decision 7's type-changing
leg ships with Deep Water, the one addition to §9's row, on §4's rule; *(b)*
`Mana productions` joins `fuzz-record.md`'s fixture table. Three review notes
changed the text above and are marked where they landed: `tapped` became
`tapped_for_mana` on the action, the pattern and the event, and decision 2
gained its "two readers" paragraph, because the reviewer asked whether the
flag was the triggers' or the replacements' — it is both, and the triggered
mana ability question shakes out of the definition; decision 7's `Fixed`
refusal was rewritten to say what board it refuses and why; and decision 9
names the `counters` collision and the rename PR it is not.

## As landed (2026-09-15)

**Every one of the thirteen decisions shipped as written after the review,
and the tree posed two more questions the section had not.** Six commits on
`replacement/re-9-mana`: the diagnostic row; the event, its arms and its legs;
the cards; the tests; the pool and the docs. **+1,423 / −29** before the docs —
types 63, engine 295, events 14, display 8, state 17, harness 7 (plus
`fuzz_ab.py`), cards 285, tests 734 — against the re-count's 1,300–1,450, with
the split landing where it was predicted: the engine half under its ~420
because `resolve_mana_effect`'s rewrite was 40 lines and not 60, the tests
over their ~650 because the payment-window board and the two refusal boards
were written after the count.

**What the tree answered that the section did not ask.**

1. **`fuzz_ab.py`'s arm-identity compare could not survive a new row.** The
   script compared the whole stripped output, so the sitting that introduces
   a diagnostic row reads every arm as `differ` from `main` for the row's
   sake alone. `comparable` drops the lines whose row label the baseline
   binary never prints before comparing; the table still shows `?` on the
   baseline, which is the honest cell. Every arm in this sitting still reads
   `differ`, and for RE-1's reason: `Memo hits` moves with the sweep — the
   fast path stops the walk and not the query — on every arm that proposes
   the event, so the check is the aggregates (§11 item 65's precedent).
2. **The engine arm carries two warnings and the committed tree none.** The
   arm with the three cards unregistered leaves `phase_re9_cards` imported
   and unused; it is a throwaway build for one sitting and is not committed,
   which is the same shape as every earlier "engine" arm.

**The exhaustive matches were exactly as counted, plus the two the design
check added.** `subject_of`, `event_amount`, `perform_action`,
`reads_the_amount`, `display.rs`; `pattern_watches` one arm; `substitute` one
leg and its catchall; `template_is_instance_invariant`,
`template_is_idempotent`, `replaces_that_many` one arm each; `kinds_of` and
`classify` needed nothing, since a pattern with no counter kind is already
`Kinds::All` and a multiplier on a pattern that reads no amount is already a
commuting member. The ordering board came out as decision 7 wrote it — index
0 the Reflection, the older row — on the first run.

**The glossary gained `retype`**, the one term of art `--suggest` found in the
108 doc-comment lines the branch added; "units" and "multiplier" are the CR's
own or the code's already.

**Corrected at the PR's review (2026-09-15), after the measurement.** The
reviewer asked whether Damping Sphere belonged to the family, and it does —
"tapped for **two or more** mana" is CR 106.12b's *amount* axis, which the
census regex could not see. Re-run with the rule's phrasing the family is
**sixteen**, not fourteen: Damping Sphere (an amount constraint that would
read the amount), Pale Moon (a nonbasic-land retype, registrable whole and
registered in the same review round) and Quarum Trench Gnomes (a *chosen*
permanent, which decision 6 said nothing prints). The design check's
"fourteen" and "six type-changers" above are left as the record of what was
reviewed; §11 item 98 carries the correction and `codebase-state.md` item
133 the three fields with their cards. Two other review notes changed the
code's comments and not its behavior: `pattern_watches`' arm now says why a
pattern field is an `Option` when the event's is a `bool`, and `substitute`'s
`Fixed` leg spells its uniformity test as two named conditions rather than a
guard clause.

## Measured (2026-09-15)

**Four arms, two seats, `--rounds 7`, 200 games / seed 12345** — `main` at
95d9e92, **engine** (the three cards unregistered), **registered** (the pool
unchanged), **pooled** — each arm's `Card pool: (N cards)` header read before
the sitting: 88 / 154, 88 / 154, 88 / 157, 89 / 157.

**The engine arm is the reading, and it held to the digit.** `Replacement
gathers` **983 → 1063** on `performance` and **1028 → 1142** on `stress`,
against `Mana productions` 81 and 114 — one gather per production, the
averages rounding apart by one — with `Restriction queries` moving by the same
number and **every gameplay row and every layer row identical to `main`**:
turns, spells, lands, combat, damage, life, walks, board walks, frames,
frames/walk, dependency checks. The one other movement is `Memo hits` **59,133
→ 59,397** (+0.4%) and 76,231 → 76,725, which is RE-1's sentence about the fast
path stopping the walk and not the query, and it is why the arms read `differ`.

**CPU/game median 13.82 → 13.98 ms, +1.2%**, rounds straddling — `main`
13.65–14.00, engine 13.79–14.03 — with `ms / 1,000 walks` +1.2%, `ms / 1,000
queries` +0.7% and `CPU/turn p50` **0.410 → 0.410**. Decision 11's rule, applied
to that number: **under the 2.5-point gate, so lever 2 is not built.** The
prediction was +0.9% to +1.2% from RE-1's and RE-2's rates over 81
productions, and the reading is its upper end. The registered arm is the
engine arm on `performance` (+1.3%, the same binary with three names in a
map), and on `stress` it is the pooled arm, since registration changes the
decks there.

**The pooled arm is a re-record and a bigger board, which is what a six-drop
that doubles mana should be.** Avg turns 29.6 → 32.1, spells 22.8 → 24.3, lands
17.6 → 18.8, walks 362 → 394, gathers 1174, `Mana productions` 86, `Replacement
prompts` 0.18 → 0.52. CPU/game **+20.1%** and `ms / 1,000 walks` +10.4% read
alarming until `Frames/walk` is beside them: 12.54 → 13.96, +11.3%, because a
board with twice the mana has more permanents for every walk to frame. Per
thousand *frames* the walk is flat — 3.04 ms against 3.02 — and `ms / 1,000
queries` is +2.7%, `CPU/turn p50` 0.410 → 0.430. More game and bigger boards,
not a slower walk. The prompts row moves with the board: no pair RE-9 adds
can ask (two Reflections commute; Reflection beside Deep Water commutes), and
the fuzz pool has no Contamination.

**`--require`:** Mana Reflection **cast 119 / resolved 118 in 82 of 200 games
(41%)**, 1.46 copies/deck, board diversity 100% — the lowest reach of any
pooled RE card, at six mana, and reach was never the point: from the turn it
resolves every land tap is a gather with a match. On `stress` the three cards
in every deck dilute the counter cards, so `Replacement prompts` 2.19 → 1.47,
RE-8's dilution again.

**Four seats, `--rounds 7`, the same four arms.** The engine arm is again
identical to `main` on every gameplay and layer row, with `Replacement
gathers` **2012 → 2163** against `Mana productions` **151** — exactly, this
time — and 2220 → 2451 against 230 on `stress`; `Memo hits` +0.5% and +0.7%.
CPU/game median **44.51 → 44.83 ms, +0.7%**, rounds straddling (`main`
44.11–44.87, engine 44.49–45.09 with one 49.36 outlier the median discards),
`ms / 1,000 walks` +0.7%, `CPU/turn p50` 0.690 → 0.700. The pooled arm moves
less than at two seats — CPU/game +4.0%, turns 58.3 → 61.0, walks 778 → 793,
`Frames/walk` 19.59 → 20.56 — because a four-player game is already the bigger
board, and the pooled arm's `ms / 1,000 queries` is **−2.4%**.

**One four-player `stress` game on the registered and pooled arms ran into
the 200-turn cap, and it is the tail rather than a stall.** `main`'s longest
four-player `stress` game is 194 turns; the pooled decks average 64 against
61.5. Re-run with `--max-turns 600` the same 200 games hit the limit **zero**
times, the longest ends at **208**, and the average moves from 64.0 to 64.1 —
one game, eight turns past a cap that was 3% above `main`'s own maximum. The
`ZERO_ROWS` flag did its job, which is to make somebody look; the row stays
pinned at zero, and a second sitting that trips it is the place to raise the
cap for four seats rather than to explain it again.

**Zero errors and zero panics on every arm at both seat counts; zero turn
limits at two seats and the one four-seat tail above; `deterministic: yes` on
every arm.** → `fuzz-record.md`, both tables re-recorded, the four-player
table beside them — the `stress` columns re-recorded once more at 158 cards
after the review registered Pale Moon, unpooled, so the sitting's
`performance` columns stand and only the stress decks moved.

**Measured again at review (2026-09-15), because the sitting could not answer
the question the phase was for.** The reviewer's objection was exact: the
engine arm measures a proposal with nothing watching it, and the pooled arm
measures a replacement applying *and* a board twice the size, so nothing
above isolates what a mana replacement costs per tap with the game held
fixed — and that, on the hottest path in the engine, was the whole worry.
Two more arms, the §8 recipe, built from the committed tree with the four
cards unregistered and compared with that same engine arm at `--rounds 7`:

- **applying** — at every game's setup, one `Duration::Indefinite` registry
  row per player from a source in exile: `ProduceMana { tapped_for_mana:
  Some(true) }`, `Amount(Multiplier(1))`, `You`. Every production is
  gathered, found, chosen, applied and re-gathered; nothing about the game
  changes, and no permanent is added for the layer walk to frame. Every
  gameplay and layer row is **identical** to the engine arm at both seat
  counts; `Replacement gathers` +81 and +151 (the re-gather after applying),
  `Memo hits` +263 and +798.
- **gated** — the engine arm with a hard-coded early return in `gather` for
  `ProduceMana`: what §8's event-kind bitmask would compute on a board with
  no mana watcher, RE-1's probe shape exactly.

| two seats | engine | applying | gated |
|---|---|---|---|
| CPU/game median | 13.95 ms | 14.32 (**+2.7%**) | 13.97 (+0.1%) |
| rounds | 13.89–14.10 | 14.28–14.40 | 13.92–14.08 |
| ms / 1,000 queries | 0.233 | 0.239 (+2.2%) | 0.234 |
| CPU/turn p50 | 0.410 | 0.420 | 0.420 |

| four seats | engine | applying | gated |
|---|---|---|---|
| CPU/game median | 47.57 ms | 48.61 (**+2.2%**) | 47.29 (−0.6%) |
| rounds | 45.44–48.25 | 47.64–49.63 | 44.89–47.73 |
| ms / 1,000 queries | 0.269 | 0.273 (+1.7%) | 0.268 |
| CPU/turn p50 | 0.740 | 0.760 | 0.740 |

(The engine arm's absolute number differs from the sitting's 44.83 ms: two
sittings on one machine drift, which is why every comparison here is inside
one sitting and no absolute is recorded as a baseline.)

**Two answers, and the second revises §8.** A mana replacement *applying*
on every tap costs **+2.7%** of a two-player game with the board held fixed —
about 4.6 µs per application, the CR 616.1 loop with a candidate: the
registry scan, `applies_to`, the ordering check, the rewrite's two `Vec`s,
the applied-set insert and the re-gather. That is the number the phase was
worried about, and it is paid only on a board that has such a replacement,
which the pool reaches in 41% of games at six mana. And **the event-kind
gate returns nothing for this event kind**: gated and engine straddle at
+0.1%, so the +1.2% the proposal costs with nothing watching is not the
sweep — it is the chokepoint's fixed cost per event, the batch opened and
closed, `is_prohibited`, the grouping and applied-set allocations, the
performer and the emit. RE-1 measured the sweep at half of its +5.0%; here
it is none of the +1.2%, because a production's sweep is three memo reads.
**Lever 2 stays unbuilt with a stronger reason than the prediction gave
it**: it would buy RE-9 nothing. The lever that would is a different one and
is named rather than built — an allocation-free fast path through
`execute_batch_inner` for a single-member batch with no candidate and no
prohibition, which is answer-preserving in §8's sense and is
`codebase-state.md`'s to size (item 136).

**On the v1 question.** The plan set no absolute budget; §8's discipline is
the per-phase gate and the record. Summing RE's per-sitting engine deltas at
two seats — RE-1 +5.0, RE-2 +0.5, RE-3 flat, RE-4 flat, RE-5 flat, RE-6
−1.0, RE-7 identical, RE-8 flat, RE-10 +0.1, RE-9 +1.2 — the ten PRs cost
the two-player `performance` game **about six points** of CPU, most of it
RE-1's three begin proposals a step, and every point of it is the
chokepoint's price for events that critical-path item 6's triggers will
read. Whether six points is inside v1's budget is a number the owner has to
set; what this sitting adds is that the next point does not come from the
gather, and that a board with a mana doubler pays 2.7 more.

### Trace-page decisions — the phases that produced a candidate and declined it

*Evicted 2026-09-13 from `plans/replacement-architecture.md`'s "Trace page" section, which keeps the rule, RE-2's page and the summary line. `engineering-practices.md` §7 owns the practice.*

**RE-3: no**, decided at its close (2026-09-12) and recorded because the phase
did produce a candidate. Generalizing the CR 616.1 suppression premise from a
list of pattern kinds to `EventPattern::reads_the_amount` looks like §7's rule
— "how a read is answered" — and is not: the same predicate asks the same
question at the same point in the same loop, and what changed is where the
answer is *written down*. Nothing reads differently, no board takes a path it
did not take, and the two boards worth walking (two Faithmenders, and the
Archive beside Tainted Remedy) are a product and a two-branch prompt that two
tests state completely.

**RE-6: no**, decided at its close (2026-09-12), and the section's own
sentence — "the sweep's shape changes" — was the candidate. It changes what is
*proposed* (four writes become four members) and not how any read is answered:
the loss goes through the same CR 616.1 loop every other kind does, the
subject-keyed dedupe is RA-3's with a second key type, and the settlement is
a read of `player_lost` after a batch. The two boards a page would walk — a
loss for two reasons replaced once, and four losses with one replaced settling
the survivor's win before the rider — are each one test with one assertion
that a trace would only narrate.

**RE-7: no**, decided at its close (2026-09-13), and the candidate was real
enough to be worth the paragraph. CR 800.4a changes what the board *is* in the
middle of a batch, and §7's rule is about a phase that changes how a **read** is
answered — which this does not: every read afterwards asks the same question of
the same accessor and gets a smaller board, which is a board and not a path.
The one seam a page would have walked is the batch-order rule (§11 item 69), and
that is a five-line change with a two-assertion test —
`a_creature_dying_in_the_same_check_is_destroyed_and_then_leaves_the_game` names
the order in the log — where the trace would only narrate it. The page this
phase *would* deserve is the one CR 800.4c wants: a permanent through three
controllers and two departures, with the layer walk's answer at each step. It
has one customer, no printed card, and a fixture board; when CV-4 or Bribery
gives it a second, it is worth writing then.

**RE-4: no**, decided at its close (2026-09-13), and it was the second phase
§9's trace-page section named in advance — for "how an entry's frame is
answered inside a plural batch". RC-5's re-size had already found that read
answered by RC-4b and walked it (`rc-5-applying-an-entry-can-move-the-board.html`'s
last trace is two entries decided against one board), so what RE-4 changed is
what is *proposed*: one plural batch where there were N singletons, and an
appearance in exile where there was a move out of a battlefield the token was
never on — RE-6's and RE-7's shape, and their answer. The three boards worth
walking are each one test: Parallel Lives over a plural creation (the outer
loop asks nothing, and is not asked again at any entry), Master Biomancer
beside Hallowed Moonlight asked once per token, and the exiled token's log
with its `ZoneChange` asserted absent. The one path a page could have added is
§11 item 77's loop of riders, and that is a page about RE-2's cards, not this
phase's — it belongs beside RE-2's, if the rider encoding is ever revisited.


**RE-5: no**, decided at its close (2026-09-13), and the candidate was the
door: a pattern arm that answers for an event of a kind it did not watch
before. §7's rule is about a phase that changes *how a read is answered*,
and the door is RC-4b's shape a second time — one CR 616.1 step over one
event, with a third pattern kind admitted to it — so every read at that step
asks the same frame, the same applied set and the same chooser it asked
before; what changed is which defs the gather returns. The one path that
looked new, a doubler falling out of applicability as another raises an
entering creature's power, is `check_order_invariance`'s existing re-gather
finding what it was built to find, and the test that shows the prompt is
one board with two picks. The three boards worth walking — the Season's
token half at a creation and its counter half at each entry, Scales beside
the Season with the affected player choosing, Vorinclex halving what an
opponent's own entry puts on — are each one test with one assertion a
trace would only narrate.

**RE-9: no**, decided at its close (2026-09-15), and the candidate was the
phase's own sentence about itself — two direct writes becoming one performer
on the hottest path in the engine. §7's rule is about a phase that changes
*how a read is answered*, and this one changes what is **proposed**: the mana
that was written is now an event, it walks the same CR 616.1 loop every other
kind walks, and every read along the way — the sweep, the filter on the
tapped permanent, the applied set, the chooser — asks what it asked for a
draw or a creation. The three boards worth walking are each one test with one
assertion: a restricted Forest under Mana Reflection (two atoms, no free
green), a Forest under Deep Water beside Mana Reflection (retype and double
commute, nobody asked), and Contamination's line beside Mana Reflection (the
real choice, in both orders). The one path that could have earned a page is
the payment window — a mana ability activated inside CR 601.2g proposing
through the same performer — and it turned out to be no path at all: the
window calls `activate_mana_ability` as the priority loop does, the cast is
not a proposal and opens no batch, so the production nests in nothing and the
trace is the priority loop's trace with a different caller at the top.

## 1. Verdict — is this the right next step?

*Evicted 2026-09-15 from `plans/replacement-architecture.md`, where the heading and a stub remain.*


**Yes, and the prerequisites are met.** Recorded so a later session does not
re-litigate it:

- `CLAUDE.md` → "Critical path to v1" item 5. Items 1–4 (layers core, CDAs,
  Layer 6, Layer 2) are ✅ as of 2026-08-24. 634 tests green, zero warnings.
- `codebase-state.md` → Deferred Migrations → "Before Replacement effects":
  item 1 ✅, item 2 ✅ (with three tagged bypasses this phase closes), item 4
  **resolved 2026-08-24** (performed-action event stream; the delta log is
  rejected). Item 3 is not a prerequisite — it is *this phase's opening ticket
  block*, and Phase RA below is that block.
- Nothing else in the tree blocks it. The chokepoint (`engine/actions.rs:86-89`)
  exists and every mutating action already routes through it or through a site
  tagged for this phase.

**Why not triggers (CR 603) first.** Triggers consume the *performed* event
stream, which only exists downstream of the CR 614 pipeline —
`GameAction` (proposed) → replacement → perform → `GameEvent` (performed).
Building CR 603 against the pre-replacement stream and refitting is the
expensive path, and it is the path the 2026-08-24 fork decision explicitly
closed. CR 614 also sits upstream in the code: `execute_action` runs before
`perform_sba_and_triggers` (`engine/priority.rs:234`, whose placement stub is at
`:240`) can ever see anything.

**Why not the CR 613.8 dependency cluster first.** It is back-stopped to land
before Phase 8 card breadth, not before this. It also gets cheaper for waiting:
§5 shows the CR 614.12 look-ahead forces `compute.rs`'s five concrete state
reads behind one accessor pair, which is plumbing 613.8's step-4 hypothetical
check would otherwise have to do itself. (Only the *seam* is shared — 613.8's
own perturbation is frame-level and much cheaper; §5 and §11 item 5 price both.) **Since 2026-09-04 it does precede
triggers** (`roadmap-v2.md` §3a): the accessor pair landed with RC-4, so the
waiting bought what it could, and the performance pool now builds a 613.8
wrong answer. Nothing here changes — RA–RE stay ahead of CR 603.

**The honest cost of going first.** Three corpus atoms in the CR 614/615 family
are tagged Phase 7 because they assert on triggers, not on replacement:
`ATOM-614.6-001` (a modified event triggers abilities), `ATOM-614.8-002`
(regeneration: damage triggers still fire), `ATOM-615.6-001`. They stay
unclaimable until CR 603 lands. That is 3 of the 58 atoms written directly
against CR 614/615/616 — the corpus already tags them, and no design decision
here changes it.

### Scope, measured

`specdb` (2026-08-24): **88 of Phase 6's 124 atoms are replacement-family.** The
other 36 are the copy system (23 × CR 707, 4 × CR 613.1–2 Layer 1), linked
abilities (2 × CR 607), and stragglers. **"Phase 6" is not one system** — it is
CR 614–616 plus Layer 1/copy, and only the first is this work.

Card breadth this unlocks (Scryfall, 2026-08-24, `unique=cards`):

| Shape | CR | Cards |
|---|---|---|
| "enters tapped" | 614.1c/d, 110.5b | **773** |
| "enters with [N] counters" | 614.1c, 122.6a | **580** |
| "prevent…" | 615 | 525 |
| regenerate | 614.8, 701.19 | 419 |
| "as [this] enters…" | 614.1c | 289 |
| "would die…instead" | 614.6, 616.1 | 98 |
| stun counters | 122.1d | 92 |
| damage replacement ("would deal…instead") | 701.10g, 609.7 | 57 |
| skip effects | 614.10 | 54 |
| "would be put into a graveyard…exile instead" | 400.6, 122.1h | 53 |
| draw replacement | 614.11, 121.2a | 47 |
| finality counters | 122.1h | 41 |
| shield counters | 122.1c | 31 |
| counter doublers | 614.16 | 26 |
| life-gain replacement | 119.10 | 21 |
| token doublers | 614.16 | 9 |

The top two rows are the point. **CR 614.1c/d is the single largest card-unlock
in the engine's remaining work**, it needs no triggers, and it is Phase RC.

### 5b. Three corrections from a judge-corpus pass (2026-08-26)

*Evicted 2026-09-15 from `plans/replacement-architecture.md`, where the heading and a stub remain.*


Five card interactions were put to this design by the owner. Two confirm it, one
moves the RC seam (see §9's RC-1…RC-4 split), and two add rules it did not state.

**The overlay is about one object, and the rest of the board is read as it
actually is — second worked example.** Elvish Archdruid ("Other Elf creatures you
control get +1/+1") enters while Master Biomancer ("Each other creature you
control enters with a number of additional +1/+1 counters on it equal to this
creature's power", a 2/4 **Elf** Wizard) is on the battlefield. Archdruid enters
with **2** counters, not 3: Biomancer's power is read off the real board, where
Archdruid's anthem is not yet applying, because Archdruid is not yet a permanent
and CR 604.3 makes its static ability function on the battlefield. This is §5's
second note working, stated from the other side — Thassa shows a *count* over the
board is not perturbed; Archdruid shows another *object's characteristics* are
not either. §5a's table gains a row:

| Read | Sees the entering object? |
|---|---|
| **Any other object's characteristics** (an `AmountExpr` reading a permanent's power, a filter's `you`) | **no** — computed against the real board |

Note the asymmetry this creates and do not smooth it over: clause (2) puts the
entering object's own anthem into *its own* frame, but that anthem does not reach
any other object's frame. One object is hypothetical; nothing else is.

**Simultaneous entries are computed against the board before any of them
entered.** Two Master Biomancers entering as one event give each other nothing —
neither is on the battlefield when the other's replacements are applied. RA-3's
`execute_actions` is the mechanism (one batch, one `BatchId`), and the rule RC
owes is that the *frame* is batch-scoped: every member's look-ahead reads the
pre-batch board. This is a different axis from §4.2's per-member `applied` set,
which is about CR 614.5 and stays per-event. **Delivered by RC-4b rather than by
the RC-5 piece that was sized for it** — once the entry is a phase-1 proposal
every member is decided before any is performed, so the frame is batch-scoped by
construction; RC-5 supplies the two-Biomancer test and §9's RC-5 entry has the
re-size.

**Not every judge example is a requirement.** Uphill Battle ("Creatures played
by your opponents enter tapped") looked like a demand for a "played by" filter
leaf until it was counted: `o:"played by"` matches **1 card in all of Magic**
(Scryfall, 2026-08-26). It stays a worked example of why CR 110.2b's default
controller matters (`codebase-state.md` item 9) and buys no `ObjectFilter`
vocabulary. Apply §8c's two-customers-before-a-variant guard to interaction
findings as well as to cards — an illuminating example is not automatically a
breadth argument.

**Two copy effects: the later one overwrites the earlier, riders included.** A
copy `Rewrite` is a *set* of the copiable values, not a modification of them
(CR 707.2), so a second copy-on-enter replacement discards the first's result
*and* its "except ..." clauses. §3.2b's algebra must not model copy as a
composable modify, and §3.2c's 0/574 completeness claim was measured without
copy-on-enter in scope — re-check it when Layer 1 lands and RC-4 fills the
CR 616.1c bucket.

## 8b. Sizing — how big do these types actually get?

*Evicted 2026-09-15 from `plans/replacement-architecture.md`, where the heading and a stub remain.*


§8a establishes that the growth axis is `GameAction`. That is only reassuring if
`GameAction` has a knowable size, so this section measures it. Asked in review:
*a bird's-eye view of roughly how big these structs get would ground the
performance and sprawl claims.* Agreed — here it is.

### Method, and what it is worth

Two independent bounds, top-down and bottom-up.

**Top-down:** CR 701 enumerates the keyword actions. `tmnt.txt` has **67** of
them (701.2 Activate through 701.68 Blight). That is the whole universe of named
game actions; it grows by a few per set and never by more.

**Bottom-up:** a keyword action needs its own `GameAction` variant only if some
card watches it **as a unit** — a replacement ("if you would X … instead") or a
trigger ("whenever … X"). Otherwise it decomposes and needs nothing. Both hooks
are counted because §2's spine gives them one vocabulary. 46 keyword actions plus
10 core mutations were queried against Scryfall (2026-08-24), one text search
per phrasing.

**These numbers are the record; there is no script behind them.** A `sizing`
subcommand existed briefly and was **deleted rather than fixed** (2026-08-24),
because two of its readings were known-wrong — Regenerate came back 0, since it
replaces *destruction* and nothing says "would be regenerated", and
`EnterBattlefield`'s 7,211 is every ETB creature ever printed. A tool that
prints answers known to be wrong is worse than no tool. This was a one-time
bird's-eye question; if it needs re-asking, write a better instrument then.

**Precision caveat, stated up front:** these are per-phrasing text searches, so
they are order-of-magnitude signals, not exact counts. Two known distortions:
Regenerate reports 0 because nothing says "would be regenerated" — it replaces
*destruction*, which is the point, not an omission; and `EnterBattlefield`'s
7,211 triggers is essentially "every ETB creature ever printed", which is true
but not informative. Read the column ordering, not the digits.

### What is watched, and by which hook

| Event kind | replacements | triggers | Needs its own `GameAction`? |
|---|---|---|---|
| Enters the battlefield | 15 | 7211 | **yes** — RC |
| Step/phase/turn begins | 9 | 2656 | **yes** — RE |
| Cast a spell | 0 | 2263 | **no** — event only, see below |
| Zone change | 110 | 1436 | **yes** — RA |
| Discard | 7 | 1045 | no — `ZoneChange { cause: Discarded }` |
| Deal damage | 145 | 817 | **yes** — exists |
| Draw | 47 | 391 | **yes** — RA/RE |
| Gain life | 21 | 351 | **yes** — exists |
| Sacrifice | 0 | 278 | no — `ZoneChange { cause: Sacrificed }` |
| Create tokens | 32 | 249 | **yes** — RE |
| Counter a spell | 1 | 172 | **yes** — RE |
| Tap / untap | 2 | 150 | **yes** — RA |
| Fight | 0 | 121 | no — two `DealDamage` |
| Lose life | 10 | 107 | **yes** — exists |
| Counters placed | 21 | 72 | **yes** — RB |
| Exile | 0 | 74 | no — `ZoneChange { to: Exile }` |
| Connive / Explore / Mill / Scry | 6 | 176 | mixed — see below |
| Lose the game | 4 | 26 | **yes** — §8a |
| Roll dice / flip coin | 8 | 27 | **yes** — CR 705/706, Phase 8 |
| Search, shuffle, reveal, vote, goad, transform, proliferate, exert, attach, activate, surveil, monstrosity, investigate, discover, manifest, amass, learn, forage, incubate, adapt, clash, play | 5 | ~200 total | mostly Phase 8, mostly decomposing |
| Exchange, Regenerate, Populate, Venture, Support, Bolster, Meld, Detain, Fateseal | 0 | 0 | **never** — watched by nothing |

**Nine of the 46 sampled keyword actions are watched by nothing at all.** They
can never need a variant, because a variant exists only to be matched against.

### Three findings that do the sizing work

**1. `ZoneChange { cause }` is where the collapse happens.** Discard (1,045
triggers), sacrifice (278), exile (74), mill (50), destroy-as-a-result, shuffle-
into, return, and put-onto-battlefield are all one variant plus a field on
`ZoneChangeCause`. That is **eight keyword actions and ~1,500 trigger-watching
cards absorbed by one arm.** It is the single most important structural decision
in §3.1 and the reason the enum does not track the card pool.

**2. The trigger vocabulary is strictly larger than the replaceable one, and
this phase only pays for the smaller.** Cast is the clearest case: 2,263 cards
trigger on it, **zero** replace it, so casting is a `GameEvent` with no
`GameAction`. Same for attack/block declarations and ability activation. So
`EventPattern` — one arm per `GameAction`, per §3.2a — stays smaller than the CR
603 matcher will eventually need, and CR 603's extra breadth lands in `GameEvent`
(already 28 variants) rather than here.

**3. Most events are of a kind nothing watches, which is what makes a gate
work.** Seven event kinds carry ~390 of the 574 replacement clauses. On a real
board the overwhelming majority of proposed actions are of kinds with zero
registered watchers, so §8's answer-preserving event-kind bitmask should skip the
gather sweep outright for most calls. That is now a measured expectation rather
than a hope — and it is still gated on measuring first.

### The projection

| Type | At end of RE | Commander-viable pool | Ceiling |
|---|---|---|---|
| `GameAction` | **~16** | ~30 | low 40s |
| `EventPattern` | = `GameAction`, by contract (§3.2a) | | |
| `Rewrite` | **5** | 5 | 5 + `AmountRewrite`'s clamp |
| `ZoneChangeCause` | **18, derived** (§11) | ~20 | ~20 |
| `ReplacementClass` | 5 | 5 | 5 (CR 616.1a–e is closed) |
| `Uses` | 4 | 4 | 4 |

The ceiling is derived, not guessed: 67 CR 701 actions, minus the ~30 that
provably decompose into other mutations, minus the 9 nothing watches, plus the
~10 non-701 core mutations (damage, life, draw, counters, zone change, tap,
untap, step begin, mana, game loss).

**For calibration, against types this codebase already runs on:**

| Existing enum | Variants |
|---|---|
| `Primitive` | 36 |
| `GameEvent` | 28 |
| `EffectModification` | 21 |
| `CounterType` | 16 |
| `ChoiceKind` | 13 |
| `Layer` | 10 |

`GameAction` at 16→30 lands squarely inside the size class of types that already
exist here and have not caused trouble — and unlike `Primitive`, its growth is
bounded by an enumerated CR chapter rather than by what cards decide to do next.

### The per-card cost, stated concretely

Adding a replacement-effect card costs, in the common case, **one `ReplacementDef`
value in `src/cards/*.rs` and nothing else** — that is §0 commitment one, and the
§3.2c pass says 549 of 574 clauses land there. When a card needs an event kind
the engine does not propose, it costs **three edits in three known files**: a
`GameAction` variant, a `perform_action` arm, an `EventPattern` arm. It never
costs a `Rewrite` arm — 0 of 574 did.

## 8c. Where card breadth actually lands — three axes, not one

*Evicted 2026-09-15 from `plans/replacement-architecture.md`, where the heading and a stub remain.*


§8b counted keyword actions, and review pushed back correctly: **plenty of
mechanically distinct cards are not keyword-shaped at all.** Alms Collector,
Don't Blink, Krark's Thumb, Tekuthal, Aeon Engine, Aboleth Spawn. The worry
behind the list — could `GameEvent` reach "thousands of entries", and would that
sink performance and maintainability — deserves a direct answer rather than
another count.

**The answer is that three different things are being sized as one.** Card
variety is real and unbounded. It lands almost entirely on the axis that is
*designed* to absorb it, and almost not at all on the enums.

| Axis | Shape | Grows with | Size |
|---|---|---|---|
| **1. Event kinds** — `GameAction`, `GameEvent` | closed enum | the engine's mutations, bounded by CR 701 | 16→50 (§8b) |
| **2. Predicates over events** — `EventPattern`'s field constraints, trigger conditions | **composed grammar**, not enumerated | card variety — **this is where breadth lands** | unbounded expressions, small grammar |
| **3. Results** — `Effect` / `then` | existing tree | card variety | already exists, already absorbs it |

Nothing in the design has a variant per card, which is the only way to reach
thousands. A card contributes a *value*, not an arm.

### The six cards, worked through

| Card | Event kind | Predicate (axis 2) | Result | New engine surface |
|---|---|---|---|---|
| **Alms Collector** — "if an opponent would draw two or more cards, instead you and that player each draw a card" | `DrawCards { player, n }`, the CR 121.2a *outer* event — Phase RE | `player is opponent && n >= 2` | `Instead(DrawCards{opp,1})` + `then:` you draw 1 | **none** |
| **Don't Blink** — "if one or more creatures would enter from exile or after being cast from exile, their owners shuffle them into their libraries instead" | `EnterBattlefield` — RC | `is creature && (from == Exile \|\| cast_from == Exile)` | `Instead(ZoneChange → Library, shuffled)` | **one struct field** — `StackEntry.cast_from: Zone` |
| **Krark's Thumb** — "if you would flip a coin, instead flip two coins and ignore one" | `FlipCoin` — CR 705, already on §8a's list | `player is you` | `Instead(FlipCoins{2})` + `then:` DP picks one to ignore | one variant, already budgeted |
| **Tekuthal** — "if you would proliferate, proliferate twice instead" | `Proliferate { times }` — Phase 8 | `player is you` | `Amount(Times(2))` on `times` | one variant, already budgeted |
| **Aeon Engine** — "reverse the game's turn order" | **not an event at all** | — | — | a `GameState` field |
| **Aboleth Spawn** — "whenever a creature entering under an opponent's control causes a triggered ability of that creature to trigger, you may copy that ability" | `AbilityTriggered { source, ability, cause }` — CR 603 | `cause is EnterBattlefield && cause.controller is opponent && ability.source == cause.object` | (a trigger, not a replacement) | **one `GameEvent` variant, carrying its cause** |

Total new engine surface across six deliberately awkward cards: **three enum
variants, all already budgeted in §8a/§8b; one struct field; one `GameState`
field. Zero new `Rewrite` arms. Zero per-card code.** Four of the six are pure
data.

Three of them teach something worth keeping:

- **Alms Collector proves the outer `DrawCards { n }` event is load-bearing**, not
  a convenience. CR 121.2a exists precisely so "would draw two or more" has an
  event to match; without the outer action the card is inexpressible. It also
  shows where "you *and* that player each draw" goes: the replaced event is the
  opponent's draw, and your draw is the `then` half (CR 615.5's shape). That
  boundary is a modelling judgment the CR does not always draw sharply — when it
  is ambiguous, ask which half can itself be replaced, because that half is the
  event.
- **Don't Blink needs one field the engine does not have** — where a spell was
  cast from. `StackEntry` carries controller, targets, modes, X and costs, but
  not the origin zone. Second customer: CR 903.8's commander tax counts casts
  *from the command zone*. (Flashback belongs to the same *class* — a stack
  object remembering how it got there — but not to this field: CR 702.34a keys
  on whether the flashback **cost was paid**, which `StackEntry.
  chosen_alternative_cost` already records.) One field, two customers, lands
  with RA.
- **Aeon Engine is the useful negative.** Reversing turn order is a
  game-rule-modifying continuous effect (CR 611.2c), not an event and not a
  replacement. The corpus already has `ATOM-611.2c-002` for exactly this class.
  **Not everything mechanically strange is an event** — some of it is a field on
  `GameState`, and mistaking the two is how an event vocabulary starts growing
  without bound.

**Aboleth Spawn is the one that looks worst and is not.** A trigger that watches
other triggers sounds like it needs the trigger system to know about itself. It
needs one `GameEvent` variant — "an ability triggered" — carrying *why* it
triggered. The card's specificity ("a creature entering under an opponent's
control causes that creature's own ability to trigger") is entirely axis 2. What
it genuinely needs beyond that is CR 706 ability copying and an inspectable
pending-trigger queue, which is CR 603's problem and already on its list.

### Performance: enum size is not the cost, and a bounded enum is the fix

Worth separating firmly, because the review put them together.

**Enum size costs nothing.** A `match` over 50 variants compiles to a jump
table. Going from 28 `GameEvent` variants to 50 is not measurable.

**What costs is watchers consulted per event** — and that scales with *board
size*, not vocabulary size. Concretely, at the v1 target: 4-player Commander,
~15–25 permanents each, so 60–100 permanents carrying maybe 100–300 abilities
between them. That is the population, and it is hundreds, not thousands.

**And this is where the closed enum pays for itself.** Because event kinds are a
small closed set, watchers can be indexed by kind in a dense array — a `u8`
discriminant into `Vec<Vec<WatcherId>>` — so an event consults only the handful
registered for its kind and skips the sweep entirely when that bucket is empty.
§8b's distribution says most buckets *are* empty: seven kinds carry ~390 of the
574 replacement clauses. An open-ended vocabulary (strings, per-card ids) would
force a hash lookup and make the empty-bucket fast path impossible. **The
bounded enum is not a constraint the performance story survives; it is the
mechanism the performance story runs on.**

### The genuine risk, named: axis 2 becoming a mini-language

The design's real exposure is not enum count. It is that `EventPattern`'s
constraint vocabulary and the filter types it borrows (`ObjectFilter`, 9
variants; `AmountExpr`, 9) grow one variant per awkward card until they are an
untyped DSL nobody can review. That is the failure mode to watch, and it is the
one this document cannot close by measurement today — it needs Phase 8 data.

Three guards, adopted now because they are cheap now:

1. **Compose, do not enumerate.** `And`/`Not`/`ByController`/`ByType` already
   compose; a new leaf must be a genuinely primitive question, never a card's
   whole condition spelled as one variant.
2. **Two customers before a variant.** A predicate leaf with exactly one card
   behind it is the warning sign. One customer is a card-specific predicate;
   several is a grammar feature.
3. **Budget an escape hatch, and measure it.** Every mature engine has per-card
   code for a long tail, and `design_doc.md` reserved `Custom(CardId)` for it.
   Keep that reservation. The boundary is principled: a card may get custom code
   for a unique *predicate* or *result*; it may never get a custom *event kind*,
   because that is what breaks the index above and hides the mutation from CR 614
   and CR 603 both. **If more than ~5% of the Phase 8 pool needs the hatch, the
   grammar is wrong and should be revisited rather than patched** — record the
   fraction in `codebase-state.md` as the pool grows.

### Should the grammar work move earlier?

Asked at merge (2026-08-24), given that the predicate grammar is the last open
item: is it worth pulling that work forward? **No — but change what RB ships,
not when it ships.**

**Reordering is not available.** The grammar's first real pressure is Phase RD's
two-sided damage predicates (CR 615.10's own Daunting Defender: "if a source
would deal damage to a Cleric creature you control"), and its strangest is RE's
*stateful* ones (Teferi's and Notion Thief both say "except the first one you
draw in each of your draw steps", which needs a per-turn draw counter, not a
filter). Neither can move: RD needs RB's pipeline and RA's batch form, RE needs
both. Moving them earlier means building the pipeline twice.

**But there is a real hazard in leaving it, and it is cheap to fix.** Everything
RB currently ships has a *trivial* predicate — shield/stun/finality counters are
"this permanent", regeneration is `SourceOnly`, and RC-2 is `SourceOnly`
unconditional by design. So `EventPattern` would be **defined in RB under no
pressure at all** and first stressed two phases later. That is the
designed-against-the-easy-case failure, and this project has paid for it before:
`ObjectSet::Filter` carried a controller snapshot until CR 109.5 proved it
wrong, because nothing at design time had a moving controller.

**The fix is one card, not a reordering.** Add a filter-based, two-sided
replacement to RB's card list so the grammar takes real weight the moment its
type is written. Kalitas, Traitor of Ghet is the natural pick — "If a nontoken
creature an opponent controls would die, instead exile that card and create a
2/2 black Zombie creature token" exercises `EventPattern` over a zone change, `ObjectSet::Filter` with
two clauses and an opponent-relative controller, and the `then` half, all at
once. It also immediately demands one grammar leaf `ObjectFilter` lacks —
nontoken — which is the *point*: it is a live test of the "two customers before
a variant" guard at the moment the guard is cheapest to apply.

**And one bounded research pass, not a census.** Predicates are free-form text,
so a regex bucketing would be the same instrument that just got the `sizing`
numbers wrong. Instead: read ~50 clauses by hand during RB and tabulate which
predicate leaves they actually need. An hour, honest, and it lands before RD
commits to a shape.

### Verdict

Manageable, on the evidence available, with one honestly open question.

Closed by measurement: axis 1 is bounded (§8b), `Rewrite` is closed and tested
at 0/574 (§3.2c), and six adversarially-chosen cards cost three budgeted
variants and two fields between them.

Open until Phase 8: whether the axis-2 grammar stays a grammar. That is the
right thing to be nervous about, and thinking about it now is not premature —
the guards above cost nothing today and are expensive to retrofit once a hundred
cards depend on the shape.

**First data point, RD-3 (2026-09-09).** The section predicted RD's two-sided
damage predicates as the grammar's first real pressure, and they landed:
`EventPattern::DealDamage` grew a `SourcePattern` and eight printed cards were
written against it. **`ObjectFilter` grew by nothing.** The leaves the source
side reached — `ByColor`, `ByController`, `And` — all existed with customers
of their own, and guard 2 refused the one leaf a card wanted: "the effect's own
host as the source", one customer (Sokrates, Athenian Teacher), recorded rather
than written. Guard 1 is what made that possible — composition meant Torbran's
whole two-sided condition was two existing leaves and an `And`, not a variant.
One data point is not the answer to the open question, but it is the answer
going the right way, and it is the *hardest* single case the phase plan named.

#### The design check — seven decisions, and the one nobody asked

*Evicted 2026-09-15 from `plans/replacement-architecture.md` §9's Phase RD, where the heading and a stub remain.*


Read against CR 615 whole, 614.7a, 614.9, 609.7a–c, 120.3, 120.3c and 701.10g
(`MTG-Rules/versions/tmnt.txt`), §4.1's six rules, §3.2c/d, §8a, §8c and §11
items 11, 15, 16, 18, 20. Every consumer named below had its oracle text and
rulings fetched from Scryfall on 2026-09-08 (`engineering-practices.md` §3.4);
the counts are from the same sitting.

**0. The affected set must name a player, and that is RD's, not RE's.**
`types/replacement.rs` records the decision that `EventPattern` gets no
`DrawCard`/`GainLife`/`LoseLife` arm until "the player-scoping mechanism" lands
in RE with the draw cards, and `gather::set_affects` returns `false` for every
`EventSubject::Player`. That was the right call for RB and it does not survive
contact with the damage family: of the consumers below, only Daunting Defender,
Samite Censer-Bearer and Pyroclasm's targets are object-only. Circle of
Protection, Reverse Damage, Guardian Seraph, Safe Passage, Pariah, Palisade
Giant, Fog, Mending Hands' "any target", Kitsune Palliator's "each player" and
Furnace of Rath's "permanent or player" all shield or modify damage *to a
player* — Scryfall: `o:"prevent all damage that would be dealt to you"` **23**,
`o:/would deal damage to (you|a player), prevent/` **12**, and nearly all of the
**46** "source of your choice" cards say "to you". A damage phase that cannot
scope an effect to a player has no consumers. **So RD-1 builds it**, and RE
inherits it.

The shape is **a second field, not a variant**. `ObjectSet` is read by three
systems — the layer walk's `row_affected`, the restriction sweep and this
pipeline — and a `Player` arm would be a variant two of the three must reject at
every match, which is the "one type with a flag" smell §11 item 2 warns about
from the other side. `ReplacementDef` gains `affected_players: PlayerSet`
(`{ Nobody, You, Opponents, Everyone, Fixed(Vec<PlayerId>) }`, resolved against
the instance's controller exactly as `Filter`'s `PlayerRef` is, CR 109.5), with
union semantics: `set_affects` consults `affected_objects` for an object subject and
`affected_players` for a player subject. Fog is `Filter { All }` + `Everyone`;
Safe Passage is `Filter { creatures you control }` + `You`; a Circle is
`Fixed(vec![])` + `You`; a targeted "any target" is filled in at resolution as
`Fixed([object])` or `Fixed([player])` the way `Primitive::Regenerate` fills its
set today. Nothing about `ObjectSet` moves, so §11 item 2 holds byte for byte
and the CR 614.12 `SourceOnly` check in `gather` is untouched.
`Restriction::ApplyReplacement { to }` needs the same second field for
"damage can't be prevented" over damage to a player; RD-4 adds it there.

Two things ride on this field. `chooser_for` already answers a player subject
with that player, so CR 616.1's chooser is free. `Rider.subject` flattens a
player subject to `None` (§11 item 16, `codebase-state.md` item 27) and its
trigger was named as RD; RD-1 carries `EventSubject` on the rider and emits
`ResolvedTarget::Player`, because Reverse Damage's "you gain life" is the
first rider that rides routinely on a player-subject event.

**`Uses::NextDamage(remaining)` is named for the rule's own phrase**, "the next
3 damage", because it counts damage and never uses: 615.7's last sentence is
"such effects count only the amount of damage; the number of events or sources
dealing it doesn't matter", and a first name, `DamagePoints(remaining)`, could
be read as either. It deliberately does not take the word "shield" in code —
**three different things are called a shield here**, and which is which is
`plans/glossary.md`, under *shield*. That disambiguation was written out in this
section, because RD-2 is where the three meet; the glossary pass (2026-09-11)
moved it, since a reader who needs it is usually not reading RD. Decision 3
below is still the one place senses 2 and 3 meet.

**1. Partial prevention is `Rewrite::Amount(AmountRewrite::PreventUpTo(n))`,
and `Instead` gets no "N − k" template.** §3.2b already lists `Amount` as the
sixth arm with its rule numbers — CR 614.5's own doubling example and 615.7 —
and it is absent from the enum only because an arm the pipeline cannot apply is
worse than a missing one. So this is the planned arm arriving, not a new claim
on the closed algebra. CR 615.10 is the sentence that permits *partial*
prevention as an operation on the event's amount ("prevents only the indicated
amount of damage in any applicable damage event") and 615.7 the same for
shields ("each 1 damage … is prevented"). Four reasons it is not an `Instead`
carrying `DealDamage { amount: N − k }`, and the first is the CDA lesson:

- **Two channels to one answer.** An `Instead` template that reads the event's
  amount and subtracts is the same arithmetic as `Amount`, spelled in the
  unbounded arm. The tree would then have two ways to say "prevent 1 of that
  damage", and a reviewer could not tell an authoring error from a choice.
- **Composition is the whole test.** Furnace of Rath's printed ruling —
  prevent 4 then double the remaining 1, or double to 10 then prevent 4 — is
  CR 616.1's choice made *observable* only because each application reads the
  amount the previous one left. `Amount` arms do that by construction; an
  `Instead` that overwrites the event does it only if its template happens to
  read the right field.
- **The pipeline has to know how much was prevented.** CR 615.5's rider may
  "refer to the amount of damage that was prevented" (Reverse Damage, Divine
  Deflection), a 615.7 shield is reduced by exactly that amount, and CR 615.13
  triggers on "some or all" of it. An `Instead` erases the number; `Amount`
  reports it.
- **`Prevent` stays a separate arm.** "Prevent that damage" (CR 615.6, the
  whole event never happens) and "prevent 3 of that damage" (a smaller event
  survives for later effects to see) are different claims, and regeneration and
  CR 122.1c already use the first. A `PreventUpTo` that empties the event
  leaves a 0-damage proposal that `never_happens` drops on the next iteration
  (CR 614.7a), so the two routes agree on the board and differ only in what
  they assert.

`AmountRewrite` ships six variants, each with a customer in the PR that lands
it: `Multiplier(u64)` (Furnace of Rath; CR 701.10g — not `Times`, which reads
as "the number of times something happens" and is the wrong noun for a factor;
§3.2c's and §3.2d's `Amount(Times(2))` are renamed with it), **`Halve(Rounding)`**
(Ghosts of the Innocent, "deals half that damage, rounded down … instead" —
the inverse, and it is printed), `Plus(u64)` (Torbran, RD-3), `PreventUpTo(u64)`
(Daunting Defender, Guardian Seraph; CR 615.10), **`PreventHalf(Rounding)`**
(Gisela, Blade of Goldnight's "prevent half that damage, rounded up" in RD-1;
Dark Sphere's "rounded down", from a resolution with a chosen source, in RD-3),
and `PreventRemaining` (CR 615.7), which reads its cap off the instance's
`Uses::NextDamage(remaining)` — the remaining amount lives in one place, the use
count, and the arm names it rather than repeating it.

**Rounding is authored, never inferred.** CR 107.1a: "If a spell or ability
could generate a fractional number, the spell or ability will tell you whether
to round up or down." So `Rounding { Up, Down }` has no `Default`, and every
halving names its direction — the same doctrine as `Duration` on
`Primitive::Restrict`. `Halve` and `PreventHalf` are two arms and not one with
a flag because CR 615.12 treats them differently and the rulings say so in
words: Ghosts of the Innocent "isn't a damage prevention effect" and still
halves Excruciator's unpreventable 7 to 3, while Gisela prevents nothing of
it; and only the prevention arm reports a prevented amount. Ghosts' other
rulings are RD-1's tests: *half of 1 rounded down is 0, so a 1-damage source
deals no damage at all* (a rewrite to 0 meets `never_happens` on the next
iteration, CR 614.7a); *three Ghosts turn 14 into 7, 3, 1* (three instances,
each once); *with Furnace of Rath the order is the affected player's, and it
matters when the amount is odd — 3 halves to 1 then doubles to 2* (the first
non-commuting CR 616.1 choice reachable from two printed statics, and
`COMP-614-DAMAGE-ORDERING-001`'s board); *redirected damage is halved once*
(CR 614.5's applied set survives a `Retarget`, RD-4). A rider can halve too —
Sokrates, Athenian Teacher's granted "each draw half that many cards, rounded
down" — but that rounding sits on the rider's `AmountExpr` (a `Half(Box<
AmountExpr>, Rounding)` leaf beside `Multiply`), not on any rewrite; Sokrates
himself waits on RS-2's hexproof and is recorded under RD-3 as the shape.
§3.2c's
Ali from Cairo clamp is **not** here: Ali's own ruling says "this effect does
not prevent damage, it prevents the damage from turning into loss of life", so
it watches the contained `LoseLife` and is RE's, once RE gives `LoseLife` a
pattern arm (decision 4 gives it the `cause` field Ali needs).

**2. Resolution-created prevention lives in `ReplacementEffectRegistry`,
through `Primitive::CreateReplacement(Box<ReplacementDef>, Duration)`, and the
shield fits a duration registry because a row already has two ends.** Settled
against its two twins rather than alone: RS-1's `Primitive::Restrict(def,
Duration)` into `RestrictionRegistry`, and `cost-architecture.md` §3.10's
`Primitive::ModifyCost(def, Duration)` into a `DurationRegistry` of its own.
This is the third instance of one pattern — a def the card authors, a
`Duration` the card authors (CR 608.2c hands scope to a human reader, so no
engine may infer it; `cant-effects-architecture.md` §9 finding 1), a row
carrying controller and `created_on_turn`, expiry through the CR 514.2 hooks
the registry already runs. The registry is not new: `Primitive::Regenerate` has
been putting rows in it since RB. The commented-out
`Primitive::ApplyPrevention(PreventionEffectDef)` in `types/effects.rs` is
deleted rather than filled in — CR 615.1 opens "like replacement effects", a
prevention effect *is* a `ReplacementDef` whose rewrite prevents, and a second
def type would be one mechanism reached through two channels.

The consumable-amount question has an in-tree answer. A regeneration row is
`Uses::Once` **and** `Duration::UntilEndOfTurn`: it ends on use through
`consume_use`'s `remove(row)` and on time through
`remove_expired_at_cleanup`, and neither end knows about the other.
`Uses::NextDamage(remaining)` is the same row with a number where `Once` has a
bit — decremented in place through `DurationRegistry::update_rows`, removed at
zero, expired at cleanup with the rest. "Expires on use, not on time" was the
wrong dichotomy: it is both, it always was, and the registry was built for it.
(`types/replacement.rs` promised the variant as `Shield(u64)`; the name changes
for the reason the glossary above gives, and RD-2 corrects the comment.)

What the primitive does at resolution follows `Regenerate` and `Restrict`: one
row per resolved target, the resolution filling an authored empty `Fixed` with
that target (CR 615.11 is exactly this — "creates a prevention shield for each
applicable creature when the spell or ability … resolves" — so Samite
Censer-Bearer is N rows of `NextDamage(1)`, one per creature it found, and Kitsune
Palliator's ruling "doesn't affect creatures that enter later" falls out of
`Fixed`); or one row as authored when the def carries a `Filter` or a
`PlayerSet` and no target (Safe Passage's ruling, the opposite one: "will
prevent damage dealt to creatures that weren't on the battlefield at the time
it resolved" — `Filter`, evaluated at the event). A pooled amount across
several subjects needs no third shape: a `Filter` + `PlayerSet` row carrying
`NextDamage(n)` *is* one pool by construction, which is Divine Deflection's
"the next X damage that would be dealt to you and/or permanents you control"
— waiting only on `AmountExpr::Variable` — and Harm's Way's 2, which waits on
nothing here and is RD-5's (finding 23).

**3. CR 615.7's allocation ships with the first shield, in RD-2, and it is
the same change as §11 item 15.** "If damage would be dealt to the shielded
permanent or player by two or more applicable sources at the same time, the
player or the controller of the permanent chooses which damage the shield
prevents." Two blockers on one attacker, or two attackers on one player, is
that board, and the registered pool builds it in every combat step — so the day
Mending Hands is registered the first-come answer the per-member loop gives is
a **silent wrong choice**, the shape `engineering-practices.md` §3.3 says a
phase must not open. It cannot be a later PR's.

Item 15 asked RD to open with CR 614.5's identity, because a creature blocking
two attackers with two shield counters loses two counters where CR 122.1c's
ruling says one. The allocation is the same question from the other side, and
one rule answers both: **decisions are per `(batch, subject)`; rewrites apply
per member.** In phase 1, members that share an `EventSubject` form a group;
the CR 616.1 loop runs once per group, with one applied set and one chooser;
a chosen instance's rewrite is applied to every member of the group, its rider
queued once, its use spent once. Checked against every ruling the batch has
had to satisfy:

| Board | Per member (today) | Per subject (RD-2) | The rule |
|---|---|---|---|
| Kalitas, N opposing creatures die | N Zombies | N Zombies — N subjects | CR 614.5 per event; Kalitas's ruling |
| two shield **counters**, two blockers | 2 counters, both prevented | **1** counter, both prevented | CR 122.1c; the SNC ruling ("that damage is prevented and only one shield counter is removed") |
| two shield **counters**, one blocker with first strike and one without | 2 counters | **2** counters — CR 510.4 makes two combat damage steps, so two batches and two groups | CR 510.4: the key is the *batch*, and this is the contrast that proves it |
| two Furnaces, one source | ×4 | ×4 | CR 614.5's own example |
| two Furnaces, two attackers on one player | ×4 each | ×4 each — chosen once, applied to each member | CR 614.5 per instance |
| Daunting Defender, Pyroclasm on two Clerics | 1 each | 1 each — two subjects | CR 615.10's own example |
| Daunting Defender, two sources on one Cleric | 1 each | 1 each — 615.10's "separately to … events that would happen at the same time" | CR 615.10 |
| a 3-shield, sources of 2 and 4 | 2 then 1, no choice | one `allocate` over the group | **CR 615.7** |

The last row is the one place a rewrite is not member-uniform, and it is the
whole of the new prompt: `ChoiceKind::AllocateNextDamage { source, remaining
}`, asked through `DecisionProvider::allocate` (the trample call), options the
members in batch order — `assign_combat_damage` walks `battlefield_ordered`,
so the order is process-independent — total `min(remaining, Σ amounts)`,
per-bucket max the member's amount. **The allocation is per instance, not per
group**, and the two words mean different things. A *subject group* is the
set of batch members about one object or player — the two attackers' damage
to *you* — and it is the unit the CR 616.1 loop decides for. An *instance* is
one replacement effect — one registry row, one static ability on one object,
one counter-derived effect — and it is the unit that owns a `NextDamage`
count. One row asks once across every member of the batch it applies to,
whatever their subjects. For Mending Hands that is one subject's group; for Divine
Deflection's and Harm's Way's "you and/or permanents you control" it spans
subjects, which their rulings spell out ("you choose which of that damage to
prevent"; "1 damage … to each of two different recipients"). The chooser is
the affected side's, and it is well-defined because every printed
multi-subject amount shield is scoped to one player and that player's
permanents; the engine asserts one chooser rather than guessing between two.
**When it is asked**: the first time the instance is chosen in any group's
loop, over the members it still applies to at their *then-current* amounts —
Divine Deflection's ruling is "you don't decide until the point at which the
damage would be dealt" — and the answer is kept for the groups decided after.
A doubling chosen ahead of the shield in a later group's loop then moves that
member, not the allocation; that is CR 616.1's own per-subject ordering
showing through, and it is recorded as a corner for RD-2's review rather than
smoothed over.
Never with one member: CR 615.7's choice exists only among "two or more", and
with one source every point is prevented unasked. The decisions live in a
batch-scoped struct on `GameState` beside `EntrySelectionScope`, on
`codebase-state.md` item 40's rule, and are empty outside phase 1. §3.2d's
lineage rule is untouched — it is about decomposition, and this is about
simultaneity — and RC's entries are unaffected, since an entry is its own
subject. Phase 1's other standing claim, `codebase-state.md` item 25 (members
are decided one whole CR 616.1f loop at a time, in APNAP order), was checked
for collateral and is untouched: grouping changes the loop's *unit*, not the
order choosers are asked in.

**4. The contained `LoseLife` never meets a prevention effect, and what
enforces that is the vocabulary, not the lineage rule.** Performing
`DealDamage` against a player proposes `LoseLife { cause: Damage { source } }`
from inside the performer, joining the damage's batch as lifelink's gain does
(CR 120.3a/f, 120.4c/d; `GameEvent::LifeChanged` keeps its `source`, so the
log line is unchanged). It re-enters `apply_replacements` with a **fresh**
applied set — containment, §3.2d — which is what lets an RE-era Bloodletter
double it while a shield that already applied to the damage does not apply
again. Fresh lineage says nothing about *which* effects may apply, and does
not need to: a prevention effect is one whose rewrite is `Prevent`,
`PreventUpTo` or `PreventRemaining` on `EventPattern::DealDamage` (CR 615.1a,
derived from the def — see decision 6), `LoseLife` has no pattern arm today,
and when RE adds one `apply_rewrite` refuses `PreventUpTo`/`PreventRemaining` on a
non-damage event with the same "its `EventPattern` and its `Rewrite` describe
different events" arm the entry rewrites use. Life loss from damage is not
separately preventable because nothing can be written that prevents it.

**A rider can refer to two different numbers, and CR 615.12 is what tells
them apart.** Reverse Damage gains "life equal to the damage prevented this
way"; Angel of Suffering mills "twice that many cards", where "that many" is
the damage that *would have been dealt*. Under damage that can't be prevented
both riders run (615.12), and Angel's own ruling says it "still mill[s] twice
that many" while Reverse Damage's amount is 0. So `Rider` carries both the
event's amount and the prevented amount, read by two `AmountExpr` leaves —
`ReplacedAmount` (a rider's "that much"/"that many") and `DamagePrevented` —
and the dovetail test in RD-4 is the one that separates them: a fixture
"damage can't be prevented" row, Lightning Bolt to the face under both cards,
3 damage dealt, 6 cards milled, 0 life gained. Angel of Suffering needs
`AmountExpr::Multiply(Box<AmountExpr>, u64)` for "twice" and a working
`Primitive::Mill`, both RD-1's (below).

`LoseLife` gains `cause: LifeLossCause { Damage { source }, Effect, Cost }` —
six construction sites, and a **fact** on §8a's triage: whether a life loss was
a result of damage is unrecoverable a moment later, Ali from Cairo's family and
CR 120.3b's poison both read it, and the `LifeChanged` line needs the source
today. The performer marks damage on a creature and removes loyalty from a
planeswalker (decision 5) in the same arm, each result independent, since
CR 120.3 says "one or more of the following results" and a creature
planeswalker gets both.

**5. CR 120.3c ships in RD-1, and its consumer is a fixture with its own
name.** The three options the brief offered, priced: registering a printed
planeswalker fails `register-a-card-only-once-the-engine-can-play-it` — loyalty
abilities have no `AbilityType` and no activation path (`phase_sba_cards.rs`
row 704.5i), so a real Jace would be a card with three dead abilities wearing a
real name, which `engineering-practices.md` §3 forbids; leaving the arm out
leaves `perform_action` marking damage on a permanent the targeting code
already validates as "any target", which is the wrong answer waiting for a
card. The fixture is the middle: **Loyalty Probe**, a `Planeswalker` with
printed loyalty 3 and no abilities, on the `graveyard_probe` convention §3.3
already admits, registered in the stress pool and not in `PERFORMANCE_POOL`.
It is a genuine consumer, not a test prop: Lightning Bolt's `SelectionFilter::
Any` validates planeswalkers, so the random agent bolts it, CR 120.3c removes
counters through a contained `RemoveCounters { Loyalty }` proposal, and CR
704.5i — **0** across the 2026-09-01 fuzz re-audit, "a planeswalker can never
die" — becomes reachable from a game. Attacking one stays out (`validation.rs`
refuses planeswalker attack targets; combat's, not this phase's).

**6. CR 615.11 needs no machinery; CR 615.12 is a property of the event, and
its three printed shapes meet at one site.** 615.11 is decision 2's per-target
rows and Samite Censer-Bearer is its consumer. 615.12 prints three shapes and
they are three different things: *"Damage can't be prevented this turn"*
(Skullcrack, Unstable Footing, 11 instants) is a **resolution's** CR 101.2
restriction with a duration — `Primitive::Restrict(ApplyReplacement { kind:
Prevention, .. }, UntilEndOfTurn)`, a registry row; *"Damage can't be
prevented"* on a permanent (Leyline of Punishment, Everlasting Torment) is a
**static ability's** restriction — `Effect::Restriction` on the static,
discovered by RS-1's sweep off the source's *effective* ability list, so
Humility strips it for free and it lasts exactly while the source is on the
battlefield with no `Duration` row at all (`cant-effects-architecture.md`
§3.4 source 1), and it may carry a filter where a printed card does; *"The
damage can't be prevented"* (Combust, Pinpoint Avalanche, 9 cards) is a fact
about **one event**, set by the effect that proposes it. The first two are
two *sources* of one `Restriction` arm, which `is_prohibited` already unions;
the third is not a restriction at all. So `GameAction::DealDamage` gains
`unpreventable: bool`, `Primitive::DealDamage` becomes a struct carrying it
(16 mechanical sites), and it is a property of the event rather than of the
source because a source's other damage is preventable and because the flag has
to survive a redirect — Retarget copies it. The two routes meet where the
prevention arms are *applied*: `prevented = 0` when the event is unpreventable
or `is_prohibited(ApplyReplacement { Prevention, to: subject })`, the rider is
queued regardless (615.12's "any additional effects they have will take
place"), and nothing is consumed (615.12's "existing shields won't be
reduced"). Protection's damage half (CR 702.16e) is a prevention effect a
keyword synthesizes — `backlog.md` §2.6's — and Leyline's ruling that "static
abilities that prevent damage (including protection abilities) don't do so"
falls out the day it exists, because it will be a prevention def like any
other. That consult is at application and **not** at `gather`'s door where
regeneration is withheld: CR 701.19c says a regeneration shield is "not
applied", CR 615.12 says a prevention effect "is still applied", and the two
`ReplacementKindFilter` arms therefore act at two different sites. 615.12a's
"just once" is the applied set — the instance was chosen, it is in the set,
and nothing re-offers it.

`cant-effects-architecture.md` §4.7 expected RD to widen `is_regeneration:
bool` into a `ReplacementKind`. **It does not**: CR 615.1a makes "prevention"
a fact about the effect's text ("effects that use the word 'prevent'"), which
the def carries in its rewrite and pattern, so `ReplacementDef::is_prevention()`
is *derived* and no card can forget to set it — the CDA and CR 614.15 lessons
(§11 item 12). Regeneration keeps its authored bit because nothing about its
def distinguishes it from any other `Prevent`-with-a-rider.

**7. A use is spent by what an application did, not by being chosen.** Not one
of the brief's questions, and it changes one line of §4.1's loop. Today
`consume_use` runs *before* `apply_rewrite`. Three rules need it after:
CR 609.7b ("if for any reason the shield prevents no damage or replaces no
damage, the shield isn't used up"), CR 614.9 (a redirect whose destination is
gone "does nothing", and ATOM-614.9-001's "the shield is NOT used up"), and
CR 615.12 (an unpreventable event reduces no shield). `apply_rewrite` returns
what it did — the event, and for a damage arm the amount it prevented or moved
— and `consume_use` spends that: `Once` iff the rewrite took effect, `Shield`
by the amount. Regeneration is unchanged, because a `Prevent` on a `Destroy`
always takes effect. 609.7b's *property* recheck needs nothing here: a source
that is no longer red fails `pattern_watches`, the instance is never gathered,
and nothing was ever there to spend.

#### RD-5 — partial redirection (Harm's Way) — ❌ gate closed 2026-09-09, moved to `backlog.md` §2.25

*Evicted 2026-09-15 from `plans/replacement-architecture.md` §9's Phase RD, where the heading and a stub remain.*


Harm's Way — "The next 2 damage that a source of your choice would deal to
you and/or permanents you control this turn is dealt to any target instead" —
is one card, never played competitively and rare in Commander, and it is
*not* out of scope the way Panglacial Wurm is: nothing in it is a rules
problem, and its rulings are a complete specification. *"If the chosen source
would deal just 1 damage … Harm's Way's effect will redirect that damage and
still have a 'shield' left for another 1 damage from that source later in the
turn"* is decision 7's rule — a `Retarget` row with `NextDamage(2)`, spent by
the amount moved. *"You choose which 2 damage is redirected … 1 damage … to
each of two different recipients"* is decision 3's per-instance allocation
across members. What it needs that RD-1 through RD-4 do not build is the
**split**: 3 damage to you becomes 1 to you and 2 to the target — one
`DealDamage` becoming two, which §3.2d's `Option` cannot return. The shape
(finding 23) is a phase-1 member insertion: the rewrite returns the residue
for this member and a split-off member that **continues the lineage** (the
moved damage is "the same damage", so an effect that already applied to the
whole does not apply again to the part), decided in the same pass and
performed in phase 2 beside its sibling.

**The gate.** The split path is entered only when a `Retarget` instance with
`NextDamage` is chosen, so it cannot cost the hot path anything by
construction; what it can cost is the per-member path's *shape*, since phase 1
would have to accept insertions while iterating. RD-5 ships if that insertion
lands without touching the code every combat step runs — the middle-arm A/B
flat within spread, and no change to `apply_replacements`' single-member
signature — and is sized ~300–400 with Harm's Way as its consumer. If the
insertion cannot be kept off that path, Harm's Way goes to `backlog.md` with
this shape attached and the measured cost as the reason, which is the only
reason one card should ever be excluded.

##### Decided at RD-4's close (2026-09-09): **the gate fails, and Harm's Way is `backlog.md` §2.25**

The criterion's second half is the one that decides it, and it decides it by
reading the tree rather than by opinion. **A split-off member has no batch
index**, and every signature between the rewrite and the performer is keyed by
one:

| Site | Count | What a split moves |
|---|---|---|
| `apply_rewrite` return sites | **20** | `(Option<GameAction>, Applied)` grows a third thing, in every arm |
| `Member.index` reads/writes | **4** | `usize` → `Option<usize>`, or a parallel tail |
| `finish(members)` | **2** | the closure that maps members back to batch positions |
| `apply_replacements` signature + its caller | **2** | `Vec<(usize, Option<GameAction>)>` cannot name an event that was not proposed |
| `execute_batch_inner`'s `decided[i] = action` | **1** | phase 2's write, which every combat damage step runs |

≈ 30 mechanical sites, and the last two rows are the code every combat step
runs. So the gate's own words — "without touching the code every combat step
runs … no change to `apply_replacements`' signature" — are not met, and the
answer §9 wrote for that case is the one that applies.

**Three things worth keeping, because they change what a later PR starts
from.**

- **The cost is a type change, not work.** Nothing in the table is reached
  unless a `Retarget` instance carries a `NextDamage` count, and no def on
  either pool does. A middle-arm A/B of the type change alone would be flat by
  construction. The gate excluded it on the *shape* half of its criterion, not
  on the speed half, and saying so is the difference between a measurement and
  a verdict.
- **RD-4 already paid the part that was hardest to see coming.** The split's
  awkward case was a member whose subject is not the group's; RD-4 made the
  chooser, the prompt and `RemoveCountersFromAffected` read the event rather
  than the group key, because CR 614.9's whole-event redirect needed it first
  (§11 item 35). A later RD-5 starts with that done.
- **What is genuinely new work, and was not in the ~300–400 sizing:**
  `next_damage_shares` would have to allocate a CR 615.7 count across members
  *and* decide how much of each member's amount moves — Harm's Way's own ruling
  ("1 damage … to each of two different recipients") makes those the same
  choice. That is new logic in the one function §11 item 24 calls the
  non-uniform rewrite, not a mechanical edit.

**So `backlog.md` §2.25 owns Harm's Way**, with the shape, this table and the
reason. Divine Deflection is *not* affected and never was: a `Filter` +
`PlayerSet` row with `NextDamage(X)` is already a pooled amount, and it waits
only on `AmountExpr::Variable` and `codebase-state.md` item 90.

#### Measured — what to expect, and why the direction is known

*Evicted 2026-09-15 from `plans/replacement-architecture.md` §9's Phase RD, where the heading and a stub remain.*


Three arms per PR through `plans/fuzz_ab.py` against a same-day `main`
worktree (`../mtgsim_v2_main`, rebuilt first), both pools, and the middle arm —
the engine with `registry.rs` and `PERFORMANCE_POOL` unchanged — is the only
one that attributes anything. Damage is on every combat step, so unlike CM-4
the counters **will** move, and each PR predicts the direction before running:

- **RD-1's middle arm:** `replacement gathers` rises by exactly the number of
  `DamageDealt` events whose target is a player (one contained `LoseLife`
  proposal each) and by nothing else; `Layer walks` flat (the sweep's fast
  path returns before any walk when no source, row or counter exists);
  40-game dumps byte-identical under the two id masks, because `LifeChanged`
  keeps its source. The shipped arm adds walks only while Furnace is on the
  battlefield — one per damage event, since the per-permanent gate walks only
  `replacement_ability_sources`.
- **RD-2:** gathers flat on the middle arm (grouping changes decisions, not
  proposals); the shipped arm's `allocate` count is the reachability row.
- **RD-3 and RD-4:** flat on the middle arm; a shipped-arm move is the card.

A middle-arm delta beyond the ±4–6% spread that the prediction does not name
wants a **fourth binary** with the suspect reverted, as CM-4 needed. The fixture
table is re-recorded in `fuzz-record.md` once per PR that moves the pool, at 50
games, after the A/B.

#### The design check — nine decisions

*Evicted 2026-09-15 from `plans/replacement-architecture.md` §9's Phase RE, where the heading and a stub remain.*


Read against CR 104, 106.6a, 119.5, 119.10, 121.2–121.6, 122.1, 122.6, 500.7,
500.11, 614.1b, 614.10–614.11, 614.16, 616.1g, 616.2, 701.9, 701.22, 704.5a–c,
704.7 and 800.4 (`MTG-Rules/versions/tmnt.txt`), §3.2d, §4.1's six rules, §8a,
§8b and §11 items 6, 18, 21, 26. Every consumer named below had its oracle text
and rulings fetched from Scryfall on 2026-09-11 (`engineering-practices.md`
§3.4); one ruling contradicted this document, and decision 1 says which.

**0. Seven kinds, one algebra, no new `Rewrite` arm.** Every kind lands on an
arm that already has a customer: draw on `Instead` and `Prevent`; life, tokens,
counters and mana on `Amount`; losing on `Prevent` with a rider; winning on
`Instead`; skips on `Prevent` (CR 614.1b's "replaced with nothing" *is*
614.6). What grows is the two open payloads §3.2b already names as open —
`AmountRewrite` gains one variant (decision 2) and `GameActionTemplate` gains
three arms (`DrawCards`, `GainLife`/`LoseLife`, `PlayerWins`), each with a
printed customer in the PR that lands it. §3.2c's census said 0 of 574
clauses needed a sixth arm; RE is the half of that census that was not yet
built, and it holds.

**1. Draw is two events, and the instruction is the outer one.** CR 121.2a:
"an instruction to draw multiple cards can be modified by replacement effects
that refer to the number of cards drawn. This modification occurs before
considering any of the individual card draws." So `GameAction::DrawCards {
player, n, cause }` is the instruction and `DrawCard { player, cause }` the
draw; the outer's performer proposes `n` inners one at a time (CR 121.2), and
**those inners inherit the outer's applied set** — §3.2d's decomposition rule,
whose parameter has waited since RB for this producer (§11 item 18,
`codebase-state.md` item 29). `test_two_teferis_draw_four_not_infinity` is the
regression and hangs rather than fails, so it is written first with a bounded
guard. **Every draw instruction proposes the outer, including "draw a card"**
— one path, and Alms Collector's ruling is the reason it is honest: "count how
many times the word 'draw' is used". Its "two or more" is
`EventPattern::DrawCards { at_least: Option<u64> }`; `EventPattern::DrawCard {
cause: Option<DrawCause> }` is everything else. CR 616.1g's "the second effect
can't be chosen until after the first" is the nesting: the outer is decided and
performed before an inner exists.

`DrawCause { TurnBased, Effect }` is a fact on the event, on RD-1's
`LifeLossCause` reasoning: ten printed cards say "except the first one you draw
in each of your draw steps" and the answer is unrecoverable a moment later. The
draw step's instruction is `TurnBased`; **its first inner is `TurnBased` and
every later one is `Effect`**, at every level of decomposition — so Thought
Reflection doubling the draw-step draw yields a first card Teferi's Ageless
Insight excepts and a second it doubles, which is what "the first one" means.

**The ruling that corrects this document.** §3.2d encodes Notion Thief as
`Prevent` with a rider ("that player skips that draw *and* you draw a card").
Its 2018-03-16 ruling describes two Thieves in a two-player game: the drawing
player applies one, then "the player whose Notion Thief's effect was chosen
repeats this process among the remaining", each "applied to the card draw only
once", and "it really will be that player who draws a card". That only holds if
the Thief's draw is **the same event with a new subject** — CR 614.5's "modified
events that may replace that event" — because a rider's draw is a fresh
proposal with a fresh applied set, and two Thieves would trade the draw forever.
So Notion Thief is `Instead(DrawCards { n: 1, player: Some(You) })`, a subject
change that keeps the lineage, and the `DrawCards` template carries `player:
Option<PlayerRef>` with `None` meaning the affected player — one arm, two
customers (Thought Reflection's `{ n: 2, player: None }`). Alms Collector stays
`Prevent` with a rider: "you and that player each draw a card" is heterogeneous
(§3.2d's own category), and its ruling that Thought Reflection may then double
the resulting draws while Alms "does not apply again" holds either way, since
neither resulting draw is "two or more". §3.2d is corrected in place, dated.

CR 614.11 / 121.6a — a draw replacement applies with an empty library — is
already true by construction: `draw_card` flags the empty library *after* the
pipeline, which is why its comment says so. CR 614.11a / 121.6b — the
replacement completes before the sequence resumes — is the outer performer
proposing one inner at a time and each inner's riders running before the next
(§4.1a). CR 614.11b / 121.6c (an additional action on a drawn card is not
performed when the draw is replaced) has no producer: nothing in `Primitive`
does something to *the card it drew*, so `ATOM-614.11b-001` stays uncovered
with that reason, on the corpus's Phase 6.

**2. Life gains its arms, `LoseLife`'s `cause` gets its first reader, and the
one new `AmountRewrite` reads player state.** `EventPattern::GainLife` and
`EventPattern::LoseLife { cause: Option<LifeLossCause> }`. Ali from Cairo —
"damage that would reduce your life total to less than 1 reduces it to 1
instead" — watches the contained `LoseLife { cause: Damage }` (its own ruling:
"this effect does not prevent damage, it prevents the damage from turning into
loss of life"; RD decision 4 built the `cause` for it) and needs the clamp
§3.2c budgeted: `AmountRewrite::LifeFloor(i64)`, the loss capped so the total
does not drop below the floor, applied against the affected player's life *now*
— the second arm of `apply_rewrite` that reads `GameState` (item 53), and a
read, not a write. The name is open; the CR has no noun for it and §3.2c said
"clamp". `is_prevention` is untouched: its first test is "the pattern is
damage", and this one is not.

Two kind-changing substitutions land here, and they are the mechanism §10's
Eligeth test wanted: `GameActionTemplate::GainLife { amount }` and `LoseLife {
amount }` with `amount: TemplateAmount::{Fixed(u64), ReplacedAmount}`. Words of
Worship turns a draw into "gain 5 life" (`Fixed`); Tainted Remedy turns an
opponent's gain into "loses that much life" (`ReplacedAmount`). Each template
arm has its two customers before it is written.

"Players can't gain life" is a **"can't"**, and Skullcrack and Leyline of
Punishment have waited on it since RD-4 (§11 item 26). `Restriction::Event {
pattern, affected, by }` has no player set, so RE-3 gives it
`affected_players: PlayerSet` — RD-1's field, RD-4's `to_players` precedent,
unioned the way `set_affects` unions — and `is_prohibited` refuses the
`GainLife` proposal ahead of the pipeline (CR 101.2). Leyline of Punishment's
own ruling then falls out of the order of the two checks: "effects that replace
an event with gaining life (like Words of Worship's) will end up replacing the
event with nothing" — the substituted `GainLife` is proposed, refused, and
nothing happens.

**3. `CreateTokens` is the outer event, each entry is contained, and a
substituted entry is a creation somewhere else.** `GameAction::CreateTokens {
defs: Vec<TokenDef>, controller }` — `Vec`, as §3.1 decided for Academy
Manufactor's "one of each" and Anointed Procession's ruling ("twice as many of
each kind"). Its performer creates the objects, then proposes **every entry as
one batch** — `codebase-state.md` item 46's first plural entry, so two tokens
entering together are decided against the pre-batch board (RC-4b's frame) and
"can't apply to … any other permanent entering the battlefield at the same
time as it" (Winding Constrictor's and Pir's rulings) is true of them the day
they exist. CR 616.1g makes the token half of Doubling Season a fresh choice per
token (§3.2d's contrast case) — the entries are *contained*, not decomposed,
and get fresh applied sets.

**The residual, item 52.** A substituted token entry — Hallowed Moonlight's
"exile it instead", whose ruling is "it's put into exile instead and then
ceases to exist" — is performed today as `ZoneChange { from: Battlefield }` for
an object that was never there, which is the line Dour Port-Mage and Aang read.
The honest event is an *appearance*: the token is created in exile, from
nowhere. `GameAction::ZoneChange.from` stays `Zone` — twenty constructions in
`src/`, five in tests, three readers of `from` in the move path, and an
`Option` there is a catchall-shaped `None` on every card move for one token's
sake. Instead the `Instead(ZoneChangeTo)` arm on an entry with `from: None`
returns a `from`-less variant — working name `GameAction::CreateTokenIn {
object, zone }` — whose performer puts the object into that zone's collection
and emits a new `GameEvent::TokenCreated { object_id, zone }`; CR 704.5d takes
it from there. The name is open. It gets no `EventPattern` arm (nothing prints
"if a token would be created in exile") and joins `Attach` as the second
deliberate exemption from one-arm-per-variant, said in its doc. A *dropped*
entry still un-creates the object (CR 111.5), unchanged.

**CR 613.7m's decision point is not asked, and the reason is RC-4's.** Tokens
created by one effect are identical and enter under one controller, so "in the
order of that player's choice" is a choice with one outcome, and RC-4's rule
("never prompt for a choice with one outcome") applies. The first creation with
distinguishable members — Bestial Menace, Academy Manufactor — is the prompt's
customer, and neither is in reach; "Before card breadth" item 4 records the
prompt as owed at that card, and the batch order is the creation's order until
then.

**4. The entry is a door for the counter pattern too, who puts the counters
on is a fact on the event, and a player can be the subject.** CR 122.6:
"putting counters on that object … refers … also to an object that's given
counters as it enters the battlefield" — Doubling Season's ruling is
"planeswalkers will enter with double the normal number of loyalty counters",
and Hardened Scales', Primal Vigor's and Corpsejack Menace's each say the same
of +1/+1. Entry counters are `EnterMods.counters` and never an `AddCounters`
proposal (RC-2's decision, kept: they are part of the entry event, CR 614.1c),
so `EventPattern::CounterChange` gains a second door exactly as
`EventPattern::ZoneChange` did in RC-4b: it watches an `EnterBattlefield` whose
`mods.counters` carries a matching kind, and `Amount` applied to an entry
rewrites that kind's count in the mods. The Master Biomancer + Doubling Season
board then needs nothing new: Biomancer's `EnterWith` makes the doubler
applicable (CR 616.2), the affected player orders them (616.1e), and 614.5
stops the doubler re-applying to counters added after it — which is the printed
interaction.

`AddCounters` gains `by: PlayerId` and `EnterMods.counters` its
`Option<PlayerId>` — `codebase-state.md` item 43, sized there at ~60–80 lines
and told to land "before RE's first doubler". Vorinclex, Monstrous Raider is
the reader: "if *you* would put one or more counters … twice that many; if an
opponent would … half that many, rounded down", and its ruling says it "cares
deeply about who is putting the counters on". `EventPattern::CounterChange {
counter, adding, by: Option<PlayerRef> }` is RD-3's `SourcePattern` on the
player axis. CR 122.6a's default — the object's controller puts entry counters
on — is what `default_enter_mods` and every `EnterWith` write unless the effect
names a player, and today none does.

**Counters on players join here, because the type is open on the table.**
CR 122.1 lets "a player" have counters, and `backlog.md` §2.16 already has the
design: `PlayerState.poison_counters: u32` becomes a kind → count map sharing
`CounterType`, since CR 701.34a's proliferate sweeps permanents and players in
one pass. `AddCounters`' subject becomes `CounterSubject { Object(ObjectId),
Player(PlayerId) }` — `DamageTarget`'s shape, `subject_of` answers the player,
CR 616.1's chooser is that player — and widening it later would be a site
sweep of exactly the kind RD-4 paid for `DealDamage` (44 sites) when RE-5 can
do it at three. The producer is a printed card with nothing else in it, Live
Fast ("you draw two cards, lose 2 life, and get {E}{E}"), whose rulings say
energy counters are "a kind of counter that a player may have" and that
"effects that interact with counters a player gets … can interact with" them;
the watchers are Vorinclex's "or player" and Winding Constrictor's "if you
would get one or more counters". CR 704.5c reads the map for poison, and RD-1's
seam for infect's poison half (`backlog.md` §2.6, 120.3b) and Commander's
experience counters are the next producers. Costs that pay energy wait for
their first card.

Two rulings fall out of RC-3 and RC-4b rather than being built: "can't apply to
itself as it's entering" (RC-3's membership rule — an entering permanent's own
filter-scoped replacement does not reach itself) and "or to any other permanent
entering the battlefield at the same time" (decision 3's plural batch).

**5. Losing and winning are proposals; the SBA sweep builds them as batch
members; CR 704.7's collapse gains a per-player leg.** `GameAction::PlayerLoses
{ player, reason: LossReason }` and `PlayerWins { player }`. The four loops in
`check_state_based_actions` that write `player_lost` (704.5a/b/c and 903.10a)
become members of the CR 704.3 batch, evaluated against the one board the rest
of the sweep already reads; a player who would lose for two reasons in one check
is **one member** (CR 704.7 — Lich's Mirror's ruling: "a single Lich's Mirror
will replace all of them"), carrying the first reason in CR order, which is
what RA-3's per-object dedupe does for zone changes. `EventPattern::PlayerLoses`
carries no `reason` field: every printed replacement applies to every reason
(Exquisite Archangel's ruling, "any time you would lose the game"), so a field
would be one nothing reads. The performer marks `player_lost`, emits
`PlayerLost`, and clears `has_drawn_from_empty_library` — today it is never
cleared, which CR 704.5b's "since the last time state-based actions were
checked" forbids and Exquisite Archangel's ruling ("you won't lose again until
you try to draw again") makes observable. `PlayerWins`' performer records the
result on `GameState` — CR 104.1, "immediately" — so `GameResult` moves there
from `Game` and `check_game_over` reads it.

**N-player, where the paragraph would have hidden a two-player assumption.**
In a game of three or more, a lost player does not leave the rotation today:
`advance_turn` takes `(active_player + 1) % num_players` and the priority loop
does the same. CR 800.4k ("if a player who has left the game would begin a
turn, that turn doesn't begin") and 800.4j (priority passes over them) are two
sites and ~40 lines, and RE-6 carries them with the `--players 4` fuzz mode
"Before Commander" item 4 sized at ~50 lines, because a `PlayerLoses` performer
that leaves the player in the turn order is the two-player shape wearing an
N-player event. **CR 800.4a–e — their objects leaving the game — is RE-7, the
PR after, and not B3's any more.** The first cut left it with `codebase-state.md`
item 108 and B3; the review's objection stands: the day RE-6 lands, item 108
flips from "unreachable" to "reachable and wrong" in the four-player run, and
a dead player's permanents on the battlefield distort every four-player number
measured after it. The ledger's own rule ("the four wrong answers are the
phase-independent output") says a reachable wrong answer is fixed first, so
RE-7 follows RE-6 immediately; CR 802's defending player stays B3's.

"Can't lose" and "can't win" are `Restriction::Event` rows over the two new
patterns through decision 2's `affected_players`, refused ahead of the pipeline;
Platinum Angel's ruling — "no game effect can cause you to lose … you keep
playing" — is that refusal, and Exquisite Archangel's "if an effect says that
you can't lose the game, Exquisite Archangel's effect doesn't apply" is the
CR 101.2 order the two checks already have. Concession (104.3a) is a *leave*
that then loses, not a replaceable loss ("does nothing if you concede", every
ruling above), and it arrives with the harness that offers it; CR 104.3f
(win and lose at once → lose) has no atom and no consumer and is recorded, not
built.

**6. Skips are the pipeline's; the proposal is built by a turn queue that
`advance_turn` drains; `pending_skips` is struck.** §11 item 6 said the first
half on 2026-08-30 and §9's RE paragraph, older, said the other thing. A
per-player counter consulted at step begin is a second mechanism for a CR 614
effect — it cannot express a static ("Players skip their upkeep steps", Eon
Hub) without a third shape, it cannot be stripped by Layer 6 or CR 305.7, and
it cannot be ordered against another effect on the same event by CR 616.1.
Three variants, because the three units name three different things and have
three different performers: `BeginTurn { player, turn }`, `BeginPhase { phase,
player }`, `BeginStep { step, player }`, each with its pattern arm (`BeginStep
{ step: Option<StepType> }`, `BeginPhase { phase: Option<PhaseType> }`,
`BeginTurn`). The subject is the player whose turn it is, so CR 616.1's chooser
is that player and Eon Hub on a four-player table asks nobody. The performer is
small: it writes `phase`/`step`/`active_player` and emits the
`TurnBegin`/`PhaseBegin`/`StepBegin` that exist today and nothing emits, which
is what item 6's "at the beginning of" triggers will read — **and that is why
this PR goes first**: §8b counts 2,656 of them, more than any other kind, and
item 6 cannot start until the three events exist through the chokepoint. Turn-
based actions stay in `on_step_begin` and `process_turn_based_actions`, run only
for a unit that began.

**The queue is the shape, and it is why `backlog.md` §2.17 graduates here.**
`advance_turn` is rewritten once, as a drainer: the next unit is the head of a
queue — an extra turn, phase or step if one is pending, else the natural next —
and it is *proposed* before it starts. A dropped proposal is proceeded past as
though the unit did not exist (CR 500.11, 614.10): a skipped phase proposes none
of its steps, a skipped turn advances no turn number and expires nothing —
"until your next turn" waits for the first turn that is not skipped, 614.10a's
own sentence — and the first turn's untap step is proposed like any other.
CR 500.7's extra turns are pushed "most recently created … first", so the
queue's turn half is a stack fed by a new `Primitive::ExtraTurn`, with Time
Walk as its consumer; the Meditate + Time Walk board — a skip consuming an
extra turn, 614.10a's "the first occurrence that isn't skipped" — is the test
that shows the two belong in one PR rather than in two rewrites of one
function. Extra phases and steps (500.8, 500.10, Relentless Assault's class)
are the same queue one level down and wait for their first card. `backlog.md`
§2.12's step- and phase-scoped durations hang off the same emitters and wait for
an "until end of combat" consumer.

Three CR sentences then cost nothing: "once a step … has started, it can no
longer be skipped" is the proposal site (Moment of Silence's ruling, "must be
used before the combat phase starts or it has no effect", is a `Uses::Once` row
that meets no proposal until the next combat); 614.10a's "two effects … skip the
next two" is two `Uses::Once` rows, the first spent on the first proposal and
the second surviving to the next; and 800.4k is a rule at the same site, ahead
of the pipeline, once RE-6 gives it `player_lost`. 614.10b ("skip …, then take
another action") has **zero printed cards** by the census and no consumer; its
atom stays uncovered with that reason.

**7. Mana is one proposal from two silent writers, and "tapped for mana" is a
fact about the activation.** `GameAction::ProduceMana { player, source, mana:
Vec<(ManaType, u64)>, special: Vec<ManaAtom>, tapped: bool }`, performed by
the one arm that writes the pool and emits `ManaAdded` — an event that has
existed since the log was written and has never once been emitted, which is how
the RA census missed the two writers (`resolve_mana_effect` for a mana ability,
`Primitive::ProduceMana` for a spell). `EventPattern::ProduceMana { tapped:
Option<bool> }`; `Amount(Multiplier)` scales every type in `mana`; CR 106.6a's
"any restrictions or additional effects … apply to all mana produced" is
`special` riding through unchanged. `tapped` is CR 106's "tapping a permanent
for mana" — Mana Reflection's ruling: "only if you're activating a mana ability
of that permanent that includes the {T} symbol in its cost" — and CM-3's
lock-in already knows what was paid. Mana abilities activated inside CM-4's
payment window propose through the same site with the same `ctx`, and
`resolve_mana_effect`'s `Fixed`-only limitation is untouched. Triggered mana
abilities ("whenever you tap a land for mana, add an additional …", 8 cards)
are CR 605.1b's and item 6's, and Mana Reflection's ruling says so in as many
words. This is the hottest path RE touches — every land tap — so RE-9 goes
last, where its A/B decides nothing else; `backlog.md` §2.19's any-color mana
rewrites the same function and is ordered after it.

**8. A CR 701 producer that a replacement customer is waiting on lands in RE,
on RD-1's precedent.** `Primitive::Mill` was a stub until Angel of Suffering's
rider needed it, and RD-1 built it rather than wait for Phase 8's CR 701 sweep.
Discard and scry are the same case with the same argument, and the first cut
of this section got it wrong by reading §8a's sentence without the precedent.
`Primitive::Discard(n)` moves cards through `change_zone(.., Discarded)` with
CR 701.9b's chooser — the discarding player by default, "at random" as the one
other shape a registered card prints (Hymn to Tourach; "another player
chooses" waits for its card) — asked through the same `ChoiceKind` the cleanup
discard uses. It proposes no outer event: nothing prints "would discard", and
the seventeen cards watch the zone change. What they need beside the producer
is `caused_by: Option<PlayerRef>` on `EventPattern::ZoneChange`, read off the
batch's resolution stamp (the cleanup discard has none, which is right — it is
not "a spell or ability"), and a to-battlefield leg on `Instead(ZoneChangeTo)`:
a substitute whose destination is the battlefield returns an
`EnterBattlefield` proposal, since RC-4b's performer refuses a `ZoneChange`
onto it, and the loop's next iteration builds the entry's frame from the
rewritten event, which it already does. `GameAction::Scry { player, n }` is an
event because Eligeth, Crossroads Augur replaces it ("if you would scry a
number of cards, draw that many cards instead" — `Instead(DrawCards { n:
ReplacedAmount })`, decision 2's template amount on decision 1's template);
its performer asks one `ChoiceKind::Scry` and reorders the library in the arm,
proposing no zone change (CR 701.22 moves nothing between zones).

#### Measured — what to expect, and why the direction is known

*Evicted 2026-09-15 from `plans/replacement-architecture.md` §9's Phase RE, where the heading and a stub remain.*


Three arms per PR through `plans/fuzz_ab.py` against a same-day `main`
worktree, both pools, the middle arm being the engine with `registry.rs` and
`PERFORMANCE_POOL` unchanged — and unlike RD, **five of the nine middle arms
add proposals**, so the counters will move on those and each PR predicts the
number before running. **Corrected at RE-4's review (§11 item 80): that
middle arm reads the engine on `performance` only.** On `stress` the registry
*is* the pool, so registering a card changes every deck there and the arm
reads the decks. A phase that registers cards and wants the engine's `stress`
cost builds a fourth binary — its engine with the cards *unregistered*, so
`main`'s registry in both pools — and that is the arm to call "engine";
"registered" is read on `performance`, and "pooled" is the re-record.

- **RE-1 — measured 2026-09-11, and it is the one number the rest of RE reads.**
  `Replacement gathers` **+447 per game** (507 → 954 on `performance`, 200
  games / seed 12345) against a predicted ~+450, and `Restriction queries`
  moved with it — one `is_prohibited` per proposal, which the prediction
  missed. `Layer walks`, `Board walks`, `Layer frames` and `Dependency checks`
  **flat**, and every gameplay row identical to `main`. `Memo hits` +1.6%,
  because the fast path stops the walk and not the query. **CPU/game +5.0%**
  (12.92 → 13.57 ms), inside the stated 2–6% spread but cleanly separated
  round to round. A fourth arm with a hard-coded event-kind gate measured
  **+2.5%**, so the sweep is half the cost and the chokepoint is the other
  half — **the gate was not built**, with the reasoning and the trigger in
  `plans/archive/replacement-architecture-landed.md`, "RE-1". **RE-9 is the PR
  that reads this**: 2.5 points is what a gate would return it, and its own
  bullet already calls it the one whose A/B could say no.
- **RE-2:** `Replacement gathers` +1 per draw instruction (the outer), so
  roughly +1 per turn plus one per cantrip; `Layer walks` flat; 40-game event
  dumps identical but for one `CardDrawn` line's neighbour. The shipped arm
  adds walks only while Thought Reflection is on the battlefield.
- **RE-3 — measured 2026-09-12, and the prediction held to the byte.** The
  middle arm is **identical to `main` outside `=== Timing ===` on both pools**,
  200 games / seed 12345: patterns over events that already flow cost nothing
  until a card watches one. CPU median 15.49 vs 15.73 ms (**−1.5%**, rounds
  straddling: main 15.36–16.11, middle 15.32–15.77) — flat.
  **The shipped arm's CPU/game is up and it is the card**, which the per-unit
  rows are what say: two sittings gave +4.0% and +2.1%, both inside the spread,
  while `ms / 1,000 walks` went +0.2% then −1.6% and `CPU/turn p50` +2.3% then
  0.0%. The walk did not get slower; there are more of them. Rhox
  Faithmender makes the games longer (avg turns 29.6 → 29.8, spells 22.8 →
  23.4, damage events 19.6 → 21.3, total damage 56.1 → 61.1, life changes
  12.9 → 13.9), and `Replacement gathers` +25/game is the gain it doubles
  rather than a sweep it added. `--require`: **cast 176, resolved 174, in 114
  of 200 games (57%), copies/deck 1.52** — between Eon Hub's 64% and Thought
  Reflection's 46%, and reach was never the point: it is the only RE consumer
  that needs no second card to do anything. `stress` moves much further (gathers
  987 → 1143, avg turns 29.0 → 32.3) with all six cards in the deck, and
  `stress` milliseconds are a threshold rather than a comparison (§3.1).
- **RE-4 — measured 2026-09-13 at landing and again after its review, and
  the prediction held everywhere it could be read.** Four arms, not three
  (§11 item 80): `main`; **engine**, the branch with the cards unregistered;
  **registered**, the old pool; **pooled**. Engine and registered are
  **`IDENTICAL` to `main` outside `=== Timing ===` on `performance` at two
  seats and at four** (200 games / seed 12345), so RE-4 and its review cost
  the pool nothing: CPU/game −0.4% and +0.1%, −1.0% and +0.2%, `CPU/turn p50`
  flat. On `stress` the engine arm is exact: at two seats 188 games
  byte-identical to `main`, eleven differing by exactly one `TokenCreated`
  line per Zombie Kalitas makes, and one re-routed — the one-exit prompt
  theme C stopped asking, on a Dryad Arbor under Containment Priest and Root
  Maze; theme A re-routed none. The pooled arm is a re-record and a bigger
  board: gathers 1002 → 1043 per game, walks 373 → 386, avg turns 29.9 →
  30.7, total damage 57.8 → 69.3, CPU/game +11.0% with `CPU/turn p50` 0.390 →
  0.400 and `ms / 1,000 queries` +1.0% — more game, not a slower walk. Two
  rows are new and are baselines from here: `Replacement prompts` (0.49 per
  game on `performance` at two seats, 2.38 at four) and `Max batch depth`,
  **7** across 1,600 games, which is what `BATCH_NESTING_LIMIT`'s 32 is
  headroom over. `--require`: Parallel Lives 168 / 168 in **116 of 200 (58%)**,
  1.54 copies/deck; Raise the Alarm 201 / 201 in **140 (70%)**, 1.49; on
  `stress`, Divine Visitation 131 / 131 in 98 (49%) and Bard 135 / 135 in
  94 (47%). Three shell runs at one seed `IDENTICAL` outside `=== Timing ===`
  at both seat counts on both pools. → `fuzz-record.md`'s tables, both
  re-recorded.
- **RE-5 — measured 2026-09-13, and the prediction held to the byte.**
  Four arms, two seats and four: engine and registered **`IDENTICAL` to
  `main` outside `=== Timing ===` on `performance`**, and the engine arm
  `IDENTICAL` on `stress` too — the door is an arm no def in the old pool
  reaches, and the subject enum changes no proposal's count. CPU/game +0.1%
  and +1.1% at two seats, −0.5% and −1.5% at four — flat. The pooled arm
  moved the row the section named: `Replacement prompts` 0.74 → 1.14 and
  1.83 → 2.44, Hardened Scales' additive pairs (§2.29), and gathers
  1043 → 1060 at two seats. `--require`: Scales 226 / 226 in **136 of 200
  (68%)**, 1.54 copies/deck. → `fuzz-record.md`'s tables, both re-recorded;
  the whole reading is in the archive, "RE-5".
- **RE-6 — measured 2026-09-12, and the prediction held to the digit.**
  `Replacement gathers` **999 → 1000** and `Restriction queries` **1001 →
  1002** per game on `performance` (200 games / seed 12345) — the loss,
  proposed once — with every gameplay row and `Layer walks` identical to
  `main`. The one other movement is `Memo hits` −0.3%, and it is CR 104.1:
  a player who had just lost no longer receives priority until the phase
  ends, so the middle arm reads `differ` outside `=== Timing ===` and the
  check is the aggregates (§11 item 65). CPU/game **−1.0% and −1.7%** across
  two sittings, `ms / 1,000 walks` the same two numbers, `CPU/turn p50`
  0.440 → 0.440 and 0.490 → 0.480 — flat. The pooled arm is a re-record
  (Laboratory Maniac, −4.7% and −5.0%, walks 385 → 373), `--require` reaches
  117 of 200 games (58%), and games ended by a win are **zero** on every
  table. The four-player table is `fuzz-record.md`'s, first recorded here as
  RE-7's baseline — after its first run turned out to be a measurement of the
  harness (item 66).
- **RE-7 — measured 2026-09-13, and both halves held.** Both two-player pools
  are **`IDENTICAL` outside `=== Timing ===`** at 200 games / seed 12345, and
  the reason turned out to be a rule rather than the code: CR 800.1 scopes all
  of CR 800 to a game that began with more than two players (§11 item 68).
  The four-player diff is the measurement, and its row is
  **"Departed-owned permanents" 32.2 → 0.0** on `performance` and 34.2 → 0.0 on
  `stress`. `Replacement gathers` 2104 → 2102 and `Restriction queries`
  2107 → 2107 — flat, as a PR that adds no proposal should be — and `Layer
  walks` 802 → **838**, up by CR 603.6c's frame — an uncached walk per permanent
  leaving, ~33 a game and none at two seats. What moves the other way is
  `Frames/walk`, 21.22 → 18.69, and everything downstream of it: `Memo hits`
  −11.8%, `Layer frames` −8.0%, `Dependency checks` 307 → 171, **CPU/game
  median −15.8% and −14.0%** across two sittings and `ms / 1,000 walks` −19.4%
  and −17.7%. That is a board the engine
  stopped carrying rather than a walk that got faster, and it is the size of
  what item 108 was costing every four-player number taken before it. The
  gameplay rows are a different board and not a delta. → `fuzz-record.md`'s
  table, re-recorded there.
- **RE-8 — measured 2026-09-14, and the middle arm's prediction held to the
  byte once a fifth arm said which change was which.** Four arms plus
  **engine-oldcleanup**, this branch with CR 514.1's one-card-at-a-time cleanup
  loop restored — and *that* arm is **`IDENTICAL` to `main` outside
  `=== Timing ===` on both pools at two seats and at four**. So the two new
  producers, `by`'s third clause in `applies_to`, a `GameAction` variant with
  its three exhaustive arms and a `TemplateAmount` on a template cost a board
  with nothing watching them nothing at all. What moved the engine arm is the
  cleanup reshape alone — one prompt of N where there were N of one, which
  spends the agent's RNG differently from that turn on: **23 of 200 games on
  `performance` and 29 of 200 on `stress`** reach a cleanup discard of two or
  more, and the aggregates move by tenths (avg turns 31.4 → 31.3, gathers
  1060 → 1052). The pooled arm is a re-record and a *smaller* board — two
  nonland cards in 36 slots dilute Hardened Scales, so `Replacement prompts`
  1.14 → **0.57** at two seats with gathers 1060 → 1018 and avg turns
  31.4 → 30.4. CPU flat or down everywhere: two seats 16.65 → 16.26 (engine),
  16.40 (registered), 15.16 (pooled); four seats 55.07 → +0.8%, −2.0%, −2.0%.
  `--require`: Mind Rot 173 / 172 in **129 of 200 (64%)**, 1.43 copies/deck;
  Opt 210 / 209 in **139 (70%)**, 1.52; on `stress`, **Eligeth, Crossroads
  Augur** 121 / 121 in 99 of 200 (50%) — the first reachability row this
  project has for a card whose name contains a comma, which is what the
  repeatable `--require` bought. Zero errors, zero panics and zero turn limits
  on every arm. → `fuzz-record.md`'s tables, both re-recorded.
- **RE-9:** +1 gather per mana ability that resolves — every land tap, several
  per turn — and the same fast-path argument as RE-1. The prediction is flat
  CPU and a moved `Replacement gathers` row; a fourth binary with the proposal
  reverted is the recipe if it is not, and the §8 gate is the fix. **If the
  gate is needed and does not close the gap, RE-9 is the PR the owner decides
  against, and the two writers keep their direct write with a Deferred
  Migrations line that names the number.**
  **Measured 2026-09-15, and the gate said yes.** Four arms, both pools, two
  seats and four, `--rounds 7`. The engine arm — the three cards unregistered
  — is **identical to `main` on every gameplay and layer row** at both seat
  counts, with `Replacement gathers` **983 → 1063** on `performance` at two
  seats against a new `Mana productions` row of **81** (one gather per
  production, the averages rounding apart by one) and **2012 → 2163** against
  **151** at four; `Memo hits` +0.4%, RE-1's fast-path sentence again, which
  is why every arm reads `differ` and the check is the aggregates. **The
  number is the proposal's cost and not a replacement's, and productions are
  not rare** — they are the commonest proposal after phases and steps. The
  engine arm has no mana replacement to find by construction, so what a
  production buys is a batch opened and closed, one `is_prohibited`, and
  `gather`'s sweep over the static replacement sources a `performance` board
  carries (about three, by the memo hits: 264 more a game over 81
  productions), which is the overhead every event kind pays at RE-1's rate.
  What a replacement *applying* costs is the pooled arm's question, where
  Mana Reflection is out in 41% of games, and that arm's movement is game
  size (below). **CPU/game +1.2% at two seats and +0.7% at four**, rounds
  straddling both times,
  `CPU/turn p50` flat — under §11 item 54's 2.5-point gate, so **lever 2 is
  not built**, which decision 11 predicted from RE-1's and RE-2's rates. The
  pooled arm is a re-record and a bigger board: +20.1% CPU/game at two seats
  with `Frames/walk` +11.3% and the walk flat per frame. `--require`: Mana
  Reflection cast 119 / resolved 118 in **82 of 200 games (41%)**, 1.46
  copies/deck. One four-player `stress` game ran to a 200-turn cap `main`'s
  longest game was already within 3% of; at `--max-turns 600` it ends at 208
  and nothing else moves. **Measured again at review with two more arms**,
  because neither arm above isolates what a mana replacement costs per tap
  with the game held fixed: a no-op row applying on every production reads
  **+2.7%** at two seats and +2.2% at four, rounds clear of the engine arm's,
  and `gather` gated for `ProduceMana` reads +0.1% and −0.6%, straddling —
  the gate returns nothing here, and the bare proposal's +1.2% is the
  chokepoint's fixed per-event work (§11 item 99; `codebase-state.md` item
  136). → the archive, "RE-9"; `fuzz-record.md`.

- **RE-10:** the one arm in RE predicted **flat in every row**, and that is the
  whole reading. The plan replaces `next_phase`'s chain on every turn of every
  game, so the middle arm walks the new cursor unforced with no pooled card;
  gathers, walks and every gameplay counter should be byte-identical to `main`,
  and CPU flat or slightly down, since an index beats a chained `match` per
  unit. **A move in any counter means the cursor changed a turn's shape**, and
  the only two that legitimately could — a turn's position count, and the
  order phases are proposed in — are exactly what the arm is checking.
  `PERFORMANCE_POOL` +0 predicted, with a `--require` row instead.
  **Measured 2026-09-14, and flat as predicted.** `performance` counters
  **IDENTICAL** to `main` — every row, turns through gathers — which is the
  whole reading: the cursor changed no turn's shape. `stress` differs, as it
  must, because registering a card changes the decks. CPU **+0.1%** at
  `--rounds 7`, with `p50`, `p99` and CPU/turn all level. `--require` on
  `stress`: Aggravated Assault cast 147 / resolved 146 in **114 of 200 games
  (57%)**, copies/deck 1.24; zero errors, zero panics, zero turn limits.
  **One real cost was found by reading the mechanism rather than the
  milliseconds**: `on_turn_begin` assigned a fresh `TurnPlan` and so
  allocated a five-element `Vec` every turn where the chain it replaced
  allocated nothing. `TurnPlan::reset` rebuilds in place. The seven-round
  median was +2.8% before it and +0.1% after, but the second sitting was
  tighter on both arms (~13.6ms against ~14.4ms), so **the two numbers do not
  compare and the fix is justified by the mechanism, not by the delta** —
  §11 item 54's lesson applied to a reading that happened to go our way.
  No `fuzz-record.md` block: no table moved.

The fixture table is re-recorded in `fuzz-record.md` once per PR that moves the
pool, at 50 games, after the A/B; from RE-6 on, the four-player table beside it.

## 11. Findings and open questions

*Evicted 2026-09-15 from `plans/replacement-architecture.md`, where the heading, the verdict table, items 3, 4 and 14, the catchall ban and one `N. **title**` line per closed item remain.*


### The `ZoneChangeCause` derivation — the catchall ban, and the list that was never the blocker (2026-08-24)

**The one that blocks — and it is not what the first draft said.** Review pushed
back that pinning `ZoneChangeCause` sounds like it needs a carefully crafted
Scryfall query, and could produce a long tail of variants with one card each.
That pushback exposed a framing error, and correcting it makes RA *less* blocked,
not more.

**`ZoneChangeCause` is not a card question.** It records *what the engine was
doing when it moved the object*, so it is derived from call sites, not
researched from the pool. The derivation is finite and already readable today:

| Source | Count | Where |
|---|---|---|
| `Primitive`s that move an object | **10** — Destroy, Exile, Sacrifice, ReturnToHand, ReturnToBattlefield, PutOnTopOfLibrary, PutOnBottomOfLibrary, ShuffleIntoLibrary, Mill, Discard | `types/effects.rs`, "Zone movement (rule 701)" |
| SBA sweep reasons | **8** — CR 704.5d tokens, 704.5f zero toughness, 704.5g lethal damage, 704.5h deathtouch, 704.5i loyalty, 704.5j legend rule, 704.5m/n attachment | `engine/sba.rs`, 5 live sites today |
| Stack exits | **4** — resolved→battlefield, resolved→graveyard, countered, fizzled | `engine/stack.rs`, the three `REPLACEMENT-BYPASS:` sites |
| Turn structure & casting | **3** — cleanup discard, draw, cast (hand→stack) | `state/game.rs`, `engine/zones.rs`, `engine/cast.rs` |

That is the whole input set, and the production tree currently has **13 zone-move
call sites**, of which **9 are labellable** — the other four are cast rollbacks
that leave the chokepoint instead (§9, RA-1). An afternoon of reading, no query
required.

Merging the 25 raw inputs down to the **18 variants** in §3.1 takes four
judgments, all checkable:

- **704.5g + 704.5h → one variant.** CR 701.8b calls both "destroyed"; no card
  distinguishes lethal damage from deathtouch as a *cause*.
- **704.5n is not a mover.** It unattaches an Equipment and leaves it on the
  battlefield, so it produces no zone change at all.
- **704.5d is not a zone change.** A token ceasing to exist is removal from the
  game, and `TokenCeasedToExist` already exists as its own event.
- **Library position is a field, not a cause.** Top, bottom and shuffled-in
  share `PutIntoLibrary`; *where* in the library is a parameter of the action.

Everything else survives one-to-one, which is what "derived" is supposed to
mean.

**Cards decide only the granularity, and they ask for less than the call sites
offer.** The demand side is short and flat (Scryfall, 2026-08-24): dies 1,287
triggers, sacrifices 278, discards 266, exiled 74, milled 50, destroyed 5. And
the feared long tail is measurably absent — **"destroyed by" appears on 1 card in
all of Magic, "was sacrificed" on 3, and "if it was destroyed" on 0.** No printed
card asks for a cause *finer* than the engine can name from its own call site, so
there is no research problem to solve.

**What actually blocks is the catchall, not the list.** Widening the enum later
is only expensive in one scenario: an existing site was labeled with a coarse
variant that should have been finer, and re-triaging it is guesswork that fails
silently. That scenario requires an `Other` / `Unknown` variant to lump into. Ban
it, and the failure mode disappears:

- **No catchall variant, ever.** Every call site names its reason. A site with
  nothing honest to say is a site whose reason nobody has worked out, which is
  the bug.
- **A genuinely new mutation arrives with its own new call site**, so it adds a
  variant and touches nothing existing — a normal diff, not an audit.
- The enum is `#[non_exhaustive]`-free and matched exhaustively wherever it is
  read (§3.1 limits that to the replacement pipeline and the trigger matcher), so
  a new variant fails to compile at every reader rather than defaulting.

So the pre-RA task is one line of policy plus an hour of labelling, and the
"get the list right first" framing was overcautious. §8b's projection of ~17 is
close to the derivation's 25 raw inputs before merging the ones that collapse
(the four stack exits share `from: Stack`; several SBA reasons differ only by
rule number) — confirm the merge at labelling time.


### Asked in review before code started (2026-08-24) — items 1, 2, 5, 6 and 7

1. **CR 903.9 is half an SBA, and `codebase-state.md` said otherwise.**
   Current Oracle splits it: **903.9a** (commander in graveyard or exile → its
   owner *may* put it in the command zone) is a **state-based action**, listed
   at **CR 704.6d**, not a replacement effect. Only **903.9b** (hand or library)
   is a replacement — and it carries the rules' only explicit exception to CR
   614.5 ("may apply more than once to the same event"). Consequence: **the
   graveyard/exile half of Commander's zone redirection is not blocked on this
   phase at all** — `check_state_based_actions` already takes a
   `&dyn DecisionProvider`, which is everything 704.6d needs. Corrected in
   `codebase-state.md` alongside this document.

2. **`ObjectSet` is reused rather than re-invented, and the reuse is
   load-bearing.** `SourceOnly` vs `Filter` is precisely CR 614.12's "affects
   only that permanent (as opposed to a general subset of permanents that
   includes it)". If a future refactor collapses those variants, 614.12 breaks
   silently.

5. **The overlay's shape — closed by performance, not by taste.**
   `layers-architecture.md` §15.2 item 3 left "clone vs. CoW overlay" open for
   the dependency algorithm. Asked again in review — *are there performance
   considerations that favor one?* — and the answer is yes, decisively, but the
   two customers have to be priced separately because §5 establishes they are
   not the same operation.

   **CR 613.8's check should clone the frame, and that is already cheap.** Its
   perturbation is `EffectiveCharacteristics`-shaped: clone `chars`, apply one
   `EffectModification`, re-run `object_matches_filter`. `layers-architecture.md`
   §12 measured frame construction at **0.37 µs at N=10 falling to 0.27 µs at
   N=80** — flat in board size, "3% and flat" in its own words, explicitly not
   the bottleneck. So the expensive thing about a
   snapshot was never the frame; it was the idea of copying the *game*. 613.8
   does not need to.

   That matters because of where the check sits: up to O(effects²) candidate
   pairs per layer, inside a walk that runs per permanent, inside a sweep that
   runs per priority check. §12's table already shows this shape going
   superlinear when a per-effect cost is added — the ungated CR 604.2 existence
   check ran 5.2× at N=10 and 8.0× at N=80 for exactly this reason. A
   game-state-shaped snapshot in that position is not a slow path, it is a
   different complexity class.

   **CR 614.12's look-ahead genuinely needs game-state-shaped perturbation** —
   battlefield membership, controller, the entering object's own registry rows —
   but it runs **once per entering permanent**, which is a few times per turn.
   Its budget would tolerate a `GameState` clone.

   **It still should not use one**, and the reasons are correctness rather than
   speed: a clone duplicates `GameState.rng`, which the determinism doctrine
   forbids reaching for a second time; and it produces a second live copy of
   every `ObjectId`, which is a v4 UUID and therefore *aliased* rather than
   distinguishable, so any code that reads an id back out of the snapshot cannot
   tell which game it belongs to. A read-side view has neither problem and needs
   the accessor pair that §5's five audited sites want anyway.

   **Decision: read-side accessor pair, no clone at either call site.** Record it
   in `layers-architecture.md` §9/§15.2 when RC-4 lands, and measure the RC-4
   overlay with `fuzz_games --games 200 --seed 12345` against the pre-RC-4
   baseline — a look-ahead that runs a few times per turn should not move the
   number, and if it does, the accessor indirection has leaked into the hot walk
   and that is the bug to find.

   **Built and recorded (RC-4, 2026-09-02).** `engine/layers/lookahead.rs` is
   the overlay, `FrameCache` carries it, and `layers-architecture.md` §9 /
   §15.2 item 3 now say so. The measurement is in §9's RC-4 subsection.

6. **CR 614.10's skips are not `execute_action` events.** They replace the
   *beginning of a step/phase/turn*, which happens in `advance_turn`, not in a
   mutation. They still go through the pipeline (they are replacement effects
   per 614.1b/614.10), but their proposal is built by the turn machinery. Note
   also that `GameState.skip_first_draw` (CR 103.8a) is a **game rule**, not an
   effect, and stays a bool. **Confirmed at RE's sizing (2026-09-11): RE-1, §9
   RE decision 6 — and §9's older paragraph, which said `pending_skips`, is
   struck; this item was right and it was not. Built and closed 2026-09-11
   (RE-1): the proposal is `GameState::advance_turn`'s, the three units are
   `GameAction::{BeginTurn, BeginPhase, BeginStep}`, and `skip_first_draw` is
   still a bool in `process_draw_step`.**

7. **Watch the `ScriptedDecisionProvider` blast radius.** Every existing test
   that reaches `execute_action` will now traverse the pipeline. The §4.1 rule
   — no DP call with fewer than two candidates — is what keeps that at zero
   prompts on today's card pool. If a phase finds itself relaxing that rule to
   make something work, it has found a design error, not a test problem.


### Answered on the review's second pass (2026-08-30)


`rb-review.md` themes D, F and H asked eight modelling questions rather than
reporting defects. Their answers live here because they are decisions about
shape, and a decision recorded in a findings ledger dies with the ledger.

8. **`apply_rewrite` does not grow per `(template, event)` pair, and the shape
   that would fix it is worse** (`rb-review.md` D1). The match is
   `template` × *{events that template can be built from}*, and every template
   so far reads at most one event kind: `ZoneChangeTo` reads `ZoneChange`'s
   `object` and `from`; `RemoveCountersFromAffected` reads nothing from the
   event at all — it takes the *subject* — so it matches `_`. Growth is
   therefore **one arm per template plus one error arm per template that
   constrains its input**, which is linear in `GameActionTemplate`.

   The symmetric alternative — make `GameActionTemplate` a projection of
   `GameAction` with `Option` fields meaning "inherit from the event", the way
   `EventPattern` is — is the wrong trade. A template's whole job is to say
   what it takes from the event, and templates cross event kinds: CR 122.1c
   turns a `Destroy` into a `RemoveCounters`. A uniform inheritance rule cannot
   express that, so the per-field rules come back as data instead of as code,
   and the error the current shape can give — "its `EventPattern` and its
   `Rewrite` describe different events" — becomes unreachable.

   **The tripwire is a template that must be built from more than one event
   kind, differently.** At that point move construction onto the template
   (`fn apply(&self, event, subject) -> Result<GameAction>`); that is
   mechanical and carries no rules content. Nothing before then. §3.2c's
   census is the evidence for the bound: 561 cards and 574 "would … instead"
   clauses put the pressure entirely on the `GameAction` vocabulary.

9. **§3.3 source 2 — static abilities functioning in other zones — is sized,
   and it should not be inside this phase** (D5, closing K3).

   **The count, Scryfall 2026-08-30: ~390 cards.** Six keyword families carry
   almost all of it — flashback 210 and buyback 39, rebound 35, aftermath 27,
   jump-start 13 (all CR-defined replacements functioning **on the stack**),
   madness 61 (functioning **in hand**) — plus five cards whose "would be put
   into a graveyard from anywhere, … shuffle it into its owner's library
   instead" functions in *every* zone: Blightsteel Colossus, Darksteel
   Colossus, Legacy Weapon, Nexus of Fate, Progenitus. Deliberately **not**
   counted: the 34 "…exile it instead" hits on the same query are disturb
   double-faced backs, and CR 712.8a gives a DFC only its front face's
   characteristics outside the battlefield and the stack, so they are ordinary
   battlefield sources.

   **The phase should close without it, and the count is not why.** The sweep
   is the easy half — one loop over a zone list instead of
   `battlefield_ids_ordered`. **That sentence was wrong, and LJ is what found
   out** (2026-09-14): the loop is not the work, because the filter's
   candidates come from `Board::members` and a graveyard card was never one.
   It is a *working-set* change — `Board::seed`, `membership` and
   `compute_non_member`'s fast exit all have to agree that an object off the
   battlefield can be reached at all — and the guard that keeps it free is
   what carries the cost, not the sweep. See `layers-architecture.md` §13c. What source 2 actually needs is **CR 113.6**, the
   fourteen-subrule answer to "which of an object's abilities function in which
   zone", and the engine has that nowhere. A graveyard sweep that does not ask
   113.6 gathers Wonder's flying grant and Bridge from Below's trigger
   alongside the one replacement it wanted.

   **And it is not this phase's to build**, because the layer system needs the
   identical facility for the identical cards: `codebase-state.md`'s layers
   section already records that timestamps must move off `PermanentState`
   onto `GameObject` precisely because Wonder — a *continuous* effect
   functioning from a graveyard, CR 113.6b — has no timestamp to read. Two
   systems, one missing facility, and building half of it inside RA–RE would
   put CR 113.6 in `engine/replacement/`.

   **Sized, then:** (a) CR 113.6's zone-function predicate, shared with layers
   and eventually triggers (CR 113.6k); (b) an object timestamp off the
   battlefield, which the layer system owes anyway; (c) a **gate leg per zone**
   — `replacement_ability_sources` is populated at ETB, so a hand, graveyard or
   stack source is invisible to `gather`'s fast path, which is `CLAUDE.md`'s own
   "a new gather source needs a gate leg" rule; (d) the sweep.

   **Re-stated against what shipped (2026-09-14).** The last sentence used to
   read "Schedule it with (b), after RE, not inside it", and `roadmap-v2.md` A5
   overruled it on 2026-09-14 — three PRs *ahead* of RE-9 and RE-10, the second
   of which is LJ. **(d) has landed** and was the whole of that PR; it turned
   out to be the working-set change above rather than a sweep, and it cost
   nothing from (a), (b) or (c). **(b) is not (d)'s prerequisite and never
   was** — CR 613.7 orders *effects*, and a row's timestamp is read off its
   **source**, so a battlefield source reaching a graveyard needs no object
   timestamp at all. (a), (b) and (c) stay owed and go together, because the
   three of them are one facility from three doors: a source that *functions*
   off the battlefield. That is A5, and Wonder is its card.

   **This is the third time this document has held two answers** — item 46 is
   about the first two, and item 41 about the exemption reversed in another
   file three days ago. The rule item 46 states applies to itself: a scope
   paragraph for an unsized phase goes stale in the merge that builds part of
   it.

   **✅ Closed 2026-09-14 by LK** (`layers-architecture.md` §13d). (a), (b) and
   (c) all landed, and the shapes are not quite the ones above:

   - **(a) the predicate** is `engine/zone_function.rs`, beside
     `engine::restriction` for that module's reason. It returns a `ZoneSet`
     rather than a bool, and it takes the object's card types as an *input*
     rather than reading them, so the layer invariant holds at the module
     boundary. Six of CR 113.6's fourteen subrules ship; §13d decision 4 is the
     triage table, and **113.6e, f, j and m are all `backlog.md` §2.3's**
     rather than this facility's, which is the split `roadmap-v2.md` A5 has
     been making since it was written.
   - **(b) the object timestamp** is `GameObject::timestamp`, CR 613.7d's, and
     `static_effect_timestamp` reads it there — which is what lets a Wonder in
     a graveyard generate an effect the layer walk can order. It cost +16.5%
     to take it off `PermanentState`, so the battlefield entry keeps a copy for
     the ordered sweeps; §13d decision 2 and `codebase-state.md` item 77 have
     the measurement and the eventual deletion.
   - **(c) the gate leg per zone** landed **narrower than this sizing
     assumed**, and deliberately. `replacement_ability_sources`,
     `restriction_ability_sources` and `cost_modification_ability_sources` all
     index a sweep over `battlefield_ids_ordered`, so LK keeps them
     battlefield-only: an entry naming a graveyard card would be a claim the
     set cannot keep. What LK built is the *registration* leg —
     `GameState::register_static_effects` takes a zone and `move_object` is its
     second caller — which is what a **continuous effect** from a
     non-battlefield source needs. A **replacement** effect from one needs the
     sweep to visit it as well, and that is still owed, now with a card against
     it: Abrupt Decay for the restriction sweep (CR 113.6g, whose *zone* answer
     is already free from the default arm), and one of the five "would be put
     into a graveyard from anywhere" cards for this one.

   **So source 2 is half-open rather than open**, and that is the honest
   summary: a static ability functioning off the battlefield now registers,
   applies and retires correctly, and the replacement pipeline still only
   gathers from the battlefield.

10. **Skullbriar is the wrong reason to change the counter model, and there is
    a right one** (F1). Two cards want CR 122.2's exception — Skullbriar, the
    Walking Grave and Me, the Immortal (`o:/counters remain on/ -is:funny`,
    2026-08-30) — which is a thin case for touching counters. But the
    *capability* they need is counters on an object that is not on the
    battlefield, and that has a real constituency: **71 suspend cards** put
    time counters on a card in exile, and CR 122.1a and 122.1b are written for
    "a card in a zone other than the battlefield" in their own words. The
    engine can express none of it — `counters` lives on `PermanentState`,
    and `perform_action`'s `AddCounters` arm errors for anything else.

    **So: do not honour Skullbriar on its own.** The change is one move, the
    `counters` map from `PermanentState` to `GameObject`, and it is contained
    — 12 direct `.counters` sites outside `src/cards`, plus the
    `add_counters` / `remove_counters` / `counter_count` accessors. The one part
    needing thought is `CounterStack.timestamp`: CR 613.7c timestamps a counter
    as it is put on, and that only means something where layers read it.

    Once the map is on the object, Skullbriar costs a predicate at
    `move_object`'s clear point — CR 122.2's "counters cease to exist" becomes
    "unless the object's effective abilities say otherwise". **Trigger:** the
    first suspend card or the first CR 122.1b keyword counter off the
    battlefield. `codebase-state.md` Deferred Migrations.

11. **"Unconditional once queued" is right, and RB cited the wrong rule for
    half of it** (F3). Three separate things were tangled:

    **The motivating card is out of scope.** Academy Rector's "you may exile
    it. If you do, search…" is a *triggered ability*; card-text "A, then B"
    resolves inside the effect (CR 608.2c) and never enters the pipeline as a
    rider at all — §4.1a's "the other 'then'".

    **The objection survives its example.** CR 615.12 is prevention-only, and
    the code cited it for every rider. The correct citation for an `Instead`
    rider is CR 614.1a + 614.6: the "and also" is part of what happens instead,
    so it happens when the substitution does. Same conclusion, different
    argument — and the case that separates them is an `Instead` queueing a
    rider and a *later* replacement in the same 616.1f loop then preventing the
    surviving event. Under 615.12 the rider still runs; under 614.6 the rules
    say nothing, and "the modified event never happened" is at least arguable.

    **Unreachable today, and checkably so.** RB's only `Prevent` producers are
    CR 122.1c's shield damage half and CR 701.19a's regeneration, which watch
    `DealDamage` and `Destroy`; RB's only `Instead`-with-a-rider is Kalitas,
    whose output is a `ZoneChange` to exile that nothing watches. No RB batch
    can build the case.

    **And "if you do" is not a rider.** It is a conditional inside an effect,
    and `then` is an `Effect`, so it lands as `Effect::Conditional` (Phase 6)
    with no replacement vocabulary at all — which is exactly what §3.2's
    "per-mechanic variety goes in `then`" was supposed to buy. Authoring an "if
    you do" as a bare unconditional `then` is a card-authoring error and should
    be called one. **RD owns the decision**, with item 15, because RD is where
    `Prevent` gets producers.

12. **CR 614.15's `SelfReplacement` class is *derived*, never authored** (F5),
    and this codebase has already settled the general form of the question.

    614.15 says a self-replacement effect "is an effect of a resolving spell or
    ability that replace[s] part or all of that spell or ability's own
    effect(s)". That is **pure provenance**: the identical sentence printed as a
    static ability on a permanent is not a self-replacement, and no text a card
    can print makes an effect self-replacing on its own.

    The precedent is CDAs. `AbilityDef.is_characteristic_defining` asserts only
    what the ability's *text* satisfies, while CR 604.3a(2)'s provenance is
    owned by whoever writes the ability onto an object — `CLAUDE.md` states it,
    and a Layer 6 `GrantAbility` must clear the flag. CR 614.15 is the same
    split with **no text half at all**, so the flag has nothing to assert.

    **What that means when item 3's producer lands:** `gather` stamps
    `class = ReplacementClass::SelfReplacement` on every instance it builds
    from `ActionContext::resolution`, and a card file never writes it. Assert
    the other direction too — no authored `ReplacementDef` carries the class —
    which is the analogue of the Layer 6 grant clearing the CDA flag. Note the
    nuance 614.15 adds and the derivation survives: "the text can be a separate
    ability, particularly when preceded by an ability word", so the class
    cannot be derived from *which ability the text sits in*; it is derived from
    the effect arriving through the resolution.

13. **`ZoneChangeCause::Returned` keeps both returns, and the reason is
    structural** (F7). §11's merge criterion asks whether a printed card
    distinguishes two reasons *as a cause*. It does not apply here: the two
    "returns" differ by **destination**, and destination is a field on the
    event. The enum exists for what `(from, to)` cannot recover — a sacrifice
    and a destruction share `(Battlefield, Graveyard)` — and return-to-hand and
    return-to-battlefield share nothing.

    Checked against the pool anyway: every card printing "is returned to" names
    its destination (5 cards, Scryfall 2026-08-30 — Azorius Aethermage,
    Justice Vance Astrovik, Puppet Master, Stormfront Riders, Warped Devotion;
    all of them "hand"). Nothing watches a return without a zone, so the
    trigger matcher always has both halves.

    **The tripwire runs the other way.** If a card ever needs "returned to the
    battlefield" as distinct from "put onto the battlefield" at the *same*
    `(from, to)`, that is a missing variant on the *put* side — not a split of
    `Returned`.

15. **RD's design must open with this: CR 614.5's identity may be per
    `(event, affected object)`, not per batch member** (H9).

    §4.2 keys the applied set per batch member. Two printed rulings pull in
    opposite directions against that, and both are real. Kalitas's says a
    board wipe killing N of an opponent's creatures makes N Zombies — different
    objects, N applications, which the per-member shape gets right. CR 122.1c's
    says that when several sources damage one shielded creature at once, **one**
    shield counter is removed — but CR 510.4 makes simultaneous combat damage
    one event, and the engine models each source as its own batch member with a
    fresh applied set, so a creature blocking two attackers with two shield
    counters loses two. (With one counter the answer is accidentally right: both
    damages are prevented and the second rider removes nothing.)

    **Keying per `(event, affected object)` within a batch reconciles them
    exactly** — Kalitas's deaths are different objects, the two damages are one
    object.

    **Why it cannot wait for a card.** CR 615.7 is "one prevention shield,
    several simultaneous sources, the shield's controller chooses how much to
    apply to each" — the same modelling question asked officially — and building
    615.7's allocation on the per-member shape gives each member its own shield
    with nothing to allocate across. What the change costs is small and is about
    *ownership*: `apply_replacements` already takes the applied set as a
    parameter, so `execute_batch_inner` would key one map by affected object
    instead of handing each member a fresh clone. §3.2d's `inherited` lineage
    rule is untouched — it is about decomposition, not simultaneity.


### Found by a rider read-through (2026-08-30)


Three items from reading `ReplacementDef.then` end to end against
`engine::resolve`. §4.1a settled *when* a rider runs; none of these was asked
there, because all three are about **what a rider can reach**.

16. **`Rider.subject` is `Option<ObjectId>`, and the `Option` is doing two jobs.**
    One is honest: an event has exactly one subject — `subject_of` is total over
    `GameAction` and returns a scalar — so a rider carrying one subject records
    the *event's* arity rather than any limit on the rider. The other is a
    silent loss. `subject_object` maps `EventSubject::Player(_)` to `None`, so
    the player case does not survive into the `ResolutionContext` at all.

    **The fix is to stop flattening, not to widen.** Carry `EventSubject` and
    let `resolve_rider` emit `ResolvedTarget::Player(pid)`; both types exist and
    `resolve_player_for_self` already reads that variant. `codebase-state.md`
    item 27 has the sizing and the trigger.

    **Why it earns a line rather than a fix now:** the failure is not an error,
    it is a rider quietly acting for the effect's controller where the card said
    "that player". Notion Thief hides it, because for Notion Thief those are the
    same player — which is exactly why the one card in the tree does not catch it.

17. **A rider's object reach is its subject, and that ceiling belongs to the
    `Effect` tree rather than to this document.** `resolve_primitive` consults
    the `EffectRecipient` only for player-directed primitives; the 24 arms that
    affect objects read `ctx.targets` and ignore the recipient's
    `SelectionFilter` and `TargetCount` entirely. `EffectRecipient::Choose`
    therefore buys a rider nothing today, and `regeneration_rider`'s filter and
    count are inert data.

    **This pins the same boundary item 11 pins, and it is worth stating once for
    both.** `then` is an `Effect`, so anything a rider cannot say is something
    the `Effect` tree cannot say. The test for whether such a gap is *this*
    document's is whether closing it changes `types/replacement.rs`. For "if you
    do" (item 11) and for set-valued recipients, it does not — which is the
    §3.2 growth contract working as intended, not a hole in it.

    `codebase-state.md` item 28 has the arm count and why it is deliberately
    left unsized.

18. **§3.2d's lineage rule ships with no producer**, and the parameter that
    carries it is handed an empty set at its only call site. Recorded because
    the failure mode §3.2d names is a **hang** rather than a wrong answer, and
    because "a correct mechanism with no customer" is precisely the shape
    `codebase-state.md`'s Deferred Migrations section exists to catch. Item 29
    there has the sizing; the regression is the one §3.2d already names, and
    **RE-2** is the PR that needs it (§9, sized 2026-09-11: the outer draw's
    performer is the producer).

    **Closed 2026-09-11 by RE-2.** The producer is `GameAction::DrawCards`'s
    performer and the regression shipped as
    `test_two_thought_reflections_draw_four_not_infinity` — on Thought
    Reflection rather than the Teferi the name predicted, because Teferi's
    Ageless Insight is legendary and the two-copy board it takes is unbuildable.
    The bound the item asked for is the `ScriptedDecisionProvider`: a correct run
    makes exactly one CR 616.1 prompt, so a provider primed with one turns the
    second into a red test at depth two rather than a binary that never returns.
    Mutation-checked both ways.


### Found by asking what RC-3's own test proves (2026-09-02)


19. **CR 616.1 prompts for a choice with one outcome on every entry, and for
    `Rewrite::EnterWith` that is a theorem rather than a coincidence.** Raised
    against `test_two_registered_cards_make_cr_616_1_ask`: Root Maze and
    Idyllic Beachfront both rewrite one `EnterBattlefield` into
    `EnterWith(tapped)`, the pipeline asks which applies first, and **no
    assertion can tell the two orders apart**. Three facts make that general:

    - `EnterMods::merge` is `tapped |= other` and per-kind counter `+` — both
      commutative and associative.
    - `EventPattern::EnterBattlefield` matches `GameAction::EnterBattlefield { .. }`
      on the variant alone, and `set_affects` keys on the event's *subject*.
      Neither reads `mods`, so applying an `EnterWith` cannot change which
      effects are applicable on the next CR 616.1f iteration.
    - So for a bucket whose members are all `EnterWith`, the loop is
      order-invariant. The prompt is real, CR-mandated and pure noise.

    **The v1 stake is the CLI harness**: "highly parallel AI games over the
    CLI" pays a decision round-trip per entry under two entry replacements, for
    an answer that cannot matter.

    **The sound rule is narrow and provable, and it is not "collapse identical
    rewrites".** That would be a semantics-assuming shortcut of exactly the kind
    `layers-architecture.md` §12 item 3 says never to reach for. The provable
    form is: *skip the prompt when every member of the bucket is a
    non-optional, non-counter-derived `EnterWith` with no `then`*. The `then`
    exclusion is load-bearing — riders queue in choice order and run in queue
    order (CR 615.5), so two candidates that both carry one make the order
    observable through the event log even though the board is identical.

    **Do not ship it without the card that keeps the branch alive**, and this is
    the whole reason it is a question rather than a patch. Suppressing the
    prompt returns CR 616.1's multi-candidate branch to dead code in a fuzz
    game — the ordering choice, the applied set across instances and CR 101.4's
    APNAP ordering among simultaneous choosers — which is the Kalitas gap for a
    third time, now caused by a fix. What keeps it reachable is a registered
    entry replacement that **does not** commute: a `Rewrite` that drops the
    event (CR 614.6) beside an `EnterWith`, where the order decides whether the
    `EnterWith`'s CR 614.5 slot is spent and its rider queues at all.
    Containment Priest is the printed shape and needs a "wasn't cast" predicate
    `EventPattern` does not have. **Order: card first, suppression second.**

    **Landed in RC-4 (2026-09-02), card first** — Containment Priest, the
    `Instead` beside an `EnterWith` — **and the predicate is narrower than the
    theorem above, because the frame falsified one premise.** "Neither
    `EventPattern::EnterBattlefield` nor `set_affects` reads `mods`" stopped
    being true the moment `set_affects` read the look-ahead: a `PowerLE`
    filter matches a 0/0 before its +1/+1 counters land and not after, so
    Adaptive Shimmerer under "creatures with power 1 or less enter tapped" is
    a real choice — tapped or untapped by the order — and
    `test_a_power_filter_beside_counters_is_a_real_choice` says so. The rule
    that shipped (`pipeline::ordering_cannot_change_outcome`) admits only members
    whose applicability no `EnterMods` field can move: `EnterWith`, mandatory,
    static, under CR 614.5, not counter-derived, no rider, and an `affected_objects`
    over leaves the counters cannot reach (`filter_is_mods_invariant`). It is
    a semantics-assuming shortcut in `layers-architecture.md` §12's sense, so
    it carries its three expiry conditions in code and in `codebase-state.md`
    and a debug-build check that re-gathers after the suppressed choice and
    asserts the rest still apply.

20. **The replaceable event must be the outermost proposal** (found by the
    RC-4 review's nesting audit, 2026-09-02). A performer may propose a
    contained event only when the outer event is real whether or not the
    contained one survives replacement — otherwise the log records the outer
    half of something the CR says never happened, and no keying rule for a
    trigger matcher can repair a wrong `from`. Every nesting in
    `perform_action` and its callers, audited against that test:

    | Nesting | Is the outer event real on its own? | Verdict |
    |---|---|---|
    | `ZoneChange { to: Battlefield }` → `EnterBattlefield` | **No** — entering *is* the zone change (CR 614.1c, 603.6a) | the entry hop; ✅ RC-4b |
    | `CreateToken` → `EnterBattlefield` | **No** — the token is created in the zone before its entry is decided | ✅ RC-4b, the cheap token answer (§9) |
    | `cast_spell`'s 601.2a move → the rest of casting | **No** — CR 601.2 rewinds it, and four rewind sites do, silently | ✅ RC-4b, design item 7 |
    | `Destroy` → `ZoneChange { Destroyed }` | Yes — CR 701.8b; regeneration replaces the outer, a finality counter the inner | fine |
    | `DrawCard` → `ZoneChange { Drawn }` | Yes — CR 121.1; a draw replacement (RE) replaces the outer and the move never proposes | fine |
    | `DealDamage` → `LoseLife` | Yes — CR 120.3's results of damage | fine |
    | lifelink → `GainLife` | Yes — CR 702.15 | fine |
    | cost payment → `Tap` / `Sacrificed` | Yes — paid after 601.2h's can-pay check, and no rewind site exists after payment begins | fine |

    The test to apply to any new performer: if the contained proposal were
    replaced with nothing, would the CR still say the outer event happened?
    Destroying a regenerated creature, drawing from an empty library and
    dealing prevented damage all answer yes. Entering and casting answer no,
    and both have to be one proposal or none.


### Found by RD's design check (2026-09-08)


Seven items from sizing Phase RD against CR 615, 614.9, 609.7 and 120.3 and
against the tree. §9's RD section carries the decisions; these are the facts
that fell out of making them, recorded here so they outlive the section that
found them.

21. **Player scoping is RD's, and `types/replacement.rs` says RE.** *Closed by
    RD-1 (2026-09-08): `ReplacementDef.affected_players: PlayerSet` shipped,
    `set_affects` unions the two sets, and both comments are corrected.* The
    module doc and `gather::set_affects` both record that an effect applying to
    a *player* waits for RE's draw cards. The damage family is where the
    pressure actually is — 23 "prevent all damage that would be dealt to you",
    12 "would deal damage to you, prevent", 46 "source of your choice … to
    you", and Furnace of Rath's "permanent or player" — so RD-1 lands
    `ReplacementDef.affected_players: PlayerSet` and RE inherits it. A second
    field rather than an `ObjectSet` variant, because that type has three
    readers and two of them would have to reject the arm. The module doc and
    `set_affects`'s comment are corrected by RD-1; §3.2a's "lands in Phase RE"
    reads as history from here.

22. **`consume_use` runs before `apply_rewrite`, and three rules need it
    after.** *Closed by RD-2 (2026-09-09): `consume_use` runs after
    `apply_rewrite` and spends `Applied { took_effect, prevented }` — `Once`
    only when the rewrite took effect, `NextDamage` by exactly the damage
    prevented. The regression is a `Once` "prevent half, rounded down" chosen
    against 1 damage, which prevents 0 and stays
    (`a_once_prevention_that_prevents_nothing_is_not_used_up_dark_sphere_is_rd_3s`).
    RD-1 had left it, correctly: it shipped no arm whose application could do
    nothing.* CR 609.7b ("if for any reason the shield prevents no damage or
    replaces no damage, the shield isn't used up"), CR 614.9 (a redirect whose
    destination is gone "does nothing"; `ATOM-614.9-001` says the shield is
    not spent) and CR 615.12 (an unpreventable event reduces no shield). Every
    RB and RC rewrite takes effect whenever it is chosen, which is why the
    order was never wrong before — regeneration's `Prevent` on a `Destroy`
    cannot do nothing. RD-2 makes `apply_rewrite` report what it did and spends
    the use from that. The applied set is *not* moved: a chosen effect that did
    nothing has still had CR 614.5's one opportunity, and re-offering it is the
    hang §4.1 warns about.

23. **One printed shape splits a damage event in two, and §3.2d's `Option`
    cannot hold it — so it is a candidate PR with a gate, not an exclusion.**
    *Gate closed by RD-4 (2026-09-09): it fails, and Harm's Way is now
    `backlog.md` §2.25. A split-off member has no batch index, and ≈ 30 sites
    between `apply_rewrite`'s return and `execute_batch_inner`'s `decided`
    write are keyed by one — including the two the gate named as
    disqualifying. §9's RD-5 section carries the table, the three things worth
    keeping, and the one piece of genuinely new work the ~300–400 sizing did
    not contain (`next_damage_shares` allocating a count and a split with one
    choice). The original entry follows, unchanged, because the shape is still
    the shape.* Harm's Way — "The next 2 damage that a source of your choice would deal to
    you and/or permanents you control this turn is dealt to any target
    instead" — redirects *part* of one event: 2 of a 3-damage Lightning Bolt
    goes to the chosen target and 1 stays on you. That is one `DealDamage`
    becoming two with different targets, which is exactly the fan-out §3.2d
    removed `Split` for, and its rulings pin the rest: the 2 may be split "1
    damage … to each of two different recipients" across *members* of a
    batch (decision 3's per-instance allocation), and redirecting 1 leaves
    "a 'shield' … for another 1" (decision 7's spend-by-amount). Whole-event
    redirection (Pariah, Palisade Giant, Kor Chant, Reflect Damage — 15 cards
    on `o:/would be dealt to you.*dealt to .* instead/`) needs none of this
    and is RD-4's. The shape for the split is **not** a `Rewrite` that
    returns two and **not** a rider (the moved damage is dealt simultaneously,
    by the original source, as part of the same event — CR 614.9): it is a
    phase-1 member insertion, the residue staying the event and the moved
    part a split-off member that continues the lineage. §9's RD-5 carries the
    gate — off the per-member path or into the backlog with the measured
    cost — because one card that was never played competitively should be
    excluded on a number, not on a feeling. Divine Deflection is *not* this
    shape: a `Filter` + `PlayerSet` row with `NextDamage(X)` is already a
    pooled amount, and it waits only on `AmountExpr::Variable`.

24. **§11 item 15 is answered: decisions are per `(batch, subject)`, rewrites
    per member, and CR 615.7's allocation is the one non-uniform rewrite.**
    *Built by RD-2 (2026-09-09): `execute_batch_inner` groups by `subject_of`
    and `apply_replacements` takes the group; the shield-counter board is
    `two_shield_counters_under_two_blockers_lose_one_counter_and_take_no_damage`,
    and its first-strike twin is what shows the key is the batch. The
    allocation is `next_damage_shares`, per instance across every member it
    applies to — later groups' members included — kept on
    `GameState::prevention_allocations`. Closed.* The
    table in §9's RD section checks the rule against every ruling the batch
    has had to satisfy — Kalitas's N Zombies, CR 122.1c's one counter under two
    blockers, 614.5's ×4, 615.10's "separately to … events that would happen
    at the same time", and 615.7 itself. The per-member shape got the counter
    board wrong and the per-batch shape §4.2 rejected got Kalitas wrong; the
    subject is the key both rulings agree on. RD-2 builds it; the shield
    counter under two blockers is the regression.

25. **"Unpreventable" is a property of the event, `is_prevention` is derived,
    and `cant-effects-architecture.md` §4.7's `ReplacementKind` is
    superseded.** ***Closed by RD-4 (2026-09-09), and all three of its claims
    shipped as written.*** `GameAction::DealDamage.unpreventable` is the
    per-event shape; `Restriction::ApplyReplacement { kind: Prevention }` is
    the other two, with the `to_players: PlayerSet` this item's own decision 0
    predicted; `ReplacementDef::is_prevention()` stayed derived and
    `is_regeneration` kept its authored bit. The three meet in one predicate,
    `pipeline::is_unpreventable`, called at the two sites a prevention arm
    applies — two rather than the one this item assumed, because RD-3 gave
    `Rewrite::Prevent` a prevented amount of its own (item 31). Not at
    `gather`'s door, exactly as argued. Pinpoint Avalanche is the registered
    per-event consumer; both restriction routes are fixtures, for item 26's
    reason. *The RD-1 note this supersedes:*  *Unchanged by RD-1, which ships
    neither the flag nor the consult: `AmountRewrite::PreventHalf` applies unconditionally there, and
    Ghosts of the Innocent's Excruciator ruling is RD-4's for exactly that
    reason. `AmountRewrite::prevented()` is the reporting half, and it shipped.* CR 615.12 has three printed shapes: a resolution's
    restriction with a duration ("damage can't be prevented this turn", 11
    instants — a registry row), a static ability's restriction ("damage can't
    be prevented" on Leyline of Punishment and Everlasting Torment — swept off
    the effective ability list, no row, filterable), and a fact about one
    event ("the damage can't be prevented", 9). The first two are two sources
    of RS-1's `Restriction::ApplyReplacement { kind: Prevention }`, which
    `is_prohibited` already unions; the third is
    `GameAction::DealDamage.unpreventable`, set by the proposer and carried
    through a redirect. All three meet at one site — where a prevention arm is
    *applied* — and not at `gather`'s door, because CR 701.19c withholds a
    regeneration shield ("not applied") while CR 615.12 applies a prevention
    effect and lets it prevent nothing. Recognising a prevention effect needs
    no authored bit: CR 615.1a defines it by the word "prevent", which the def
    carries as `Prevent` or `Amount(PreventUpTo | PreventRemaining)` on
    `EventPattern::DealDamage`, so `ReplacementDef::is_prevention()` is a
    method and a card cannot forget to set it — the CDA and CR 614.15 argument
    (item 12) for the third time. `is_regeneration` keeps its bit, since
    nothing in a regeneration def distinguishes it from any other
    `Prevent`-with-a-rider.

26. **Every "damage can't be prevented" card but two carries a half the engine
    lacks.** *Discharged by RD-4 as far as it can be (2026-09-09): Pinpoint
    Avalanche is registered and CR 615.12's per-event route is exercised by a
    printed card; both **restriction** routes are built and tested as fixtures
    in `tests/phase_rd4_integration_test.rs` — a `RegisteredRestriction` row for
    the turn-scoped form and a static `Effect::Restriction` on a fixture
    permanent for Leyline's — and neither has a registrable consumer, which the
    PR said in as many words. The item stays open as the note that says why,
    and it closes when RE's `GainLife` pattern arm lets Skullcrack land the row
    in a game.* Skullcrack and Leyline of Punishment print "players can't gain
    life" (RE's `GainLife` pattern arm), Unstable Footing has kicker, Stomp is
    an adventure, Flaring Pain has flashback, Wild Slash is a `Conditional`,
    Everlasting Torment has wither, Combust "can't be countered" (§8a's missing
    counter event). Pinpoint Avalanche is clean and is RD-4's per-event
    consumer; the restriction consult is built against a fixture row and
    Skullcrack lands it in a game in RE-3 (§9, sized 2026-09-11), which also
    gives `Restriction::Event` the `affected_players` the row needs — see
    item 45. Recorded so the arm's producer
    status is not misread as an omission.
    **Closed by RE-3 (2026-09-12).** Skullcrack is registered, both of its rows
    are `Primitive::Restrict`, and the prevention one is RD-4's fixture row
    printed on a card. Leyline of Punishment stays unregistered on its
    opening-hand clause, and the RD-4 fixture now carries the static form of
    both of its other sentences rather than only one.

27. **CR 120.3 has eight results, RD ships two of them, and the other four
    have an owner each as of today.** *Closed by RD-1 (2026-09-08), with one
    addition the sizing did not name: 120.3e is now gated on the target being a
    creature, because writing the arm as a list of results made the ungated
    marking visible as bookkeeping the rule does not have.* 120.3a (life loss) and 120.3c (loyalty)
    are RD-1's. 120.3b and 120.3g (poison — infect and toxic) and 120.3d
    (wither's and infect's counters) have no keyword flag behind them and are
    `backlog.md` §2.6's, which now names the three keywords, their results,
    their size and RD-1's seam; 120.3h (a battle's defense counters) has no
    card type behind it and is `backlog.md` §2.23's, new today because no doc
    owned CR 310. The ledger's `T21c` was the only thing pointing at them,
    and a `T##` names work without owning it. The performer's arm is written
    as one `match` per result on the source's keywords and the target's type
    so each lands as one arm; none is stubbed, and each gets a dated Deferred
    Migrations line at RD-1's commit. 120.3e (marked damage) and 120.3f
    (lifelink) were already there.

28. **The inverse of a doubler is printed, it rounds, and the rounding is
    the card's.** Asked on review: "half that damage, rounded down" (Ghosts
    of the Innocent, an `Instead`-shaped halving), "prevent half that damage,
    rounded up" (Gisela) and "rounded down" (Dark Sphere), and a rider's
    "half that many, rounded down" (Sokrates) are four printings of one
    arithmetic in three places. CR 107.1a puts the direction on the card, so
    `Rounding { Up, Down }` carries no default, and it appears in exactly the
    places the text puts it — `AmountRewrite::Halve` and `PreventHalf` on the
    rewrite, `AmountExpr::Half` on an amount — never as an engine-wide
    convention. `Halve` and `PreventHalf` stay two arms because CR 615.12
    separates them (Ghosts halves unpreventable damage; Gisela cannot
    prevent any of it) and because only the prevention arm reports what it
    prevented. `Multiplier`'s inverse is therefore not a `Divide(n)`: nothing
    printed divides by anything but two, and a general divisor would be an
    arm with no second customer.


### Found by the RD-1 review (2026-09-08)


29. **Two Furnaces prompt, and multiplication commutes — so §11 item 19's
    suppression theorem has a second candidate, and it is narrower than "N
    identical effects".** Asked on review: is a bucket of N identical effects
    that ask the player nothing always order-invariant, so CR 616.1's prompt is
    noise? For *triggers* the answer is trivially yes — they go on the stack in
    one APNAP pass and their relative order is fixed at that moment. For
    **replacement effects it is not**, and the reason is the one the reviewer
    reached unprompted: CR 616.1f re-gathers after every application, so the
    bucket is re-formed between members and "the other members" is not a fixed
    set. That is exactly the premise `ordering_cannot_change_outcome` spends its
    longest clause on.

    **What is provable is narrower and still worth having.** A bucket every
    member of which is `Amount(Multiplier(n))` is order-invariant: multiplication
    over `u64` is commutative and associative (saturating included, since
    saturation is monotone), and no multiplier can remove another's
    applicability — an amount above 0 stays above 0 under any `n ≥ 1`, so
    `never_happens` cannot fire between members, and `EventPattern::DealDamage`
    carries no amount predicate for a member to fall out of. Two Furnaces are
    that bucket, and the prompt in `two_furnaces_multiply_by_four_and_ask_once`
    is noise a human player would resent.

    **What breaks the moment the bucket is mixed** is the phase's own headline
    board: `Halve` beside `Multiplier` does not commute (3 → 1 → 2 or 3 → 6 → 3),
    which is `COMP-614-DAMAGE-ORDERING-001`. So the clause cannot be "all
    members are `Amount`"; it has to be "all members are `Amount(Multiplier)`",
    and `PreventHalf` and `Plus` each need their own argument if they ever want
    one. `Plus` beside `Plus` commutes; `Plus` beside `Multiplier` does not.

    **Is it worth it?** The cost is one more clause on a
    semantics-assuming shortcut that already carries expiry conditions
    (`layers-architecture.md` §12 item 3), plus a `check_order_invariance`
    arm — and the clause goes false the day an `EventPattern` field reads the
    amount, or a `Multiplier(0)` is printed, or `Uses::NextDamage` makes a
    member spendable mid-bucket. **The verdict is: not RD-1's, and RD-2 decides
    it**, because RD-2 changes the loop's unit and the suppression predicate is
    read at exactly the site it changes. Deciding it earlier would mean writing
    the clause twice.

    **Decided at RD-2's close (2026-09-09): yes, and built.**
    `ordering_cannot_change_outcome` — the predicate, renamed for item 65's
    reason now that "entry" had become wrong as well as implementation-shaped —
    admits a set of choosable effects that is entirely `Amount(Multiplier(n ≥ 1))` on
    `EventPattern::DealDamage` beside the all-`EnterWith` one, under the same
    shared clauses (mandatory, static, no rider, under CR 614.5, not
    counter-derived). Two Furnaces ask nothing; the composite atom
    `COMP-614-616-DOUBLE-REPLACEMENT-001` drops to `COVERS-PARTIAL`, because
    "Player A chooses order" is the half deliberately not built. The debug
    re-gather checks per member, since a candidate may apply to a subset of a
    group from RD-3 on. `codebase-state.md` item 47 carries the two new expiry
    conditions: an `EventPattern::DealDamage` field that reads the *amount*,
    and a printed `Multiplier(0)`, which the `n ≥ 1` clause refuses. Measured
    as its own arm of RD-2's A/B (§9, RD-2 as landed).

30. **A test that asserts "nothing happened" cannot say *why* nothing
    happened, and the trace sink is the instrument.** Raised against Ghosts of
    the Innocent's "half of 1 rounded down is 0, so a source that would deal 1
    damage won't deal damage at all": the test asserts 0 damage marked and no
    `DamageDealt` event, and neither distinguishes "Ghosts halved 1 to 0 and
    CR 614.7a dropped the emptied proposal" from "the damage never reached the
    pipeline". **The differential closes most of it** and was added — the same
    1 damage without Ghosts marks 1, and 2 damage with Ghosts marks 1, so the
    instance demonstrably applies on that board — but the *rule* that dropped
    the event is still not observable, and no assertion available today makes
    it so.

    That is a general property of `Rewrite::Prevent` and of every arm that can
    empty an event, so it is the trace sink's case rather than this test's, and
    the sink is already scheduled as its own PR (`roadmap-v2.md` A4c, moved out
    of A6 on 2026-09-08 and now after CM-4). **Nothing about RD moves it
    earlier**: the differential is available on every board RD can build, and
    RD-2's boards — a shield that prevents 0 without being spent — are where
    the argument for the sink actually gets stronger, since "nothing was
    consumed" has no event at all. Re-ask at RD-2's close, alongside the trace
    *page* decision §9 already schedules there.

    **Re-asked at RD-2's close (2026-09-09): A4c stays where it is.** RD-2's
    "nothing was consumed" boards turned out *not* to strengthen the case,
    because a CR 615.7 count is materialized state: `Uses::NextDamage(remaining)`
    sits on the row before and after, so "the shield was not spent" is a direct
    assertion (`counts(&game)` in `phase_rd2_integration_test`), and a `Once`
    row that prevented nothing is still in the registry to be counted. RD-1's
    Ghosts board had only an absence to assert; RD-2's have a number. The
    trace page walks the same boards by hand, which is tier 1's job; the
    sink's argument and its slot are unchanged.


### Found by RD-3 — sources, and its review (2026-09-09)


31. **`Rewrite::Prevent` reports how much damage it prevented, and RD-4 will
    read it.** CR 615.6's "prevent that damage" is the whole amount, and
    CR 615.5's rider may refer to it — Reverse Damage's "you gain life equal to
    the damage prevented this way" is the printed reader, and it was the first
    def in the crate to ask a whole-event `Prevent` what it prevented (RD-1's
    Angel of Suffering rides on `ReplacedAmount`; RD-2's counts reach the number
    through an `Amount` arm). The arm reports it rather than the caller
    deriving it from a dropped event, and the reason is the same line
    `ReplacementDef::is_prevention` draws: **only the arm knows the event was
    damage**, and a `Prevent` on a `Destroy` is regeneration, which prevents no
    damage at all (CR 615.1 is about damage, CR 701.19c treats the two
    differently). A caller-side derivation would have had to re-ask the pattern
    and would have got regeneration wrong the day a regeneration shield gained
    a rider that read a number.

    **The RD-4 consequence is direct.** CR 615.12's "those effects won't prevent
    any damage, but any additional effects they have will take place" needs
    `Applied.prevented` to be **0 on an unpreventable event while the rider
    still runs** — so RD-4's consult belongs at the same site this number is
    computed, not at `gather`'s door, and the `Prevent` arm is now one of the
    two places (with `Rewrite::Amount`'s prevention arms) that has to ask.

32. **A new `EventPattern` field costs nothing until a def writes it, and the
    A/B is the proof.** RD-3's middle arm — the whole engine with `registry.rs`
    and `PERFORMANCE_POOL` unchanged — is **byte-identical to `main` outside
    the timing block on both pools at 200 games**. `pattern_watches` reads
    `source` and `combat` through `Option`, and every def written before RD-3
    carries `None`, so the added match arms are not reached. That is worth
    keeping as a general fact about §3.2a's growth contract: widening an arm is
    free at the measurement, and the cost of a phase like this is its *cards*.
    It also means a middle-arm timing delta with identical counters is spread
    and must be read as such — RD-3's was +4.8%, from one slow round out of
    three, with round 1 at +0.1%.

33. **The `--require` reachability instrument counts casts, and an activated
    ability's *activations* are invisible to it.** RD-3 predicted Circle of
    Protection: Red would be rare because "its activation competes for mana the
    random agent rarely has"; the `--require` row says it resolves in 130 of
    200 stress games, which answers a different question. The activation count
    had to come from `--dump-events` and a grep: **12,660 activations across
    200 games, in 129 of them**, about 98 per game the Circle is on the
    battlefield — a `{1}` with nothing else to spend on is something a random
    agent does until it runs out. Two things follow. The prediction is struck.
    And **a phase whose consumer is an activated ability should measure the
    activation, not the cast** — the report has no counter for it, and until it
    does the recipe is one `--dump-events` run and
    `grep "AbilityActivated: <name>"`.

34. **Asked at the RD-3 review: should CR 616.1's prompt be skipped by
    *simulating* both orders — `Lookahead`, compare the states, suppress if
    they agree — rather than by a static predicate? No, and the case that
    prompted the question is the reason why.**

    The case is two Guardian Seraphs, both `Amount(PreventUpTo(1))`,
    `Uses::Static`, no rider — the shape of item 29's multiplier bucket, and
    the arithmetic does commute: `max(a − p − q, 0)` whichever applies first.
    `ordering_cannot_change_outcome` does not admit it, and that is correct
    rather than an omission.

    **Three reasons, in increasing order of how hard they are to work around.**

    - **A rewrite is not a pure function of the event, so "simulate it" is not
      free of consequences.** `apply_rewrite` takes `&mut GameState` because
      CR 614.13 is the rules' own statement that applying an entry replacement
      *moves other objects* — `EnterAfterMoving` performs auxiliary zone
      changes through `execute_actions_new_batch` and prompts a player for the
      set. An ordering containing one cannot be speculatively executed and
      rolled back without the `DecisionProvider` having been asked a question
      that did not happen. `Lookahead`/`EntryFrame` is a *characteristics*
      look-ahead — it answers "what would this permanent be" — and it is not a
      board fork; the fork-and-search question (`codebase-state.md` item 44)
      is a different and much larger piece of work.
    - **Board equality is the wrong equivalence.** CR 616.1 gives the choice to
      a player, so suppressing it is sound only when *no rules-legal question*
      distinguishes the orders — and the event log is such a question.
      CR 615.13 triggers "each time a prevention effect is applied to one or
      more simultaneous damage events and prevents some or all of that damage",
      which counts *applications*, not life totals. Two orders that reach the
      same board by applying a prevention once versus twice are different games
      the moment item 6 lands. A `GameState` comparison cannot see that; a
      comparison that included the log would be comparing the thing the
      shortcut exists to avoid producing.
    - **And `PreventUpTo` is exactly that case.** Take `a = 2` against
      `PreventUpTo(5)` and `PreventUpTo(1)`. Apply the 5 first: the event is
      emptied, CR 614.7a drops it on the next iteration, and the 1 is never
      gathered — **one** application. Apply the 1 first: 2 → 1, then the 5 →
      0 — **two**. Same board, different number of CR 615.13 events. Deciding
      which case a given board is in requires the **amount**, and
      `ordering_cannot_change_outcome`'s signature is `(&[Candidate],
      Option<ObjectId>)` — `Candidate` carries the instance and the member
      *indices*, never their events. Reading the amount is precisely what
      `codebase-state.md` item 47's condition (d) forbids, so the predicate
      **structurally cannot** admit this arm. Two Guardian Seraphs keep their
      prompt, and the prompt is real.

    **The frequency says this was never a hot-path question.** Two Guardian
    Seraphs share a battlefield in **19 of 200 games with the card forced into
    every deck** (10%; four at once at the extreme, measured 2026-09-09). A
    simulator would run on every multi-candidate prompt to remove a prompt that
    rare.

    **What is available, if a prompt ever does need removing:** another clause
    on the static predicate, arriving the way item 29's did — with the rule
    number that makes the orders indistinguishable, and with the event log
    counted among the things that must agree. The bar is a proof, not a
    comparison.


### Found by RD-4 — redirection and unpreventable damage (2026-09-09)


35. **A group's subject stops being its members' the moment a redirect
    applies, and three questions were reading the wrong one.**
    `apply_replacements` captured one `EventSubject` at entry — the key
    `execute_batch_inner` grouped by — and passed it to `apply_rewrite`, to the
    CR 616.1 prompt and into every queued `Rider`. That was exactly right while
    no rewrite could move a member's subject, which is every rewrite RB, RC and
    RD-1 through RD-3 shipped. CR 614.9 moves it.

    **The failure is loud rather than subtle, which is the only good news in
    it.** Redirect damage from a player onto a creature carrying a shield
    counter: the next iteration gathers CR 122.1c's replacement half against the
    *rewritten* event, correctly, and its `Instead(RemoveCountersFromAffected)`
    then asks the group key which object to take a counter from — and the key is
    a player. `subject_object` answers `None` and the arm returns `Err`, failing
    the whole batch mid-performance.

    **The fix is to read the event, which is what CR 616.1f already says.**
    `subject` is re-derived per iteration from the first live member and
    per *member* at application; the rider takes the subject of the first member
    the application touched, read before its own rewrite, because CR 615.5's
    "that much" is about the event the effect replaced. The group key keeps its
    one job — one applied set, one chooser, one loop — and stops being consulted
    about objects. `a_redirect_hands_the_event_to_the_destinations_own_shield_counter`
    is the regression.

36. **`Rewrite::Retarget` makes `codebase-state.md` item 25 reachable for the
    first time, and it is still not worth building.** Item 25 — CR 101.4d's
    APNAP restart, a nonactive player's choice forcing an earlier player to
    choose again — has been unreachable since RB because nothing in phase 1
    could change *who* chooses about a member. A redirect can: an opponent's
    Pariah enchanting **your** creature moves damage aimed at them onto an
    object you control, so the CR 616.1 chooser for that member becomes you,
    and your own group may already have run to completion.

    **Reachable is not the same as reached.** It needs a two-sided board — one
    batch, two subjects with different choosers, and a redirect that crosses
    between them — and no card in either pool builds one, because both printed
    redirects here are `PlayerSet::You` scoped and Pariah on an opponent's
    creature is a play a random agent makes only by accident. The sizing is
    unchanged and still the one item 25 records: turning phase 1 inside out,
    a per-member state machine under an outer APNAP round-robin. **Revisit at
    Phase 6**, where triggers make the second half of CR 101.4 live anyway —
    not before, and no longer "at RD".

37. **The gate that decides an arm ships fired against `RetargetSpec`, and
    caught the arm §9 named.** `AmountRewrite`'s rule — every arm ships with a
    printed customer in the PR that lands it — admitted `ToEffectSource`,
    `ToHost` and `ToDamageSourceController` and refused `ToFixed(DamageTarget)`.
    The refusal is not "no card wants it": Harm's Way and Divine Deflection both
    do. It is that **no card can author it**, because a damage target is chosen
    at cast and a card file cannot name one — so the arm's only possible filler
    is `RegisteredReplacementEffect.targets`, threaded onto the instance, which
    is `codebase-state.md` item 90's work and arrives with its own card. An arm
    whose customer cannot reach it is worse than a missing one for exactly the
    reason §3.2's growth contract gives, and this is the first time that rule
    has excluded something a *design doc* wrote down rather than something a
    build wanted.

38. **`ObjectFilter::EachOther` was refused in an affected set, and the
    instrument that measured it as harmless measured the wrong thing.**
    `codebase-state.md` item 103 ran the three causes of
    `object_matches_filter`'s swallowed `Err` over 600 fuzz games, found zero,
    and left them on the argument that a mid-game panic is worse than a card
    doing nothing. Palisade Giant made one of the three live on its first
    board.

    **The measurement was sound and the inference was not.** Zero reachability
    over a pool that contains no card using a leaf says nothing about the leaf;
    it says the pool does not use it. The two remaining causes — an id with no
    object behind it, and `PowerLE` against something with no power — are
    genuinely card-authoring errors, and item 103's argument still holds for
    them. `EachOther` never belonged in that list: the layer walk has answered
    it off `FilterPlayers::source` since the layer system, `set_affects` has
    carried the same `source` since RB, and the two simply were not connected.
    **The general lesson, worth more than the fix:** a reachability zero is
    evidence about the *pool*, and it can only retire a concern that the pool
    could have exercised.


### Found by the RD-4 review (2026-09-09)


39. **Two of RD-4's three "missing destination" tests were the same branch, and
    the `COVERS-PARTIAL` on one of them claimed a leg no test reached.** Asked
    on review whether an unattached Aura is a real board. It is — `change_zone`
    detaches every attachment when a permanent leaves and **leaves the Aura on
    the battlefield** for CR 704.5m to find, so a single resolution that
    destroys a creature and then damages its controller reaches it. But that
    means `pariah_whose_host_has_left_the_battlefield_does_nothing` exercises
    `retarget_destination` answering `None`, not `redirection_is_legal`'s
    "no longer on the battlefield" — the same branch as the
    attached-to-nothing test beside it. Verified by probing `attached_to` after
    the host's zone change: `None`.

    **CR 614.9's first clause needs a destination that still exists and is
    still named**, and an Aura structurally cannot supply one. A **registry
    row** can: it keeps the `source` it was created with and CR 608.2c's
    duration outlives the permanent, so a resolution-created `ToEffectSource`
    redirect whose source has died still names it.
    `a_registry_row_whose_source_has_left_the_battlefield_redirects_nothing`
    is that board, and the `COVERS-PARTIAL` moved onto it. Mutation-checked:
    replacing the `contains_key` guard with `true` fails it.

40. **Nothing tested that `unpreventable` survives a redirect, and "the field
    is copied in one line" is not a reason it did not need to.** CR 614.9 moves
    "the same damage", so the flag travels with `is_combat` — and `is_combat`
    had a test (Pariah's first ruling) while the flag did not. A `Retarget` arm
    that forgot `unpreventable` left **every other test in the file green**;
    measured, not assumed, by setting it to `false` in the arm and running the
    suite. `a_redirect_carries_the_unpreventable_flag_onto_the_destination` is
    the twin, and it is the only test the mutation fails.

41. **`Restriction::ApplyReplacement`'s `to` was half a pair wearing the name of
    the whole thing.** `{ kind, to, to_players }` reads as an affected set with
    a player modifier hung off it; the two are unioned and neither is primary.
    Renamed `to_objects` — 12 sites, no behaviour. `ReplacementDef`'s
    `affected`/`affected_players` keeps its own names, where `affected` reads
    as the generic noun rather than as a preposition, and the asymmetry there is
    the older one.

    **The exemption was reversed 2026-09-14** — `codebase-state.md` item 124,
    built as `refactor/object-set-rename`. It did not survive its own argument
    being applied twice more: RE-3 renamed `Restriction::Event.affected` on it,
    which left `ReplacementDef` as the last type spelling the set bare, and "the
    generic noun" stops reading as generic once it is the odd one out. Both
    fields are `affected_objects` now and the type is `ObjectSet`. Written here
    and not only in `codebase-state.md` because item 46 below is about this
    document holding two answers for twelve days, and an exemption reversed in
    another file is exactly that shape.


### Found by RE's sizing (2026-09-11)


42. **§3.2d's Notion Thief encoding contradicts the card's ruling, and the
    lineage rule is what catches it.** "That player skips that draw and you
    draw a card" was filed as `Prevent` + `then`, the heterogeneous case.
    Its ruling walks two Thieves and says each is "applied to the card draw
    only once" and that in a two-player game "it really will be that player
    who draws" — which a rider's fresh proposal cannot deliver, since two
    Thieves would trade the draw forever. The Thief's draw is the same event
    with a new subject, `Instead(DrawCards { n: 1, player: Some(You) })`,
    and §3.2d is corrected in place. Alms Collector stays a rider. Worth
    keeping because it is the second time (after Furnace of Rath's) that a
    printed ruling decided a rewrite's *arm* rather than its numbers — the
    rulings pass is doing design work, not just test work. → RE-2.

    **Landed 2026-09-11, and it has a twin.** The encoding shipped as written
    and `two_notion_thieves_hand_the_draw_across_the_table_and_back` is the
    two-player board the ruling's last sentence names. What this item did not
    see is that Alms Collector is the *same* mis-filing — item 53 — and that
    the rule underneath both is CR 614.5's "any modified events that may replace
    that event" rather than anything about the word "instead". §3.2d now states
    it as a rule about where to split a printed "instead X and Y".

43. **Mana production is a chokepoint violation, and RA's census could not
    have seen it.** `resolve_mana_effect` (`mana.rs:91`) and
    `Primitive::ProduceMana` (`resolve.rs:337`) both write the pool directly,
    and `GameEvent::ManaAdded` — in the enum since the log was written — is
    emitted at zero sites. RA's audit walked *emissions* (§6's table was built
    from `events.emit` sites), so a mutation that emitted nothing was invisible
    to it, the mirror of the lifelink finding, which emitted without proposing.
    Two printed replacements (Mana Reflection, Nyxbloom Ancient) and item 6's
    "whenever you tap a land for mana" family read the event. → RE-9, and
    `codebase-state.md` item 111. **✅ Closed 2026-09-15 (RE-9).** One
    performer, `GameEvent::ManaAdded` emitted, both cards registered and one
    pooled; the family turned out to be fourteen cards rather than two, and
    six of them retype rather than multiply (item 94). The census lesson
    stands as written: a mutation that emits nothing is invisible to an
    emissions walk, and the fix was a proposal, not an emission.

44. **A lost player keeps taking turns and receiving priority in any game of
    three or more, and `has_drawn_from_empty_library` never clears.**
    `advance_turn` takes `(active_player + 1) % num_players` and the priority
    loop rotates the same way; nothing reads `player_lost` on either path.
    `fuzz_games` plays two, where a loss ends the game in the same sweep, so it
    has never shown. This is exactly the two-player assumption `CLAUDE.md`
    said `PlayerLoses` would hide, found by looking where it said. The flag is
    the same shape one rule over: CR 704.5b's window is "since the last time
    state-based actions were checked", and Exquisite Archangel's ruling ("you
    won't lose again until you try to draw again") is what a never-cleared
    flag gets wrong. → RE-6; `codebase-state.md` items 112 and 113 — and
    the objects a departed player leaves behind (item 108) are RE-7's, the PR
    after, since the day RE-6's four-player mode lands they are reachable and
    wrong. **✅ Closed 2026-09-12 (RE-6)**, its turn half having closed at
    RE-1: the priority loop rotates through `next_player_in_game`, the flag
    clears at the check that reads it, and the four-player mode measures the
    one part that stays open — item 108's permanents, 32 per game at the end
    — as RE-7's starting point.

45. **`Restriction::Event` carries no `PlayerSet`, so four printed "can't"
    families have no row shape.** "Players can't gain life" (25 cards),
    "can't lose life" (2), "you can't lose the game" (11), "your opponents
    can't win the game" (9) all scope a prohibition to players, and
    `Restriction::Event { pattern, affected, by }` has only the object set.
    RD-1 added `affected_players` to `ReplacementDef` and RD-4 added
    `to_players` to `ApplyReplacement`; this is the third instance of the same
    field and the last type without it. → RE-3 (Skullcrack), read by RE-6
    (Platinum Angel); `codebase-state.md` item 114; closes item 26.
    **Closed by RE-3 (2026-09-12)**, unioned by the same `set_affects` call the
    other two use. Twelve literal constructions took the field and one
    exhaustive destructuring took the binding — the row's "~6" counted the type
    name, comments and `..` patterns included. Platinum Angel is now a def
    rather than a design question.

46. **This document held two answers on skips for twelve days.** §11 item 6
    (2026-08-30) said skips go through the pipeline with the proposal built by
    the turn machinery; §9's RE paragraph (older) said a per-player
    `pending_skips` counter consulted at step begin. Nobody read them against
    each other until RE was sized, and the paragraph was the one a builder
    would have started from. Same for §8a's discard row, which said the
    pattern arm did not exist for sixteen days after RB built it. The rule
    that follows is `state-of-play.md`'s reason generalized: **a scope
    paragraph for an unsized phase goes stale in the merge that builds part of
    it, and re-reading it is the first step of sizing, not a courtesy** — the
    prompt for this sizing said so, and it was right twice. The review the
    same afternoon found a third: the first cut sent discard's producer to
    Phase 8 on §8a's sentence while RD-1 had built `Primitive::Mill` inside
    a replacement PR for the same reason — a precedent seventeen days old
    that a re-read of §9's own RD-1 section would have surfaced.


### Found by building RE-1 (2026-09-11)


47. **A turn queue needs a second field, and two players hide it.** The
    sizing asked whether the queue holds the player or the `(player, turn)`
    pair and got the right answer (the player). It did not ask how the
    *natural* rotation survives an extra turn, and `(active_player + 1) % n`
    does not: CR 500.7 inserts an extra turn **after** a turn, so the rotation
    resumes from the player whose natural turn it was. Every printed extra turn
    in RE-1's reach says "you take an extra turn", and on two players with a
    sorcery that is the same answer by accident — which is why nothing in the
    sizing caught it. Final Fortune is an *instant*, so a four-player table has
    P1 taking an extra turn during P0's and then their own natural one, and the
    arithmetic skips the second. `GameState.turn_rotation` is the field, and it
    advances when a natural turn is **proposed** rather than when one begins,
    because CR 614.10a proceeds past a skipped turn rather than re-offering it.

    **And a printed ruling says the field out loud** (Timesifter, fetched at
    the review on 2026-09-11): *"Remember which player would have taken the
    next turn if Timesifter's ability hadn't triggered the first time. After
    Timesifter leaves the battlefield and all extra turns have been taken, that
    player takes the next turn."* That is `turn_rotation`'s whole
    specification, written by the people who adjudicate it, on the card
    notorious for the deepest queue in the format — *"with two Timesifters on
    the battlefield, two extra turns are created for each turn taken"*, in a
    four-player game. **Third time a rulings pass has decided a design rather
    than a test** (after Furnace of Rath's numbers and Notion Thief's arm, item
    42), and the first where the ruling arrived *after* the code and confirmed
    it. Timesifter itself waits on item 6's triggers; the shape is tested
    without it — `phase_re1_integration_test::
    a_deep_queue_drains_most_recent_first_and_leaves_the_rotation_where_it_was`.

    Worth keeping because it is `CLAUDE.md`'s "write new systems N-player-shaped
    from the start" biting a *queue* rather than a player set — the shape was
    right and the cursor it moved was wrong. → RE-1, `codebase-state.md`
    item 115.

48. **The game's first turn had never begun, and no test could see it.**
    `GameState::new` wrote turn 1, player 0 and the beginning phase's untap
    step, and nothing ever called `on_step_begin` for that step — so turn 1's
    untap sweep and land-drop reset had not run in any game the engine has
    played. It is invisible because the untap step of an empty battlefield does
    nothing and `lands_played_this_turn` is already zero, and it was about to
    become visible for a much worse reason: item 6's 2,656 "at the beginning
    of" triggers would have read every turn's events except the first's.
    `Game::setup` now calls `start_first_turn`. **The class is the finding**:
    a constructor that writes a *state* the engine elsewhere reaches through a
    *transition* is a missing event, and RA's census could not have seen this
    one either, because it walked emissions and this site emitted nothing —
    the same blind spot as finding 43's mana pool. → RE-1,
    `codebase-state.md` item 116.

49. **"`advance_turn` is written as a turn queue so `backlog.md` §2.17 does not
    rewrite it a second time" is an overclaim, and it covers the turn level
    only.** Decision 6's own sentence, and the argument that pulled CR 500.7
    into a skips PR. It holds for extra *turns*: the queue is a list beside a
    cursor and the cursor never had to change shape for it. It does not hold
    for CR 500.8's extra **phases**, and the reason is the cursor RE-1 wrote.
    `next_turn_unit` answers "what follows" from `(Option<PhaseType>,
    Option<StepType>, phase_began)` — a phase **type** — so a turn holding two
    combat phases cannot say which one the cursor is at. The shape that can is
    the per-turn `TurnPlan` that `state::game_state::next_phase`'s pre-RE-1
    TODO already described, indexed rather than chained; RE-1 rewrote that TODO
    into a pointer instead of building it. So the function *is* rewritten
    twice, and the second time is a cursor change rather than an addition.

    **Raised by the review, on the fixture** that
    `a_phase_skip_cast_during_combat_is_spent_on_the_next_combat_phase` needs:
    a registered card (Moment of Silence) has a ruling with no engine-produced
    board. Three things bound the cost of having waited, and all three were
    checked rather than assumed: `advance_turn` is named in **no other RE PR's
    sizing** (RE-6's line for it was CR 800.4j/k, which RE-1 spent), the corpus
    files 500.8–500.10 as **DEFERRED with no atom ids**, so no phase's exit
    criteria move, and CR 500.10 needs item 6's triggers whatever happens to
    500.8 (Obeka is the only card for it). So waiting costs one extra rewrite
    and no interest — which is what makes this an **ordering call rather than a
    debt**. **Called 2026-09-11 by the owner: it comes into RE as RE-10**, on
    the argument that the sentence above is unkept and that 46 cards plus a
    registered card's fixture-only ruling is more pressure than several of RE's
    nine carry. §9's RE-10 section is the design; `backlog.md` §2.17 graduates
    with it.

    **Closed 2026-09-14, and the second rewrite cost less than the first
    charged for.** `advance_turn`'s cursor is an index into
    `GameState.turn_plan` and `next_phase` is gone. What the finding did not
    see is where the cost actually sat: not in the drainer, which lost lines,
    but in **46 sites that write `GameState.phase` by hand**, 19 of which then
    drain. The cursor is a second fact about the position, and a fixture that
    writes one of two facts is a fixture that drains from somewhere else.
    `GameState::set_position` is the seam that makes them unwriteable apart;
    the `debug_assert` in `advance_turn` is what found all 40 affected tests
    in one run. **The general shape, beside the one above:** a claim that a
    rewrite is cheap because the *function* is small is a claim about the
    function and not about its callers' picture of the state it reads.

    Worth keeping for the general shape: **a "we built it once so nobody
    rewrites it" claim is only as wide as the cursor it is made about.** RE-1's
    was made about a queue and is true of the queue; the sentence did not say
    which level it covered, and nobody read it against the level below until
    the fixture forced it.


### Found by building RE-2 (2026-09-11)


50. **`GameState::draw_cards` has had no caller since before RA and is a draw
    path with no proposal.** A `pub fn draw_cards(player, count, ctx)` that loops
    `draw_card` — the *performer* — so an N-card draw through it would make N
    library-to-hand moves with no `DrawCard` event, no CR 121.2 instruction and
    no lineage. It was correct-and-unused while `Primitive::DrawCards` looped
    `execute_action` itself; after RE-2 it is a second spelling of the outer
    performer that skips the pipeline entirely. The census counted `DrawCard`'s
    *producers* and this is not one — it produces no action at all, which is
    exactly the shape RA's emission-walk audit missed for mana (item 43) and
    §8a's table missed for discard. **Deleted rather than routed**, because a
    plural helper beside a plural `GameAction` is the ambiguity, not the
    convenience: `Primitive::DrawCards` is the one way to ask for N draws.

51. **`Game::setup`'s opening-hand comment claimed a route the code does not
    take, and had since RA-2.** Fixed in the same PR that found it; kept as one
    line because the general shape is worth a reader's second: *a comment that
    names a mechanism ages with the mechanism*, and this one was written about a
    route the same commit chose not to take.

52. **CR 121.2c orders two players' draws and nothing can express it.** "If more
    than one player is instructed to draw cards, the active player performs all
    of their draws first, then each other player in turn order does the same."
    Alms Collector's rider is the first thing in the crate that instructs two
    players to draw from one effect — `Effect::Sequence([draw for you, draw for
    that player])` — and a `Sequence` resolves in text order, so when the
    affected opponent is the active player the engine draws in the wrong order.
    Observable in the event log today and in gameplay the day item 6's
    "whenever you draw" triggers land. **Not built here**: the facility is
    APNAP ordering over *an effect's recipients*, which no `Effect` arm carries
    and which CR 121.2d (shared team turns) extends; one rider is one customer
    and the rule wants two. `codebase-state.md` has the line and the sizing.
    → RE-6, which is where a lost player stops being in turn order at all.

53. **Alms Collector as `Prevent` plus riders is an infinite loop, and its own
    ruling says so.** §9's RE decision 1 kept it as the heterogeneous case on
    §3.2d's rule — "you and that player each draw a card" is two unlike things —
    and that filing is what a rider's fresh applied set punishes: an opponent's
    Thought Reflection doubles the rider's one draw back to two, Alms applies
    again, and the two effects trade cards forever. Found by
    `alms_collector_does_not_apply_again_to_the_draws_it_produced`, which
    overflowed the stack the first time it ran; the ruling that decides it is
    the same one the test is named for.

    **Third time a printed ruling has decided a rewrite's arm rather than its
    numbers** — Furnace of Rath's, then Notion Thief's (item 42), now this —
    and the first time one has decided it *against* a correction made three
    hours earlier. Worth the entry for what the two Thief/Collector corrections
    share once they are side by side: the misreading is not about "instead"
    versus "and", it is about assuming `then` is where the second clause goes.
    CR 614.5's "any modified events that may replace that event" is the test,
    and §3.2d now states it as a rule about where to split rather than as a
    description of two cards.

    **And the failure mode is worth a line of its own.** Both mis-filings
    produce a loop rather than a wrong number, both loops are the kind CR 104.4b
    calls a draw, and CR 731 loop detection is §12's — explicitly out of scope.
    So an encoding error in this corner is a crash the engine cannot diagnose
    for itself, which is the argument for the bound
    `test_two_thought_reflections_draw_four_not_infinity` carries and for
    putting one on every board in RE-2's file that can hold two of anything.

54. **A three-round median said +6.0% and the number was noise.** RE-2's middle
    arm is the engine with the pool unchanged, and `fuzz_ab.py`'s default three
    timing rounds put `main` at 14.78 / 15.97 / 20.32 ms — a 27% outlier inside
    one arm, which makes the median 15.97 and the middle arm's tight 16.67–17.05
    look like a 6% regression. Seven rounds put `main` at 15.74–16.62 and the
    delta at **+0.5%**, which is what +34 gathers a game should cost beside
    RE-1's +5.0% for +447.

    **The rule, and it is cheap:** when one arm's rounds straddle another arm's,
    raise `--rounds` before writing the delta down. `fuzz_ab.py --rounds 7
    --no-fixtures` re-runs the timing block alone in about 70 seconds, against
    35 for the whole default sitting. Worth an item rather than a footnote
    because the number would have been *published* — §8's whole discipline is
    recording measured CPU deltas, and a phase that records a wrong one poisons
    every later phase that reads it. RE-9's decision is scheduled to turn on a
    2.5-point gate, and 6 points of noise is more than that gate is worth.


### Found by the RE-2 review (2026-09-12)


55. **Two draw doublers commute, and RE-2 shipped a prompt between them.**
    Raised in review against trace A: if two Furnaces of Rath do not need
    CR 616.1's question, why do two Thought Reflections? They do not. The
    theorem is the multiplier shape's **one level out** — a doubler's output is
    a `DrawCards`, which no `EventPattern::DrawCard` watches, so the composition
    happens through the *decomposition*, where each inner meets whichever
    doublers have not applied — and the total is the product of the members' `n`
    in either order. Checked exhaustively over the printed population before a
    line was written.

    **The premise needs one clause the multiplier shape does not**, and it is the
    clause the cause-stamping rule creates: a substituted instruction's inners
    carry the parent's cause once and `DrawCause::Effect` thereafter, so a member
    that admits the parent and *not* `Effect` applies to the first inner and to
    none of the rest. `None` beside `Some(TurnBased)` at `n = 2` and `n = 3` on a
    turn-based draw gives **4 one way and 6 the other**. No card prints a
    turn-based-only draw replacement, so the exclusion costs nothing and leaving
    it out would have cost the theorem. `player: None` is the other clause:
    Notion Thief's `Some(You)` moves the subject, and two subject-moving
    applications commute with nothing — which is the board the suppression is
    tested against.

    **`check_order_invariance` had to learn a second question.** The existing
    check asserts that every suppressed candidate still applies to the *rewritten
    event*; for draws it does not, because the suppressed doublers apply to the
    inners. It now probes the first inner instead. Worth the entry for the
    general shape: **a semantics-assuming shortcut's debug-build check is part of
    the shortcut**, and a new shape that cannot be checked the same way is a new
    shape that needs its own check rather than an exemption.

    **Reachable in a measured game**, which is why it is a fix and not a note:
    Thought Reflection is pooled at copies/deck 1.58.

56. **The acid test's bound was the prompt, and suppressing the prompt took it
    away — so the bound moved into the engine, derived.** `execute_actions_decomposing`
    now carries `GameState::decomposition_depth` and asserts
    `depth <= inherited.len() + 1` in debug builds. That is not a cap: a call at
    depth `d` exists because `d - 1` substitutions happened above it, and each
    inserted an instance into the applied set, so the bound *is* CR 614.5's set
    computed the other way and has no number in it. Break the lineage and depth
    climbs while the set does not; it fires at depth 2 naming the rule.

    **Strictly better than what it replaced**, and the reason is worth keeping:
    the provider bound only existed on boards that prompt, so it would have
    protected trace A and nothing else. This one protects every draw board.

    **The other half of the question it was asked: can a fuzz game loop?** Not
    from anything registered. The loop item 53 describes is a property of the
    encoding that was *rejected*; with what shipped, every board built from the
    four registered cards terminates, and all four were forced through 200 games
    apiece with zero errors and zero panics. What is true is that the harness has
    **no watchdog** — `--max-turns` cannot see a loop inside one resolution, and
    a stack overflow aborts the process rather than being counted as a panic, so
    it is the one failure `fuzz_games` cannot report. The debug assertion covers
    every test run; a release-mode ceiling is a number nobody has measured, and
    it is recorded here as the owner's call rather than guessed at. CR 731 proper
    stays §12's: it is about *game* loops and would answer "the game is a draw",
    which is a different question from "this engine ran out of stack".


### Found by building RE-3 (2026-09-12)


57. **`Primitive::Restrict` could only build an object-scoped row, and its first
    printed customer is the one that needed a player-scoped one.** The primitive
    looped `collect_battlefield_targets` and overwrote the def's affected set
    with each target, with a debug assertion demanding the card author an empty
    `Fixed`. Skullcrack breaks it in both directions at once: "players can't
    gain life this turn" names no object and every player, and its *target* is
    the player it then damages, so the loop would have written a row about that
    player's objects — or, with no object target, no row at all; and "damage
    can't be prevented this turn" authors `Filter { All }`, which the assertion
    forbids.

    **The fix is the assertion's own sentence promoted to a condition**: an
    empty `Fixed` beside `PlayerSet::Nobody` is the *marker* for "the resolution
    supplies the subject", and anything else is complete as authored and gets
    one row. RS-1's existing test authors exactly that pair and is unchanged.

    Worth the entry because of how long it hid: RS-1 shipped the primitive with
    no registered consumer, RD-4 needed both of its row shapes and built them as
    `RegisteredRestriction` fixtures because every printed carrier was blocked
    on something else, and the gap between "the type can express it" and "a
    resolution can create it" survived two phases that each touched one side.
    **A `Primitive` with no registered consumer is a `Primitive` whose shape is
    a guess**, which is the same rule as "an arm the pipeline cannot apply is
    worse than a missing one", read at the producer end.

58. **The CR 616.1 suppression premise was written about damage for the third
    time in three PRs, and it is now a property of the type.** RD-1 wrote
    `ordering_cannot_change_outcome`'s multiplier shape as `matches!(pattern,
    EventPattern::DealDamage { .. })`; item 19 found the entry shape's leaf table
    needed the same clause, item 55 found the draw shape needed it stated
    differently, and RE-3 found a `Multiplier(2)` over `EventPattern::GainLife`
    falling straight through the damage gate into a real prompt. Two Rhox
    Faithmenders commute for exactly the reason two Furnaces of Rath do, and the
    card's own ruling is the arithmetic.

    The clause the theorem needs is **not** "the pattern is damage"; it is "no
    member can stop applying as another member changes the number", which is
    `EventPattern::reads_the_amount` — matched exhaustively, and it answers
    `true` for exactly one arm today (`DrawCards { at_least }`, Alms Collector's
    "two or more").

    **It lives on `EventPattern` rather than beside its caller, and that is the
    finding.** Each of the three misses happened because the axis that moved was
    that enum, which grows one arm per `GameAction` variant every replacement
    phase — so the question has to be in front of whoever writes the arm, not in
    a predicate they have no reason to open. An exhaustive match makes it a
    compile error either way; *where* it sits decides whether the author answers
    it at the moment they are in a position to. The contrast is
    `filter_is_mods_invariant`, which stays at its caller because it classifies
    an `ObjectFilter` against a property of `EnterMods` — a relation between two
    types, and so a fact about neither.

    **The remaining shape lists are on the closed axis and stay lists.** The
    entry and draw shapes enumerate `Rewrite` arms, and `Rewrite` is a closed
    algebra whose every addition arrives with a CR cite and a review (§3.2b). The
    axis that grows silently is the one now covered.

59. **A phase can close `specdb`'s claim about its own corpus and should check
    it.** §9's RE-3 section said "nothing else in the corpus is filed under
    life-gain replacement". Two atoms are: `ATOM-119.7-004` is CR 119.7's last
    clause — *"a replacement effect that would replace a life gain event
    affecting that player won't do anything"* — which is Skullcrack's third
    ruling and is now covered; `ATOM-616.2-001` is the CR's own gain→draw→
    graveyard chain, already carrying an RB partial and now carrying a second
    from Words of Worship into Rhox Faithmender.

    Neither moves `specdb owed` (both are filed under Phase 8 and Phase 6, and
    `owed`'s scope is the three shipped phases), which is exactly why the claim
    rotted unnoticed: **a sentence about the corpus is not checked by the gate
    that checks the corpus.** The prompt for this phase said to verify it before
    relying on it, and that instruction is the generalization — item 46's rule
    about stale scope paragraphs, applied to the atom list rather than to the
    design.

60. **The fourth suppression shape has the shortest premise of the four, and it
    was nearly not built because the reason for skipping it was a cost
    argument.** RE-3 left two Tainted Remedies prompting and wrote down "no
    pooled card reaches this board" — true, and not the question. RC-4's rule is
    *never prompt for a choice with one outcome*, and this choice provably has
    one; the review that asked "why not?" was reading the rule and the PR was
    reading the budget.

    What the check produced is worth more than the fix. The premise the other
    three shapes carry is about *what the members do to the number* —
    multiplication commutes, entry mods merge, doublers compose one level out.
    This one needs none of that: **if every member carries the same `Rewrite`
    and applying it is a pure function of the event, the event after one
    application is the same whichever member applied it**, so the set of members
    still applicable to it is the same, and by induction so is the whole trace
    — however many end up applying, and whatever their patterns are. It is the
    only one of the four that does not mention `EventPattern`.

    **The clause that is not free is instance-invariance**, and it is what makes
    the leaf table earn its place rather than answering `true` everywhere:
    `GameActionTemplate::GainLife` embeds CR 609.6's source and
    `DrawCards { player: Some(You) }` the applying effect's controller, so two
    otherwise-identical statics on different permanents substitute *different*
    events. Extracting `substitute` from `apply_rewrite`'s `Instead` arm is what
    makes the purity a fact of the signature rather than a comment.

    And the shape needed its own debug check, on item 55's rule: a suppressed
    member here does not have to keep applying — Tainted Remedy's ruling is that
    it stops — so what is asserted is that its own `substitute` would have
    produced the same event.

    **The premise as first written had a false step, found by the review asking
    what the check actually checks** (2026-09-12, after merge). It argued that
    after one application "the same set of members is still applicable, since
    applicability is decided against that event". It is not: `applies_to`
    resolves the affected sets against *each instance's own* controller and
    source, so two defs that are `==` as data can differ on one event —
    `PlayerSet::Opponents` around two different permanents is the printed case.
    So order can change **which** members apply and **how many**.

    What was holding the theorem up unstated is **idempotence**: an `Instead`
    overwrites rather than accumulates, and the only field a template reads off
    the event is one the previous application already set to the value it will
    read — `ReplacedAmount` applied to a gain of 3 makes a loss of 3, and applied
    to that reads 3 again. So every trace ends at `T(e)` whatever `k` is. Now a
    second leaf table (`template_is_idempotent`), a second release-mode clause
    and a second debug assertion, because the arm that would break it is easy to
    write: "loses twice that much life instead" as a *template* rather than as an
    `AmountRewrite`, which is why doubling lives on that type.

    **Worth the entry for the general shape.** The first three shapes were each
    got wrong by under-specifying the *pattern* side (items 19, 55, 58); this one
    was got wrong by under-specifying the *iteration* side, and the two failures
    rhyme. A suppression premise has to say what is true of one application and
    what is true of repeating it, and a premise that only argues the first is a
    premise that has assumed the second.


### Found by building RE-6 (2026-09-12)


61. **CR 704.3's "performed" is not the performed set, and it took a ruling to
    say so.** The state-based check repeated only when `execute_actions`
    returned a member — right for the indestructible creature whose `Destroy`
    a "can't" drops, and wrong the first time a replaced state-based action was
    *all* rider: Exquisite Archangel's loss performs no member, and Stunning
    Reversal's eighth ruling — a short library loses "immediately after" —
    needs the check to repeat, or a priority window opens between the rider's
    draw that re-arms CR 704.5b and the check that reads it. CR 614.6 makes the
    rider's work the state-based action in modified form, so it *was*
    performed. The check now asks whether the event log grew, which a refused
    proposal never makes it do. **What this admits, and what closes it:** a
    `Uses::Static` replacement whose rider does not clear the condition is
    CR 104.4b's mandatory loop — Lich's Mirror controlled but not owned,
    ruling twelve — and the rules' own answer is a draw, so
    `check_state_based_actions_loop` caps its performing checks
    (`MANDATORY_LOOP_CHECKS`) and settles `GameResult::Draw` at the cap. A
    fixture that gains 1 life instead of losing, at ten poison, spins it
    (`a_loss_replaced_forever_is_a_draw_not_a_hang`).

62. **CR 104.2a is a fact about a batch, and CR 104.1 is a line at the
    chokepoint.** The section said `PlayerWins`' performer records the result
    and `check_game_over` reads it; it did not say where the losses' outcome is
    decided, and the first place to hand was the `PlayerLoses` performer —
    which, asked "is anyone left" after the first of two simultaneous losses,
    crowns the second loser's opponent where CR 104.4a says draw. Stunning
    Reversal's four-player ruling is the same rule from the other side: four
    members, one replaced, and the survivor "wins the game as soon as everyone
    else has lost". So `execute_batch_inner` settles once every member has
    performed, and `GameResult` lives on `GameState` because that is where the
    batch is. "Immediately" then has one home too: a check at the top of the
    same function, which stops a decomposition's later inners, a resolution's
    later instructions and the ending batch's own riders at one line — the
    survivor never draws the seven cards the ruling says they could lose to.

63. **A rider runs after the batch, and one printed ruling wants it inside.**
    Exquisite Archangel's first ruling: lethal to it and to you at once, "its
    effect applies ... You choose whether Exquisite Archangel is moved to exile
    or to your graveyard." The loss and the death are two members of one batch
    decided against one board, so the replacement applies; but §4.1a's timing
    — riders after the performed events, CR 615.5 — means the death has
    happened when the rider's exile looks, and CR 400.7 makes the graveyard
    card a new object it does not find. The engine takes the graveyard outcome
    and offers no choice — one of the ruling's two answers, tested as it
    behaves, with the other unreachable by construction. **The size is
    `backlog.md` §2.25's, not a patch's.** The Archangel's "instead" is two
    events about two subjects, and a rewrite yields one (§3.2d); getting the
    exile *into* the batch beside the death is the same one-event-becoming-
    two that RD-5 gated closed for Harm's Way (≈30 mechanical sites in
    `apply_replacements`' return shape and phase 2's write), and on top of it
    two members moving one object to two zones turn the CR 704.7 collapse into
    a prompt. §2.25 carries the Archangel as its second customer now; item 125
    is the record that the choice is missing until it is built.

64. **The first `Condition` a replacement effect reads is asked at gather, and
    the argument is two rules that were already there.** CR 604.2 makes a
    conditional static's effect exist while its condition holds; CR 614.4 asks
    whether the effect exists *before the event*; so the "as long as" is part
    of "which replacement effects exist right now", which is the gather's
    question and not the rewrite's. Asked at application instead, a Laboratory
    Maniac with cards left would be a candidate that does nothing — a prompt
    beside Thought Reflection with one live option, which RC-4's rule forbids.
    CR 121.6a is what makes the board reachable: the draw reaches the pipeline
    with nothing to draw, and the condition is true at exactly that gather; the
    doubled draw whose first inner takes the last card and whose second is the
    win is the test that separates the readings. One evaluator —
    `settled_holds`, shared with CR 613.11's cost effects — so a leaf is one
    question wherever it is asked. The restriction sweep has the same leg
    (the review found it missing there and it was eight lines), so a
    conditional "can't" reads the way a conditional replacement does.

65. **CR 104.1 removed the post-mortem tail, and that is the one two-player
    stream change — read the way RE-3's review said to.** Since the harness
    existed, a player who had just lost kept receiving priority until the phase
    ended, and a random agent in that window cast spells and drew from its RNG.
    Nobody receives priority in a game that has ended now, so the middle arm
    reads `differ` outside `=== Timing ===` on both pools while every gameplay
    row is identical to the printed digit and `Layer walks` to the integer;
    `Memo hits` −0.3% is the size of the tail. The A/B's byte-identity line
    is a *sufficient* check and not the check, and this is the second phase to
    say so (RE-3's suppression was the first).

66. **The four-player run's first table was a measurement of the harness.**
    Avg turns 86.6, total damage 496 per game, and a departed player at −516
    life in the one turn-limit game's log: the attack-target list was
    `(0..n).filter(pid != active)`, so the random agent spent a hundred turns
    hitting empty chairs and the two survivors' game never had to end. CR 506.2
    — the defending players are the active player's *opponents*, and a player
    who has left is nobody's — is one `in_game` read, and the table it leaves
    (61 turns, 156 damage) is the one §3 records. Two things worth keeping:
    a wider table's first number is a measurement of the harness until the
    harness is checked, and the check that found it was reading the event log
    of the outlier rather than the averages — the same recipe as
    `engineering-practices.md` §3.2's tail rule.

67. **The Deferred Migrations list had become a queue of small fixes, and the
    review said so.** RE-6's first cut recorded six items, and four of them
    were wrong answers with fixes shorter than their entries: `Exile`
    reaching only the battlefield (ten lines), the CR 104.4b loop with no cap
    (fifteen), the restriction sweep lacking the conditional leg the gather
    had just gained (eight), and `fuzz_ab.py` reading two seats (fifteen). A
    fifth, CR 104.3f, was a catch-all the CR keeps for rules text and no card
    can produce. All four are fixed in the same PR, with a fixture each; the
    fifth is a sentence in "Out of RE". The list's own foot now says the rule:
    a wrong answer under about thirty lines with a fixture to prove it is
    fixed in the PR that found it, and the list is for what a later system
    has to carry. The entry that *does* belong there is item 125, whose fix is
    RD-5's gated facility — and writing it honestly meant sending the
    Archangel to `backlog.md` §2.25 as that facility's second customer, which
    is more useful than a "~40 lines" that was never true.

    Two more things the review caught: a departed player was still a legal
    *target* (CR 800.4a; a "target player draws" resolving after its target
    left would have drawn for an empty seat, and Laboratory Maniac there
    would have reached the `PlayerWins` performer's refusal as a game error);
    and three source comments narrated what the code used to do, which is the
    commit message's job (`engineering-practices.md` §2).


### Found by building RE-7 (2026-09-13)


68. **CR 800.1 is the gate, and without it "byte-identical on both two-player
    pools, by construction" was going to be false.** §9's Measured bullet gave
    the reason as "nothing here runs before a third player exists", and nothing
    in the first cut made that true: CR 800.4a's clauses hung off the
    `PlayerLoses` performer, which fires at two seats as readily as at four, so
    a two-player loser's whole board would have vanished a statement before the
    settlement crowned the winner. Three RE-6 tests said so immediately. The
    fix is the section's own scope rather than a guard invented for the
    measurement: CR 800.1 — "a multiplayer game is a game that begins with more
    than two players" — and CR 800.4's own first sentence, "unlike two-player
    games, multiplayer games can continue after one or more players have left
    the game". `GameState::is_multiplayer` reads the seat count the game
    *began* with, so a four-player game down to two keeps the rules; the A/B
    then reads `IDENTICAL` outside `=== Timing ===` on both two-player pools,
    which is the first RE phase since RE-3 to manage it. **The general lesson
    is about where a scope claim is checked**: "this cannot run yet" is a
    property of the code until a rule is found that says it, and the rule was
    one section heading above the one being implemented.

69. **A member that removes objects performs after the members decided against
    them, and CR 704.3 is why that is not a contradiction.** Phase 2 performs
    in batch order and every performer is loud about the board it finds; the
    `PlayerLoses` member is the only one whose performer removes *other*
    members' subjects, because CR 800.4a empties the departing player's zones
    inside it. A creature they own and that is dying in the same check is both
    a `Destroy` member and, a moment later, gone — and with the losses first,
    as `sba.rs` had gathered them since RE-6, the `Destroy` would have met "not
    on the battlefield". Moving the losses to the end of the batch costs
    nothing the rule cares about: CR 704.3 makes the events simultaneous, the
    subject-keyed CR 704.7 dedupe is indifferent between a player subject and
    an object one, and CR order among the losses is preserved by gathering them
    in it. What moves is the log's order, which is the only place a
    simultaneous event has one. The two alternatives were rejected on
    invariants rather than on taste — a removal "tolerant" of members already
    decided is a quiet performer, and deferring the leave past phase 3
    contradicts 800.4a's "as soon as the player leaves the game" and would run
    the riders against a board the departure should have emptied.

70. **`ATOM-800.4c-001`'s board could not reach its own rule, and the CR says
    so in one sentence.** The atom had the creature's *owner* leave the game
    and then asked what happens when the control effect on it ends; CR 800.4a's
    first clause takes every object its owner owns, whoever controls it, which
    the rule's own Bribery example states — *"If Bianca leaves the game, Serra
    Angel also leaves the game."* So the creature never survives to reach
    800.4c. The rule needs a **default controller who is not the owner**, which
    is CR 110.2b's gap and main item 9's subject, and session-1 had already
    written the right board down (line 1197, the Gonti scenario) six months
    before session-10 wrote the wrong one. Corrected in place with the reason.
    **Worth generalizing:** the corpus is authored, and an atom whose board
    contradicts a rule's own example is a defect the `owed` gate cannot see,
    because an atom nobody has tried to cover looks exactly like an atom that
    is waiting.

71. **CR 608.2n's tail had to learn that the object might be gone.** A spell
    whose owner leaves the game during its own resolution — `Primitive::LoseGame`
    aimed at its own controller, in a game of three or more — is taken out of
    the stack and out of `objects` by clause 1, and `resolve_taken`'s
    post-resolution block then asked `get_object(object_id)?` and turned the
    whole resolution into an error. CR 608.2m already had the answer — "if a
    spell or ability leaves the stack while resolving, it will continue to
    resolve fully" — and what does not continue is the CR 608.2n graveyard
    trip, because there is no card to make it. Six lines, a fixture, and the
    same shape as RE-6's `Primitive::Exile` correction: a performer that
    assumes its own object outlives the effect it is performing.

72. **The A/B could not see the row this PR exists to zero.**
    `fuzz-record.md` says of the four-player table that
    `fuzz_ab.py --players 4` "prints all of it"; RE-6 added "Turns after a
    departure" and "Departed-owned permanents" to `fuzz_games` and not to the
    script that diffs two of its runs, so the two rows in the table's bold
    middle were the two the instrument could not read. Added by name to
    `ROWS` and skipped by name in a two-player table — not by "every arm reads
    `?`", because a row that vanished because the harness stopped printing it
    is a regression and these two are the only ones legitimately absent.
    **The recipe this generalizes:** a harness row and the script that diffs it
    are two edits, and RE-6 made one of them.

73. **CR 603.6c answers the question RE-7 recorded as open, and it answers it
    in the sentence that names this event.** The first cut left
    `GameEvent::LeftTheGame` without a CR 603.10a frame on the reasoning that
    "whether a leaves-the-battlefield ability fires for a permanent that leaves
    the *game* is CR 603's and the rules do not say in one sentence". They do,
    and it is the sentence: *"leaves-the-battlefield abilities trigger when a
    permanent moves from the battlefield to another zone, **or when a phased-in
    permanent leaves the game because its owner leaves the game**"*. So the
    frame rides the event, captured in the same one-statement window
    `perform_zone_change` uses, and the deferral was a rules search that stopped
    at CR 800.4 instead of following the trigger side. **The residual is the
    qualifier, not the answer**: "phased-in" is a condition this engine cannot
    express, because phasing is not built, so the frame is unconditional and
    every permanent is phased in — `codebase-state.md`, "Before Triggered
    abilities" item 6, which now names the two things that would make it
    reachable rather than the question it used to hold.

    **The general shape is `engineering-practices.md` §8, written at this
    finding and for it:** a rule that is *about* an event is not always written
    where the event is. 800.4a says what happens to the permanent; 603.6c says
    what watches it. The habit is three greps — the chapter that owns the
    thing, the chapter that owns whatever watches it, and the CR's own words
    for what you are building — and it is hoisted out of this list because a
    lesson buried six thousand lines into a phase doc is a lesson nobody
    reaches.

74. **CR 111.4 names the token, and every def in the tree had been naming it
    wrong.** "If the spell or ability doesn't specify the name of the token,
    its name is the same as its subtype(s) plus the word 'Token'" — so
    Kalitas's Zombie is "Zombie Token", and the RB tests asserted "Zombie".
    Found by §8's rules pass on RE-4's first morning, reading CR 111 before
    touching `TokenDef`; `backlog.md` §2.27's measurement had counted fields
    against fields and could not see it. Two corrections to the type
    followed from the same read: `name` is an `Option` (CR 111.9 and 111.10
    give a name, CR 111.4 derives one), and power and toughness are
    `Option`s, because CR 208.3 gives a noncreature none and the lowering had
    been writing `Some(0)` onto one. `Subtype::word` is the printed word the
    default name is built from (`codebase-state.md` item 130 is its one
    residual).

75. **`TokenCreated` has two emitters, and CR 111.13 is the line between a
    token the event announces and one it does not.** Decision 3 gave the
    event to `CreateTokenIn`'s performer alone; the entry performer's token
    arm announces it too, ahead of `PermanentEnteredBattlefield`, because
    CR 111.2 is two sentences (created, then enters) and a "whenever you
    create one or more tokens" trigger wants one key whatever the zone.
    CR 111.13 — a copy of a permanent spell becoming a token "is not
    'created'" — is what settled it: that token enters from the stack with a
    `from`, takes the card arm, and is announced by its zone change alone.
    One emitter function, two callers, each announcing the placement it
    performed; the archive's "Decided before writing" has the argument.

76. **The first performer to propose a batch from inside a performer set the
    precedent: the outer reports the event as decided, the log counts what
    happened.** Three tokens proposed and one refused is `CreateTokens {
    defs: [3] }` in the performed list and two `TokenCreated`s in the log —
    `DrawCards { n }` against an empty library, one level up. Nothing reads a
    token count off the outer; a rider reads the count *proposed*.

77. **A rider's proposals carry the replaced event's applied set, and the
    loop RE-4's A/B found was the engine's, not the rules'.** First recorded
    here as a CR 104.4b mandatory loop with a cap that ended the game in a
    draw; the owner's review read CR 614.5 the other way, and the CR agrees
    with the review. Alms Collector's "you and that player each draw a card"
    is one replaced event with two draws, and its own ruling — an applied
    effect "can't be applied again to the resulting events" — covers both.
    So two Reflections and two Collectors across two seats terminate: P0
    draws two and P1 one, every effect having had its one opportunity. The
    engine looped because §4.1a's "fresh lineage" bullet gave a rider's
    proposals a new applied set, which is exactly the re-application the rule
    forbids. `Rider::lineage` now carries the group's final set and
    `resolve_rider` hands it to the rider's proposals through
    `GameState::rider_lineage`; the fixture asserts the CR's numbers. The
    nesting cap stays as an **engine guard** — with every nested batch
    carrying its lineage, CR 614.5 bounds every chain, so a nesting past
    `BATCH_NESTING_LIMIT` is a lost lineage and returns an error rather than
    a rules answer — and `Max batch depth` is a fuzz row so the bound is a
    measured number rather than a magic one. Reached by the four-player
    `stress` A/B (seed 12523, registered arm) and by nothing at two seats in
    200 games; two seats suffice.

78. **`decomposition_depth`'s assertion counted across a rider, and fired on
    a legal board.** Its invariant — depth bounded by the inherited set — is
    a fact about one lineage, and a rider inside a doubled draw starts a
    second one at depth zero. One Thought Reflection beside one Alms
    Collector (the board RE-2 registered both cards for) tripped it in every
    debug build; release builds carry the counter without the assertion,
    which is why 600 fuzz games had not. A fresh-set batch now zeroes the
    depth for its extent. Found by item 77's fixture's *control*.

79. **Four arms left absent, each with its customer named**
    (`codebase-state.md` items 126–129): `EventPattern::CreateTokens` has no
    kind field (Divine Visitation, Ojer Taq, Jinnie Fay, Xorn, Academy
    Manufactor); `AmountRewrite::Plus` over a creation is refused (Xorn);
    there is no `GameActionTemplate::CreateTokens` (Divine Visitation, Bard
    King of Dale — and the reason `ATOM-614.16-001` is `COVERS-PARTIAL`);
    and a creation carries no `EnterMods` (121 printed "tapped and
    attacking"). Each is a `GainLife`-shaped wait: the retrofit is sized and
    nothing registered reads it.

80. **The middle arm was the wrong instrument for `stress`, not the
    byte-identical check.** The owner's review read the fourth arm as a
    workaround for the identity check; the check is the strongest instrument
    the A/B has — whole-game trace equality, which is what made "flat" a
    checkable claim rather than a timing spread — and what was wrong was the
    *arm's definition*. "Registered, old pool" is an engine reading only on
    `performance`, because on `stress` registration *is* the pool change, so
    that arm reads the decks there (RE-3 noted it; RE-4's read `differ` in
    every row). The arm that reads the engine on both pools is the engine
    with the cards *unregistered* — `main`'s registry, `main`'s decks — and
    RE-4's was `IDENTICAL` on `performance` and differed on `stress` by
    exactly one `TokenCreated` line per Zombie Kalitas makes (26 in 200
    games), the one extra gather's walk of Kalitas showing as `Layer frames`
    7,166 → 7,169 per game. It is also the arm that showed item 77's loop was
    the decks' to reach and not the engine's to cause. So the three arms are
    **engine** (cards unregistered — the engine reading on both pools),
    **registered** (old pool — read on `performance`, and the `stress`
    re-record's first half), and **pooled** (the re-record); the Measured
    section's recipe says so.

81. **An exit beside `EnterWith`s is one outcome, and the predicate now
    says so** — the owner's review (`plans/handoffs/re-4-review.md`, R15),
    §4.1's standing question applied: *what does this check* — exactly one
    `Instead(ZoneChangeTo)` beside `EnterWith`s on an entry, every member
    static, rider-less and not optional; *what if it runs twice* — after the
    exit applies nothing entry-shaped matches, and after an `EnterWith`
    applies the exit still does and its substitute carries no mods, so the
    event that performs is the same whichever went first. Master Biomancer
    beside Hallowed Moonlight on a token was the board: counters on a token
    that ceases to exist in exile either way. RC-4's Dryad Arbor pair under
    Root Maze and Containment Priest — "the prompt that is real", item 19 —
    was this shape too, and is not asked now; the Shimmerer board is the
    real choice and carries `ATOM-616.1-001`'s partial. The debug check
    substitutes against the entry with its mods disturbed and demands the
    same event. **R10 from the same review, answered in the predicate's
    doc**: three of the four expiry conditions were already compile errors
    (an exhaustive `ObjectFilter` match, an exhaustive `EventPattern` match,
    a pattern arm that names every field) and the fourth is now —
    `EnterModsTemplate::is_fixed` destructures the struct — so whoever adds
    the field is sent to the premise by the compiler and not by a sentence.

82. **Four ledger lines were arms the PR had the type open for, and the
    review made three of them code** (`plans/handoffs/re-4-review.md`, theme
    B; the rule is `engineering-practices.md` §4's). `EventPattern::
    CreateTokens { kind }` — a `TokenKind` asked of each def, since a
    creation's tokens are not objects when the pattern is — with Divine
    Visitation as its customer; `GameActionTemplate::CreateTokens { def,
    count, mode }`, the fourth template arm decision 0 did not foresee, whose
    `Replace` is Divine Visitation and whose `Append` is Chatterfang's and
    Xorn's shape; and `TokenDef::enters_tapped`. **`AmountRewrite::Plus` over
    a creation was the wrong arm for Xorn**, and the review's "why leave it
    half done" is what found it: the printed "plus" adds *an additional
    Treasure token*, a named def, which is the template's `Append` with
    `Fixed(1)` and not arithmetic — so `Plus` stays refused, for a reason
    that is now the right one. Bard, King of Dale came in with them once R5
    found it is a draw doubler and a token doubler with both halves built,
    not a draw-to-token card (that is Hullbreacher). What is still recorded
    — a draw-to-creation leg, a template with a choice of def, "attacking"
    — is three lines instead of five, each naming the facility it waits on.


### Found by building RE-5 (2026-09-13)


83. **CR 122.6a's named putter was closed on an empty Scryfall query, and
    the review reopened and built it.** RE-5 sized item 43's field, ran the
    rules pass, found no printed effect that specifies who puts entry
    counters on, and closed the item on that — finding on the way that the
    premise behind it (Doubling Season "doubles counters *you* put on") and
    `ATOM-122.6a-001`'s expected result name the wrong card: the Season's
    counter half reads "a permanent you control", and Vorinclex is the
    reader. The owner's review rejected the close: a rule the CR states is
    owed whether or not a card prints it, since a card can be printed next
    set and custom card creation is a post-v1 goal — the CR is the customer,
    a printed card is the test (`engineering-practices.md` §4). And Bold
    Plagiarist shows the shape on a *proposal*, which RE-5 had not looked
    for: "whenever an opponent puts one or more counters on a creature they
    control, *they* put the same number and kind of counters on this
    creature" — the opponent puts counters on a creature they do not
    control, so `Primitive::AddCounters` writing its own controller as the
    putter was the same shortcut. Built at the review (theme A):
    `EntryCounters { counter, n, by: Option<PlayerId> }` and its template
    with `Option<PlayerRef>`, `merge` keyed on `(kind, putter)`, the door and
    the CR 101.2 check reading each row's putter ahead of the entry's
    controller, and `by: Option<PlayerRef>` on the two primitives, resolved
    by `resolve_putter`. The atom stays covered on its default half with the
    card correction in the test's doc; the corpus line is not edited, since
    the session files are authored and this is an erratum against a card.

84. **Item 47's condition (c) fired, from the multiplier side, and the
    re-derivation is recorded.** The condition was written as "`EventPattern::
    EnterBattlefield` gains a field that reads `mods`"; what arrived is a
    *different* arm reading `mods` — `AddCounters`'s entry door — which is
    the same hazard for the multiplier shape rather than the `EnterWith` one.
    Two reads: which kinds the mods carry, and whether each carries one or
    more. A multiplier of one or more changes neither, so a suppressed
    member stays applicable — but its `affected_objects` filter over an entering
    permanent reads the CR 614.12 frame, which +1/+1 counters feed, and a
    `PowerLE` doubler stops applying once another doubler has raised the
    count past it. The multiplier clause now asks
    `object_set_is_mods_invariant` of an entry's members, the clause the
    `EnterWith` shape always asked, and
    `a_multiplier_reading_power_is_asked_at_the_entry_door_only` shows the
    prompt is real there (2 or 4) and absent over a proposal (4). §4.1's two
    halves, answered: the clause compares one `ObjectSet`'s leaves to the
    fields `EnterMods` feeds; run twice, a multiplier of one or more leaves
    every kind on "one or more"'s side, which `check_order_invariance`'s
    re-gather confirms per member in debug builds.

85. **Additive beside mods-adding commutes on one kind and not across
    kinds, so §2.29's table needs the kind axis.** Hardened Scales beside
    Master Biomancer at an entry's second iteration has one outcome — both
    add to +1/+1, and 2 + 1 is 1 + 2 — and is asked. A plus beside an
    `EnterWith` that adds a *different* kind is a real order: CR 614.5
    gives the plus one opportunity, so a charge counter the `EnterWith`
    writes after it is not raised, and
    `a_multiplier_beside_an_enters_with_is_a_real_order` shows the same for
    a multiplier (6 loyalty and 1 charge, or 6 and 2). Recorded on
    `backlog.md` §2.29 at landing; **built at the review (theme B,
    2026-09-14)**: `ordering_cannot_change_outcome` is the commutation table
    §2.29 designed, per `(class, kinds)` with one board read — the kinds an
    entry's mods hold — so two Scales, Scales beside Biomancer on a present
    kind, disjoint-kind arithmetic and Divine Visitation beside Parallel Lives
    ask nothing, while a plus beside an `EnterWith` writing a new kind, a
    multiplier beside a plus on a shared kind, and a multiplier beside any
    `EnterWith` it touches stay real. `classify` is exhaustive over `Rewrite`
    and `AmountRewrite`, which is where the predicate's expiry conditions are
    a compile error now.

86. **A cost that puts counters must not be an effect that puts counters,
    and the event has no field for it yet.** Doubling Season's ruling: loyalty
    paid as a cost "isn't doubled … because those counters are put on as a
    cost, not as an effect." `Cost::AddCounters` is unimplemented (`costs.rs`
    returns `Err` for both its validation and its payment), so nothing can
    be asserted; when `backlog.md` §2.11 builds loyalty abilities, the
    payment's proposal needs a fact `pattern_watches` can refuse —
    `LifeLossCause::Cost`'s shape on `AddCounters` — so that CR 614.16's
    "the effect of a resolving spell or ability" is what the pattern reads.
    `codebase-state.md`, "Found by RE-5", item 129.

87. **`gather` has no source that asks a card off the battlefield, and
    five of RE-8's six consumers needed one.** Found by `engineering-
    practices.md` §8's rules pass: CR 701.9's own neighbours say nothing about
    where a discard-watching ability lives, and the rule that does is
    CR 113.6 — whose 113.6m puts an ability whose effect moves the object it
    is on out of a zone *in that zone*. Dodecapod's and Wilt-Leaf Liege's
    clause is therefore in the **hand**, and `gather`'s five sources are the
    CR 903.9b rule, the entering permanent (CR 614.12), the battlefield sweep,
    the counters and the registry. None asks a card in a hand, and
    `replacement_ability_sources` is populated at ETB, so no gate would see
    one either.

    **Counted rather than assumed** (Scryfall, 2026-09-14): eleven cards print
    "onto the battlefield instead", six are a sorcery's own instruction, and
    every replacement effect among them is that one family — Loxodon Smiter,
    Nullhide Ferox, Obstinate Baloth, Wilt-Leaf Liege, with Dodecapod's
    variant wording. So `GameActionTemplate`'s entry arm would have shipped
    with no printed customer at all, which is the exception
    `engineering-practices.md` §4 carves out of its own ship-the-arm rule: an
    arm whose customer needs a facility the PR does not have is a ledger line
    pointing at that facility. Both the arm and the five cards are one, under
    critical-path item 6a.

    **Nephalia Academy is what kept `by` honest.** It is a *Land*, so the
    battlefield sweep finds it and its `ObjectSet::Filter` reaches a card in
    hand the way every `Filter` already reaches any object in any zone — which
    is also the reading that says this is **not** item 9's source 2. That item
    is a **sweep over other zones** for effects about *other* objects
    (flashback, madness, Wonder, Leyline's opening-hand clause) and it needs
    CR 113.6's predicate; what the Dodecapod family wants is source 1a's
    shape, the object the event is about contributing its `SourceOnly`
    replacements. The two are one PR's worth of work together and critical-path item 6a
    owns both.

88. **CR 514.1's cleanup discard was N events and the rule says one.** The
    rule: the active player "discards **enough cards** to reduce their hand
    size to that number" — one turn-based action over N of them. The engine
    asked one card at a time in a `while` loop and moved each through its own
    `change_zone`, so a hand of ten made three batches where CR 603.2c's
    "whenever one or more cards are discarded" should see one. Fixed in this
    PR rather than recorded, on `codebase-state.md`'s thirty-line rule, and it
    is the one thing that moved RE-8's engine arm — a fifth arm with the loop
    restored is **`IDENTICAL` to `main` on both pools**, which is what
    attributes the movement exactly.

89. **A discard redirected into a hidden zone has undefined characteristics,
    and RE-8 is the first PR that can produce one.** CR 701.9c: "If a card is
    discarded, but an effect causes it to be put into a hidden zone instead of
    into its owner's graveyard without being revealed, all values of that
    card's characteristics are considered to be undefined." Nephalia Academy
    does exactly that — hand to library, both hidden — and its "you **may**
    reveal that card" is the clause that avoids the rule, which the engine
    does not model because §2.9's information model does not exist.

    **Unreachable, and the rule says why in its own second sentence**: the
    consequence is that a *cost* specifying a characteristic of the discarded
    card becomes an illegal payment, and `Cost::Discard` returns `Err` at both
    its validation and its payment arms. So nothing reads a characteristic of
    a card discarded this way. `codebase-state.md`, "Found by RE-8".

90. **A discard whose graveyard move is replaced loses the fact that it was a
    discard.** Found by reading a `--dump-events` log at RE-8's review, on a
    board the pool builds unforced: under Leyline of the Void a Hymn to
    Tourach's two cards leave the hand as `ZoneChange { to: Exile, cause:
    Exiled }`, because `GameActionTemplate::ZoneChangeTo` carries the
    substitute's cause and overwrites the original's. The card was still
    discarded — Dodecapod's and Wilt-Leaf Liege's rulings say so in as many
    words ("you've still discarded it. Abilities that trigger whenever you
    discard a card will trigger"), and CR 701.9c calls such a card "discarded"
    while describing exactly this move into a hidden zone. So critical-path item 6's
    discard-watchers would miss it.

    **Not RE-8's to fix, and not a thirty-line one.** The field is RB's and its
    three customers set it deliberately (CR 122.1h's finality counter,
    CR 903.9b's commander, Kalitas). What the CR seems to want is that a
    substitution about the *destination* keeps the reason the object was
    moving, which is a change to `ZoneChangeTo`'s shape — `cause:
    Option<ZoneChangeCause>` meaning "keep the original", or a rule that the
    original's cause survives unless the template names one — and to what every
    RB def means. It is reachable **13 times in 200 `stress` games** already and
    wrong the day critical-path item 6 lands, which is why it is a Deferred Migration rather
    than a backlog entry. `codebase-state.md`, "Found by RE-8", item 131.

91. **`fuzz_games` counted a resolution by its cause, and CR 608.2m's move is
    replaceable like any other.** The `--require` block read `resolved` as a
    stack departure with `ZoneChangeCause::Resolved`, so a spell that resolved
    under a Leyline of the Void — leaving the stack as `Exiled` — read as never
    having resolved. It was **13 of Hymn to Tourach's 142 casts** in one
    200-game `stress` run, which is what made that row look wrong on review and
    is the only reason it was found. The cause cannot discriminate, because
    Leyline replaces a *countered* spell's graveyard move too; the fix tracks
    the countered and fizzled ids and subtracts them. **Every `--require` row
    this project has recorded on a board holding Leyline, Kalitas or a finality
    counter under-counted the same way** — `performance` has none of the three,
    so only `stress` rows are affected, and RE-8's is re-read here (Hymn to
    Tourach 149 / **146**).

92. **`substitute` grows with templates, not with templates × actions, and the
    count says when to split it.** Asked at RE-8's review, whose worry was a
    1,500-line function once card breadth starts. Seven `GameActionTemplate`
    variants today and 235 lines, 86 of them comment — about 21 lines of code
    each — and only two carry a nested match over the event's kind
    (`ZoneChangeTo` and `DrawCards`); the other five take any event through
    `subject_of` and `template_amount` in four lines. Templates went 3 → 7
    across RB, RC, RD and RE, roughly one a phase, against §3.2c's census of
    574 printed "would … instead" clauses that needed zero new `Rewrite` arms.
    A thousand lines would take about fifty templates. **The split is
    mechanical whenever it is wanted** — one function per template, since
    nothing in the match shares state — and the trigger is written at the
    function: **when a third template needs a nested match.**


### Found by RE-9's design check (2026-09-15)


93. **"Tapped for mana" is a CR definition, not a ruling, and it names the
    activation cost.** Decision 7 grounded `tapped` on Mana Reflection's
    ruling and left "where `tapped` comes from" open between CM-3's payment
    plan and a field. CR 106.12 answers both: *"To 'tap [a permanent] for
    mana' is to activate a mana ability of that permanent that includes the
    {T} symbol in its activation cost"*, and CR 106.12b says a replacement so
    worded "modifies the mana production event" — the event's name in the
    CR's own words. So the flag is `Cost::Tap` in `ability.costs`, read where
    the effective ability is already in hand, and the plan is never consulted.
    Found by §8's third grep (the phrase, across the whole file) after the
    first two — CR 106 and CR 605 — had been read for a day without it.
    → RE-9 decision 2.

94. **The census undercounted the family by a factor of two, and the miss is
    a substitution leg.** §9's RE table read "if you tap a permanent for mana
    **2**; produces twice/three times 3; would add … instead 1". Re-run with
    `o:/tap.* for mana/ o:instead` and `o:/tapped for mana/ o:instead` the
    family is **fourteen**, and six of them change the mana's *type* —
    Contamination, Infernal Darkness, Deep Water, Hall of Gemstone, Naked
    Singularity, Harvest Mage — which is CR 106.12b's "of a specific type
    and/or amount" and needs an `Instead` template the row never priced. One
    of the six registers whole today (Deep Water, on Fog's
    `CreateReplacement` shape), which is what put the leg inside
    `engineering-practices.md` §4's eighty-line rule rather than in the
    ledger. The lesson is the one item 46 already states, from the other end:
    a census regex written for the *amount* family cannot see the *type*
    family standing next to it, and the check is to search the rule's own
    phrase — 106.12b's — rather than the card's. → RE-9 decisions 6 and 7.

95. **Decision 7's "`special` riding through unchanged" is wrong for a
    multiplier.** CR 106.6a: restrictions "apply to **all mana produced**". A
    `ManaAtom` is one unit, so a doubled production of one restricted {G} is
    two restricted atoms, and passing the `Vec` through unchanged would have
    produced one restricted and one free — `ATOM-106.6a-001`'s board with the
    wrong answer, on the atom this PR exists to cover. The correct shape is
    RE-4's `repeat_n` over defs, and CR 106.6a's per-mana sentences (a
    delayed trigger and a continuous effect "created once for each mana
    produced") are then the atom's own `grants` and `persistence` on each
    copy. → RE-9 decision 5.

96. **`GameEvent::ManaAdded` has carried a `HashMap` since the log was
    written, and the first emitter would have been the determinism
    regression.** `--dump-events` renders every event through `display.rs`,
    whose `ManaAdded` arm iterates the map; three shell runs at one seed
    would have disagreed on the line the first time a production carried two
    types. Unobservable for as long as nothing emitted the event — which is
    exactly item 43's shape, a defect invisible to an emissions census —
    and caught here only because RE-9 is the first PR that has to read the
    event it emits. The event becomes a `Vec` in proposal order in the same
    PR. → RE-9 decision 8; `CLAUDE.md`'s determinism rule, "any collection
    reaching a choice, log or count".

97. **`GameState.counters` is the diagnostics and its two sibling fields are
    CR 122's, and the review is where the collision was seen.**
    `PermanentState.counters` and `PlayerState.counters` hold +1/+1, loyalty,
    poison and energy; `GameState.counters` holds `EngineCounters`, the layer
    walks and gathers `fuzz_games` prints. Three fields, one spelling, two
    meanings, and RE-9's design check wrote "a permanent counter" about a
    diagnostic row without noticing which of the two it meant. Sized as a
    mechanical sweep — six `EngineCounters` sites, ~75 `.counters.` calls —
    and, on `codebase-state.md` main item 124's precedent, its own PR rather
    than a rider on a rules change; the proposed name is `EngineMeters` /
    `game.meters`, a word the CR never uses. RE-9 adds one method under the
    existing name and leaves the collision where it is. → RE-9 decision 9.

98. **The census regex read the card's phrase and not the rule's, and it hid
    two of CR 106.12b's three axes.** `o:/tap.* for mana/` matches "tap a
    permanent for mana" and misses "tapped for **two or more** mana"
    (Damping Sphere), "taps a **nonbasic land** for mana" (Pale Moon) and
    "target Plains is tapped for mana" (Quarum Trench Gnomes); the union of
    `o:/tapped for .*mana/` and `o:/tap.* for .*mana/` with `instead` is
    **sixteen** cards, not fourteen. Classified rather than counted: three
    multipliers; seven constant-type retypes (Contamination, Infernal
    Darkness, Deep Water, Pale Moon, Ritual of Subdual, Damping Sphere,
    Quarum Trench Gnomes); five chosen or mapped types (Hall of Gemstone,
    Harvest Mage, Pulse of Llanowar, Naked Singularity, Reality Twist); and
    Chaos Moon's even half. The two the regex hid are the two axes the rule
    names beside the permanent: an **amount** — Damping Sphere's "two or
    more", an `at_least` on the pattern that *reads the amount* exactly as
    Alms Collector's does, so a doubler beside it is CR 616.1's real
    question and Damping Sphere's own first ruling walks it ("choose one to
    apply. After that, determine if any others are applicable") — and a
    **chosen permanent**, Quarum Trench Gnomes' "target Plains", which is
    `SourcePattern.object` and which decision 6 declined on the sentence
    "nothing prints a chosen permanent tapped for mana", now false. Found by
    the review asking whether Damping Sphere belonged in the family
    (2026-09-15). Pale Moon registers whole on Deep Water's shape and does
    in this PR; the two axes and the type field are `codebase-state.md`
    item 133's, each with its card and the facility it also needs. The
    lesson is item 94's one step further: a census that searches the card's
    words finds the cards that share them, and CR 106.12b's own sentence
    names three axes in a row — search each.

99. **The event-kind gate returns nothing for mana, so §8's one pre-approved
    lever does not touch the cost the phase was worried about.** The review
    asked how the sitting knew the hottest path was safe when its engine arm
    had nothing to find and its pooled arm was a bigger board, and the
    answer was two more arms. A no-op `Multiplier(1)` row per player,
    applied on every tap with the game held fixed, costs **+2.7%** at two
    seats and +2.2% at four — the CR 616.1 loop's work with a candidate,
    about 4.6 µs an application, paid only where a mana replacement is on
    the board. And `gather` returning early for `ProduceMana`, RE-1's probe
    exactly, reads +0.1% and −0.6% with rounds straddling: the sweep is
    three memo reads, and the +1.2% a bare proposal costs is the batch,
    the prohibition check, the grouping allocations, the performer and the
    emit. RE-1 found the gate worth half its cost; here it is worth none,
    and the difference is what a production's sweep finds — nothing, fast.
    **§8's lever list is one short**: the fixed per-batch work of a
    single-member batch with no candidate is the cost every cheap event
    kind pays, and an allocation-free path through it is answer-preserving
    in §8's own sense. `codebase-state.md` item 136 sizes it and names the
    budget question it is waiting on. → RE-9's archive, "Measured again at
    review".

