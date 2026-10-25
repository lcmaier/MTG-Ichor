# Handoff — `concept-census-and-register`, the design note

**Opened 2026-10-24**, A6e's cleanup PR 1d (`item-6-audit.md`'s cleanup
plan, row 1d: `item-6-audit.md#the-cleanups-plan-session-6-2026-10-23`).
This file is the design note, reviewed before the build; the build runs on
the same branch and the PR that lands it deletes this file. Every count
below was read from the tree at `e4779a07` (#286's merge) on 2026-10-24 by
the pilot script in §1, unless it says otherwise.

## 0. The problem

The item 6 audit's model pass asked whether a fact is *computed* twice,
which is a question about functions. Nobody asked whether a CR concept is
*held* in several places, which is a question about data. That is how CR
113.7a's last known information came to live in about twenty fields, each
kept by the reader that needed it for as long as it needed it, until #285
gave it one store keyed by existence. The rule that came out of it is
`engineering-practices.md#key-a-concept-before-storing-it`: ask what the CR
identifies a concept by, and who owns it, before deciding where it lives.

1d asks that question of the whole engine, once, and then keeps it asked:

- **The census.** Every CR concept that names a thing in a game, its key,
  its one owner, and every type and field that holds it. A concept with two
  owners is a finding, and goes to the PR whose types it touches.
- **The register.** The census's result, kept as a list a gate reads: a new
  field that holds an owned concept fails until the register has a line for
  it.

It extends the type-surface method (`cr-coverage-audit.md#2-the-method--the-type-surface`),
which asks of a type what the CR can require that it cannot say, and whose
gate, `check_type_surface.py`, reads only `GameState`'s own fields
(`cr-coverage-audit.md#4b-gamestates-fields-and-where-each-was-read`).

**Words this note uses.**

- A **concept** is something the CR names as a thing in a game: an object's
  existence, its owner, its controller, its last known information, a
  timestamp, a target, a paid cost, an attachment.
- Its **key** is what the CR identifies one instance of it by. Last known
  information is one per *existence* of an object (CR 400.7), not one per
  object id; a target is one per (spell or ability, instance of "target").
- A **holder** is a struct field or an enum variant's payload.
- A holder's **role** for its concept is one of:
  - *owner*: the one place the concept's value is held for its key;
  - *key*: it holds only the key, an id that points at the owner. A
    handle to the owner's own immutable value (an `Arc` clone of it) counts
    as a key. A key holder must hold the CR's key: an `ObjectId` where the
    concept is keyed by existence (an `ObjectRef`) is a **narrow key**, a
    finding of its own;
  - *copy*: it holds the value again, for the same key. A copy is a
    finding unless the register keeps it with a reason;
  - *moment*: a frame of the concept at a moment the CR names, such as CR
    603.10's "immediately prior to the event", which is a concept of its own
    with its own key;
  - *none*: no CR concept (an id allocator, an index, a cache, a log line).
- **The state types** are the types `GameState` holds, transitively, not
  counting the types that spell out what a card's text says (`CardData`,
  `AbilityDef`, `Effect`, `ObjectFilter` and the rest of that vocabulary).
  Card text aside, a clone of the game holds exactly these.

## 1. What the tree holds

A pilot script (in the scratchpad, to become
`plans/references/concept-census.py`) lists every struct field and enum
payload outside test code, with its type and doc comment, using
`rust_source.py`'s reader. It finds **300 structs, 144 enums with payloads,
1,196 fields and 1,020 payloads**: 2,216 holders, close to the brief's
regex counts. Split by where they sit:

| Scope | Holders | What a reader does with them |
|---|---:|---|
| The state types (137 types) | 785 | read each: this is where a second owner costs something |
| `GameAction`, the proposals | 63 | read each: a proposal carries values through the pipeline, as last known information's `source_lki` did |
| Card text (`CardData` and its vocabulary) | 468 | class by type: what a card says, not a fact about a game |
| Working values (views, dispatch frames, plans) | 588 | class by type; read only one that outlives a decision |
| Tools (`bin/`, `ui/`, `scenario/`, `cards/`) | 312 | class by directory |

