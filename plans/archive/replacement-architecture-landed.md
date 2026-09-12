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
  orders* → the Furnace/Ghosts test generalised to a prevention (Gisela's
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
- **Two identical kind-changing substitutions are not a fourth suppression
  shape.** Two Tainted Remedies do prompt: both are applicable at the first
  iteration, and CR 616.1's question is "choose one to apply", not "choose one
  if it matters". Its ruling says the *outcome* is unaffected, which the test
  asserts the stronger way — the prompt happens and both answers are three
  life. A fourth semantics-assuming shortcut would carry its own expiry
  conditions for a board no ruling calls a choice and no pooled card reaches.

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

**The shipped arm is +4.0% CPU, and it is the card.** `ms / 1,000 walks` is
+0.2%: the walk did not get slower, there are more of them, because Rhox
Faithmender makes the games longer.

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
| CPU/game median | 15.73 ms | 16.36 ms | **+4.0%** |
| ms / 1,000 walks | 42.40 | 42.49 | **+0.2%** |

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
