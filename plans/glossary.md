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

**It is checked.** `python plans/check_glossary.py` asserts three things:

1. every term defined here, and every code anchor an entry names, still resolves
   in `mtgsim/src` — so a rename fails CI in the commit that renames;
2. every word on the script's watch-list is defined here — so a new coinage
   cannot land undefined;
3. a word the script lists as polysemous carries **all** of its senses here,
   numbered. That is the assertion with a scar: `Rewrite::Retarget`'s arms
   reached the build as `ToSource` and `ToSourceController` — two *different*
   sources, adjacent in one enum — and both had to be renamed mid-PR.

## Words that mean more than one thing

**source** — **(1)** CR 609.7's source of damage or of an effect: the object
that dealt or produced it, carried by `GameAction::DealDamage` and matched by
`EventPattern`. **(2)** The object whose ability an effect *is* —
`ReplacementInstance`'s `source` field, `RegisteredReplacementEffect`'s,
`ContinuousEffect`'s. Also CR 609.7, and in a damage event it is a different
object from sense 1. **(3)** A *gather source*: one of the numbered routes a
discovery pass looks down, not an object at all. → senses 1 and 2 are adjacent
in `RetargetSpec`, which is why its arms are `ToEffectSource` and
`ToDamageSourceController`; sense 3 is `replacement-architecture.md` §3.3's five
and `cost-architecture.md` §4's two.

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
harness plays. `PERFORMANCE_POOL` is the frozen set every recorded baseline was
measured on; `--pool stress` is every registered card. →
`engineering-practices.md` §3.

**step** — **(1)** CR 500's turn step: `StepType`, an untap step, a combat
damage step. **(2)** a rung of CR 616.1a–e's choice ladder — `ReplacementClass`,
returned by `must_choose_among`. Sense 2 is the newest collision in this file:
it arrived when `codebase-state.md` item 65 renamed `forced_bucket` to the
rule's own word, which was right for that name and made this pair.

**blocked** — **(1)** combat: a creature a blocker blocked, CR 509.
`AttackingInfo`'s `is_blocked`. **(2)** prose only: an event a CR 614.17 "can't"
forbids, checked ahead of the replacement pipeline. There is no function by that
name — the mechanism is `is_prohibited` with a `Query::Event`. `CLAUDE.md` and
two doc comments still point at `engine::replacement::is_blocked`, which has
never existed; `codebase-state.md` item 110.

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
one loop, one chooser, one applied set, so an effect applies to the pair once.
→ `engine/actions.rs`'s phase-1 comment; `replacement-architecture.md` §9.

**member** — `Member`: one proposal inside a batch as the loop carries it — its
index and its event as decided so far, `None` once dropped (CR 614.6, 614.7a).

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
applied set is a hang. A rider re-enters with a fresh applied set.

**ladder** — CR 616.1a–e's ordered classes, walked top-down: the first non-empty
step decides the whole question and everything below it is not a choice the
player has yet. `ReplacementClass` derives `Ord` in the rule's order, so the
minimum present class *is* the first non-empty step.

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