Classing the working values by type rests on a rule `CLAUDE.md` already
states: a decision site may not hold outcome-bearing state off `GameState`,
so a working value cannot be a second owner across a decision. 306 of the 785 state holders
have no doc comment, so about four in ten need their source read.

### The calibration

`cr-coverage-audit.md#3-calibration--run-the-question-against-what-you-already-know`
sets the bar: a method that cannot rediscover a finding already known is the
wrong method. Run against two:

- **Item 318, last known information.** At `0e599b93`, #284's merge and the
  tree #285 started from, a query for holders whose type or name says `Lki`
  lists 35. Among them are every keeper's list #285 removed:
  `PendingTrigger.kept_lki`, `StackEntry.kept_lki`,
  `ResolvingObject.kept_lki`, `DelayedTrigger.source_lki`,
  `MatchedTrigger.source_lki`, `HistoryRow.last_known`,
  `PaidCosts.source_lki` and `Unattached`'s two frames. On `e4779a07` the
  same query lists 23. The query finds the holders; reading them is what
  finds that they are one concept keyed by existence.
- **Item 77, the timestamp's copy.** The `Timestamp` query lists
  `PermanentState.timestamp`, whose doc comment says it copies
  `GameObject.timestamp` for the ordered sweeps and names item 77 as what
  deletes it. A known copy, found and already owned.

### What the pilot read found

The 191 holders of `events/event.rs` and `state/battlefield.rs` were read
twice to price decision 2: once in this session, and once by a single agent
that did not see that reading. These are inputs to the census, not its
verdicts. Both readings found these:

- **Attachment is held both ways.** `PermanentState.attached_to` and
  `attached_by` are one relation (CR 301.5, 303.4), written by every attach
  and unattach, which is `engineering-practices.md#one-record-never-two-kept-in-step`'s
  shape. So are `AttackingInfo.blocked_by` and `BlockingInfo.blocking`.
- **A permanent's controller has four holders, each a different key.**
  `EffectiveCharacteristics.controller` is Layer 2's output (CR 613.1b);
  `PermanentState.controller` is who put it onto the battlefield (CR 110.2);
  `announced_controller` is the "from" of CR 603.10d's next change;
  `frame_before_control_change` is a moment. The register says why there
  are four, so a fifth is a question.
- **The owner is copied onto records.** `ZoneChange`, `LeftTheGame`,
  `SameZoneMove` and `TokenCreated` each carry `owner`, which never changes
  (CR 108.3), and a departure's `lki` holds it too. Whether a copy of a fact
  that cannot change is a finding is decision 6's rule.
- **CR 400.7d's facts about how a permanent was cast** are on
  `PermanentState` (`cast`, `cost_choices`, `x_value`), on the `StackEntry`
  they came from, and in a departed spell's last known information
  (`LkiBody.entry`). Which is the owner once the spell has resolved is the
  census's question.

The agent's reading found more, and these survive a check:

- **Records name an object by a narrow key.** Most `GameEvent` payloads carry
  a bare `ObjectId`, while the CR keys the object by existence. `ZoneChange`
  carries the new existence's epoch and no key for the departing existence
  unless its frame is present. `DelayedTrigger.source_left_at` exists to
  work around exactly this: its doc says "a departure record names the
  object's id and not which existence of it moved".
- **A record restates its own frame.** `SpellCountered.controller` is the
  controller inside the same record's `lki`, and `ControlChanged.from` is
  too.
- **Three concepts the draft list lacked:** the replacement effects that
  applied to an event (CR 614.5), "continuously controlled since the most
  recent turn began" (CR 302.6, `entered_battlefield_turn` and
  `controller_since_turn`), and deathtouch damage since the last check (CR
  704.5h, `damaged_by_deathtouch`).

One claim did not survive: that a departure's frame is held twice, by its
record and by the LKI store. `GameState::depart` writes an `Arc` clone of the
one `Lki` to the store and hands the record the same value, which cannot
disagree with itself, so the record's handle is a key. Whether the record
should carry the departing existence and read the store instead, as
`Unattached` does, is the narrow-key finding above, not a copy.

