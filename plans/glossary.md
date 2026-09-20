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

**member** — two, and they are one word for a reason: each names the unit its
subsystem *iterates*. **(1)** `Member`: one proposal inside a batch, as the
replacement pipeline works through it — the proposal's index into the batch,
plus its event as rewritten so far. The event becomes `None` when a replacement
drops the proposal outright (CR 614.6, 614.7a, 614.17); the member itself stays,
so the index it carries stays valid. **(2)** One object in the layer walk's
**working set** — what `Board::members` holds and what a pass computes a frame
for. Its opposite is a *non-member*, an object no row can reach, which gets
`compute_non_member`: a solo walk of its own CDAs, exact precisely because
nothing can reach it. `Membership` is the three-way answer the top-level entry
dispatches on — `Member`, `NonMember`, and `ZoneOnly` for an object in the
battlefield *zone* with no entity yet (a token mid-creation), which is a member
of the one pass that asks about it and of no other. Added at LI-1's review
(2026-09-06), renamed from `Query`, because the question is not "what is being
asked" but "is this thing in the set". → sense 1 is
`engine/replacement/pipeline.rs`; sense 2 is `engine/layers/board.rs::membership`
and `layers-architecture.md` §13b decision 2.

**instance** — two, and they never meet: each subsystem counts a different
thing. **(1)** CR 115.3's *instance* of the word "target": one clause of a
spell or ability that asks for a choice — "target creature", printed once,
however many atoms act on what was chosen for it. It is the unit CR 601.2c
announces, CR 608.2b re-checks, and CR 115.3's "the same object can be chosen
once for each instance" counts. `TargetInstance` is one with its choice;
`AbilityDef::instances` and `CardData::spell_instances` are the printed-order
list `Effect::instances` derives once, at `CardDataBuilder::build`;
`EffectRecipient::SameInstanceAs` refers back to one by its position in that
list; `DeclaredInstances` is where a resolution reads them from. Neither a
*target* (CR 115.1's word for what was chosen) nor an *atom* (an effect, which
may act on one instance twice — Act of Treason — or once each on three —
Seeds of Strength). Added at A4i (2026-09-17), numbered at A4n's review.
**(2)** `ReplacementInstance`, keyed by `ReplacementInstanceId`: one
replacement *effect*, which is the granularity CR 614.5's "affects an event
only once" is about. Not the object that generated it and not the card. →
sense 1 is `engine/targeting.rs` and `types/effects.rs`; sense 2 is
`engine/replacement/instance.rs`.

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
`codebase-state.md` item 65 renamed the class to the rule's own word, which was
right for that name and made this pair.

**blocked** — **(1)** combat: a creature a blocker blocked, CR 509.
`AttackingInfo`'s `is_blocked`. **(2)** prose only: an event a CR 614.17 "can't"
forbids, checked ahead of the replacement pipeline and winning over it
(CR 101.2). Sense 2 had a function — RB's `engine::replacement::is_blocked`,
which RS-1 deleted; the mechanism is `engine::restriction::is_prohibited` asked
with a `Query::Event`. Four pointers outlived the name, and a grep after one
lands on sense 1.

**window** — **(1)** the records one dispatch matches over: every record one
batch stamped, read at the close of the outermost `execute_actions`, or the
one record of an unbatched emission — `EventLog::records_from(mark)` filtered
by the batch's id. CR 603.2c's "one or more" is one trigger per window; CR
603.6a's "all permanents ... are checked" is why the window is the event and
not the record (`triggers-architecture.md` §4.1). **(2)** CR 601.2g's mana
ability window, `run_mana_ability_window`: the chance to activate mana
abilities while a cost is being paid — the older sense, and the one "inside
the mana window" means.

**tier** — **(1)** CR 603.3b's two-part placement, `Tier::First` and
`Tier::Second`: a trigger whose condition is another ability triggering goes
on the stack after the ones that are not, whatever APNAP says
(`TriggerCondition::tier`). **(2)** the trace practice's three tiers
(`engineering-practices.md` §7, §7.1): hand-authored pages, the engine's
sink, the codebase map.

