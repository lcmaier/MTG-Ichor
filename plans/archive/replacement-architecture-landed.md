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
