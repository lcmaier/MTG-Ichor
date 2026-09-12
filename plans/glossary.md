# Glossary — the words this codebase invented

Every phase that coined a word defined it wherever that phase was standing: a
doc comment on the type that uses it, a commit message, one architecture doc's
§9. A reader meets *subject group* in `pipeline.rs`, *leg* in
`cost_determination/gather.rs` and *rider* in `types/replacement.rs`, and until
this file existed there was nowhere to look any of them up.

**Definitions point; they do not restate.** An entry says what the word means
here and names the file or section that owns the reasoning — the same budget
`CLAUDE.md` keeps for invariants. An entry that re-argues its owner's case is a
second copy, and the second copy is the one that goes stale. It also **defines
without renaming**: where this pass found a name genuinely wrong, the finding is
a `codebase-state.md` line and its own PR.

**It is checked.** `python plans/check_glossary.py` asserts that every term and
every code anchor here still resolves in `mtgsim/src`, that every word on its
watch-list is defined, and that a word it lists as polysemous carries **all** of
its senses, numbered.

**And a phase does not close until it has run `--suggest`.** The watch-list is
the gate's *input*, so it catches a coinage you remembered to add and is blind
to one you forgot — which is how `lineage`, `decomposition` and `containment`
reached a merged branch undefined, and were caught by a reviewer rather than by
the check. `python plans/check_glossary.py --suggest` reads the doc-comment
prose the branch added, drops what the CR itself says and what this file already
mentions, and prints the rest: 17 words for the PR that prompted it, against 587
for the same signal run over the whole crate. It reports and never fails, on
purpose — the residue is ordinary English, and a gate over a heuristic is a gate
people learn to silence. Triage it, and add what is a term of art to
`WATCHLIST`, which is what makes the gate keep it. The last is the one with a scar: `Rewrite::Retarget`'s
arms reached the build as `ToSource` and `ToSourceController` — two *different*
sources, adjacent in one enum — and both had to be renamed mid-PR.

## Words that mean more than one thing

**source** — **(1)** CR 609.7's source of damage or of an effect: the object
that dealt or produced it, carried by `GameAction::DealDamage` and matched by
`EventPattern`. **(2)** The object whose ability an effect *is* —
`ReplacementInstance`'s `source` field, `RegisteredReplacementEffect`'s,
`ContinuousEffect`'s. Also CR 609.7, and in a damage event it is a different
object from sense 1. **(3)** A *gather source*: one of the numbered places
`gather` looks to find the replacement effects that could apply to a proposed
event — a permanent's static abilities, a registry row, a counter. Not an object
at all, and the list being wrong is the failure that shows up as a card silently
doing nothing. → senses 1 and 2 are adjacent in `RetargetSpec`, which is why its
arms are `ToEffectSource` and `ToDamageSourceController`; sense 3 is
`replacement-architecture.md` §3.3's five and `cost-architecture.md` §4's two.

**shield** — three, and RD-2 is where they meet. **(1)** CR 614.1's metaphor:
every replacement and prevention effect "act[s] like a shield" around what it
affects. That is `ReplacementDef`, and nothing in code borrows the word for it.
**(2)** CR 615.7's prevention shield and CR 701.19a's regeneration shield: a
resolution-created registry row with a `Uses`. Regeneration is `Uses::Once`;
615.7's amount-bearing one is `Uses::NextDamage`, named for the rule's own "the
next 3 damage" because it counts damage and never uses. Neither takes the word
"shield" in code. **(3)** CR 122.1c's shield **counter**, `CounterType::Shield`:
a counter that *creates* one replacement and one prevention effect, synthesized
by `gather`. It is not a sense-2 shield — no amount, a whole-event `Prevent`,
and its "use" is the rider removing a counter. → `replacement-architecture.md`
§9, whose RD decision 3 is the one place senses 2 and 3 meet.

**pool** — **(1)** a mana pool, CR 106.4: `ManaPool`, one per player, emptied at
the end of each step and phase. **(2)** a *card* pool — which registered cards the fuzz
harness plays. `PERFORMANCE_POOL` is **representative, not frozen** (it was
frozen for a year; revised 2026-09-01, and a phase that opens a new engine path
adds one card to it); `--pool stress` is every registered card. →
`engineering-practices.md` §3.