**probe** — **(1)** one read of a gate — a hash-set `is_empty` or `contains`,
nanoseconds and no allocation — as in "a dispatch on today's pools is five
probes and nothing else" (`triggers-architecture.md` §11). **(2)** a
throwaway build or test that measures a claim before the claim is trusted —
the panic-on-gate-pass binary TR-1's A/B ran first, the thread-local counters
A4n and 7a were sized with — and is never merged.

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

**plan** — two, and the older one is not a type at all.
**(1)** a *payment* plan — `plan_payment`, `pay_with_plan`, `plan_and_pay` and
`planned_sacrifices`: the cost system working out what to tap and sacrifice
before it does either (CR 601.2f–h). A local idiom, and a verb. **(2)** a
**turn** plan — `GameState.turn_plan`, `TurnPlan`, `PlannedPhase`: CR 500.1's
phase sequence as data the drainer indexes, so CR 500.8 can splice into it. A
noun, and a stored fact. The collision was caught by RE-1's glossary pass
*before* sense 2 was written rather than mid-PR, which is what this gate is
for — `planned_sacrifices` sits one letter from `PlannedPhase`. Sense 2 keeps
the name because it is the one `state::game_state::next_phase`'s own pre-RE-1
TODO used. → `replacement-architecture.md` §9's RE-10; `cost-architecture.md`.

**splice** — **(1)** CR 702.47's keyword ability: adding a card's rules text to
an Arcane spell as it is cast (CR 612.10). **Unimplemented, and the name is
taken by sense 2 before the mechanic arrives**, which is the collision this
entry exists to record. **(2)** inserting phases into `GameState.turn_plan` at
the cursor — CR 500.8, `Primitive::ExtraPhases`, literally `Vec::splice`. Sense
2 keeps the word because it is the standard-library method's, and the CR's own
sense is a keyword ability that will be spelled `KeywordFlag` when it lands and
so cannot be confused with a call. **Found by `--suggest` only because the
prose says "splices"**: the filter drops words the CR uses and the CR uses the
singular, which is one thing that report cannot see. →
`replacement-architecture.md` §9's RE-10.

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

**cell** — one pair of commutation classes in `pipeline::commutes`, the table
`ordering_cannot_change_outcome` reads: two members whose applications reach
one outcome in either order, on the counter kinds both touch. A pair with no
cell is asked (CR 616.1). "The exit cell", "the substitute cell" name the
class a chosen member fell in, which is what `check_order_invariance`
dispatches on. → `backlog.md` §2.29, graduated at RE-5's review.