## 2. The four candidates #285's review named

Each is confirmed or cleared by the census; here is what the tree says now.

**Survivor frames**: an object as it was immediately before an event it
survived (CR 603.10). Held on each record (`ControlChanged.lki`,
`SameZoneMove.lki`), per batch (`LookBackSnapshot.frames`, each an
`ObjectSnapshot` holding an `Lki`), and on the permanent between sweeps
(`PermanentState.frame_before_control_change`, which the next
`ControlChanged` takes). The key is (event, existence), or (batch,
existence) for a snapshot. A record holding its
own event's frame is that key's owner; the open question is the staging
field, which holds the same frame before the record exists.

**Which record ended an existence**, held by each keeper that reads it:
`DelayedTrigger.source_left_at`, `RememberedObject.left_at` and each
`Referred` entry's `left_at`, all written by one loop in
`engine/triggers/delayed.rs`. The key is the existence, which is the LKI
store's key, but the store's entries do not hold the record that ended them.
Beside them, `DelayedTrigger.owner` copies the source's owner because the
object store may lose the source (a token at CR 704.5d), which since #285
the LKI store answers for any departure it framed. This is the shape the
store replaced, one level down.

**What paid for a cost** (`codebase-state.md` item 30, "nothing records what
was spent to pay a cost"): nothing holds the objects that paid. The
mana half is owned (`StackEntry.mana_spent`, carried to `CastFacts`). A
source that paid its own cost is read through the LKI store by its
existence. Another object that paid (Altar's Reap's and Bone Splinters'
sacrificed creature, CR 601.2h) has no holder; no registered card reads it
back. So the concept has a key (the spell or ability's existence) and no
owner. It gets a register line saying so, pointed at item 30's slot,
`spell-copies`, so the first holder written is the owner.

**`ObjectInformation::Characteristics`**, the catch-all arm: an object read
with characteristics its caller already holds (`oracle/characteristics.rs`).
It is built at six sites for five moments, and every fact beyond the
characteristics is read off the live board:

| Site | Moment | The characteristics | The live facts | An `Lki` in hand? |
|---|---|---|---|---|
| `cost_determination/gather.rs` | a spell being cast (CR 601.2f) | now | now | no |
| `replacement/gather.rs`, `replacement/entry_copy.rs` | a permanent about to enter (CR 614.12) | as it would enter | its old zone's | no |
| `triggers/dispatch.rs`, `TriggerCandidateFrame::Live` | a live list already walked | now | now | no |
| `triggers/dispatch.rs`, `TriggerCandidateFrame::Before` | a survivor before a batch (CR 603.10) | before the batch | after it | **yes**: `ObjectSnapshot.lki`, dropped to its characteristics |
| `state/history_log.rs`, `RowFrame::AsTheEventLeftIt` | a history row's object | as the event left it | whenever the row is read | for a `SameZoneMove` row, **yes**, dropped in `engine/triggers/history.rs` |

The brief's first question was whether a filter on these paths reads
anything beyond characteristics. The matcher can
(`engine/object_filter.rs`'s `evaluate`, with no `Lki` given):

- `WithCounters` reads the battlefield's counters now;
- `ThisTurn` and `ThisCombat` read the object's *current* existence by its
  id, which for a row whose object has moved since is the wrong existence,
  and `Entered` and `WasCast` read its battlefield entry now;
- `PlayerRef::DefendingPlayer` and the arm's own `attacking()` and
  `defending_player()` read combat now;
- `Token` and `ByOwner` read the object store, which is safe: neither
  changes within an object's life.

`TriggerSubject::Host` answers "not attached" for the arm, which is right
for an entering permanent and wrong for a walked live list; no caller sends
one there today, and nothing stops one. No registered card's filter on these
paths reads a moment-sensitive leaf: Sphere Grid's counters and Relentless
Assault's "attacked this turn" are read live, through `Current`. So the
mismatch is not reachable by a card today; it is reachable by the CR.

## 3. Decisions

### Decision 1 — what counts as a concept, and where the list comes from

**The shared problem.** The census's claim is "every concept has one owner,
and every holder is listed". That is only as good as the two lists under it:
the concepts, and the holders.

- **(a) Top-down, from the CR's glossary.** Its 726 terms, less 197 keyword
  abilities and actions and 20 obsolete, leave about 509 to triage once:
  names a thing in a game, names a procedure, or names a mechanic. The claim
  becomes "every glossary term triaged". **It misses concepts the glossary
  does not list**: there is no entry for "controller", "life total", an
  object's existence, or CR 603.10's look-back frame. Two of those are the
  concepts this PR exists for.
- **(b) Bottom-up, from the tree's holders.** Every holder in §1's first two
  rows is classed, and the concepts are whatever the holders hold. The claim
  becomes "every holder classed". **It misses a concept with no holder**:
  "what paid" is exactly that, and so is anything the engine has not built.
- **(c) Both, reconciled.** *(Recommended.)* The concept list starts from the
  glossary's triage; the holders come from the script. A holder no concept
  claims either adds a concept, with the CR rule that names it (checked
  against `tmnt.txt` by script), or takes a *none* class with its reason.
  A concept no holder holds gets a register line saying so and where its
  owner is slotted. Then both enumerations are closed and each checks the
  other. The triage is kept as a list (`plans/references/glossary-triage.tsv`,
  each term and its class), so the next CR version's new terms are the ones
  without a class.

### Decision 2 — method and cost

**The shared problem.** About 850 holders need reading (§1's first two
rows), four in ten without a doc comment, and the item 6 discovery
workflows spent 99.4M tokens against a 3–6M estimate (#271), because
refuters scaled with findings. Each option below was piloted on the same
191-holder slice.

- **(a) A script and one reader, by concept.** *(Recommended.)* The script
  prints each concept's holders by its query (decision 4), then the holders
  no query claims. One person reads them in sequence. **Piloted:** about 30k
  tokens in this session for the slice (13k of listing, about 10k of source
  read where a doc comment did not settle a holder, the rest the
  classification). That reading was quicker than the agent's and missed the
  narrow key and the restated frames, so price it at twice that for the
  agent's thoroughness: about 270k for the 850 holders, plus about 30k for the
  glossary triage and 20k for the unclaimed list. **About 320k tokens, one
  long session or two.**
- **(b) A read-only fan-out of agents.** Needs your opt-in. **Piloted:** one
  agent read the same slice in **190,609 tokens and 20 tool calls** (about
  seven minutes), six times this session's cost, because each tool call
  re-sends its context. About 140 of the 191 holders were mechanical from
  the type alone, and it raised 36 copies. Extrapolated to 850 holders:
  about 850k tokens of finders. Item 6's discovery measured about 100k per
  refuter (#271), so one refuter per finding over about 160 findings would add
  about 16M. Deduplicating first, and refuting only what changes a
  disposition, might bring that to 3–5M. The agent read well: it found more
  than this session's pilot did, and one of its claims was wrong.
- **Either way**, a doubtful finding may get a single verifier, as the item 6
  handoff's §2 allows a fix session, and no fan-out runs. The pilot also says
  where the reading goes: a quarter of the holders are the work, and the
  script should class keys and allocators so the reader sees only them.

### Decision 3 — where the register lives

**The shared problem.** The register is the census's standing result,
read by a gate and by anyone adding a field; the census's one-time record
(the method, the triage, the calibration) needs a home beside it.

- **(a) A section of `cr-coverage-audit.md`.** *(Recommended.)* A new
  `## 4c. Concepts and their owners` after §4b, which is the same shape for
  `GameState`'s fields: the type-surface method's third application. The doc
  already owns the method, its calibration bar and the findings register, so
  a finding lands in its §5 as every other has. `CLAUDE.md`'s row for the doc
  gains three words ("and the concept register") in place, no new line. The
  doc grows from 899 lines by an estimated 350–450.
- **(b) A doc of its own**, `plans/concept-register.md`, with a pointer from
  `cr-coverage-audit.md`. Shorter files, but a second home for one method,
  and `CLAUDE.md`'s table would want a row for it, which its rule ("a new
  subsystem extends this row, never adds one") argues against even though
  the register is not a subsystem.

### Decision 4 — the gate's shape

**The shared problem.** A register no one is made to read lapses: §4's
promise that a new type joins the sweep the day it joins `GameState` lapsed
between `3c322e5` and 2026-09-24 while `GameState` went from 28 fields to
52. But `check_type_surface.py`'s docstring records your call of 2026-09-24
against a gate on the fields of the record structs: they are the most-edited
in the tree, and a gate on their fields "would fail every rename there". 1d's
gate must not bring that back.

Each register entry is a concept with a **query**, the rule that finds its
holders: a type of its own (`Lki`, `Timestamp`, `CastFacts`, `CostChoices`,
`ManaSpent`, `CopiableValues`, `CounterStack`), or, for a concept held in a
shared type, that type and a word in the field's name (`PlayerId` named
`owner`; `EventId` named `left_at`).

- **(a) Key on a concept's canonical type.** A field rename passes, since
  the type does not change. Blind to every concept held in a shared type:
  owner, controller and "which record ended it" all are, and the last is
  one of §2's four.
- **(b) Key on its vocabulary in field names.** Sees the shared-type
  concepts. A rename that drops the word takes the field out of the gate's
  view silently, one that keeps it fails if the gate checks names, and over
  the whole tree the words match views and working values: 44 engine-side
  holders are named `controller`, a dispatch's working values among them
  beside every registry row's.
- **(c) Fail only a new struct, or a new field in a struct already listed.**
  A scope rule rather than a key: it bounds what is read, but within a listed
  struct a renamed field is a new field, so it fails every rename there,
  which is the objection.
- **(d) A review checklist, no gate.** No upkeep, and the lapse above.
- **(e) (a) and (b) as one query per concept, checked per holding type, never
  per field.** *(Recommended.)* The register says, for each concept, which
  types hold it, in which role, and how many fields of each type do; it may
  name the fields for a reader, but the gate does not read the names. The
  gate runs each query over the state types and `GameAction` (decision 1's
  scope) and fails a type holding a concept the register does not list for
  it, or holding more fields of it than listed. **A field rename passes**; a
  new holder fails; a type rename fails, which is rare and reviewed anyway.
  What it does not see, said in its docstring as the current one says its
  own: a concept held in a shared type under a word its query lacks.

**Where it runs.** Recommended: `check_type_surface.py` itself, grown a
second half, since its subject is the type surface and its docstring holds
the 2026-09-24 call that this shape answers. `CLAUDE.md`'s check line and
CI's "Type surface" step then stay as they are; nothing needs swapping in.
The alternative is a new `check_concept_register.py` swapped onto the check
line beside it and given a CI step of its own.

### Decision 5 — `ObjectInformation::Characteristics`

**The shared problem.** §2's table: one arm serves five moments, the live
facts it reads are "now" for two of them and another moment for three, and
two of the three callers had an `Lki` in hand and dropped it.

- **(a) One arm, honestly named.** Rename it and say in its doc that only
  the characteristics are from the caller's moment. No code moves. A
  moment-sensitive leaf still answers for the wrong moment, silently, the
  day a card reaches one.
- **(b) A moment tag on the one arm** (#285's recommendation):
  `Characteristics(id, frame, moment)`. A reader that does not care ignores
  the tag; one that does matches it. All five moments stay on the arm, so
  the two with an `Lki` keep reading counters and combat live.
- **(c) One arm per moment.** Every match on the arm decides for every
  moment (nine matches today), though most of them do not care.
- **(d) (b), after handing the two callers that hold an `Lki` the `Lki`.**
  *(Recommended.)* `TriggerCandidateFrame::Before` carries its snapshot's
  `Lki`, and the `SameZoneMove` history row keeps its `RowFrame::LastKnown`;
  both then read through `ObjectInformation::LastKnown`, whose readers already
  answer counters, attachments and combat as of the frame (its doc already
  says a survivor's frame rides it). The tag then has three values, each one
  moment: *now* (a spell being cast, a walked live list), *entering* (CR
  614.12), and *as the event left it* (a history row), carrying the row's
  existence so `ThisTurn` reads the right one. Under *entering* and *as the
  event left it*, a leaf the frame cannot answer (counters, combat, "entered
  this turn") refuses in a debug build rather than reading now, and
  `TriggerSubject::Host` answers per tag. **Code shape:** about 150–250
  lines with tests; the two handed `Lki`s are an `Arc` each, no walk.
  **Pools:** identical predicted, since no registered card reaches a
  moment-sensitive leaf on these paths and no prompt changes; the close-out
  says so if they move.

### Decision 6 — what a record's values are (raised by the pilot)

**The shared problem.** 124 of the slice's 191 holders are `GameEvent`
payloads, and many name an owner, a controller or a frame that some other
holder also answers. The agent had to choose a rule before it could class
them, and its rule made 24 of them copies. The rule decides whether the
census's findings on records are a handful or a few dozen per hundred
payloads, so it is yours to set before the reading starts.

- **(a) A record owns what happened, as of its event.** Everything it carries
  is that event's moment, never a copy; findings come only from state outside
  the records. Fewest findings, and it misses a record that restates a value
  it already holds, as `SpellCountered` does.
- **(b) A record holds keys.** A value another holder answers for the same
  key is a copy, and the record should name the key and read the owner, as
  `Unattached` does with the store. The most findings, and each one is a
  change to a `GameEvent` payload, which every arm of the trigger matcher
  reads.
- **(c) A copy only where it can disagree.** *(Recommended.)* A record's
  value is a finding when another holder answers it for the same key at the
  same moment and the two could differ. An unchangeable fact (the owner, CR
  108.3) and a shared `Arc` handle cannot differ, so each is kept, with the
  reason in the register. A controller frozen as the event happened is that
  event's moment, since a stack object's controller can change later. What is
  left is a short list: a record restating its own frame, and every narrow
  key, which is a key finding and not a copy.

## 4. Sizing

`engineering-practices.md#sizing-a-phase`'s band counts code and tests, with
docs beside. Under the recommendations:

| Piece | Kind | Lines |
|---|---|---:|
| `plans/references/concept-census.py`: the lister, the queries, `--holders <concept>`, `--unclaimed` | tooling | 250–350 |
| `plans/references/glossary-triage.tsv`: 726 terms, each with its class | data | ~730 |
| `cr-coverage-audit.md#4c-…`, the census and the register: about 50–70 concepts | docs | 350–450 |
| Findings: `codebase-state.md` items and `**Slotted:**` lines, the cleanup plan's rows | docs | 100–200 |
| `check_type_surface.py`'s second half, and its probe | tooling | 150–250 |
| Decision 5 (d) | code and tests | 150–250 |

So about 150–250 lines in the band and about 1,600–2,200 of docs, data and
tooling beside it. If decision 5 takes (a), the PR is docs and tooling
alone, as the brief said it would be. The reading itself is decision 2's
price, about 320k tokens. The findings' fixes are not in this PR: each
goes to the PR whose types it touches.

## 5. The build, commit by commit

1. The census script and the glossary's triage.
2. The census, concept by concept, into `cr-coverage-audit.md`'s new section,
   under decision 6's rule for records, every key holder checked against
   the CR's key, and the unclaimed holders read last.
3. Each two-owner finding dispositioned: merged into `one-fact-one-type`
   (item 291's types, item 296's `Condition`), merged into
   `trigger-matcher-merges`, a new numbered item with its route slot, or kept
   with its reason in the register.
4. The gate, with a probe that adds a holder of an owned concept and shows it
   failing, and a field rename that shows it passing.
5. Decision 5's build.
6. The close-out: the 1d entry names the PR, this note goes, the board is
   regenerated, every check passes, and the pools are compared.