**step** — **(1)** CR 500's turn step: `StepType`, an untap step, a combat
damage step. **(2)** a rung of CR 616.1a–e's choice ladder — `ReplacementClass`,
returned by `must_choose_among`. Sense 2 is the newest collision in this file:
it arrived when `codebase-state.md` item 65 renamed `forced_bucket` to the
rule's own word, which was right for that name and made this pair.

**blocked** — **(1)** combat: a creature a blocker blocked, CR 509.
`AttackingInfo`'s `is_blocked`. **(2)** prose only: an event a CR 614.17 "can't"
forbids, checked ahead of the replacement pipeline and winning over it
(CR 101.2). Sense 2 had a function — RB's `engine::replacement::is_blocked`,
which RS-1 deleted; the mechanism is `engine::restriction::is_prohibited` asked
with a `Query::Event`. Four pointers outlived the name, and a grep after one
lands on sense 1.

**gate** — **(1)** a cheap precondition deciding whether to do expensive work:
"a gate, not an answer". It may over-approximate and cost a walk; it may never
under-approximate, or the answer it guards is silently lost —
`replacement_ability_sources`, `cost_modification_ability_sources`.
**(2)** a process checkpoint a phase must pass before it closes: "a gate, not a
report" — `specdb.py`'s `owed`, the three `check_*.py` scripts. →
`engineering-practices.md` §5.

**census** — **(1)** a *card* census: a Scryfall query partitioning printed
cards by mechanism, to size a phase before designing it.
`plans/references/*-census.py`, one per subsystem, re-runnable.
**(2)** a *call-site* census: counting the sites a change will touch before
writing it, and recording the number so the prediction can be scored afterwards.
Sense 2 is what missed `apply_lifelink` — it censused `emit` sites, and lifelink
wrote `life_total` by hand while emitting loudly.

**registry** — three tables wear the name and only two are game state.
**(1)** `CardRegistry`: card name → constructor, the definitions themselves,
nothing to do with a game in progress. **(2)** `ContinuousEffectRegistry`:
CR 613 continuous effects created by a resolution, each with a `Duration`.
**(3)** `ReplacementEffectRegistry`: CR 614.3 / 615.7 / 701.19a rows, with a
`Duration` and a `Uses`. Senses 2 and 3 are both `DurationRegistry` instances,
and for both, **membership is not effect existence** — CR 305.7 or Layer 6 can
take the ability away without touching the row. → `CLAUDE.md`;
`layers-architecture.md` §5.2.