**customer** — what a facility exists for. The *customer* of an arm, a field
or a rule-stated facility is the CR rule that states it; a *printed customer*
is a card in print that exercises it, and it is the facility's **test**, not
the reason it is owed — a facility with none gets a fixture test and a
reachability line that says "no printed producer" (`engineering-practices.md`
§4, adopted at RE-5's review). "An arm the pipeline cannot apply is worse
than a missing one" is about an arm with no customer of either kind.

**putter** — the player putting counters on an object or player: CR 122.6a's
"which player puts those counters on it", carried as `AddCounters::by` and
asked by `EventPattern::AddCounters`'s `by`. For counters a permanent enters
with it is the player the effect named, else the rule's default, the
controller it enters under (`EntryCounters::putter`). Vorinclex, Monstrous
Raider is the reader; a removal has none.

**subject group** — the members of one batch that share a subject — two
blockers' damage to one attacker. CR 616.1's unit is the group, not the member:
one pass of the rule, one chooser, one applied set — so an effect applies to the
pair once, not once each. → `engine/actions.rs`'s phase-1 comment;
`replacement-architecture.md` §9.

**candidate** — `Candidate`: one applicable effect in a group's iteration, with
the members it applies to. A candidate is not yet applied; `must_choose_among`
narrows candidates to the ladder's first non-empty step.

**def** — a *definition*: the data a card author writes and the engine
reads — `TokenDef`, `ReplacementDef`, `RestrictionDef`, `AbilityDef`. Not
an **instance** (sense 2: a def on a board, with a source and a controller) and not
an object; a `TokenDef` in particular is the description a token is created
*from*, which is why a creation's kind is asked of the def and never of an
object (`EventPattern::CreateTokens`). → `types/effects.rs`,
`types/replacement.rs`.

**applied set** — the `ReplacementInstanceId`s already applied to this group,
CR 614.5's memory for one event. Declining an optional effect is tracked
**separately**: CR 903.9b is exempt *and* optional, so a decline recorded in the
applied set is a hang. A **decomposition** inherits this set and so does a
**rider** — the rest of a replacement's effect is "a modified event that may
replace that event" — while a *contained* event starts a fresh one. See
**lineage**; a rider given a fresh set was RE-4's loop.

**retype** — to change the type of every unit a mana production carries and
nothing else about it: CR 106.12b's "tapped for mana of a specific type"
replacement, `GameActionTemplate::ProduceMana` with `ReplacedAmount`, and
`pipeline::substitute`'s leg for it. Deep Water's "produces {U} instead of any
other type" retypes; Contamination's "instead of any other type *and amount*"
does more than retype, and is the same template with a `Fixed` amount. A
retyped restricted unit keeps its restriction (CR 106.6).

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
one, and §3.2d's discriminator is whether the derived event is *the original
in modified form* — a decomposition is, a rider's proposals are, an event the
replacement merely *causes* to be nested (an entry inside a creation) is not.
Answered at the call — `perform_action`'s `lineage` argument, threaded by
`execute_actions_decomposing`; `Rider::lineage`, handed to a rider's proposals
by `resolve_rider` — never inferred from the event. → `replacement-architecture.md`
§3.2d, §4.1a.

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

**acid test** — the one board in a phase whose failure mode is a **hang or a
crash rather than a wrong number**, written first and named in full so it cannot
be quietly deleted. Two Thought Reflections (RE-2) and the CR 614.5 lineage are
the original: get the applied set wrong and the engine does not answer badly, it
recurses until the stack ends. So the test carries a derived bound as well as an
assertion — `execute_actions_decomposing`'s depth check — because a test that
cannot fail is not a test. **Not a synonym for "the important test"**: a board
that fails as a wrong number, or as an unexpected prompt, is an ordinary
regression however much the phase turned on it.

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

**working set** — written **W** where the docs reason about it. The objects
one layer pass computes frames for,
and the unit `Board::members` holds. Not "everything in the game" and not "the
battlefield": it is *every object some registered row can reach*, derived from
the rows rather than listed — the battlefield in CR 613.7 timestamp order,
then the entering object, then whatever a `Fixed` row names, then (since LJ)
the objects in the zones `RegistryScopeSummary::reachable_zones` names. An
object outside it is a **non-member** and gets its printed characteristics plus
its own CDAs, which is exact precisely because no row can reach it.
→ `engine/layers/board.rs::Board::seed`, `layers-architecture.md` §13b
decision 2 and §13c.

**seed** — two related things, and the walk does both at layer 0. To *seed the
working set* is `Board::seed`: decide which objects the pass computes frames
for at all, before any layer runs (see **working set**). To *seed a frame* is
`seed_frame`: fill one object's frame with its **printed** characteristics —
the card's own types, colors, P/T, abilities and keyword flags — which is what
every layer then modifies. The noun is the value before any effect has touched
it, which is why "the seed" and "layer 0" name the same moment.
→ `engine/layers/board.rs`, `layers-architecture.md` §13b decision 2.

**ceiling** — the layer a frame was computed *up to*, and the termination
argument for `compute_characteristics` re-checking ability existence at every
layer: each re-check reads a strictly lower ceiling, so the recursion descends
and cannot oscillate. Not an optimization. → `layers-architecture.md` §5.2.

**host** — the object an Aura or Equipment is attached to; as an `ObjectSet`
arm, `ObjectSet::Host` resolves `attached_to` during the walk. Named `Host`
rather than `AttachedTo`, whose longer form read as "the things attached to me"
— the wrong direction — and because "host" is what every attach site already
called it. → `layers-architecture.md` §13a decision 4.

**reach** — the zones a row's `ObjectSet` can name an object in, as a
`ZoneSet` read off the row without touching the board. The union over the
registry is `RegistryScopeSummary::reachable_zones`, and what makes it a union
of *syntax* rather than a search is why the zone lives on `ObjectSet::Filter`
and not as an `ObjectFilter` leaf. → `types/zones.rs`, `layers-architecture.md`
§13c decision 3.

**donor** — in a copy effect, the object whose copiable values are copied *from*
— Mirrorweave's target, Cytoshape's chosen permanent. The word exists because
`exclude_donor` needed a noun for the thing a copy must not also be applied to.
→ `copy-effects-architecture.md`.

## The turn structure (CR 500, 614.10)

**drain** — the **verb**, and `engine::turns::drain` is the loop that does it:
propose the next unit of turn structure, and if it does not begin, propose the
one after it, until one does. Named for what it consumes rather than for what
it produces — each pass spends something that cannot be spent twice (a queued
extra turn, a place in the rotation, a plan entry), which is also the
termination argument. One call to `advance_turn` is one drain and ends at one
**position**. → `engine::turns::drain`.

**drainer** — the **thing** that drains: `advance_turn`, as opposed to a step
function. The distinction is CR 614.1b's — "skip" is a replacement effect, so
the next unit in CR 500.1's sequence is not necessarily the one that happens,
and a function that returned "the next step" would be answering a question the
rules do not have. → `replacement-architecture.md` §9's RE-1.

**cursor** — the drainer's place in the sequence: the last unit **considered**,
which is not the last that happened. CR 500.11's "proceed past it as though it
didn't exist" is the whole difference — a skipped phase advances the cursor and
begins nothing. Since RE-10 it is an **index** into `GameState.turn_plan` and
not a phase *type*, because CR 500.8 lets one turn hold two combat phases and a
type cannot say which of them is meant. Distinct from the **position**
(`GameState.phase`), which is where the drainer *stopped*: the two agree at a
drain boundary and diverge inside one, which is why
`GameState::set_turn_position` writes both and `advance_turn` asserts they
still match. → `replacement-architecture.md` §9's RE-1 and RE-10.

**position** — where the drainer *stops*: a step, or a main phase, which has
none. Not a synonym for unit — a phase with steps is a unit and never a
position, since it is entered together with its first step. The word is load-
bearing in fixtures, which count positions to walk a turn: with no attackers
a turn is **ten** of them, because CR 508.8 refuses three combat steps. →
`engine::turns`.

## Words that are not about one subsystem

**spelled** — how a fact is *written* in the tree, as opposed to what it means.
"CR 113.6 was spelled as which function calls it" says the rule was real and
enforced and had no name: `register_static_effects` ran only from
`place_on_battlefield`, so "a static ability functions on the battlefield" was
true by construction and appeared nowhere as a statement. The word is doing
work a reviewer needs, because the two are not the same defect — a rule spelled
somewhere odd is *correct and unfindable*, where a missing rule is wrong. Most
of this project's refactors are re-spellings: the behaviour is already right
and the change is where a reader would look for it.

**dispatch** — the matcher's run over one window: the candidates behind the
gate, each triggered def against each record, the queue written, `AbilityTriggered`
emitted per queued trigger, a mana trigger resolved at once (CR 605.4a). The
first of CR 603's two instants; placement is the second. → `engine::triggers::dispatch`.

**binding** — what a pending or stacked trigger remembers about its event:
the def, the records that matched (by `EventSeq`, never by copy), which arm
matched, and the subject's `ObjectRef` (id and epoch). "That creature", "that
player" and "that many" are read back through the arm's projections at
resolution, so there is one copy of every fact and the frame comes with the
record. → `TriggerBinding`, `engine::triggers::binding`.

**elision** — the engine declining to ask a prompt whose every legal answer
leaves the same game, with the conditions under which that stops being true
written beside it. The project's word, from `cost-architecture.md` §3.4 and
`codebase-state.md` item 47 — `pipeline::ordering_cannot_change_outcome`
skips CR 616.1's prompt when no order can change the outcome — and not
Rust's, where it names the compiler filling in omitted lifetimes. Two things
it is not: a *forced* prompt, which has one legal answer and which `ui::ask`
declines under the same rule (A4e); and a decorator *answering* such a
prompt, which is a second home for one fact (`backlog.md` §2.22, rule 1).
An elision always carries its expiry conditions, because the theorem that
licenses it is about the symbols the engine pays or the fields a filter
reads today, and the phase that widens either owes the prompt back.

**quadrant** — one of the four buckets CR 702's 189 keyword abilities fall
into, on two axes: does the engine **branch** on the keyword or **execute** it,
and does it take a **parameter**? The map decides what a keyword *is* in this
codebase, and the four answers are four different types:

| | no parameter | parameter |
|---|---|---|
| **branch** | ① a `KeywordFlag` — flying, trample, vigilance | ② a set of *values* on the frame — protection from [quality], [type]walk |
| **execute** | ③ a plain `AbilityDef` — storm, prowess, devoid | ④ an `AbilityDef` with arguments — equip [cost], ward [cost], cycling [cost] |

Quadrant ① is `types::keywords::KeywordFlag`, 16 variants, every one consumed
by combat, SBA, casting or damage; it is a **characteristic**, so `seed_frame`
seeds it from the card and Layer 6 writes it through
`EffectModification::GrantKeywordFlag`. Quadrants ③ and ④ are ordinary
abilities on `CardData::abilities`, granted through
`EffectModification::GrantAbility`. **Quadrant ② has no representation yet** —
it lands with the first card that needs one.

Why the split is load-bearing rather than taxonomy: a fieldless variant cannot
hold what a ②/④ keyword is *made of* (CR 702.6d lets one permanent have
several equip abilities at different costs), and a quadrant-① keyword is not
an `AbilityDef`, so **anything that reasons over abilities does not see it** —
which is `codebase-state.md` item 133, CR 113.6 not reaching a graveyard card's
printed flying. → `types/keywords.rs`, whose doc comment has the per-keyword
detail and the five variants that were removed; `codebase-state.md` "Before
Layers" item 10.

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

**departed** / **departing** — a player who has left the game (CR 104.5), and
the objects and effects that were theirs. The CR's own phrase is "a player who
has left the game"; this is the adjective form, and it is the word `fuzz_games`
already prints in "Departed-owned permanents". **It says nothing about *why***
— every loss is a departure, and so is a concession — and it is scoped by
CR 800.1: at two seats a departure ends the game, so nothing is ever *about* a
departed player there. → `engine/leaving.rs`; `GameState::is_multiplayer`.

**atom** — one atomic test in the spec corpus: a single checkable claim read off
the CR, with an id like `ATOM-614.9-001`. Tests claim atoms with `// COVERS:` at
write time, and a phase does not close until `specdb.py`'s `owed` is clean for
it. → `engineering-practices.md` §5.

**trace** — one game's records from the trace sink, a JSON object per line,
and the file that holds them; a *trace page* is tier 1's hand-authored walk
and a trace is what tier 2 writes. The two meet at the spine. →
`engineering-practices.md` §7, §7.1; `state::trace`.

**sink** — the trace sink, `TraceSink`: the one writer every emit point's
record goes to, shared by every branch of a game and off by default. An
observer and never a participant — it draws from no rng, asks no provider and
changes no control flow, and the check is `IDENTICAL` counters with it off
and on. → `state::trace`'s module doc.

**emit point** — a site where the engine hands the sink a record: the batch
(`execute_batch_inner`), the CR 616.1 iteration (`apply_replacements`), the
layer walk (`compute_characteristics` and its entering and LKI forms), the
decision boundary (the four `validate_*` helpers, and the priority loop's
rejection) and the performed event (`emit_event`). Not an *emitter*: an
emitter announces a performed event to the log and there is one per event
(`announce_zone_change`); an emit point observes and announces nothing. →
`state::trace`.

**spine** — a trace page's step rows, one per read, each labelled by what it
consulted: what the sink generates and `plans/trace_spine.py` renders. Not the
page — the map's questions, the comparison table and the closing section are
the *argument*, and stay authored. → `engineering-practices.md` §7.1.

**branch** — of a trace, which fork of a game a record was written by, the
number every record carries beside `seq`. `TraceHandle`'s hand-written `Clone`
is the fork marker: a `GameState` clone takes a fresh branch and writes one
`fork` record naming both. Not *lineage*, which is which CR 614.5 applied set
a proposed event starts from, and not the verb the quadrant map uses for what
the engine does with a keyword. → `state::trace`.