**queue** — three, and only one is a queue. **(1)** `GameState`'s `turn_queue`:
CR 500.7's extra turns, and it is a **stack** — "the most recently created turn
will be taken first" is `Vec::pop`, so the name says FIFO and the rule is LIFO.
**(2)** `ScriptedDecisionProvider`'s expectation queue, which really is one:
enqueued in test order, popped per decision. **(3)** a *queued* rider — CR 615.5's
"and" clause, pushed during the CR 616.1 loop and resolved after the event
(`Rider`, `execute_batch_inner`'s phase 3). →
`replacement-architecture.md` §9's RE-1, §4.1a.

**unit** — **(1)** one of CR 614.10's three replaceable pieces of turn
structure: a turn, a phase or a step. `TurnUnit`, returned by `next_turn_unit`,
and the thing a skip replaces with nothing. **(2)** a unit of *work* — one
branch, one PR (`CLAUDE.md`'s Git workflow). Sense 2 predates sense 1 and is
prose only; nothing in the crate is named for it.

**schedule** — **(1)** what the turn machinery will propose next: `turn_queue`
and `turn_rotation` together, read and **consumed** by `advance_turn` as it
builds a proposal. Naming it is load-bearing — the schedule is an *input* to
CR 614 rather than state a replacement effect or a trigger can see, which is
why writing it outside `perform_action` is not a chokepoint violation. **(2)**
an ordering of work over time, which `backlog.md` is explicitly **not** ("an
inventory, not a schedule") and which a `T##` label in
`cards-unlocked-ledger.md` never is. → `engine::turns`; `backlog.md` §0.

## The action pipeline (CR 614–616)

**proposal** — a `GameAction` handed to `execute_action` or `execute_actions`,
before anything has mutated. The pipeline reads the proposal, not the event,
which is why a direct write is invisible to CR 614 no matter how loudly it is
emitted. → `CLAUDE.md`, the chokepoint invariant.

**batch** — the proposals `execute_actions` decides against one board and then
performs, CR 704.3's single event. A loop is not a batch: 704.7's collapse and
615.7's allocation are unreachable from one. → `replacement-architecture.md` §2.

**chokepoint** — `perform_action`'s own match arms, the only place observable
game state is mutated. Everything else proposes. → `CLAUDE.md`.

**performer** / **emitter** — one function performs, a different one announces.
`move_object` moves and says nothing; `announce_zone_change` is the only emitter
of `GameEvent::ZoneChange`, and each of its callers performed the move it
announces. Performers are loud about failure; callers check legality.

**subject** — the one object or player an event is *about*, read by `subject_of`
and carried as `EventSubject`. A damage event has a subject and a source, and
they are not the same field.

**subject group** — the members of one batch that share a subject — two
blockers' damage to one attacker. CR 616.1's unit is the group, not the member:
one pass of the rule, one chooser, one applied set — so an effect applies to the
pair once, not once each. → `engine/actions.rs`'s phase-1 comment;
`replacement-architecture.md` §9.

**member** — `Member`: one proposal inside a batch, as the replacement pipeline
works through it — the proposal's index into the batch, plus its event as
rewritten so far. The event becomes `None` when a replacement drops the proposal
outright (CR 614.6, 614.7a, 614.17); the member itself stays, so the index it
carries stays valid.

**candidate** — `Candidate`: one applicable effect in a group's iteration, with
the members it applies to. A candidate is not yet applied; `must_choose_among`
narrows candidates to the ladder's first non-empty step.

**instance** — `ReplacementInstance`, keyed by `ReplacementInstanceId`: one
replacement *effect*, which is the granularity CR 614.5's "affects an event only
once" is about. Not the object that generated it and not the card. →
`engine/replacement/instance.rs`.

**applied set** — the `ReplacementInstanceId`s already applied to this group,
CR 614.5's memory for one event. Declining an optional effect is tracked
**separately**: CR 903.9b is exempt *and* optional, so a decline recorded in the
applied set is a hang. A rider re-enters with a fresh applied set; a
**decomposition** inherits this one — see **lineage**.

**ladder** — CR 616.1a–e's ordered classes, walked top-down: the first non-empty
step decides the whole question and everything below it is not a choice the
player has yet. `ReplacementClass` derives `Ord` in the rule's order, so the
minimum present class *is* the first non-empty step.

**outer event** / **inner event** — a pair, and only where the CR names both.
CR 121.2a's "instruction to draw multiple cards" is the outer, `DrawCards`; the
individual draw it is carried out as is the inner, `DrawCard`. Different cards
watch each — Alms Collector the instruction, Thought Reflection the draw — and
the rule that keeps them apart is the printed one: *count how many times the
word "draw" is used*. CR 701.8's destruction and the graveyard move it proposes
are the same shape and the older instance. Not a general layer of the design:
an outer exists when a rule gives the instruction its own replaceable identity,
and `Primitive::Mill` deliberately has none. → `engine::actions::GameAction`.

**lineage** — which CR 614.5 applied set a proposed event starts from. An event
derived from another either **continues** its parent's set or starts a fresh
one, and §3.2d's discriminator is whether the derived event is *the same kind of
thing as its parent*. Answered at the call — `perform_action`'s `lineage`
argument, threaded by `execute_actions_decomposing` — never inferred from the
event. → `replacement-architecture.md` §3.2d.

**decomposition** — one event expressed at finer grain, which **continues** the
lineage: CR 121.2's "draw N" carried out as N individual draws. The engine's
only instance, and the reason there is one is that CR 121.2 says draws happen
one at a time and no other rule says it of its own plural. Without the
continuation, Teferi's Ageless Insight re-applies to its own output and the
recursion does not stop. → `GameState::execute_actions_decomposing`.

**containment** — a *different* event that a performed event caused, which
starts a **fresh** lineage: CR 120.3a's life loss inside damage, an entry caused
by a token creation, anything a **rider** proposes. The common case, and what
plain `execute_action` gives. The card-authoring rule that follows: whatever a
printed "instead X and Y" needs CR 614.5 to cover belongs in the rewrite, not in
the rider — §3.2d, corrected twice.

**rider** — CR 615.5's "and" clause on a replacement effect — Reverse Damage's
"you gain life equal to the damage prevented this way". `Rider`. Riders resolve
after the performed event, never mid-loop, and are unconditional once queued
(CR 615.12).

**bucket** — a recipient slot in an `allocate` decision: one blocker in a
damage assignment, one color in a generic mana payment, one member across which
a 615.7 shield's remaining amount is split. The CR never uses the word, which is
why it was retired from the choice ladder — see **step**.

**frame** — a computed `EffectiveCharacteristics` snapshot of one object. Two
scopes, one meaning: the layer walk's frame for an object on the board, cached
per `layer_epoch` under a descending layer **ceiling**; and `EntryFrame`, CR
614.12's look-ahead — the object *as it would exist* on the battlefield, built
at most once per pipeline iteration. → `engine/layers/board.rs`,
`engine/replacement/lookahead.rs`.

## The layer system (CR 613)

**walk** — one run of `compute_characteristics` over CR 613's layers for one
object, producing that object's frame. A *walk input* is anything that would
change its answer, which is what bumps the epoch.

**epoch** — `GameState`'s `layer_epoch`, bumped by anything that changes a walk
input. A frame computed at the current epoch is a hit; anything older is
recomputed. It is the cache key, not a clock.

**memo** — the frame cache itself, and what `fuzz_games`' `memo_hits` counts.
One walk fills every member's frame at its epoch, so the next object asked is a
hit. → `engine/layers/compute.rs`.

**ceiling** — the layer a frame was computed *up to*, and the termination
argument for `compute_characteristics` re-checking ability existence at every
layer: each re-check reads a strictly lower ceiling, so the recursion descends
and cannot oscillate. Not an optimization. → `layers-architecture.md` §5.2.

**host** — the object an Aura or Equipment is attached to; as an `AffectedSet`
arm, `AffectedSet::Host` resolves `attached_to` during the walk. Named `Host`
rather than `AttachedTo`, whose longer form read as "the things attached to me"
— the wrong direction — and because "host" is what every attach site already
called it. → `layers-architecture.md` §13a decision 4.

**donor** — in a copy effect, the object whose copiable values are copied *from*
— Mirrorweave's target, Cytoshape's chosen permanent. The word exists because
`exclude_donor` needed a noun for the thing a copy must not also be applied to.
→ `copy-effects-architecture.md`.

## The turn structure (CR 500, 614.10)

**drainer** / **cursor** — `advance_turn` is a *drainer*: it walks CR 500.1's
sequence proposing each unit and stops when one begins, because CR 614.1b makes
"skip" a replacement effect and the next unit in the sequence is not necessarily
the one that happens. Its *cursor* is the last unit **considered**, which is not
the last that happened — CR 500.11's "proceed past it as though it didn't
exist" is the whole difference, and a skipped phase advances the cursor while
beginning nothing. → `engine::turns::drain`; `replacement-architecture.md` §9's
RE-1.

**position** — where the drainer *stops*: a step, or a main phase, which has
none. Not a synonym for unit — a phase with steps is a unit and never a
position, since it is entered together with its first step. The word is load-
bearing in fixtures, which count positions to walk a turn: with no attackers
a turn is **ten** of them, because CR 508.8 refuses three combat steps. →
`engine::turns`.

## Words that are not about one subsystem

**arm** — a variant of a closed enum, and by extension the `match` arm that
handles it. `Rewrite` and `EventPattern` grow by arms, each needing the CR rule
that permits it; **an arm the pipeline cannot apply is worse than a missing
one**, which is why `RetargetSpec` shipped three and not four. →
`replacement-architecture.md` §3.2a.

**leg** — one of the alternative routes a gate has to cover. Discovery off an
object's effective ability list has three — printed, granted, copied — and a
gate that grows a new route without growing a leg on every gate is dead code
that looks live.

**sweep** — an iteration over a zone, usually the battlefield, performed for one
rule. Routing a sweep through the chokepoint makes its order observable, so it
needs `battlefield_ids_ordered` even where the old direct-write loop did not.

**atom** — one atomic test in the spec corpus: a single checkable claim read off
the CR, with an id like `ATOM-614.9-001`. Tests claim atoms with `// COVERS:` at
write time, and a phase does not close until `specdb.py`'s `owed` is clean for
it. → `engineering-practices.md` §5.
