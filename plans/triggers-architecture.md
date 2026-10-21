# Triggered Abilities — CR 603

> **Status:** design, authored 2026-09-18 — critical-path item 6's architecture
> doc, `roadmap-v2.md` row A6 step 2. Written after the trigger survey
> (`plans/references/trigger-survey.md`, PR #171) and before a line of the
> phase. No code exists for any of it; **the owner reviews this before TR-1
> starts.**
> **Authority:** type shapes, the dispatch and placement algorithms, the
> trackers, the LKI reader, the delayed-trigger registry, and the phase
> sequencing for CR 603, with the rules that hand a trigger its moment —
> 117.2a/117.5, 405.3, 500.6, 502.4/503.1a, 508.2b/509.2a/510.3a, 513.2,
> 605.1b/605.4a, 608.2a, 702.131d, 704.3's trigger half, 800.4d's second
> sentence — and CR 603.10's look-back, which the LKI reader serves. Where
> this contradicts `codebase-state.md`, that file wins on *what exists*; this
> file wins on *what is being built*. `CLAUDE.md` → "Critical path to v1"
> owns the ordering of this phase against the others.
> **Companions:** `replacement-architecture.md` §2 is the spine this reads
> (a `GameEvent` is a performed record, emitted from inside the chokepoint,
> post-replacement truth); `layers-architecture.md` §13b decision 5 is the
> `Condition` this reuses; `state-tracking-architecture.md`'s resolution
> postscript is the owner's decision on the substrate, which this document
> builds on and does not reopen; TR-7's persist loop is the integration
> test (§12), and `cost-architecture.md` §3.11's Ironworks loop moved to
> `roadmap-v2.md` B13 on 2026-10-13.

---

## 0. The budget — why this stays tight

Written in answer to the question the replacement doc's §0 was written for:
*what stops this sprawling once 14,149 paper cards carry a trigger?* Five
commitments, each falsifiable, and the first two already tested against the
card pool by the survey.

**One. A card is data, not code.** A triggered ability is an `AbilityDef`
whose effect is `Effect::Triggered(Box<TriggerDef>)` — a condition, an
optional intervening "if", an optional once-per-turn limit, and the effect —
the same arm-in-the-tree shape `Effect::Replacement`, `Effect::Restriction`
and `Effect::CostModification` already use for the other ability kinds whose
body is a definition. Adding a trigger card touches `src/cards/*.rs` and
nothing else. A card that needs an engine branch is a design failure at
review, not something to absorb.

**Two. There is exactly one growth axis, and it is not card-shaped.** A
trigger can only watch an event the engine **performs**, so the set of
watchable event kinds is the set of `GameEvent` variants — a question about
`emit_event`'s 34 call sites, not about 30,000 cards. `TriggerEvent` (§3.3)
is a mechanical projection of that enum, one arm per variant, the way
`EventPattern` is of `GameAction` (`replacement-architecture.md` §3.2a).
Growth arrives as *new performed events*, and the survey's table two already
names every one the printed pool wants (items 10–18, §3.12). Per-mechanic
variety — "and that player loses 1 life", "put that many counters" — lands
in the `Effect` tree, which is where the replacement track put it too.

**Three. The predicates reuse the vocabulary that exists.** Within an arm,
"whose", "which object", "from which zone", "why" are `PlayerRef`,
`ObjectFilter`, `Zone`, `ZoneChangeAct`, `CounterType`, `StepType` — the
leaves the filters, the patterns and the conditions already share.
`Condition` (twelve leaves, `types/effects.rs`) is the intervening "if" and
the state trigger's predicate; it grows a leaf only when a registered card
needs it, three edits per leaf (`layers-architecture.md` §13b decision 5).
Nothing here has a variant per card.

**Four. Detection and placement are two instants, and the split is fixed.**
Detection is synchronous with the mutation (§4.1); placement waits for
CR 603.3's moment (§5). CR 603.8's mid-resolution state triggers, 702.131d's
"reapplied before the check" and 603.6a's batch-wide entry check all fall
out of that split, and every alternative the project considered (a scan at
priority, a delta log replayed later) was rejected by the owner on 2026-08-24
for reasons the postscript records. This document does not reopen them.

**Five. Performance has one designated lever and a measurement gate.** The
matcher sweeps effective ability lists, which is the same read `gather` and
`is_prohibited` make, behind the same three-leg gate — printed, granted,
copied — so a board with no trigger source pays one set probe per dispatch
and the pools that exist today pay nothing. The lever, pre-approved, is a
per-source event-kind mask so a Soul Warden is never asked about a shuffle;
`fuzz_ab.py` at two seats and four is the gate, per PR, against
`engineering-practices.md` §3.1's budget. Semantics-assuming shortcuts are
ruled out in advance; `can_change_abilities()` is still the cautionary tale.

**What this does *not* claim.** The event vocabulary is incomplete and says
so — nine gaps are sized in §3.12 and one (control change) needed a design
rather than a field. The history log's facts are the CR's "this turn"
quantities as of today's cards, and a new lookback quantity is a `RowFact`
variant and a writer arm, authored with the card. Modes on a trigger (603.3c), linked
abilities (603.11/607), ward's payment (702.21a's "unless"), the monarch's
source-less abilities (724.2) and the loop detector's state hash (731) are
named, given owners, and not built here (§16).

---

## 1. Verdict — the prerequisites are met, and the survey's four decisions hold

**Yes, item 6 is next, and everything it was scheduled behind has landed.**
Recorded so a later session does not re-litigate it.

- **The stream is post-replacement truth** (critical-path item 5, RA–RE,
  closed 2026-09-15 and audited): every mutation is a `GameAction` through
  `execute_actions`, `apply_replacements` runs between proposal and
  mutation, and `GameEvent` is emitted only from inside the chokepoint,
  through one door (`GameState::emit_event`, A4c). CR 603.2g — a prevented
  or replaced event triggers nothing — is therefore a property of the
  stream, not a check the matcher makes.
- **The layer system is complete except Layer 3 and 1b**, with the
  board-wide pass and CR 613.8 (item 7, LI-1–LI-3), attachment as an input
  (6b, LH), the zone-reaching `ObjectSet` (LJ) and CR 113.6 (6a, LK + RF).
  The matcher reads *effective* ability lists and gets Humility, Blood Moon
  and a Layer 6 grant right for free; the intervening "if" is a reader over
  `Condition`, "a reader, not a language" as `CLAUDE.md` says.
- **The between-phases slot is empty.** A4e–A4q landed: the decision
  counters, the two levers, the fork test, the rulings ledger, per-instance
  targets (A4i — CR 603.3d builds on `announce_targets`), the prompt-subject
  rule (A4j — every new `ChoiceKind` here names its subject at birth), the
  middleware census (A4k — CR 603.3b's ordering prompt is classified before
  it exists, `codebase-state.md` item 163), the stack filter (A4o — a
  triggered ability is uncounterable by "counter target spell" for free if
  it is the same ephemeral object with `is_spell: false`, which §3.13
  makes it), departed seats (A4p), and the trace sink (A4c — the dispatcher
  is its sixth emit point, item 9).
- **The survey settled what this decides for.** Its four decisions stand
  and this document inherits them: the CR's phrasing names an event, a
  missing field is a filed item and never a patch, no trigger AST was
  proposed there, and no card was registered.

**The bar this is measured against**, named once. §3b's five problems and
three seams: P1 (603.8 mid-resolution) §4.5; P2 (603.1b) §3.10 and §14
question 16's neighbor; P3 (cross-turn) §3.10; P4 (603.7h) §6.5; P5 (731)
§4.9. S1 (visibility) §4.2; S2 (121.2c) §6.6; S3 (per-instance identity)
§3.6. The nine stream gaps, items 10–18, each absorbed or declined in §3.12.
The sixteen questions of the survey's §6, each answered or deferred with
the rule that lets it wait, in §14. The corpus — 133 Phase 7 atoms across 36
CR sections, 52 of them CR 603, six partially covered and none fully — owed
phase by phase in §13, with `specdb owed` the gate at each close.

---

## 2. The shape — one event, end to end

```
 chokepoint performs        the dispatch (§4)                 the stub (§5)             the stack
┌────────────────┐   ┌──────────────────────────────┐   ┌─────────────────────┐   ┌──────────────┐
│ GameEvent      │──►│ at the batch's close, or at  │──►│ perform_sba_and_    │──►│ StackEntry   │
│ (performed,    │   │ an unbatched emission:       │   │ triggers drains     │   │ is_spell:    │
│  stamped)      │   │  write the history log rows  │   │ pending_triggers:   │   │  false,      │
└────────────────┘   │  match every candidate       │   │  800.4d refusal     │   │ trigger: Some│
        ▲            │  ability against the window  │   │  tier 1 then tier 2 │   │ (the binding)│
        │            │  check state triggers        │   │  APNAP over seats   │   └──────┬───────┘
  every emit_event   │  check delayed / reflexive   │   │  ordering prompt    │          │ resolves:
  goes through one   │  resolve mana triggers NOW   │   │  modes, targets     │          │ intervening
  door (A4c)         │  queue the rest; emit        │   │  (601.2c–d)         │          │ "if" again
                     │  AbilityTriggered per queued │   │  the stack object   │          │ (608.2a),
                     └──────────────────────────────┘   └─────────────────────┘          │ bound facts
                                                                                          ▼
```

Three properties, each load-bearing:

1. **Detection reads the performed record plus live state, at the instant
   the mutation settles.** "That instant is now", so the layer walk answers
   for free and nothing is replayed (the postscript's argument, adopted).
2. **Nothing happens when an ability triggers** (CR 603.2, 117.2a). The
   dispatcher writes a `PendingTrigger` onto `GameState` and one record; it
   never touches the stack, never prompts, and never resolves anything —
   with one exception the CR makes, the triggered mana ability (605.4a).
3. **Placement is where the choices are** (603.3b–d): the order, the modes,
   the targets, the refusal. It runs inside the CR 117.5/704.3 loop the stub
   already sits in, and every prompt it asks is a `DecisionProvider` prompt
   with a subject and two or more options.

### 2.1 What the engine has today

The survey's §2 counted it and this restates only what the design leans
on. **34 `GameEvent` variants, 31 emitted, 3 never** (`PhaseEnd`,
`StepEnd`, `TurnEnd` — deleted in TR-1, §3.12). **34 `emit_event` sites,
one door**, each record carrying `EventStamp { batch, resolution }`. **23
`GameAction` variants, 23 `perform_action` arms.** `AbilityIdentity` is a
`(source, ability)` pair, which cannot tell two instances of one ability
on one object apart since A4g (item 149). The stub at
`engine/priority.rs` (`perform_sba_and_triggers`, `let triggers_placed =
false; // Phase 7 stub`) places nothing. **No registered card has a
triggered ability**; `AbilityType::Triggered` appears at two sites, the
enum and a display label. `Effect::Conditional` and `Effect::Optional`
resolve to `Err` — the intervening "if" and "may" are this phase's. The
`Condition` enum has twelve leaves and one evaluator (`condition::holds`,
the pass's live board; `settled_holds` for a post-layer reader). The CR
603.10a frame is captured on every battlefield departure (`ZoneChange.lki`,
`LeftTheGame.lki`) as an `EffectiveCharacteristics` — characteristics and
controller, no status (item 14), and for no other departure (item 15).

### 2.2 The two instants, and what each may read

| | Detection (§4) | Placement (§5) |
|---|---|---|
| **When** | the close of the outermost batch, or an unbatched emission; state and reflexive checks at the same instant | `perform_sba_and_triggers`, after SBAs settle (117.5, 704.3); the cleanup step's 514.3a check |
| **Reads** | the window's records, the live board, the LKI frames the records carry, the history log, the delayed registry | the pending queue, the seat list, the board for targets and modes |
| **Writes** | `pending_triggers`, the history log's rows, the two once-per-turn gates, `AbilityTriggered` records, the trace's `trigger` record | the stack, `AbilityTriggered`-shaped nothing (placement announces no event), the trace's `pending` record |
| **Prompts** | none — except CR 603.7b's simultaneous-events choice for a delayed trigger, which the CR puts at the trigger | `OrderTriggers` (603.3b), modes (603.3c, deferred), targets (603.3d via `announce_targets`) |
| **Outcome-bearing state** | all on `GameState` — item 40's invariant, the one constraint with a deadline (A4h's fork test) | same |

---

## 3. Type surface

### 3.1 `TriggerDef` — the body of a triggered ability

```rust
/// A triggered ability's definition (CR 603.1): "[When/Whenever/At]
/// [trigger condition or event], [effect]". The `Effect` arm that carries
/// it is `Effect::Triggered(Box<TriggerDef>)`, on an `AbilityDef` whose
/// `ability_type` is `Triggered` — the shape `Effect::Replacement` gave
/// static replacement abilities, so no `AbilityDef` literal changes.
pub struct TriggerDef {
    /// What it watches (§3.2).
    pub condition: TriggerCondition,
    /// CR 603.4 — checked as the event happens and again as the ability
    /// resolves (608.2a). `None` is an ability with no "if" clause; an
    /// `if` anywhere else in the text is ordinary `Effect::Conditional`.
    pub intervening_if: Option<Condition>,
    /// The two once-per-turn gates and "for the first time each turn" (§3.5).
    pub limit: Option<TriggerLimit>,
    /// What it does. Its targets are `Effect::instances` of this tree,
    /// announced at placement (603.3d); its bound facts read `TriggerBinding`.
    pub effect: Effect,
}
```

`AbilityDef` is untouched: `ability_type: Triggered`, `costs: Vec::new()`,
`effect: Effect::Triggered(..)`. `Effect::instances` descends into the arm
(item 153's note: "CR 603.3d will want the same walk"), so a trigger's
target instances are stored on the def the way a spell's are (A4n), and
`announce_targets` at placement is the same call `activate_ability` makes.
`is_mana_ability(def, record)` (item 11) reads a `Triggered` def too: no
target, a `ProduceMana` in the effect, and a triggering record that is mana
being added or a mana ability's activation — CR 605.1b's three criteria,
derived, never a tag. Since TR-5c's review the record decides the second:
"whenever you activate an ability" is a mana ability only when a mana
ability's activation triggers it. `could_be_a_mana_ability(def)` is the
def's half, which the mana supply asks.

### 3.2 `TriggerCondition` — event, state, and their compositions

```rust
pub enum TriggerCondition {
    /// CR 603.2's "game event": one performed record matches (§3.3).
    Event(TriggerEvent),
    /// CR 603.8's "game state": no event; a predicate over live state,
    /// checked at every dispatch and at every priority grant (§4.5).
    State(Condition),
    /// "Whenever [A] or [B]" — one ability, several events, the first that
    /// matches triggers it (CR 603.2's "a game event ... matches"). Each
    /// arm is matched on its own; two arms matching one record trigger once.
    AnyOf(Vec<TriggerEvent>),
}
```

CR 603.1b's "more than one trigger condition, and an instruction referring
to whether *all* of them happened this turn" is `AnyOf` plus an
`intervening_if` over the history log (§3.10): the ability triggers on any
of its events and the "if all" is a condition, which is exactly how the rule
reads ("regardless of whether that ability has triggered based on those
conditions"). Avatar Aang, the one printing, is a fixture until its four
bending actions exist; the shape costs no arm of its own.

**Shape is a property of the arm, not a wrapper.** "Whenever one or more
creatures die" and "whenever a creature dies" differ in one field —
`multiplicity: Multiplicity` on the arms that can carry a plural — and CR 603.2c
says which is which: `PerOccurrence` triggers once per matching record in the
window, `OncePerEvent` once per window (§4.4). Delayed and reflexive
triggers are not conditions but *origins* (§3.8): a delayed trigger's
condition is one of these three, registered rather than printed.

### 3.3 `TriggerEvent` — one arm per `GameEvent` variant

The projection, in `GameEvent`'s declaration order, with the fields each arm
reads and the phase that lands it. **Growth contract, adopted from
`replacement-architecture.md` §3.2a: exactly one arm per performed event
kind, and no other axis.** A card that needs a predicate no arm can express
is a missing field on a `GameEvent` or a missing variant, and the fix goes
there — a `TriggerEvent` arm with no record behind it is the smell this
contract exists to catch. An arm ships with its first consumer (§4's rule
from `engineering-practices.md` §4: an arm the phase's type opens, with a
printed customer, ships in that PR); an arm whose customer needs a facility
the phase lacks is a line in §15.

Within an arm, `subject` is one of three readings of "which object": `ThisObject`
(CR 603.6a's "when [this object] enters", read against the ability's own
source), `Filter(ObjectFilter)` ("whenever a creature you control ..."), or
`Any`. "Another" is `Filter(And(filter, Not(Self)))` — the leaf
`ObjectFilter::NotSource` is the one new filter leaf this phase adds, a
sibling of A4i's `OtherThanInstance`. "Whose" is `PlayerRef` (`You`,
`Opponent`, `Player(_)`, and `Each` for "each player's", "a player") and
`None` for "any". `Option` on a field means the arm does not ask.
`multiplicity` is CR 603.2c's question — one trigger per matching record
(`PerOccurrence`) or one per window in which any matched (`OncePerEvent`),
§4.4 — and it is on the arms that can carry a plural.

| `GameEvent` variant | `TriggerEvent` arm and its fields | Look-back? | Lands |
|---|---|---|---|
| `ZoneChange` | `ZoneChange { subject, change: ZoneChangePattern { from, to, act }, by: Option<SourceFilter>, owner: Option<PlayerRef>, multiplicity }` — dies is `from: Battlefield, to: Graveyard`; "leaves the battlefield" `from: Battlefield, to: None`; sacrificed/discarded/milled/countered by `act`, which a redirect keeps (CR 614.6); exiled by `to: Exile`, since CR 701.13a's act is its destination; "from anywhere" `from: None`; "a spell or ability an opponent controls causes" by `by`, which asks the cause of what the arm names (since RH-1). **The record's whole contract** (the act, the cause, the applied set, the object the move made, and when the record is matched) is `replacement-architecture.md` §2b | iff `from == Some(Battlefield)`, `from == Some(Graveyard)`, or `to ∈ {Hand, Library}` from a zone all players can see (CR 603.10a's three classes, the third written about visibility; §4.3) | TR-1 (battlefield departures), TR-4 (the other two classes, item 15) |
| `Tapped` / `Untapped` | `BecomesTapped { subject }` / `BecomesUntapped { subject }` — transition-only by the record's own contract (603.2e) | no | TR-1 (fixture), Phase 8 (card) |
| `CardDrawn` | `DrawsCard { player: Option<PlayerRef>, multiplicity }` — never a library-to-hand `ZoneChange` (121.5) | no | TR-2b |
| `ManaAdded` | `ManaAdded { source: Option<ObjectFilter>, tapped_for_mana: Option<bool>, mana: Option<ManaType> }` — CR 106.12a's "tapped for mana" reads `tapped_for_mana` | no | TR-1 |
| `DamageDealt` | `DamageDealt { source: Option<SourcePattern>, recipient: DamageRecipient, combat: Option<bool>, excess: bool, multiplicity }` — "is dealt damage", "deals damage", "deals combat damage to a player"; `combat` is item 10's field. The record names two objects, and "that creature" means either: the source in "put a +1/+1 counter on that creature", the recipient in "destroy that creature". The binding keeps both, and each instruction names the one it reads, `EventRole::DamageSource` or `EventRole::DamageRecipient` (§3.12a, `codebase-state.md` item 255; TR-5a, §12). `excess` is "is dealt excess damage" (CR 120.10), whose "that much" is the record's share of the batch's excess, read `OncePerSubject` by the recipient (TR-5d, §12) | no | TR-1; `excess` TR-5d |
| `PhaseBegin` / `StepBegin` / `TurnBegin` | `PhaseBegins { phase, whose }` / `StepBegins { step, whose }` / `TurnBegins { whose }` — "your upkeep", "each upkeep", "the monarch's end step"; `whose` reads item 10's `player` | no | TR-1 |
| `PermanentEnteredBattlefield` | `EntersBattlefield { subject, controller: Option<PlayerRef>, from: Option<Zone>, was_cast: Option<bool>, multiplicity }` — `from` and `was_cast` are joined from the same object's `ZoneChange` or `TokenCreated` in the window (§4.4; question 9) | no (603.6a reads the board after, with 603.6b's effects applied) | TR-1 |
| `LifeChanged` | `GainsLife { player, multiplicity }` / `LosesLife { player, multiplicity }` — two arms for one record because the sign decides which printed family reads it; per record (question 3). `cause` waits for its one customer, CR 727's rad counters | no | TR-1, TR-2a |
| `AttackersDeclared` | `Attacks { attacker, defender: Option<DefenderFilter>, occurrence: AttackOccurrence, alone, multiplicity }` — CR 508.3a–e's and 506.5's seven readings as three filters, what one triggering event is (an attacker, or what is attacked) and `alone`; `occurrence` was `per: AttackEventPer` until TR-5c, which named the block arm's field with CR 603.2c's word; reads item 11's defender, and only a declared attacker (CR 508.4). **Revised at TR-5a's design (2026-10-16, §12's TR-5e, decision 5)** from an enum of seven, `Creature`, `CreatureAgainst`, `PlayerIsAttacked`, `PlayerAttacksWith`, `PlayerAttacks`, `PlayerAttacksPlayer` and `Alone`, which this row had called five | no; 508.2a's snapshot is the dispatch instant | TR-5e |
| `BlockersDeclared` | `Blocks { blocker, attacker, occurrence: BlockOccurrence, count: Option<CountIs>, alone, multiplicity }` — CR 509.3a–g's readings as two filters, what one triggering event is (a blocker, a block, a blocked creature or an unblocked one), 509.3e's count and 506.5's `alone`; "that creature" of "blocks or becomes blocked by" is `EventRole::OtherCombatant`. One record per declaration step, written also when nobody blocks, with its unblocked attackers (item 11, CR 509.1h). **Revised at TR-5c's design (2026-10-18)** from `shape: BlockShape`, an enum of five readings with two spellings of one trigger and no place for the count | no; 509.3f's characteristics are the declaration's | TR-5c |
| `SpellCast` | `CastsSpell { caster: Option<PlayerRef>, spell: Option<ObjectFilter> }` — the stack object is live at dispatch (types, colors, mana value through the layer walk). `from` waits for casting from a zone other than the hand (`backlog.md` §2.3) | no | TR-2a |
| `AbilityActivated` | `ActivatesAbility { player: Option<PlayerRef>, source: TriggerSubject, mana_ability: Option<bool> }` — both producers write the record, a mana ability's after its costs (CR 605.3a–b), with whether it is one, where its source was and the source's frame if a cost moved it; the arm reads only a source on the battlefield (Harsh Mentor's second ruling). A trigger on a mana ability's activation that could add mana is a mana ability (CR 605.1b). The zone waits for an activation off the battlefield (item 282), a loyalty field for CP-2's loyalty abilities (item 283). "Copy that ability" needs the stack object, which the record does not name (§3.12a, item 256, CV-4) | no | TR-5c |
| `AbilityResolved` | `AbilityResolves { identity: IdentityRef }` — `ThisAbility` for CR 603.7h's delayed form, read with the resolution count (§6.5) | no | TR-3 |
| `SpellCountered` / `AbilityCountered` | `IsCountered { spell: Option<ObjectFilter>, caster: Option<PlayerRef> }`, reading `SpellCountered` only — **never `SpellFizzled`** (question 2; Multani's Presence's ruling). The look-back reads the frame the record carries: the stack object's `Lki` with its `entry` (§3.12a, item 253), whose `StackEntry::caster()` is "a spell you've cast". No printed trigger reads an ability being countered (§12, TR-4d) | yes (603.10e) | TR-4d |
| `SpellFizzled` / `AbilityFizzled` | **no arm.** CR 608.2b's "doesn't resolve" is not "countered" under the baseline CR, and no printed trigger reads it. An ability's is its own record since #231, named by its identity as `AbilityResolved` is | — | — |
| `PlayerLost` | `PlayerLoses { player }` — 603.9's "regardless of the reason", so no reason field: why is the record's cause, and `LossReason` folded into it at RH-2. Its "unless as the result of a draw": a draw is `GameResult::Draw`, not a `PlayerLost`. The loser's permanents leave in the same window and are asked off their departure frames; one the loser controlled is kept off the stack (CR 800.4d), and one another player controlled triggers for that player | yes (603.10f) | TR-4d |
| `PlayerWon` | **no arm** — no printed trigger; recorded so the projection stays one-to-one | — | — |
| `Scried` | `Scries { player }` — "that many" is the record's `looked_at`, the cards a scry looked at, which a short library makes fewer than its number (Elrond, Master of Healing's first ruling); a scry 0 writes no record (CR 701.22b) | no | built in TR-5g (fixture) |
| `LibraryShuffled` | `ShufflesLibrary { player }` — two printed watchers, Cosi's Trickster and Psychic Surgery ("whenever an opponent shuffles their library"; the survey's 168 on this row is the substring over-count); one record per shuffle, an empty or one-card library included, and never for cascade's random bottom (Cosi's Trickster's rulings) | no | TR-2b |
| `CountersPutOn` / `CountersRemovedFrom` | `CountersPutOn { subject: CountersOn, counter_type, by, multiplicity }` / `CountersRemovedFrom { .. }`, over two records since TR-5b split `CountersChanged` by direction: a put always has a putter (CR 122.6a, 120.3d), a removal has a remover only where a player removed them. An occurrence is a counter by default (Protean Hydra's fourth ruling), `OncePerSubject` is "one or more ... on a creature" (Enduring Scalelord's second), `OncePerEvent` "on one or more". Counters a permanent enters with are put on it (CR 122.6). CR 122.7's "the Nth counter" is no field: it is the ability's intervening "if", `Condition::CounterCountReached`, reading the count each record left (`now`), as CR 714.2b words a Saga's chapter | no | TR-5b |
| `CountersAnnihilated` | **deleted in TR-5b**: CR 704.5q is two `RemoveCounters` members of the state-based batch, so its removals are `CountersRemovedFrom` records the removal arm reads (Protean Hydra's fifth ruling) | — | TR-5b |
| `Attached` | `BecomesAttached { attachment, host }`, each a `TriggerSubject`, read after the event (Siona, Captain of the Pyleas' fourth ruling) — transition-only (701.3b), so re-equipping the same creature announces nothing (603.2e-002), and an Aura entering attached is one (CR 303.4f; item 228). "That creature" is the host | no | TR-4b |
| `Unattached` (was `EquipmentDetached`) | `BecomesUnattached { attachment, former_host }` — one record for CR 701.3d's four routes (question 5), named by `UnattachRoute`, the cause on the stamp (§3.12a). Both ends are `ObjectRef`s, and "that permanent" is the existence the record names (CR 400.7) | yes (603.10c): each end as it was, off its departure's frame when it left in the window; an attached ability-list source's move snapshots the lists before it (§4.3) | TR-4b |
| `LeftTheGame` | folded into `ZoneChange`'s leaves-the-battlefield reading: a `LeftTheGame` from the battlefield matches `from: Battlefield, to: None` and nothing narrower, which is CR 603.6c's own sentence; the phased-in qualifier is item 6's and waits for phasing | yes (the record carries the frame since RE-7) | TR-1 |
| `TokenCreated` | `CreatesToken { owner, definition: Option<Box<TokenDef>>, multiplicity }` — keyed here and never on `is_token` at entry (item 8; CR 111.13). A reflexive "when you do" on a creation asks whether each token is of the definition the instruction named, which the record carries after any replacement: Ajani, Nacatl Avenger's eighth ruling triggers once per Cat Warrior under Doubling Season and not at all for the Angels Divine Visitation makes instead. *Corrected at TR-5g:* this row named `zone`, which no printed trigger asks, and `kind: Option<TokenKind>`, which the G2 amendment (2026-09-23) said was the definition | no | built in TR-5g (fixture) |
| `TokenCeasedToExist` | **no arm** — CR 704.5d names no trigger event, and what cards observe is the *absence*: Flickerwisp's ruling has an exiled token "cease to exist and won't return", which is a delayed trigger's `ObjectRef` finding nothing (§3.9), not an event to match | — | — |
| `StateBasedActionPerformed` | **no arm** | — | — |
| `Targeted` (item 12; the record since #231) | `BecomesTarget { target: TargetSubject (an object or a player), by: TargetedBy (a spell, an ability, either; "an Aura spell"), controller: Option<PlayerRef>, multiplicity }` — once per spell or ability, however many instances (question 11), one window per spell or ability (CR 601.2c). "For the first time each turn" is `TriggerDef::limit`'s, read off the turn's targeting rows | no | TR-5c |
| *new* `ControlChanged` (item 13) | `ControlChanges { subject, from: Option<PlayerRef>, to: Option<PlayerRef> }` — "you lose control of" is `from: You`; "an opponent gains control of a permanent from you" adds `to: Opponent` (Zidane, Tantalus Thief); "a permanent you own" is the subject's filter. Losing control includes leaving the battlefield (Khârn the Betrayer's first ruling), so an arm with `to: None` also reads a battlefield departure whose frame's controller is `from` | yes (603.10d), off the frame the sweep kept (§12, TR-4d) | TR-4d |
| *new* `DamagePrevented` (item 16) | `DamageIsPrevented { recipient: DamageRecipient, multiplicity }` — one record per prevention applied per subject group (CR 615.13), read at the batch's close; an object recipient is `EventRole::DamageRecipient` | no | TR-5d |
| `AbilityTriggered` (new, §4.8) | `AbilityTriggers { caused_by: Option<TriggerEvent> (Strict Proctor's "a permanent entering causes"), source: Option<ObjectFilter> }` — **the arm that is CR 603.3b's second tier**: `TriggerCondition::tier()` reads it | no | TR-1 (the event and the tier), TR-5 (the arm's card) |

Thirty-four variants, four deleted (three never emitted, and
`CountersAnnihilated` once CR 704.5q proposed), four added (three from the
survey's gaps and `AbilityTriggered`), one renamed, and one split
(`CountersChanged`, in TR-5b): **thirty-five performed kinds after TR-5,
thirty-one with an arm and four with none, each of the four named above with
the reason.** One kind carries two arms, life, where a signed total splits
two printed families; that is the one place the projection is not
one-to-one, and it is a property of the record, not a second axis. `PhaseType` and `StepType` are
the engine's own; "at end of combat" is `StepBegins { step: EndCombat }`
(511.2) and "at end of turn" is `StepBegins { step: End }` (513.1a), which
is why the three `*End` variants die.

**Not one enum with `EventPattern`** (the TR-1 review, 2026-09-22). The two
watch different streams: an `EventPattern` reads a *proposal*, before it
happens and rewritable by the closed `Rewrite` algebra; a `TriggerEvent` arm
reads a *performed* record, after, with frames, look-back and multiplicity.
Ten of TR-1's twelve arms have a sibling in shape (`Attacks` and
`AbilityTriggers` have none), and the vocabulary is already shared at the
leaf: `ObjectFilter`, `PlayerRef`, `Condition`, `ZoneChangeAct`. What is
worth sharing is a *field struct* — `ZoneChangePattern { from, to, act }`,
which both `ZoneChange` arms hold since RH-1 (`replacement-architecture.md`
§2b.7), and `SourcePattern` for damage where the trigger arm has a
`TriggerSubject`. Never a shared enum. Layers watch nothing.

### 3.4 Subjects, "you", and the bound facts — `TriggerBinding`

A trigger's effect refers back to its event: "that creature", "that player",
"that many", "it" in "return it to the battlefield", "damage equal to that
creature's power". CR 608.2k says the reference survives characteristic
changes; CR 603.6 says a zone-change trigger looks for the object in the
zone it moved to and finds nothing if it left; CR 113.7a and 608.2h say
information about an object that is gone is its last known information.
The engine's answer is a struct filled at dispatch and carried on the
`PendingTrigger` and then the `StackEntry`. It was built to **point at the
records and copy nothing they hold**. **As built in the bounded-state PR
(2026-09-25; `codebase-state.md` item 42):** the log has left `GameState`, so
the binding **copies the records it matched, whole**. A projection of the
facts each reader needs would be a second schema for an event, with a second
chance to drop a fact. The copy is cheap because a record's CR 603.10a frame
is the layer memo's own `Arc`, which the same PR made it:

```rust
pub struct TriggerBinding {
    /// The records that matched, copied at dispatch: one for a
    /// `PerOccurrence` trigger, every matching record of the window for a
    /// `OncePerEvent` one. Each carries its `EventId`, the stream's
    /// monotonic number and never an index, and its `EventStamp`, which the
    /// reflexive check and CR 603.7h's "this ability" read.
    pub records: Vec<EventRecord>,
    /// Which of the condition's events matched — the `TriggerEvent` whose
    /// projections (below) say what "that object", "that player" and
    /// "that many" are for these records.
    pub event: EventIndex,
    /// The one fact a record does not carry and the resolution needs: the
    /// subject's `zone_change_epoch` at dispatch (CR 400.7 — a later move
    /// makes it a new object the reference cannot find;
    /// `codebase-state.md` item 10's field). `None` for an event about no
    /// object.
    pub subject: Option<ObjectRef>,
    /// The ability whose triggering this is (CR 603.3b's second tier), or
    /// for a reflexive trigger, the action within the resolution.
    pub triggered_by: Option<TriggerSeq>,
}
```

**Amended 2026-10-10 (`replacement-architecture.md` §2b.5), as built in RH-3
(#245):** a zone change's record carries the object the move made
(`new_epoch`), so `subject` is read off the record rather than off the store at
dispatch (`subject_after`, item 177). "Them" on a "one or more" binding is
`EffectRecipient::TriggeringObjects`, each matched zone change's new object;
over another kind of record it is refused (`codebase-state.md` item 250).

**Amended 2026-10-16 (TR-5a, §12):** the binding keeps its triggering
events (`TriggeringEvent`), each the record that holds it and the objects it
is about (`TriggeringEventObjects`), by the part each played, as `ObjectRef`s
taken at the dispatch. That replaces `records` and `subject`. Each instruction
names the participant it reads: `EffectRecipient::Triggering(participant)` is
"that creature" off a trigger on each event and "them" off "one or more", over
any kind of record (items 250 and 255).

**The projections live on the arm, and they are exhaustive matches.**
`TriggerEvent` carries three methods — `subject_of(&GameEvent) ->
Option<ObjectId>`, `player_of(..) -> Option<PlayerId>`, `amount_of(..) ->
Option<u64>` — and the dispatcher's `triggering_events_in` one more,
each a `match` whose last arm names every arm with `_` for the record (the
triggers midpoint audit, item 231: a bare wildcard had stood there since
TR-1). So a new arm must say what its "that many" is (damage
dealt, cards looked at, counters put on, damage prevented) or fail to
compile, and must say what an occurrence is (a record for most kinds; a
counter for the counter arms, per Protean Hydra's ruling; an attacker for
the attack shapes). The readers resolve through them at resolution:
`EffectRecipient::TriggeringObject` is `subject` checked against the live
object's epoch — CR 603.6's "unable to be found" and CR 400.7 in one
comparison; `EffectRecipient::TriggeringPlayer` is `player_of` on the
records; `AmountExpr::TriggeringAmount` is `amount_of` summed over them
(one creature's records for Simic Ascendancy's "that many", §4.4); and
`AmountExpr::TriggeringPower` reads the live object when it is where the
event left it and its last known information otherwise (CR 608.2h) — the
record's `lki` when the event was the departure, the entry's
`kept_lki` when the object left afterwards (§6.1). All
three leaves ship in TR-1 because the type that carries them opens there.

**Why no stored amount.** A single `Option<u64>` on the binding would mean
a different field per arm with nothing forcing a new arm to declare which,
and a number copied at dispatch is a second copy of a fact the record
already materializes. Holding the record keeps one copy of each fact, and the
frame comes with it. Pointing at the record cost one constraint, that no
record a pending or stacked trigger referenced could be evicted. Copying the
record removed it, and with it the last reason for the log to outlive a
dispatch.

### 3.5 `TriggerLimit` — the two once-per-turn gates, and "first time"

The survey's question 16 found two rules where the text has one phrase
family, and they are two trackers with two writers:

```rust
pub enum TriggerLimit {
    /// CR 603.2h — "Do this only once each turn." An action-taken gate:
    /// the ability keeps triggering until its action is taken; written by
    /// the RESOLUTION that takes the action; read at dispatch (no trigger
    /// once taken) and at resolution (a second instance on the stack does
    /// nothing). Nykthos Paragon's four rulings, each a test.
    DoThisOnlyOnceEachTurn,
    /// "This ability triggers only once each turn" — no rule of its own in
    /// the baseline CR (CR 702.179d's speed is the one use), so the earliest
    /// printing's ruling defines it: Elvish Warmaster's, "once the triggered
    /// ability has triggered once during a turn, it can't trigger again,
    /// even if [it] is still on the stack, has been countered, or has
    /// otherwise left the stack". A triggered gate: written by the
    /// DISPATCHER as it queues; once per source object per instance; once
    /// for a batch; read before CR 603.2d's multiplier applies, so
    /// Panharmonicon cannot double it.
    TriggersOnlyOnceEachTurn,
    /// "Whenever [event] for the first time each turn" (Kira, Vengeful
    /// Warchief): not a gate on the ability but a predicate on the event —
    /// this record is the first of its kind among the player's rows of the
    /// turn (§3.10). Written by nobody; read at dispatch, record by record.
    FirstTimeEachTurn,
}
```

The two gate sets live on `GameState` beside the history log:
`action_taken_this_turn` and `triggered_this_turn`, each keyed by the full
identity (§3.6), so a bounced and replayed permanent — a new object, CR
400.7 — starts clean, and both cleared by the `BeginTurn` performer. "Each
turn" is the game's turn, not the controller's.

**Amended 2026-09-24 (`cr-coverage-audit.md` §4a, pass 2): the action-taken
gate is also keyed by the controller.** CR 603.2h: the ability triggers "only
if its source's controller has not yet taken the indicated action that turn".
If a permanent changes control mid-turn, its new controller has not taken the
action, so `action_taken_this_turn` is a set of `(AbilityIdentity, PlayerId)`
pairs. Nykthos Paragon's rulings don't reach a control change, and the rule's
wording decides it. `triggered_this_turn` stays keyed by the identity alone:
Elvish Warmaster's ruling makes "triggers only once each turn" a fact about
the ability. 34 cards print "Do this only once each turn"
(`o:"do this only once each turn"`).

**As built (TR-2a, 2026-09-24).** "The first time" is each record's place
among its player's records of the same kind this turn, which the history's
advance returns beside the rows (`TurnOrdinals`). So two losses in one batch
are the first and the second. `triggered_this_turn` is written as the
dispatcher queues, so a second match in the same window finds it taken.
`begin_turn` clears both sets and §6.5's count.

**Amended at TR-3c's review (2026-10-10): the place is kept on the record.**
`TurnOrdinals` lived only for the dispatch that advanced the history, so a
reflexive trigger made later in the resolution could not read it, and
TR-3c first answered "no place" for one. `EventRecord::place_in_turn` is
written as the history advances past the record, and every reader of "the
first time each turn" reads it there.

### 3.6 `AbilityIdentity` gains the object's epoch, and a granted instance its grant (S3)

```rust
pub struct AbilityIdentity {
    /// The object, and which existence of it (CR 400.7 — the epoch
    /// `move_object` stamps, item 10's field): two activations across a
    /// bounce are two abilities' worth of counting. One `ObjectRef`, since
    /// that type is exactly this pair (`types/ids.rs`).
    pub source: ObjectRef,
    /// Which ability, and for a granted one which grant: `AbilityId {
    /// definition, grant }`, the grant the granting row's `EffectId`.
    /// A4g made an `AbilityId` per definition, so two sources granting one
    /// ability put it on an object twice (item 149). For a mana ability
    /// nothing tells the two apart in outcome; for a TRIGGERED one the
    /// outcome differs — Diffusion Sliver's ruling: the abilities Slivers
    /// grant "are cumulative", so a Sliver under two Diffusion Slivers
    /// triggers twice and the opponent pays twice — and a dispatcher keyed
    /// on the definition alone would fold the two into one trigger. Each
    /// instance triggers, is placed, and is its own gate; CR 603.7h counts
    /// the ABILITY (§6.5), so its key takes `ability.definition()`.
    pub ability: AbilityId,
}
```

**Decided: the instance joins the identity, as the brief recommended,
with one refinement the rulings forced.** Ashling the Pilgrim's ruling
(2023-07-28): the count is of "that same ability from this creature", a
copy of the ability (Rings of Brighthearth) counts, another creature with
the same name does not, and a countered one does not — so 603.7h's key is
`(source, zone_change_epoch, ability)`, and CR 707.10b's copy rule for
CV-4 is one line: a copied ability's stack entry carries the original's
identity. The instance is for dispatch and placement, where two instances
are two triggers, and for the gates. The `(source, ability)` pair sites —
13 by item 149's count — take the two new fields; `AbilityActivated` and
`AbilityResolved` carry them. An ordinal among same-id instances rather
than an index into the whole list, so a Layer 6 grant later in the turn
does not renumber an earlier instance's gate.

**Amended 2026-09-22 by the owner (the TR-1 review, theme D): provenance
ids replace the ordinal, built in the review's theme E before TR-2 keys a
gate on the identity.** The ordinal is stable under a later grant but not
under an earlier grant *ending*: if the first of two Diffusion Slivers
leaves mid-turn, the survivor's instance 1 becomes instance 0, and a
once-per-turn gate keyed on instance 1 is orphaned while instance 0's reads
as unused. So the Layer 6 grant site (`compute.rs:962`, where
`is_characteristic_defining` is already cleared) mints the granted def's id
from the granting row's source and epoch, an id constructor beside
`AbilityId::derived_on` (`types/ids.rs:140`). Two grants carry two ids, stable
while each grant exists and gone with it, so the "one or more" fold's
`(identity, arm)` key (`dispatch.rs:519`) tells them apart with no ordinal.
The elision (`placement.rs:106`) is re-keyed on def equality rather than id
equality, and item 149's closure is amended. "Loses all abilities" goes by
list, so CR 113.10b is unaffected. Diffusion Sliver itself waits for TR-5's
target event, so nothing printed reads the field yet.

**Built 2026-09-22 (theme E), with three calls the amendment left open —
the owner's, each as recommended.** The mint changes every granted def's
id, not only a triggered one's, and three sites wanted the shared id,
because each means "this ability, whichever instance": CR 113.10b's
`LoseAbility`, the mana window's dedupe, and a granted static's rows, which
`register_granted_static_effects` filed under the def's id for CR 604.2's
existence check to find on the grantee. *Where the definition lives:*
inside the id, `AbilityId { definition, grant }` with the grant `0` unless a
Layer 6 row granted the instance, so no `AbilityDef` literal changed,
equality is per instance by default, and `definition()` is the opt-in —
taken on the condition that the A/B's CPU line shows no cost of the wider
key (`fuzz-record.md`'s theme E block), a field on `AbilityDef` otherwise.
*The mint's input:* the granting row's `EffectId`
(`AbilityId::granted_by`), not its source and epoch. A resolution's row is
sourced at the resolving stack object — a spell, whose epoch moves when it
reaches the graveyard and again whenever it leaves, or an ability object
CR 608.2n deletes — and two grants of one def from one source to one
object would share a pair. The row is the grant: a counter no row reuses,
gone when the grant goes. *The mana window* dedupes on the definition, so
it lists what it listed. A granted static's rows now carry its instance's
id, which keeps the existence check exact, and CR 614.12's look-ahead names
its would-be rows by the ids their registration will assign. One
consequence rather than a call: the replacement gather keys a static
replacement by `(ObjectId, AbilityId)`, so two granted instances of one
replacement ability are two effects, as the CR has them; nothing registered
grants one.

### 3.7 `PendingTrigger` and the queue

```rust
pub struct PendingTrigger {
    /// Monotonic per game; the stable order key within one player's
    /// triggers, and the handle a tier-2 trigger's "that ability" resolves.
    pub seq: TriggerSeq,
    pub origin: TriggerOrigin,            // §3.8
    /// CR 603.3a — the player who controlled the source as it triggered,
    /// locked here; for a delayed trigger, 603.7d–g's.
    pub controller: PlayerId,
    /// What `GameObject::new` needs to build the stack object — the name and
    /// display the source's card gives it, as `activate_ability` clones the
    /// source's `card_data` today. Held here, behind an `Arc`, because the
    /// source may be gone by placement: a dies trigger's source is in the
    /// graveyard and a `LeftTheGame` source is not in the object map at all.
    /// CR 603.3 gives the object "the text of the ability that created it,
    /// and no other characteristics", and nothing reads this card's types.
    pub source_card: Arc<CardData>,
    /// The bound facts, and the def: shared out of the source's effective
    /// list once, at dispatch, so a Humility that lands between triggering
    /// and placement cannot un-trigger it (CR 113.7a). CR 603.3b's tier is
    /// read off it (`PendingTrigger::tier`) rather than stored beside it,
    /// and so is CR 605.1b's mana class (`is_mana_ability`).
    pub binding: TriggerBinding,
    /// CR 603.8's one-shot: a state trigger stays armed-off until its stack
    /// object leaves (§4.5).
    pub is_state_trigger: bool,
}

// on GameState:
pub pending_triggers: Vec<PendingTrigger>,
pub next_trigger_seq: u64,
```

On `GameState` and nowhere else — item 40's invariant, and A4h's fork test
is what enforces it: a clone taken at a priority prompt with triggers
waiting must replay identically, which it cannot if the queue lives on the
dispatcher's stack.

### 3.8 `TriggerOrigin` — object, delayed, reflexive, rule

```rust
pub enum TriggerOrigin {
    /// A printed, granted or copied ability of an object.
    Object(AbilityIdentity),
    /// CR 603.7 — registered by a resolution, a replacement's rider, or a
    /// special action; provenance per 603.7d–g on the registry entry.
    Delayed(DelayedTriggerId),
    /// CR 603.12 — a delayed trigger checked at creation against the
    /// creating resolution's own records.
    Reflexive(DelayedTriggerId),
    /// CR 724.2 / 725.2 / 727.1 — "inherent triggered abilities [that] have
    /// no source", CR 113.8's exception. Named here so `AbilityIdentity`
    /// never grows a fake source; built with the designation that owns it
    /// (Phase 9). Question 6.
    Rule(InherentAbility),
}
```

**As built (TR-3a, 2026-10-07): `Delayed(AbilityIdentity)`, not
`Delayed(DelayedTriggerId)`.** Every reader of an origin wants its source and
its identity: placement builds the stack entry's `ability_identity`, CR
603.7h's count keys on it, and an entry's kept LKI names the source. A `Once`
entry leaves the registry as it queues, so an id alone would name nothing by
placement. The identity is the entry's source (CR 603.7d–g) and
`AbilityId::delayed` of its registry number, a derivation in the role
`UNASSIGNED` leaves free above zero, so a delayed trigger's resolution never
advances its creator's count. The arm's tag grows `GameEvent` from 64 to 72
bytes (`AbilityTriggered` carries the origin), priced in `fuzz-record.md`'s
TR-3a block.

**Amended at TR-3c (2026-10-09): there is no `Reflexive` arm.** CR 603.12
has a reflexive trigger "follow the rules for delayed triggered abilities",
and what every reader of an origin wants is what `Delayed` already carries:
the source CR 603.7e gives it, and the identity its registry number gives
it. A reflexive trigger is a delayed entry checked as it is made and never
left waiting (§4.6), so its number names nothing in the registry once it is
made, as a `Once` entry's names nothing once it has triggered. A second arm
would be a second spelling of one origin, and placement, CR 603.7h's count
and the kept LKI would each have to learn it.

### 3.9 The delayed-trigger registry (CR 603.7)

```rust
pub struct DelayedTrigger {
    pub id: DelayedTriggerId,
    pub condition: TriggerCondition,
    pub effect: Effect,
    /// CR 603.7d–g: the source, remembered — the spell's card and the
    /// object id it had as it resolved (its stack object is gone, 608.2n),
    /// the ability's source, the static ability's object.
    pub source: DelayedSource,
    /// 603.7d–g: the controller as of the creating instant.
    pub controller: PlayerId,
    /// When it was created — CR 603.7a (never retroactive), 513.2 (a step
    /// that has begun does not "back up"), and the reflexive window's end.
    pub created: Instant { turn: u32, record: usize },
    /// CR 603.7b — once, or a stated duration.
    pub duration: DelayedDuration,   // Once | ThisTurn | UntilEvent(..)
    /// CR 603.7c — the objects it refers to, by id and epoch (CR 400.7),
    /// filled from the records the creating instruction performed, never
    /// from its count: under Doubling Season "exile it at the beginning of
    /// the next end step" exiles each token made (Twinflame's and
    /// Kiki-Jiki's rulings), and a creature that later becomes a copy of
    /// one is not among them (Twinflame's other ruling).
    pub refs: Vec<ObjectRef>,
    /// CR 107.3n — X, inherited from the creating spell when unstated.
    pub x: Option<u64>,
    /// CR 603.12 — `Some(stamp)` for a reflexive trigger: checked at once
    /// against the records that carry this stamp up to `created.record` —
    /// "earlier during the resolution" (amended 2026-09-23, §4.6).
    pub reflexive: Option<ResolutionStamp>,
    /// A named extra turn, for "at the beginning of that turn's end step"
    /// (Final Fortune): CR 500.7's queue entries gain an id and a turn
    /// records which entry it came from, so a skipped extra turn (614.10)
    /// never fires it.
    pub turn: Option<ExtraTurnId>,
}
// on GameState: delayed_triggers: Vec<DelayedTrigger>, next_delayed_id: u64
```

`Primitive::CreateDelayedTrigger(Box<DelayedTriggerTemplate>)` is the
producer for 603.7d/e (a resolution) — the template names the condition
and the effect and the resolver fills provenance from `ResolutionContext`
(`source` or `ability_source`, `controller`, `x`). A `ReplacementDef.then`
containing it is 603.7f: the rider's context carries the static ability's
object and its controller *as the replacement applied*, which is
`Rider`'s instance source and the chooser of that group (question 14). 603.7g
has no producer until a static ability lets a player take a special action
(`backlog.md` §2.8), and gets a fixture. CR 610.3's "until [event]" return is
the registry's second kind, `DelayedDuration::UntilEvent`: the returning
one-shot is created "immediately after" the event and does not use the
stack, so it resolves at dispatch the way a mana trigger does (§4.7), with
610.3d's simultaneity falling out of the window. Banishing Light is the
consumer.

**Amended 2026-09-24 (`cr-coverage-audit.md` §4a, pass 4): the id rides the
proposal.** "A turn records which entry it came from" has to happen before the
turn begins, on `GameAction::BeginTurn`, because four printed replacement
effects read it there: Stranglehold, Ugin's Nexus, Gerrard's Hourglass Pendant
and Trouble in Pairs replace "a player would begin an extra turn" (CR
614.10). Today `next_turn_taker` pops the queue and proposes the turn with its
player and number only, and `BeginTurn`'s own doc says who takes the turn is
not on the event. The proposal carries `extra_turn: Option<ExtraTurnId>`, the
turn's `EventPattern` arm reads it, and the turn that begins takes it from the
proposal. "During that turn" (Alchemist's Gambit, Kang the Conqueror) is a
duration on the same id. ~20–30 lines, with this id or with the first of the
four cards.

**As built (TR-3a, 2026-10-07).** `engine/triggers/delayed.rs`, with these
departures from the sketch, each forced by a board:

- **The def, not a condition and an effect**: the entry holds an
  `Arc<TriggerDef>`, so an intervening "if" and the binding come as they do
  for an object's trigger, and placement needs nothing new.
- **`source: ObjectRef` and `source_card`, not a `DelayedSource` enum**: every
  CR 603.7d–g rule fixes its source and controller at creation, after which
  nothing reads which rule it was. The resolution reads the context: an
  ability's `ability_source` (603.7e), else the resolving spell or the
  rider's object (603.7d, 603.7f), never `ctx.source` for an ability
  (`codebase-state.md` item 223). `create_delayed_trigger` (named
  `register_delayed_trigger` until TR-3c's review) is the one door, and
  603.7g's special action will call it.
- **`source_left_at`**: a departure record names the object's id and not
  which existence of it moved, so the movers note the record that ended a
  delayed trigger's source. "When this creature leaves the battlefield"
  matches that record and no later object's.
- **`source_frame`**, added at the review, and `source_lki`, an `Lki`, since
  TR-4a (§3.11): CR 113.7a's last known information, the frame the source
  left with. `hand_over_lki`,
  the one writer of kept LKI, gives it to the entry as it gives it to
  every keeper reading the mover, and `objects_lki_is_kept_for` names the
  registry's sources, so a spell is framed as it leaves the stack. The
  trigger carries it to the stack in its `kept_lki`; its first reader is
  item 223's attribution (Dragonhawk's damage, lifelink read off the frame).
- **`created_at` and `created_in` replace `created`**: CR 603.7a is the
  filter, and its batch half was found by a test. A rider runs inside the
  batch it rides on, and Destroy's move closes its own nested batch, so a
  rider can run between two members of one simultaneous event. An entry it
  creates reads nothing of that batch.
- **`turn: TriggerTurn`**: `Any`, `LaterThan` the creating turn (the Ice Age
  cantrips' "the next turn's upkeep", which a second upkeep this turn, CR
  500.10, is not), or the one `Extra` turn "that turn" names. `ExtraTurnId`
  rides `GameAction::BeginTurn` as `extra_turn` and becomes `GameState::extra_turn`;
  the turn's `EventPattern` field waits for the first of the four cards. An
  `Extra` entry is dropped as the next turn begins if its turn has gone.
- **No `refs` and no `reflexive` yet**: no TR-3a board names an object other
  than its source. Objects a creating instruction names, Flickerwisp's "that
  card" and the Tatsumasa board's "that token", come with TR-3b, beside the
  reflexive window, both reading the resolution's own records.

**Added in #233 (2026-10-08), the dev GUI's notes on TR-3a.** The template and
the entry carry `rules_text: AbilityText`, the card's words for the trigger,
which no `TriggerDef` carries, written by the card's author as an ability's
are. `create_delayed_trigger` announces `GameEvent::DelayedTriggerCreated`
ahead of the entry, so its creation is no record it reads; nothing triggers on
it. `AbilityId::delayed_trigger` reads back the role above, so a log names the
trigger by its registry number on its triggered and resolved lines.
`ui::waiting` reads the registry for a client.

**As built (TR-3b, 2026-10-08).** `refs` is `Effect::Remember`, decided at
#234's review. A card marks the instruction whose objects a later delayed
trigger names, and once that instruction has performed the resolution reads
back its own records (`EventWindow::resolution_records`), which the window
holds until the resolution ends. What it remembers:
- each object the verb moved where the verb puts them, as the move left it
  (a move a replacement sent elsewhere is not "that card");
- each token it made;
- each object a verb that moves nothing acted on;
- each player it named.

They ride on the entry as `Referred`, each object's departure noted as the
source's is, and on the trigger's binding to the stack.
`TriggerSubject::Referred` matches "when that token dies", and
`EffectRecipient::Referred` finds each object only while it is the same
object (CR 603.7c). CR 610.3's return is not `DelayedDuration::UntilEvent`:
it is no triggered ability, so it lives on `GameState::until_returns`
(`engine/returns.rs`), made by `Primitive::ExileUntil` and performed at a
dispatch as one batch for one event (610.3d). Its event is any
`TriggerEvent`, matched as a trigger arm is (`take_returns_due`), and
`refers_to` names a target the event watches as `TriggerSubject::Referred`
(Calix, Destiny's Hand). CR 610.3a/b's "already occurred" is answered for a
leaving, by identity, and refused for any other event until that event's
history is kept (`codebase-state.md` item 230). A token the exile moved
stays where it went (CR 111.8). A return carries an `UntilReturnId`, minted
as a delayed trigger's number is, and `wait_to_return` announces
`GameEvent::UntilReturnMade` ahead of it, as `DelayedTriggerCreated` is
announced; the Waiting panel keeps a row by the number its log line gave it
(item 234, the TR-3b dev GUI PR).

**As built (TR-3c, 2026-10-09; the door renamed at its review,
2026-10-10).** The sketch's `reflexive: Option<ResolutionStamp>` is
`DelayedDuration::Reflexive(ReflexiveForm)`, `Does` or `Doesnt`, beside the
resolution creating it, which `DelayedProvenance::resolution` carries
(`CreatingResolution`: the stamp, and "that action", the records of the last
instruction before it that took one). The door is `create_delayed_trigger`,
CR 603.7a's word: a waiting entry is registered for a later event, and a
reflexive one asks about that action now (§4.6) and never reaches the
registry — the review's question was why a function named for registering
read the past, and the name was the answer. A reflexive template with no
resolution creating it is refused, since CR 603.12 makes one only as a spell
or ability resolves, so the door returns a `Result`. Its
`DelayedTriggerCreated` reads "reflexive, if it was done" (or "if it was not
done") whether or not it triggered: the owner's call at TR-3c's design
questions, since a dropped entry otherwise leaves no trace in the log. Cards
write `when_you_do(event, …)` and `when_you_dont(event, …)`, so the call site
says it is checked once.

### 3.10 The history log, `PlayerHistory`, and the game scope (item 42; P2–P4; question 15) — rebuilt in TR-5h

```rust
/// One event a reader of the past can ask about, as it was then, kept for
/// the whole game (`state::history_log`).
pub struct HistoryRow {
    pub seq: EventId,                    // the record it was written from
    pub turn: u32,
    pub combat: u32,                      // which of the turn's combats had begun
    pub player: Option<PlayerId>,         // whose event
    pub object: Option<ObjectRef>,        // the existence it was about (CR 400.7)
    pub frame: Option<Arc<EffectiveCharacteristics>>, // that object then (CR 608.2h)
    pub fact: RowFact,
}
// RowFact: Cast { from }, Drew, GainedLife(n), LostLife(n), EnteredBattlefield { from },
// LeftZone { from, to, act, became }, LeftTheGame { from },
// DealtDamage { to, amount, combat }, Attacked(target), Blocked(attacker),
// RemovedFromCombat { attacking }, BecameTarget { by_spell, controller },
// Stated { fact, count } (a scenario's count, §5.1 of setup)

/// "[Players'] [fact] [of what]", summed over a span's rows. A `Condition`
/// leaf compares it with a `CountIs`; `AmountExpr::History` reads it.
pub struct HistoryCount { pub players: PlayerSet, pub fact: TurnFact, pub of: Option<Box<ObjectFilter>> }
// TurnFact: SpellsCast, CardsDrawn, LifeGained, LifeGainEvents, LifeLost,
// LifeLossEvents, DamageTaken (dealt to this player: bloodthirst),
// ControlledCreaturesDied (controlled as it died: morbid sums the players),
// AttackersDeclared (raid is at least one)

/// Beside the log, per player: "your last turn" and the draw step's count.
pub struct PlayerHistory { own_turn: Option<u32>, previous_own_turn: Option<u32>, this_draw_step: u64 }
```

**One store, written once.** Every count a card reads and every "what did
this object do" is a read of one log, written by one writer
(`engine::triggers::history`) record by record as each event ends (§4.1).
A row records an event, never the board: its frame is a departure's record's
last known information, or else the object as the window left it (CR
603.10), the window's own rows included. That last clause costs a second
look only when the layer memo's gate says a frame read a kind of row the
window wrote (the archive's "TR-5h", what the build corrected). A new fact is a
`RowFact` variant and a writer arm, never a new store: a fact a filter can
narrow is narrowed by `of` over the frames, so "noncreature spells you've
cast this turn" and "creatures with power 3 or greater that died this turn"
need no field.

**Every span is a read.** "This turn" is turn T's rows; "last turn" is T-1's,
whoever's turn it was (Paladin of Atonement's ruling); "since your last turn"
is every turn after your most recent one to have ended, this one included,
so on your own turn the one before (CR 730's day/night, Arboria, Concert
Kaboomist); "this game" is every turn's. `GameHistory::turns_of` maps a span
to its turns for every reader. CR 103's opening hands are drawn on turn 0,
which no span counts. *A game-scoped quantity is a scope on the log, not a
store*: Approach of the Second Sun's casts and CR 903.8's commander tax
(`cost-architecture.md` §3.8, waiting on designation) are counts over
"this game", each with its first card.

**The readers.** `Condition::ThisTurn`, `LastTurn`, `SinceYourLastTurn` and
`ThisGame` compare a `HistoryCount`; `AmountExpr::History` reads one as a
number ("the life you gained this turn", Betor, item 280);
`ObjectFilter::ThisTurn` and `ThisCombat` ask what an object did (attacked,
blocked, was blocked, entered, was dealt damage, dealt damage to a player);
a record's place in its turn, the targeting limit (CR 115.1's "first time
each turn") and a removed creature's defending player (CR 508.5) are lookups
on the same rows. A `HistoryCount`'s players resolve against the reader's
"you", so a static's `condition_reads` arm declares the source's controller.

**The gate.** The layer memo keys a history read by what it touches: a key
per kind of row, the turn number and the combat count
(`GatedWalkInput::Rows`, `TurnNumber`, `CombatCount`), noted inside the
lookup that scans them, so no caller can leave a dependency out. The writer
bumps the kinds it wrote; a turn's start bumps the turn number. The memo
audit re-walks fresh against each hit in a debug build.

**What it holds (item 179, restated 2026-10-19).** A clone copies one
pointer for the log, however long the game: the rows sit in chunks of
sixteen behind `Arc`s, newest first, and a fork's first write copies only
the open chunk. What a *game* holds now grows with its events, never with a
clone: 2.7–4.6 KB a turn over `history_held_test`'s 23 games, 378 KB in the
largest. Nothing truncates the log; if what a game holds ever needs to
shrink, an old row's frame can become a compact copy of what filters read,
and identical frames can be shared, each waiting for a measurement that asks.

**A departed player's rows stay (CR 800.4i).** "If an effect requires
information from the game about actions players have taken, the effect can
find actions that were taken by a player who has left the game." The log
keeps their rows, a read over every player (`PlayerSet::Everyone`) sums
them, and "an opponent" includes them. No test pins this yet: session 10
deferred CR 800.4i to Phase 9 with no atom.

**A count "before it" is a read by order (`cr-coverage-audit.md` §4a, pass
3).** Storm copies a spell "for each other spell that was cast before it
this turn" (CR 702.40a; 33 cards plus Thousand-Year Storm). A spell cast in
response is not before it, so the count is the turn's cast rows ordered
before the storm spell's own, by `seq`, readable at resolution with nothing
carried by the trigger. It is authored with the first storm card.

**A record is counted as its event ends (A6r, at #248's review).** That is
its window's dispatch, as before, for every record but one kind: a card of a
draw instruction is counted as that card's draw ends. CR 121.2 draws an
instruction's cards one at a time and 614.11a decides each after the last,
all inside one window, so a replacement asking "is this the draw step's
first card" of the second card must see the first counted. A simultaneous
batch is still counted once, at its end, and triggers are still matched
once per window. The draw step's count is a field beside the log, reset by a
draw step's `StepBegin`, because no turn can hold it: a turn can have two
draw steps (CR 500.8, Sphinx of the Second Sun). A draw's record carries its
place in the draw step beside its place in turn (`codebase-state.md` item
254).

**What a row does not keep.** A frame, not the store's facts beside it: a
filter leaf the frame cannot answer — whether it was a token, its counters,
the owner of an object the store has lost — reads the store by the row's id,
which answers for the object now (`codebase-state.md` item 286).

**The history before TR-5h** (TR-2a's three summaries per player, item 179's
bound by the table, TR-5b's one struct, the combat and targeting rows of
TR-5f and TR-5c) is in `plans/archive/triggers-architecture-landed.md`,
each under its phase, and TR-5h's design there says why the log replaced
them.

### 3.11 `Lki` — the last known information, widened (items 14, 15) — built in TR-4a

CR 603.10 says "the appearance of objects immediately prior to the
event"; CR 113.7a and 608.2h call the same fact "last known information",
and the tree has called the field `lki` since RA — so the type takes the
rules' name for the fact rather than for the look. Persist, undying, an
Aura's 603.6e trigger and the four "becomes unattached" Equipment each
read a **status**, which `EffectiveCharacteristics` deliberately does not
carry (it is the layer walk's output). So the type carries both.

**As built (TR-4a, 2026-10-14): one type, one body behind one `Arc`**
(`types/triggers.rs`). A hand-over clones one pointer, the record and every
entry that keeps it share one copy, and `GameEvent` keeps its 72 bytes. The
body is private, and the readers on the type are its whole surface:

```rust
pub struct Lki(Arc<LkiBody>);
struct LkiBody {
    object: ObjectRef,                     // the existence that left (CR 400.7)
    frame: Arc<EffectiveCharacteristics>,  // as it last existed, shared with the memo
    owner: PlayerId,                       // CR 108.3
    cost_choices: CostChoices,             // CR 707.10's decisions
    permanent: Option<PermanentFacts>,     // `None` off any other zone
}
struct PermanentFacts {
    tapped: bool,                          // CR 110.5
    counters: Vec<(CounterType, u32)>,     // CR 122, by kind in `CounterType`'s order
    attached_to: Option<ObjectId>,         // CR 301.5, 303.4
    attached_by: Vec<ObjectId>,
}
```

Each fact carries a reader that TR-4a built or one that TR-4b to TR-4d name:

| Reader on `Lki` | Who asks it | CR |
|---|---|---|
| `object()` | a frame candidate's identity (item 168), `carried_lki`, `CurrentOrLastKnown::existence` | 400.7 |
| `chars()`, `frame()` | leg 2's candidates, `subject_matches`, `bound_characteristics`, a damage source's reads | 603.10a, 608.2h |
| `owner()` | the filter table's `ByOwner` through `HeldFrame::LastKnown`; `owner_of_source` | 108.3 |
| `cost_choices()` | `SpellWasKicked`, for a source that left. CR 118's cost decisions only: kicked, bargained, evoked. A mode (CR 700.2) is the spell's entry's, TR-4d's `entry`, and a choice made as the permanent enters (Citadel Siege's "choose Khans or Dragons", CR 614.12) is a fact nothing records yet | 702.33d, 707.10 |
| `was_tapped()` | `SourceTapped`, `SourceUntapped` | 110.5 |
| `counters(kind)` | `CounterCount` of `ThisObject` or `TriggeringObject` (persist, undying); `AmountExpr::SourceCounters` | 122.8, 122.9, 702.79a, 702.93a |
| `host()` | `HostMatches`, for a source that left; a frame candidate's own host, the dispatcher's and the audit's | 303.4 |
| `had_attached(id)` | a look-back `TriggerSubject::Host` (item 252) | 603.6e, 603.10a |

**Left out, each until its reader arrives.** `damage_marked` and
`controller_since_turn`, which this section planned, have no reader in
TR-4a to TR-4d. `cast`, CR 400.7d's facts (the 2026-09-23 amendment's
Vibrance, "if {R}{R} was spent to cast it" after it left), has no condition
leaf that reads mana spent, and the card is unregistered. `entry`, a stack
object's copy for CR 603.10e and CV-4's `copy_of`, was built in TR-4d with
item 253: `last_known_information` keeps it for any object framed on the
stack, and `entry()` reads it.

**The capture.** `last_known_information` builds the whole body before the
batch performs (`capture_frame`); the frame is the memo hit it always was,
and the facts are copied off `PermanentState`; the cost decisions are
`GameState::cost_choices_of`'s, the one reader of them wherever the object
is: a permanent, a spell on the stack, or the resolving spell, whose entry
the resolution has taken. The dispatch audit's snapshots are `Lki`s too, so a
candidate the audit finds gone keeps its host and counters as the
dispatcher's frame candidate does. `DelayedTrigger.source_lki` replaced the
bare `source_frame`: kept as the source leaves, or seeded as the trigger is
made when its source has already gone, as a dies trigger's own has.

**The owner** (A6k's amendment, item 235's follow-up): carried and read.
`owner_now_or`'s fallback to the controller is gone. `owner_of_source`
reads the store by id, since ownership never changes (CR 108.3), and else
the resolving entry's kept `Lki`; a source with neither is a missed capture,
and the creation refuses. The filter table reads `ByOwner` off a held `Lki`,
so "a creature an opponent owns" answers for an object whose owner left
the game (CR 800.4a). `Token` still refuses a gone object: no reader asks it.

The two amendments below are the design record this was built from.

**Amended 2026-10-10 (the LKI rename, A6n, #241): one type, not two.** This
section planned `LastKnownInformation` as the frame inside TR-2b's pair,
`DepartedFrame { object, frame }`. The rename called the pair `Lki`, the
tree's spelling of the fact (`lki`, `lki_of`, `source_lki`), and a second
type spelled the same way would be two names for one concept
(`engineering-practices.md` §2b). So TR-4a widens `Lki` itself, and whether
the fields share the frame's `Arc` was TR-4a's to decide.
`DelayedTrigger.source_frame` becomes an `Lki` with it, or the widened facts
never reach a delayed trigger's source (§3.12a).

**Amended 2026-10-09 (A6k's review, item 235): the owner** (CR 108.3; CR
111.2 for a token). A delayed trigger or an "until" return stores its
source's owner as it is made, since "its owner" is fixed. A source already
gone from the store by then has only its last known information left to say
who owned it: a token that ceased to exist before its ability resolved (CR
704.5d), or an object whose owner has left the game (CR 800.4a).

`ZoneChange.lki` and `LeftTheGame.lki` carry an `Lki` since TR-2b, and the
new `ControlChanged.lki` will too, so widening the type reaches every
record. **The capture covers CR 603.10a's three classes** (item 15, built in
TR-4c): `zone_change_looks_back(from, to)` — from the battlefield, from a
graveyard, and into a hand or library from a zone all players could see, CR
603.10a's own words for the third class; a Brainstorm put-back from a hand
was never a shared fact, so it has nothing to look back at and stays `None`.
Leaving the game counts only from the battlefield (CR 603.6c): a card leaving
the game from a graveyard leaves no graveyard (Erebos's Titan's third
ruling). **The
reader methods live on the type** (the table above), and `TriggerBinding`'s
readers answer live when the object is still where the event left it and
from the frame otherwise (CR 608.2h, 113.7a); there is no second view type.
`triggering_object` is the one order, which `bound_characteristics` and a
condition's "it" both read: the record's frame when the matched event was
the subject's departure **and the arm looks back** (item 251's resolution
half: a "from anywhere" subject is read where it went), the live object
while it is still there, and the resolving entry's kept LKI after that
(§6.1). Item 3's "three ad-hoc reads in `sba.rs`" are two existence probes
today (`sba.rs:327`, `:425`, both `?` ahead of a subtype read) and not
frames; the item closes on the reader existing, not on those lines.

### 3.12 The stream's records — items 10 to 18, decided

Each of the survey's nine gaps, absorbed or declined:

| Item | Decision | Phase |
|---|---|---|
| **10** — three performers drop a proposal field | absorbed as three fields: `StepBegin.player` and `PhaseBegin.player` (`GameAction::BeginStep`/`BeginPhase` already carry it; the survey's 1,167 + 976 + 313 + 391 cards read "whose"); `DamageDealt.is_combat` (804 cards, and not live-derivable — a trigger resolving during the combat damage step deals noncombat damage); `LifeChanged.cause: LifeLossCause` (one card, CR 727.1a, and the field is free), which RH-1 folded into the stamp's cause, leaving the record its `source`. Literal sites: `StepBegin {` 5 in `src` and 12 in `tests`, `DamageDealt {` 10 and 7, `LifeChanged {` 10 and 9 | TR-1 |
| **11** — `AttackersDeclared` carries no defender | absorbed: `attackers: Vec<(ObjectId, AttackTarget)>` at the one emit site; `BlockersDeclared` becomes **one record per declaration step** carrying every defending player's pairs, because 509.3g's "attacks and isn't blocked" is a fact about the whole declaration and several defenders declare in turn | TR-5 |
| **12** — no event announces a target being chosen | absorbed: `GameEvent::Targeted { target: TargetRef, by: ObjectId, ability_source: Option<ObjectId>, controller: PlayerId, instances: u32 }`, one record per (spell or ability, distinct target), emitted where CR 601.2c's announcement is final — `cast_spell` (ahead of `SpellCast`, both at 601.2i), `activate_ability` once its costs are paid (ahead of `AbilityActivated`, which moved there: 602.2b runs 601.2i), and placement's 603.3d. `ability_source` names an ability durably, since `by` is the object CR 608.2n removes; the source's id and not the whole identity, which would grow `GameEvent` from 64 bytes to 88 | the record #231, for the dev GUI's log; the arm TR-5 |
| **13** — no event announces a control change | absorbed as **a sweep, not a write hook**: control is a computed value and "gains control" is a "becomes" on the walk's output, so the dispatcher's state check (§4.5) compares every permanent's effective controller against `PermanentState.announced_controller` — a materialized field with one meaning, the controller the stream last announced — gated on `RegistryScopeSummary.any_control_changing`, and emits `ControlChanged { object, from, to, lki }` per change. Every route lands there: the registry write, its expiry at cleanup, a stripped granting ability, a conditional row flipping. The sweep cannot tell which effect changed control, so its record's cause is a `GameRule` arm of its own, and a resolution that writes a control row runs it before its next instruction (§3.12a) | TR-4 |
| **14** — the frame carries no status | absorbed: `Lki`, widened (§3.11) | TR-4 |
| **15** — the frame is captured for battlefield departures only | absorbed: the three classes (§3.11) | TR-4 |
| **16** — no event for a prevention applying | absorbed: `GameEvent::DamagePrevented { target, amount, by: ReplacementInstanceId }` (no `source`, since one subject group can hold several, and the printed readers say "prevented this way", which `by` answers, §3.12a), emitted by `apply_rewrite`'s prevention leg once per prevention instance per subject group (CR 615.13's "each time a prevention effect is applied to one or more simultaneous damage events"); inside the batch, ahead of the `DamageDealt` it reduced. Not a performer emitting — the application is the event. The record came with #260, for the dev GUI's log, announced by `apply_replacements` where a subject group's prevented amounts are summed; the arm `DamageIsPrevented`, TR-5d | the record #260; the arm TR-5d |
| **17** — entry counters announce nothing | absorbed, the survey's question 4 decided: the `EnterBattlefield` performer announces one `CountersChanged` per `EntryCounters` row after `PermanentEnteredBattlefield`, inside the batch, with a new `by: Option<PlayerId>` on the record (CR 122.6a's putter, which the proposal has and the record dropped). Announced, never proposed, so CR 614.16's doublers see nothing twice; CR 122.7's "before" reads the live count minus `added`, which for an entry is zero | TR-5 |
| **18** — three variants never emitted | absorbed: `PhaseEnd`, `StepEnd`, `TurnEnd` deleted with their `format_event` arms. No printed trigger reads an end (511.2, 513.1a), and an arm the stream cannot carry misleads the reader. `CountersAnnihilated` is the fourth deletion, in TR-5, once CR 704.5q proposes (§3.3) | TR-1 |

Plus one record this design adds for its own use: **`AbilityTriggered {
seq, identity: TriggerOrigin, controller, caused_by: usize }`**, the event
CR 603.3b's second tier watches (§4.8). And one renamed, in TR-4b:
`EquipmentDetached` became `Unattached { attachment, former_host, route:
UnattachRoute }` (named for CR 701.3d's route since RH-1 gave "cause" to the
stamp, §3.12a), emitted only by `announce_unattached`, whose callers are the
`Attach` and `Unattach` arms and the two leave performers (question 5).

### 3.12a The records' contract — every kind, the binding and the frame (A6p)

*Designed 2026-10-12 in a pull request of its own (`roadmap-v2.md` A6p),
reviewed before any code. Docs only.*

**Where it sits.** On the event path, an event is proposed, decided (CR 616.1),
performed and recorded. The dispatch then matches the record (§4), a trigger's
binding copies it (§3.4), and the resolution reads it back (§6.3). Phase RH gave
the zone-change record a contract along that whole path
(`replacement-architecture.md` §2b): which object it names, what caused it,
what replaced it, and when it is read. This section asks the same questions of
the other 35 `GameEvent` kinds, of `TriggerBinding`, of `Lki` with the kept
LKI, and of `RememberedObject`. §3.12 decided each record's fields one gap at
a time; this is the whole surface read at once.

**The five questions.** Each cell of the table below answers one of them, for
one kind, and cites the CR rule that asks it.
1. **Which object.** Does the record name the existence (CR 400.7) each "that
   object", "it" or "them" reader needs, at the moment the CR fixes it (CR
   400.7e, 400.7j, 603.6c, 608.2h)?
2. **What was done, and what caused it.** These are two facts. Does the record
   answer every act word and every cause word its readers use, on every route
   the event can take? The routes are an effect, a cost, a special action (CR
   116), a turn-based action (CR 703), a state-based action (CR 704), a rule's
   move (CR 608.2), and an event a replacement itself makes (CR 614.13, 615.5).
   Does each field answer one question?
3. **What replaced it.** Can a reader that asks which replacement applied (CR
   614.5, 615.13) read it? Can one that needs the modified event (CR 614.6)?
4. **When it is read.** For every fact a reader takes live: the instant the CR
   fixes (CR 603.2, 603.3a, 603.4, 603.6a, 603.10, 608.2), and everything the
   engine does between the event and that read.
5. **As it left.** Does the record carry the last known information its
   readers need (CR 113.7a, 603.10, 608.2h)?

Each fact also goes through `cr-coverage-audit.md` §2's two checks:
- is it recorded when it exists, on the path production takes;
- does its shape survive a copy, a zone change, a control change and a
  replacement, per unit as well as in aggregate.

A cell says **holds**, or names its gap by the `codebase-state.md` item that
carries it, with the CR rule and the item's home. The item has the card count
with its query beside it. "Holds (item N)" is a gap that an earlier pass found
and homed, re-checked here against the tree after RH.

#### Calibration: the questions re-found the record RH fixed, on the second try

`cr-coverage-audit.md` §3 requires an instrument to re-find what is already
known before it sweeps. The known facts are items 175, 176 and 177.

Both runs were blind:
- A subagent worked in a detached worktree at `bf60f1f`, `main` before #242.
- `plans/` was deleted, the CR copied in, and the four source comments naming
  item 176 reworded.
- It was not told what RH changed, and ran no git history command.
- Each run read the zone-change record, its emitter and its readers, with no
  examples given.

| Run | Questions | Item 177 | Item 176 | Item 175 |
|---|---|---|---|---|
| 1 (9 minutes) | as the brief worded them | ✅ Q1: no arrival existence; the binding's epoch read at dispatch; a "one or more" binding keeps no subject | ½: Q3 found the dropped applied set (madness against Rest in Peace). Q2 judged the act and the cause to hold, since `ZoneChangeCause` and `EventStamp::resolution` told them apart | ❌ Q4 checked what each read is, not what happens between the event and the read |
| 2 (7 minutes) | Q2 and Q4 revised as above | ✅ Q1: a bare id, rebuilt live by four readers, three of them side channels | ✅ Q3 as run 1. Q2: "who caused it" was split between the act (destruction's two variants) and the stamp, whose `None` covered a turn-based action, a state-based action, a cost and combat damage alike; a cost's sacrifice recorded what an effect's did | ✅ Q4 traced phase 3: the riders ran inside the open batch, so every live read the dispatch made came after them (CR 615.5, 603.4) |

Run 1 missed one item and a half, so the questions were fixed before the
sweep, as §3 says:
- Q2 now names two facts, every route, and "one field, one question".
- Q4 now traces the code from the event to the read.

Run 2 re-found all three. Unprompted, it also drew the line that
`replacement-architecture.md` §2b.11 leaves open: an "instead ... and" rider
is part of one modified event (CR 614.6), and a prevention rider comes
"immediately afterward" (CR 615.5).

**Run 1 found two gaps nobody had filed, and both are in today's tree.**
"From anywhere" arms read the battlefield frame (item 251). A surviving
attachment's host is read after the event detached it (item 252). Neither came
from a known item, so the calibration also bought two findings.

#### The table

| Kind | 1. Which object | 2. Done, and caused | 3. Replaced | 4. When read | 5. As it left |
|---|---|---|---|---|---|
| `ZoneChange` | holds (`new_epoch`, §2b.5); **item 258**: a move into a hand or library is still found (CR 400.7e, 400.7j, 701.17c) → TR-4d | holds (`act`, the stamp's cause, §2b.2–3); **item 131**: a same-zone substitute leaves no record (CR 614.6, 701.17c) → TR-4d; a redirect's destination is the resolving spell's too (item 261, closed by A6r) | holds (`applied`, §2b.4) | holds at the dispatch (§2b.6); item 251, a non-look-back arm reading the frame (CR 603.6c, 603.10), closed by TR-4a | holds (item 14, closed by TR-4a; items 15 and 249, closed by TR-4c) |
| `LeftTheGame` | holds: it left the game, so there is no new object | holds: the leaving's cause | holds: a replaced loss leaves no record (CR 603.2g) | holds | holds: a graveyard card leaving the game leaves no graveyard (Erebos's Titan's third ruling; item 15, closed by TR-4c) |
| `Tapped`, `Untapped` | holds; "them" item 250 → TR-5a | holds: a cost's tap is `Cost` (crew) | holds: no reader | holds: transition-only (CR 603.2e) | holds: nothing left |
| `CardDrawn` | holds: the card in hand at the dispatch | holds: the draw's place in its turn and in its draw step, counted as the card's draw ends (`place_in_turn`, `place_in_draw_step`; item 254, closed by A6r) | holds: a replaced draw has no record (CR 603.2g) | holds: one batch per card (CR 121.6b) | holds |
| `ManaAdded` | holds | holds: `tapped_for_mana` (CR 106.12) | holds: the record is the modified production (CR 106.12b) | holds: resolves at the dispatch (CR 605.4a) | open: a source sacrificed for its own mana (decision 6) |
| `DamageDealt` | **item 255**: two objects, one subject (CR 603.2, 608.2h) → TR-5a; "them" item 250 → TR-5a | holds: `is_combat` and the cause (CR 510.2) | holds (item 16, closed by TR-5d) | holds; excess damage (CR 120.10), each record its share, decided with the batch (TR-5d) | holds (`source_lki`, CR 608.2h) |
| `DamagePrevented` | holds: the recipient; no source, since one subject's damage can come from several | holds: `by` is the effect, and the stamp's cause the prevention's own | holds: announced only for damage prevented, so none under CR 615.12 | holds: announced as the batch decides, ahead of what it performs, and dispatched at its close; the arm is `DamageIsPrevented` (TR-5d) | holds: nothing left |
| `PhaseBegin`, `StepBegin`, `TurnBegin` | holds: whose (item 10) | holds | holds: a skip leaves no record (CR 614.10) | holds | holds |
| `PermanentEnteredBattlefield` | holds; "them" item 250 → TR-5a | holds: `was_cast` off CR 400.7d's facts, `from` off the joined zone change (§4.4) | holds; entry counters item 17 → TR-5b | holds since RH-3 (CR 603.6a) | holds (the kept LKI, §6.1) |
| `LifeChanged` | holds | holds: per source (CR 119.9); RH-1 folded item 10's `LifeLossCause` into the cause | holds: the modified amount | holds | holds |
| `AttackersDeclared` | **item 255**: attacker and defender; defender item 11, "them" item 250 → TR-5a | holds: the active player declares (CR 508.1); one put onto the battlefield attacking never attacked (CR 508.4, item 128 → TR-5a) | holds: not replaceable | holds: CR 508.2a's instant is the dispatch | holds |
| `BlockersDeclared` | **item 255**: blocker and attacker → TR-5a; item 11: one record per declaration step, also when nobody blocks (CR 509.3g) → TR-5c | holds | holds | holds | holds |
| `SpellCast` | holds: the stack object is live | holds: a copy is not cast (CR 707.10) | holds | **item 257**: storm's "before it" (CR 702.40a) → CV-4 | holds |
| `AbilityResolved` | holds: the identity (CR 603.7h) | holds | holds | holds: the count's last step (§6.5) | holds |
| `AbilityActivated` | **item 256**: no stack object for "copy that ability" (CR 707.10) → CV-4 | holds | holds | holds | holds |
| `Targeted` | **item 255**: target and spell or ability (Frost Titan's "that spell") → TR-5a; the arm item 12 → TR-5c | holds: once per spell or ability (§14 question 11) | holds | holds: CR 601.2i | holds |
| `AbilityTriggered` | **item 256**: a placed trigger keeps no `TriggerSeq` → CV-4 | holds: `caused_by` (CR 603.3b) | holds | holds: dispatched as it is emitted | holds |
| `SpellCountered`, `AbilityCountered` | holds | holds: `countered_by` and the cause | holds: a redirected counter keeps its act (CR 608.2c) | holds | **item 253**: no frame for CR 603.10e → TR-4d |
| `SpellFizzled`, `AbilityFizzled` | holds: no reader (§3.3) | holds | holds | holds | holds |
| `PlayerLost` | holds | holds: why is the cause (CR 603.9) | holds | holds: the leaving's records share its window | holds: CR 603.10f reads the `LeftTheGame` frames |
| `PlayerWon` | holds: no reader | holds | holds | holds | holds |
| `Scried`, `LibraryShuffled` | holds | holds | holds | holds: `looked_at` taken at the event | holds |
| `CountersChanged` | holds; "them" item 250 → TR-5a | holds (item 17's putter → TR-5b) | holds | holds, with §3.3's `nth` corrected below → TR-5b | holds |
| `CountersAnnihilated` | deleted in TR-5b (item 6) | — | — | — | — |
| `Attached` | holds | holds | holds | holds (item 228: an Aura entering attached "becomes attached", Siona, Captain of the Pyleas' first ruling → TR-4b) | holds |
| `EquipmentDetached` | becomes `Unattached` in TR-4b, its field renamed below | — | — | — | holds (CR 603.10c's frame, item 14) |
| `TokenCreated` | holds: the token by id, in the zone it was made (CR 111.8) | holds | the definition it was made from, which a substitution changes (Divine Visitation): built in TR-5g | holds | holds |
| `TokenCeasedToExist` | holds: no reader | holds | holds | holds | holds |
| `DelayedTriggerCreated`, `UntilReturnMade` | holds: records for a log | holds | holds | holds | holds |
| `StateBasedActionPerformed` | holds: no reader | holds | holds | holds | holds |
| `TriggerBinding` | **item 255**: each occurrence keeps both of its record's objects; item 250 → TR-5a | holds | holds | holds (§6.3) | holds |
| `Lki`, the kept LKI | holds (`object`; item 168, closed by TR-4a) | holds | holds | item 252, a survivor's host, closed by TR-4a; item 259, a newcomer to a zone asked look-back arms, closed by TR-4c | holds (item 14, closed by TR-4a; item 15, closed by TR-4c; the type unified below) |
| `RememberedObject` | holds: by epoch, with `left_at`; item 260, an Aura's new object after its own CR 704.5m move (CR 400.7f), closed by TR-4c; item 258 → TR-4d | holds | holds | holds | holds |

All 36 kinds are in the table; kinds whose answers are the same share a row.

#### Why the cells that hold, hold

Most of the "holds" above rest on three facts the tree now has:
- **Every record names its cause** (RH-1, decision 11), so a cause word reads
  the stamp on every kind.
- **The dispatch runs immediately after the event** (RH-3, decision 5). A live
  read at the dispatch is the CR 603.4 instant for every kind, not only the
  zone change. Between a record and its dispatch the engine runs only the rest
  of its batch, which CR 704.3 and 603.6a make one event.
- **A replaced event leaves no record** (CR 603.2g), except the zone change,
  whose modified event is the record (CR 614.6).

The rest, row by row where it is not obvious:
- `PlayerLost`: CR 603.10f looks back, and the performer runs the player's
  leaving in the same batch. Leg 2 asks the frames the `LeftTheGame` records
  carry, and RH-2's `departs_with` snapshot.
- `ManaAdded`: CR 106.12a triggers "whenever such a mana ability resolves and
  produces mana", and the record is the production a replacement modified.
- `Targeted` names its target as an `ObjectRef` and its spell or ability by
  `by`. CV-4's copies and CR 115.7's changed targets each announce a new one,
  since a copy with new targets is a new targeting.
- `RememberedObject.left_at` is noted by the two movers whose record follows
  the move at once (`note_delayed_source_moving`). A same-zone move notes
  nothing, which is right while it makes no new object (CR 400.7).
- No printed delayed trigger reads an object it refers to after that object has
  left. `o:"at the beginning of the next" (o:"that creature's power" or
  o:"that creature's toughness" or o:"its power")` is 5 cards, and each reads
  its own target at resolution.

#### The gaps

Eleven new items, and one re-slotted, in route order. Item 261 came at #246's review. Counts are Scryfall's
`total_cards` for the query with `game:paper -is:funny`, run 2026-10-12.

- **Item 251 (TR-4a): "from anywhere" arms read the battlefield frame.**
  `occurrences_matching_arm` passes the record's `lki` to `subject_matches` for
  every `ZoneChange` arm. CR 603.6c's last sentence makes "from anywhere" no
  leaves-the-battlefield ability, and CR 603.10 reads its object after the
  move. So "a creature card is put into a graveyard from anywhere" refuses a
  land that was a creature card (Song of the Dryads), and matches an animated
  land. 34 cards: `(o:"when " or o:"whenever ") o:"from anywhere"
  (o:"creature card is put" or o:"creature cards are put" or o:"land card is
  put" or o:"land cards are put" or o:"a card is put" or o:"cards are put" or
  o:"artifact card is put" or o:"permanent card is put" or o:"nontoken creature
  is put")`. Found by calibration run 1. Compost registers for it: a Clone
  copying a black creature dies blue, and so does not trigger Compost.
  Gandalf the White's multiplier looks back at "an artifact ... leaving the
  battlefield" while the arm it doubles does not, which TR-5d's tests stress
  on one board (the owner, at #246's review).
- **Item 252 (TR-4a): a surviving attachment's host is read after the event
  detached it.** `TriggerSubject::Host` compares the candidate's host, which the
  dispatch reads live, and `cleanup_zone_state` detached the Aura or Equipment
  as its host left. So "When enchanted creature dies" and "Whenever equipped
  creature dies" never trigger. CR 603.10a makes them leaves-the-battlefield
  abilities, which look back. Item 14's `Status.attached_to` is the departing
  object's own host: it answers the Aura that leaves, not the Aura that stays.
  The departing host's frame keeps what was attached to it, and a look-back
  `Host` reads that. 81 cards: `(o:"enchanted creature dies" or o:"equipped
  creature dies" or o:"enchanted creature leaves" or o:"equipped creature
  leaves" or o:"enchanted permanent leaves" or o:"enchanted land leaves" or
  o:"enchanted creature is put into" or o:"enchanted permanent is put into" or
  o:"enchanted artifact is put into")`, the Zendikons among them; Demonic Vigor
  registers. Found by calibration run 1.
- **Item 259 (TR-4c): a newcomer to a zone is asked look-back arms of the event
  that brought it.** A Bridge from Below that reaches the graveyard in the same
  wipe is a live candidate there, and a live candidate answers every arm
  (`TriggerCandidateFrame::asks`). So it triggers on the creatures that died
  beside it. Its third ruling says it does not: "If Bridge from Below isn't in
  your graveyard immediately before a creature dies, its abilities won't
  trigger at all" (CR 603.10a). It is unreachable until item 173 lets Bridge
  function in a graveyard. 17 cards state the zone that way: `o:"if this card
  is in your graveyard" (o:whenever or o:"at the beginning" or o:"when ")`.
  Found by TR-4's re-count. Closed by TR-4c (§4.3).
- **Item 260 (TR-4c): an Aura's trigger does not find the Aura after its own
  CR 704.5m move.** CR 400.7f's second sentence lets "When enchanted creature
  dies" find the new object the Aura became in its owner's graveyard. Angelic
  Destiny's "return this card to its owner's hand" resolves after the
  state-based check moved it there. The source is found by identity, which
  that move ended. 17 cards: `(o:"enchanted creature dies" or o:"enchanted
  permanent dies" or o:"enchanted creature is put into" or o:"enchanted
  permanent leaves" or o:"enchanted creature leaves") o:"this card"`. It owes
  ATOM-400.7f-001 and ATOM-603.6e-001. §12 had given both to Rancor, whose
  trigger is on its own move (CR 400.7e, built in RH-3). Found by TR-4's
  re-count. Closed by TR-4c (§4.3).
- **Item 253 (TR-4d): the countered records carry no frame.** CR 603.10e looks
  back. `SpellCountered` and `AbilityCountered` carry no `lki`, `GameEvent::lki`
  reads only the zone change and the leaving, and a move from the stack is no
  CR 603.10a class. §3.11 planned the frame's `entry` for this look-back, and
  §6.1 says the `IsCountered` arm reads "the same frame", but no record carries
  it. 1 card: `(o:"when " or o:"whenever ") o:"spell" o:"is countered"
  -o:"countered this way"`, Multani's Presence, which TR-4d registers.
- **Item 254 (A6r), closed: a draw's place among its player's draws.** The
  history counts each card of a draw instruction as that card's draw ends,
  not at the window's dispatch (§3.10's amendment), and keeps the draws of the
  draw step in progress, reset when a draw step begins (CR 504). A doubled
  draw's second card is proposed inside the window that holds its first, so a
  count taken at dispatch would make it the step's first. The record carries
  its place in turn and in the draw step, and one `DrawFilter` (the turn's
  Nth, the draw step's first, or not the step's first) is asked by
  `EventPattern::DrawCard` of the proposed draw, off the history, and by
  `TriggerEvent::DrawsCard` of the record. When a
  replacement takes the step's own draw, the next card drawn in that step is
  its first, as miracle counts (CR 702.94a): Words of Worship then Opt under
  Teferi's Ageless Insight draws one card. `DrawCause` went. 78 triggers
  (`(o:"when " or o:"whenever ") o:draw (o:"second card" or o:"third card" or
  o:"first card")`), 3 more with "except the first" (Orcish Bowmasters, Leela,
  Xyris) and 7 replacements (`o:"would draw" (o:"second card" or o:"first
  card" or o:"except the first")`) read a draw's place.
- **Item 131, re-slotted to TR-4d: a same-zone substitute leaves no record.**
  RH did not close it, and its slot named RH. TR-4d gives a performed same-zone
  substitute a record of its own kind, which keeps the act, the applied set and
  the card as it was milled, and which act words read (decision 3). 26 cards
  read a mill:
  `(o:"when " or o:"whenever ") (o:"mills one or more" or o:"cards milled" or
  o:"milled this way")`.
- **Item 255 (TR-5a): a record names two objects, and the binding keeps one.**
  The combat and targeting records each name two parties:
  - the attacker and its defender (Hellrider's "the player or planeswalker
    it's attacking");
  - the blocker and the attacker (Loyal Sentry's "that creature");
  - the damage's source and its recipient;
  - the target and the spell or ability (Frost Titan's "that spell or
    ability").

  `TriggerEvent::subject` makes the damage arm's subject its source, and
  `subject_of` projects the recipient. So in "Whenever a creature you control
  deals combat damage to a player, put a +1/+1 counter on that creature",
  "that creature" binds nothing. 21 cards: `o:"deals combat damage to a
  player" o:"that creature"`, about ten of which mean the source; 96 with
  "it": `(o:"deals combat damage to a player" or o:"deals combat damage to an
  opponent") (o:"that creature" or o:" on it" or o:"untap it" or o:"return
  it")`. One facility with item 250: an occurrence keeps both of its record's
  objects as `ObjectRef`s at the dispatch.
- **Item 256 (CV-4): "copy that ability" has no stack object to copy.**
  `AbilityActivated` names the ability by identity, which two activations of
  one ability share. A placed trigger's stack entry keeps no `TriggerSeq`, so
  `AbilityTriggered.seq` finds nothing on the stack. `TriggerBinding.triggered_by`
  is never written. 15 cards: `(o:"when " or o:"whenever ") o:"copy that
  ability"`, Illusionist's Bracers and Aboleth Spawn among them. CV-4 builds
  the copy of an ability, and these are its readers.
- **Item 257 (CV-4): storm's count is planned and has no slot.** §3.10's
  amendment takes "each other spell that was cast before it this turn" (CR
  702.40a) at the `SpellCast` record, "authored with the first storm card". 33
  cards: `kw:storm`. A storm card copies spells, so CV-4 is the first phase
  that can register one, and the count goes in it.
- **Item 258 (TR-4d): CR 400.7j's, 400.7e's and 701.17c's "public zone" is not asked.** `GameState::find` answers in a hand or a library, so a
  trigger or an effect that moved a card there could still act on it. No
  printed card does:
  - the triggers that watch a move into a hand or library act on nothing they
    moved: 7 cards, `(o:"when " or o:"whenever ") (o:"put into your hand" or
    o:"put into a hand" or o:"put into your library" or o:"put into a library"
    or o:"put on top of")`;
  - the effects that return "that card" to a hand find it in a graveyard or in
    exile first: 25 cards, `(o:"owner's hand" or o:"owner's library") (o:"when
    that card" or o:"that card is" or o:"return that card" or o:"exile that
    card" or o:"cast that card")`.

  The rule turns on the zone. CR 701.17c finds a milled card "in the zone it
  moved to from the library, as long as that zone is a public zone". So a
  Darksteel Colossus shuffled back is not found, and Fateful Tempest's "total
  mana value of cards milled this way" does not count it (the owner's judge
  Discord, citing CR 701.17c), though it was milled (item 131). In the other
  direction, a milled card that Leyline of the Void exiled is found in exile,
  and `put_into_since` reads only the verb's zone. It goes beside item 131 in
  TR-4d, where the mill's same-zone record first asks the rule.

#### Planned shapes corrected here, with no item

Each is a sketch in this document, written before RH or before a rule reached
it. Its phase builds the corrected one.
- **`nth` reads the event, not the record** (§3.3's `CountersChanged` row,
  TR-5b; built there as the ability's condition over the counts the records
  keep, §12). CR 122.7 and the Sagas' CR 714.2b compare the count before the event
  with the count after it. "Count now minus `added`" is right only for the last
  record of a kind on an object, and two wither sources in one combat damage
  step write two records on one creature. So `nth` is the count now minus the
  window's `added` for that object and kind.
- **`Unattached` names its route, not a cause** (§3.12, TR-4b). Since RH-1,
  "cause" means `EventCause` (`replacement-architecture.md` §2b.3, decision 8).
  CR 701.3d's ways to become unattached are what was done, so the field is
  `route: UnattachRoute`, and the stamp carries the cause. Re-attaching to a
  new host is one of the routes (ATOM-603.10c-002), beside an unattach, the
  attachment leaving and the host leaving.
- **`ControlChanged` carries the frame before, its own cause, and an earlier
  moment** (§3.12 item 13, §15 item 4, TR-4d). Three corrections:
  - **The frame before.** "When you lose control of this" reads the controller
    the object had. A frame taken after the change names the new controller,
    whom CR 603.3a would make the trigger's controller, so the trigger never
    matches. §15 item 4 said no card could tell; 9 cards can: `(o:"when you
    lose control of" or o:"whenever you lose control of")`, Khârn the Betrayer
    among them.
  - **Its own cause.** The sweep cannot tell which effect changed control, so
    its record's cause is a `GameRule` arm of its own. No printed control
    trigger reads a cause: `(o:"when " or o:"whenever ") o:"gains control"
    o:"causes"` is 0.
  - **An earlier moment.** The sweep runs at the head of `execute_actions` as
    well. Act of Treason's change is then announced before its "untap that
    creature", which is CR 603.4's instant.
  - **Two more, found by TR-4d's re-read (§12).** The sweep also runs once
    after the last control-changing effect ends, or a steal's end goes
    unannounced. And the frame before the first control row is kept as that
    row is written (TR-4d's decision 1).
- **`DamagePrevented` keeps its instance and drops its source** (§3.12 item
  16, TR-5d). One subject group can hold several sources, as two attackers into
  a Fog do. The printed readers say "prevented this way", which the instance
  answers, and none reads the prevention's controller. §2b.4's
  `AppliedReplacement` would widen `GameEvent` past the 72 bytes
  `the_event_keeps_its_size` pins, so `by` stays a `ReplacementInstanceId`. It
  is the one record emitted while a batch decides, and its doc says so. Built
  with #260, its stamp's cause the prevention's own (`caused_by_replacement`).
- **`TokenCreated` carries the definition it was made from** (§3.3's
  `TokenCreated` row, built in TR-5g). Divine Visitation's substitution yields a
  creation that looks the same as an instructed one. So Ajani, Nacatl
  Avenger's "when you do" needs the record to say which definition made the
  token.
- **One type for last known information** (§3.11, built in TR-4a).
  `DelayedTrigger.source_frame` became `source_lki`, an `Lki`. Otherwise the
  widened facts never reach a delayed trigger's source.
- **S1's exemption is "it was in a public zone"** (§4.2, TR-4c). A frame
  candidate is exempt from CR 603.2f because it was visible. With CR 603.10a's
  second and third classes, that covers a graveyard card or a revealed one,
  not only a permanent. Built in TR-4c as the instant the list is read
  (`visible_as_read`, §4.2).

#### §3.12's decisions, re-checked after RH

| Item | Still holds? |
|---|---|
| 10 | Two of three fields hold. `LifeChanged.cause: LifeLossCause` is gone: RH-1 folded it into the stamp's cause and kept `source` |
| 11 | Holds; closed by TR-5c: one `BlockersDeclared` per declaration step, also when nobody blocks, with its unblocked attackers (CR 509.3g) |
| 12 | Holds; closed by TR-5c, which reads it |
| 13 | Holds, with the frame, the cause and the moment above |
| 14 | Holds, with item 252 beside it |
| 15 | Holds; closed by TR-4c, with items 259 and 260 |
| 16 | Holds without `source`, which one subject group cannot name |
| 17 | Holds. `by` is CR 122.6a's putter, a player, which is not the cause. Built in TR-5b |
| 18 | Holds. `CountersAnnihilated` went in TR-5b |

#### Decisions, and what is left open

Taken here, for the owner's review:
1. **The contract lives here, beside §3.12**, since the records are the
   trigger stream's. `replacement-architecture.md` §2b stays the zone change's,
   and this section points to it rather than restating it.
2. **`GameAction` is out of scope.** `replacement-architecture.md` §8a owns
   the proposal vocabulary's completeness, and this pass is about what is
   recorded and read. One proposal-side observation is left for §8a:
   `Primitive::CounterAbility` removes the stack object directly, with no
   proposal.
3. **A same-zone substitute gets a record of its own kind** (item 131). CR
   614.6 performs the modified event in place of the mill. A discard redirected
   to exile is still discarded (Wilt-Leaf Liege's ruling), and CR 701.17c finds
   a milled card "in the zone it moved to from the library". So Darksteel
   Colossus, shuffled back, was milled, and a count of milled cards reads 3 on
   the item's board. Not a `ZoneChange` with `from == to`: about 19 readers of
   `from` and `to` would each need an exclusion, since a move within a zone is
   no departure and makes no new object (CR 400.7). **Settled 2026-10-13** by
   Jess Dunks (WotC rules manager) on Zellix, Sanity Flayer under Leyline of
   the Void and under Wheel of Sun and Moon: "The cards are still milled"
   (`codebase-state.md` item 131). Zellix's "creature cards" reads what the
   card is, so the record carries the card as it was milled.
4. **TR-4 is four PRs and TR-5 is four** (§12, "Re-counted 2026-10-12").
   Every one of the four sub-phases the 2026-09-24 plan named crossed the band
   once its cards were hunted clause by clause and the multiplier applied, and
   each split on a rule seam with a consumer on each side.
5. **Storm and "copy that ability" go to CV-4**, the phase that can register
   their cards. Neither is reachable before it.
6. **A mana source sacrificed for its own mana holds, pending a ruling.** CR
   106.12a triggers as the ability "resolves and produces mana". Crystal Vein
   is gone by then, and CR 603.10's default reads the objects after the event,
   so "a land you control is tapped for mana" does not see it. No ruling says
   otherwise: Zendikar Resurgent's, Mana Flare's, Heartbeat of Spring's and
   seven more were read. 23 triggers read "tapped for mana" (`(o:"when " or
   o:"whenever ") o:"tapped for mana"`), and 180 lands sacrifice themselves for
   mana (`t:land o:"{T}, Sacrifice" o:"Add"`). If a judge reads it as a
   look-back, the record gains a `source_lki` as damage has.
7. **Item 254 was reachable and wrong, and A6r fixed it before TR-4** (the
   owner, 2026-10-13), with item 261 and the swallowed dispatch error of item
   262. The count is taken as each card is drawn, on both sides; the CR 103.5
   cause arm it was sized with was not needed, since the stamp's
   `GameRule::OpeningHand` already says it and a draw before any turn has no
   place in one.
8. **A redirected move's destination is the resolving spell's** (item 261;
   reversed at #246's review, 2026-10-13, and built in A6r). #242's review gave Hero of
   Bretagard's exile under Rest in Peace to the replacement. Judges gave it
   to the resolving Doom Blade, which causes "everything ... even if some of it
   gets replaced" (`replacement-architecture.md` §2b.4). A replaced event keeps
   its cause through the replacement too: an opponent's cycling ability, under
   your Plagiarize, still causes your draw for Unpredictable Cyclone.

### 3.13 The stack object

A placed trigger is the same ephemeral object an activated ability is:
`GameObject::new(card, controller, Zone::Stack)` with a fresh `ObjectId`, a
`StackEntry { is_spell: false, cast_from: None, ability_identity:
Some(..) for an object origin, chosen_targets from announce_targets,
effect: def.effect.clone(), trigger: Some(TriggerBinding) }`, pushed by
`set_stack_entry`. `StackEntry` gains one field, `trigger:
Option<TriggerBinding>` (13 literal sites in `src`, 5 in `tests`). A4o's
constraint holds by construction — `is_spell_on_stack` refuses it to
"counter target spell" — and `SelectionFilter::Ability`, the complement
Stifle's class wants, is a normal arm when its card comes. CR 603.3's "no
other characteristics": the object's `card_data` is the source's for
display and nothing reads its types (an ability on the stack has none —
`has_permanent_type` is asked only of spells).

---

## 4. Detection — the dispatch

### 4.1 When: the close of a batch, or an unbatched emission (question 8)

**A dispatch is the matcher's run over one window: every record stamped
with one `BatchId`, run when the outermost `execute_actions` that opened
that batch is about to return, after its riders; or one unbatched record,
run inside `emit_event` before it returns.** The window is
`records_since(mark)`, with `mark` the stream's sequence number at the
outermost `open_batch`.

**Amended 2026-10-10 (`replacement-architecture.md` §2b.6, decision 5):** a
window whose outermost batch queued riders is dispatched in two parts. What
the batch performed is dispatched before its riders run, and the riders'
records at its close, so every live read is taken immediately after the event
(item 175). Built in RH-3 (#245) as two batches: the batch closes and its
window is dispatched, and its riders then run as a new batch with its own
dispatch (`execute_riders`), so "one window, one batch id"
still holds.

**Where the records live, and when they go (the bounded-state PR,
2026-09-25; item 42).** `GameState.events` is an `EventWindow`. It holds each
record until the outermost dispatch that reads it has returned with no batch
open, then flushes it, to a recorder if one is attached
(`events::recorder`) and otherwise nowhere. Every other reader runs inside
that dispatch and flushes nothing:
- the tier-2 `AbilityTriggered` dispatch, which reads the record it names;
- a mana trigger's batch;
- an auxiliary batch's window, which closes mid-phase-1 of the batch it
  interrupts.

An entry's zone change, which `EntersBattlefield { from }` joins, is emitted
by the entry's own performer, so it is in the window whenever the entry is.
Priority is given outside every batch and dispatch, so at every priority
prompt the window is empty and a clone copies no record.

This is a deliberate refinement of "at `emit_event`, once per record", and
CR 603.6a is the rule that forces it: "each time an event puts one or more
permanents onto the battlefield, all permanents on the battlefield
(including the newcomers) are checked". The *event* is the batch. Two
permanents entering together each see the other's static abilities
(603.6b) before either is checked, and Humility entering beside a creature
strips that creature's ETB before it can trigger — a per-record match run as
each `PermanentEnteredBattlefield` is emitted has the first newcomer's
record before the second newcomer is on the board, and RC-5's finding
("applying an entry can move the board") is the same fact from the
replacement side. Batch close is also where CR 603.2c's "one or more" needs
no second pass, where lifelink's gain and its damage are one event (CR
120.3f, 120.4d — the reason a nested batch joins), and where 704.3's
"single event" of state-based actions is one event to a trigger.

**What it keeps from the resolved design.** The instant is still
synchronous with the mutation — before the next proposal, before the
state-based check, before anyone receives priority — so "that instant is
now" holds and the layer walk answers live. CR 603.8's example — "discard
your hand, then draw that many" — is two `execute_actions` calls and two
dispatches; the state trigger sees the empty hand between them (P1). CR
702.131d's "continuous effects are reapplied before the game checks"
happens because the batch bumped the layer epoch and the check walks
fresh. An `execute_actions_new_batch` (the `// AUXILIARY-MOVE:` exception)
is its own window and dispatches at its own close, mid-phase-1 of the
enclosing batch — which is what CR 614.13 means by "not a result of" the
entry: a devoured creature's death triggers before the entry's ETB, and
both are placed together at the next priority.

**Order within a dispatch.** Records are taken in window order, and for
each: its place in its turn is counted (§3.10), then the record is matched. So
"for the first time each turn" reads a count that includes this record and
asks whether it is one; two life losses in one batch are the first and the
second, which is Vengeful Warchief's answer. Then the state check (§4.5),
then the delayed and reflexive checks (§4.6), then mana triggers resolve
(§4.7), then every queued trigger's `AbilityTriggered` is emitted (§4.8) —
each an unbatched record with its own dispatch, which is how a tier-2
trigger sees its event.

### 4.2 Who is asked: the candidate set, the three-leg gate, S1, and CR 113.6k

For each record, the candidates are the union of:

1. **Every permanent's effective ability list**, in `battlefield_ids_ordered`
   order (the order is observable — 603.3b lets a player order their own
   triggers but the *default* order a prompt offers must be
   process-stable), through `get_effective_abilities` — a memo hit on every
   object the batch did not touch, and one board pass otherwise, which the
   state-based check that follows would have paid anyway.
2. **The frames the window carries**: for every `ZoneChange` or
   `LeftTheGame` with `lki: Some`, the departed object's ability list *as it
   was* — CR 603.10a's look-back, which is what lets an artifact with
   "whenever a creature dies" trigger twice in the wipe that killed it
   (ATOM-603.10a-001) and a creature's own dies-trigger fire (-002). A frame
   candidate is asked only look-back conditions (§4.3).
3. **Objects off the battlefield whose ability functions where they are**
   (CR 113.6b, 113.6k): `zone_trigger_sources: IdMap<ObjectId, Vec<AbilityId>>`,
   registered by `register_static_effects`' door at `arrive_in_zone`
   exactly as `zone_replacement_ability_sources` is (RF), with
   `zone_function::functioning_zones` gaining a `Triggered` arm that reads
   the condition: **113.6k is derived, not stated** — a condition about
   `ThisObject` moving from zone Z functions in Z ("when this card is put into a
   graveyard from anywhere" functions everywhere it can be; Guerrilla
   Tactics' discard, amended below); a condition about other
   objects functions on the battlefield unless 113.6b states otherwise.
   **Amended 2026-10-11 (RH-1):** the map loses the card as it moves, before
   the window's dispatch, so a condition about this object *leaving* such a
   zone is asked off the departure's frame, on leg 2 (`codebase-state.md`
   item 249, TR-4c).
   **Amended 2026-10-15 (TR-4c): not off a frame, and where it went.** A move
   CR 603.10a does not look back across is asked after the event (CR 603.10),
   so CR 113.6k's "zones it can trigger from" are the zones the move can leave
   the object in, a hidden one included: whether the card triggers there is CR
   603.2f's question, of the object (Library of Leng's board,
   `plans/atomic-tests/supplemental-docs/603-2f-complexity.md`, which Future
   Sight's revealed top card turns around). CR 702.29c says so of a cycling
   trigger, which fires "from whatever zone the card winds up in", and Yixlid
   Jailer's second ruling of Narcomoeba's, which fires from the graveyard
   where the Jailer stops it. Guerrilla Tactics' discard functions in the
   graveyard, in Rest in Peace's exile and in Library of Leng's library. A look-back move still functions
   in the zone it left, asked off the frame on leg 2. And an intervening "if"
   that states a zone replaces the derivation for every condition of its
   ability (CR 113.6b; Bridge from Below, item 173), in `functioning_zones` and
   `condition_functions_in` alike.
4. **The record's own subject**, wherever it is now — CR 603.10's default
   is "objects that exist immediately after an event", and a card
   discarded into a graveyard exists there with its printed abilities.
5. **The delayed registry** (§4.6) and **rule-owned abilities** (§3.8, none
   built).

**CR 113.6 is asked of each trigger condition, with a per-ability fast
path.** A candidate is in a sweep because *one* of its abilities functions
where it is, so the rest must still be asked. Dread in a graveyard is a
candidate for "when Dread is put into a graveyard from anywhere" and must not
be asked "whenever a creature deals damage to you" there; and because CR
113.6k's second sentence says "other trigger conditions of the same triggered
ability may function in different zones", the same holds one level down,
inside a single ability (Absolver Thrull's "enters or the creature it haunts
dies" is the CR's example). Two checks, one derivation in `zone_function`:
the pre-pass skips an ability none of whose conditions function in the
candidate's zone (`functions_in`, the union), and `match_def` skips a
condition that does not (`condition_functions_in`). On every leg, including
the battlefield, so the rule has one home — `replacement::gather`'s reason
for asking it the same way. (F1, found in review and fixed 2026-09-22: the
dispatcher read the zone map's *keys* and then the whole effective list.)

**The matcher's shape: candidates, rows, then records × rows.** The
candidates are each leg's objects, in CR 613.7 order, each with its
effective frame read once — a live object's off the layer memo, a departed
one's off the CR 603.10a frame its record carries (`TriggerCandidateFrame`).
The rows are one per triggered ability of each candidate
(`TriggerCandidateDef`), holding what does not depend on the record: CR
113.6's per-ability answer and the identity. Then every
record is asked of every row. The rows exist because the first cut was
records × candidates × abilities, recounting the ordinal by a prefix scan
and rebuilding the identity on every record (#16).

**The gate, three legs on each of two sets**, mirroring `gather` exactly
because `CLAUDE.md` says a new reader of the effective list is dead on every
board a gate skips: `trigger_sources: IdMap<ObjectId, EventKindMask>` —
permanents that *printed* a triggered ability or a trigger multiplier, each
against the record kinds its printed defs read (§11's lever, built
2026-09-22), written by `place_on_battlefield`, removed by
`cleanup_zone_state`, over-approximating in one direction only; `RegistryScopeSummary.unattributed_trigger_zones`,
one field for the Layer 6 grant and the copy (`copy-effects-architecture.md`
§4.7's leg) because the gate only ever reads them OR-ed; and the zone set
above. `puts_a_triggered_ability(def)` is the
predicate beside `puts_a_replacement_ability`. A dispatch on a board where
every set is empty and the delayed registry is empty returns after four
probes — the whole of what today's pools pay. **The battlefield probe is
source first**: it selects the sources whose masks meet the window's kinds
and orders only those, so the whole board is ordered only when the summary's
leg — a granted or copied trigger, which may be on any permanent — asks.
**Amended 2026-10-15 (TR-4c, item 266): the zone set's sibling.** An object
off the battlefield whose every functioning trigger is on itself changing
zones (`TriggerDef::triggers_only_on_this_object_moving`: Guerrilla
Tactics' discard, Guile's "from anywhere") can be triggered only by the
window that moved it, so it waits on `newcomer_trigger_sources`, and the gate
and the zone leg take it only as one of that window's newcomers
(`newcomer_trigger_carriers`). On the zone map it passed every window of a
game it was in from the library on: on `stress`, windows past the gate at
6.6× `main`'s.

**Why those four, and why four is enough.** A trigger can only come from a
triggered ability on some object's *effective* list, and an ability reaches
one of those by three routes — printed, granted, copied (`CLAUDE.md`'s three
legs) — on an object that is either still findable or gone inside this very
window. Cross the routes with the locations and every candidate falls in
exactly one probe: printed and on the battlefield is `trigger_sources`,
whose probe is the mask's — not "is any source present" but "does any source
read a kind this window carries";
printed and functioning where it is off the battlefield (CR 113.6k) is
`zone_trigger_sources`; granted or copied is the summary's zone set, because
neither printed set has ever heard of that object; and **departed** is the
frames the records carry, the one leg the other three cannot cover —
`cleanup_zone_state` took the object out of both sets as it left, so a board
whose only source just died would return at the gate and CR 603.10a's dies
trigger would be lost. Each is necessary by its own board (Soul Warden; Guile
in a graveyard; a Layer 6 grant of a triggered ability; any dies trigger), and
together they are sufficient **for the origins TR-1 ships** — `TriggerOrigin`
has one built arm. The two unbuilt ones are the gate's next legs, named here
so a later phase cannot forget them: the delayed registry (TR-3, §4.6) and the
state check (TR-6, §4.5), which has no event and so no window to gate on at
all. Over-approximation runs one way only — a probe may pass on a board where
nothing matches, which costs a walk that finds nothing.

**S1 — visibility (CR 603.2f).** "The object with that triggered ability is
at no time visible to all players" is asked per candidate at the instant its
list is read (`GameState::visible_as_read`, TR-4c): a frame's immediately
before the event (CR 603.10), in the zone the object left, and any other list
after it, of the object as the event left it, through `visible_to_all(game,
id)`, whose body is `zone.is_public() && !face_down` today and whose body is
`backlog.md` §2.9's to replace. Only a move in one of CR 603.10a's classes
carries a frame, and each leaves a public zone, so a frame always answers
yes: a creature returned to its owner's hidden hand still triggers its
leaves-the-battlefield ability. A library card asked about a creature
entering is refused (ATOM-603.2f-001), and a discarded card is asked in the
graveyard it went to, where all players see it. The 603-2f postscript's
reading is adopted: the instant is after the event, not the object's
history, so Coercion's reveal before the discard does not count and Future
Sight's revealed top card does. **Amended 2026-10-15 (TR-4c):** the brief
offered "is the object, where it is now, visible?" for every candidate.
Asked of a frame, that refuses the returned creature, whose fixture fails
under it, so a frame is asked where it was. `Zone::is_public()` gets its first caller
here and `backlog.md` §2.9 is not pulled forward.

### 4.3 Look-back (CR 603.10), and "from anywhere" (question 13)

A condition looks back — reads the frame for the ability's existence and
the object's appearance — iff it is on CR 603.10's list, and the list is a
property of the arm's fields, never a flag a card sets:

- `ZoneChange { from: Some(Battlefield) }` — leaves-the-battlefield, dies
  (603.10a's first class, 603.6c);
- `ZoneChange { from: Some(Graveyard) }` — leaves a graveyard (second class);
- `ZoneChange { to: Some(Hand) | Some(Library) }` — put into a hand or
  library from a public zone (third class), with `from` public or not asked:
  a move from a hidden zone carries no frame, and its subject is read as it is;
- `BecomesUnattached` (603.10c, TR-4b), `ControlChanges` (603.10d),
  `IsCountered` (603.10e), `PlayerLoses` (603.10f). 603.10b (phases out)
  waits for phasing; 603.10g (planeswalks) is out of scope. CR 603.10's "the
  existence of those abilities" reaches an unattach as it does a leaving: an
  Equipment that grants its creature an unattach trigger takes it away in
  the move that triggers it, so an `Attach` or `Unattach` of an attached
  ability-list source snapshots the lists before it (TR-4b).

**"From anywhere" is `from: None`, and `from: None` is not on the list.** CR
603.6c: an ability that triggers when a card is put into a zone "from
anywhere" is never a leaves-the-battlefield ability. So Guile's "when
Guile is put into a graveyard from anywhere" is matched against the board
*after* the move — it triggers from the graveyard even when Lignify took
the ability on the battlefield, and not when Yixlid Jailer takes it in the
graveyard (the survey's row, Guile's ruling) — which is the one case a
record carrying a frame is matched without one. The matcher's rule is the
sentence above, and the test is Guile's two boards with Yixlid Jailer
already registered.

For a look-back candidate the ability's controller (603.3a's "you") is the
frame's `controller`, and the "if" clause reads the frame where it names
the object (persist's "if it had no -1/-1 counters", off `Status.counters`).

**A source that survives the event** (main item 167, built 2026-09-22 in
the TR-1 review's theme E). CR 603.10's "existence of those abilities ...
immediately prior to the event" holds for every look-back source, not only
one that left: a Blood Artist that survives the wipe that took Humility had
no abilities before it, and triggers on none of the deaths. A survivor's
list can differ across the event only if the batch departs the source of a
row that writes ability lists — a copy, a Layer 6 grant or removal, or a Layer
4 type change, since CR 305.6–305.7 give and take abilities with a land's type
(`RegistryScopeSummary::ability_list_sources`) — so that is the trigger, asked
of the batch's *decided* members between deciding and performing: replacement
decides whether anything departs, and the lists are still the ones before. The
batch then keeps each look-back reader's frame, an `Arc` off the memo, as a
`LookBackSnapshot` over the records it performs — legs 1, 3 and 4, read before
a departure can end the grant leg 4 walks for — and the close asks look-back
arms of those frames and every other arm of the live list. **A nested batch
that joins the window takes its own**, and a record reads the outermost
snapshot whose batch performed it, since a nested batch inside a performer is
the enclosing event at finer grain (CR 704.3's one event); else the first
taken after it, since no list changed in between; else the live list, which is
then also the list before it. Both signs are fixtures: Humility beside a
surviving Blood Artist, and a grant whose source dies in the wipe, whose
carrier triggers for each death. Bridge from Below's graveyard half is the
zone map's, built in TR-4c with main item 173: under Yixlid Jailer, a wipe
that takes the Jailer gives the card its abilities back only after the event,
and it does not trigger, in either batch order.

**A source that leaves in the same event** (main item 174, TR-1b's first
commit). A departure record's frame is taken at the same point, between
deciding and performing, for the same
reason: taken as each member moved, a later member's frame showed an earlier
one gone — Blood Artist framed after Humility left had its ability back, and
an artifact March of the Machines animated, framed after March left, was no
creature. So the batch frames every permanent its decided members take off
the battlefield before any of them performs — every permanent when a player
leaves, since CR 800.4a's fourth clause decides its exiles only after its
first two — and the move reads that frame (`capture_lki`). A
destruction's move is a nested batch, and the permanent keeps the frame the
outer batch took: the event it is part of. Both halves are fixtures, each in
both batch orders.

**An object the event brought** (main item 259, TR-4c). A live candidate the
window made, a zone change's new object or a created token, was somewhere
else immediately before the event, so it answers no look-back arm of the
window (`newcomer_from`, read in `match_candidates`, which the audit
shares). The whole window, not
the records up to its move to the graveyard: a window is one batch and so one event
(§4.1). A Bridge from Below put into the graveyard by the wipe that killed the
creatures beside it does not trigger, its third ruling, in either batch order.

**An Aura's new object** (main item 260, TR-4c; CR 603.6e, 400.7f). An
Aura's trigger on its enchanted permanent leaving
(`TriggerEvent::watches_its_host_leave`) finds the card the Aura became in its
owner's graveyard: `TriggerBinding.source_found_in_graveyard`, set at the
dispatch when the Aura's move into the graveyard is in the window (the
rule's first sentence, the same event), and by the performer of CR 704.5m's
move into the graveyard, whose stamp names it, for each pending or stacked
trigger of that Aura (the second). Only the graveyard: an Aura a replacement
exiles instead (Leyline of the Void) was never put there, and stays where it
went. That is the CR's text; CR 400.7e and 603.6c would find it in the first
public zone it went to, and `engineering-practices.md` §3.4b's
`aura-found-in-graveyard` row keeps that reading one switch away. `this_object` reads it once the source itself is gone. Angelic
Destiny's "return this card" is the consumer; Rancor's "return it" is CR
400.7e's, read through the binding's subject since RH-3.

### 4.4 Multiplicity: per record, per window, and the multiplier

CR 603.2c's two answers, and the `multiplicity` field is which one an arm
gives. The *occurrence* it counts is one triggering event
`triggering_events_in` returns, below; the word is the CR's, and
`plans/glossary.md` carries it.

- **`PerOccurrence`** (the default): one trigger per occurrence, and the
  matcher's arm for each says what an occurrence is — a record for most
  kinds, a counter for the counter arms (Protean Hydra's ruling: several
  removed at once trigger that many times), an attacker or what is attacked
  for the attack readings (TR-5e, §12). A wipe of three lands is three `ZoneChange`s in one batch and
  three triggers (ATOM-603.2c-001); Soul Warden entering beside two
  creatures triggers twice (its ruling).
- **`OncePerEvent`** ("one or more"): one trigger per window in which any
  record matches; the binding holds every matching record,
  `TriggeringAmount` is `amount_of` summed over them, and `subject` is `None`.
  CR 603.2c's boundary is the `BatchId`, which the envelope was built to
  carry (`events/event.rs` says so in as many words).
- **Per subject** ("one or more ... on a creature"): one trigger per
  subject in the window, `TriggeringAmount` summed over that subject's
  records. Simic Ascendancy's "one or more" collapses the counters on one
  creature, not the creatures: counters put on three creatures at once are
  three occurrences (CR 603.2c). Added 2026-09-24; built in TR-5b as
  `Multiplicity::OncePerSubject`, which only an arm where one event can make
  several records about one subject may carry
  (`TriggerEvent::supports_once_per_subject`,
  `TriggerDef::readings_its_arms_cannot_give`). The
  damage arm is the next to need it: "whenever one or more creatures you
  control deal combat damage to a player" triggers once for each player
  dealt combat damage at once (Pyrewild Shaman's and Frostcliff Siege's
  rulings; 32 cards, Scryfall `o:"one or more creatures you control deal
  combat damage to a player"`), its subject the player. No registered card
  reads it yet; the first one does.
- **The join** for `EntersBattlefield { from, cast }`: the entry record and
  the same object's `ZoneChange` or `TokenCreated` are both in the window;
  the matcher reads `from` off the zone change and "cast" off
  `PermanentState.cast` (main item 9, CR 400.7d — who cast it and from
  which zone, two facts the permanent keeps, recorded by the entry
  performer off the stack entry the proposal's zone change came from;
  Coal Stoker reads "from your hand", Prized Amalgam "from your
  graveyard"), and a token has neither. No field is added to the entry record (question 9).
- **CR 603.2d's count** is a static ability, discovered off the effective
  list as the dispatch queues what a window triggered, behind its own gate:
  `Effect::TriggerMultiplier(Box<TriggerMultiplierDef { caused_by:
  Option<TriggerCondition>, abilities_of: ObjectFilter, additional: u32 }>)`.
  `caused_by` is a condition for Gandalf the White's two events and `None`
  for Roaming Throne's none; `abilities_of` names the doubled ability's
  permanent, its "you" the multiplier's controller. A multiplier adds when it
  is on the board the matched arm reads (after the event, or before it for an
  arm that looks back, through the frames the dispatch reads a trigger's
  source through), the source is a permanent it names as the dispatch read
  it, and a triggering record is its event, asked with its own referents and
  frames; an arm naming an act its event does not is another event (Teysa
  Karlov's fifth ruling). Panharmonicon's rulings are the edges: only the
  object's own triggered abilities, never 603.6d's entry statics or a
  replacement, never a delayed or reflexive trigger it creates; two
  Panharmonicons make three, not four (additive); each instance is its own
  `PendingTrigger` with its own `seq`, choices and targets, and a triggered
  mana ability resolves as many times. `TriggersOnlyOnceEachTurn` is read
  before the count, so a capped ability triggers once under Panharmonicon
  (the survey's judge-literature note); `DoThisOnlyOnceEachTurn` is read at
  the trigger and as each instance resolves, so both go on the stack and one
  acts. *Corrected at TR-5g:* this paragraph named `caused_by: TriggerEvent`
  and `source: ObjectFilter`.

### 4.5 State triggers (CR 603.8; P1; question 7)

`TriggerCondition::State(cond)` has no event. The check is a pass over the
candidates of §4.2 leg 1 and leg 3 whose def is a state trigger, evaluating
`cond` through `settled_holds` against the live board, at **three
schedules**: after every dispatch (§4.1 — the mid-resolution case, P1,
Emperor Crocodile's ruling); at every iteration of `perform_sba_and_triggers`
before placement, which is where a registry expiry at turn start, a row
dropped by `cleanup_zone_state`, and any epoch bump outside a batch are
seen (question 7 — the doc names what a dispatch is when nothing was
performed: it is this schedule); and inside the cleanup step's CR 514.3a
probe, so "at the beginning of the next cleanup step" and a state that
became true when "until end of turn" ended both reach the second cleanup.
CR 702.131d is free: each schedule reads frames the epoch invalidated.
The predicate is the same `Condition` a static's "as long as" and a
trigger's intervening "if" use, leaf for leaf: threshold's "seven or more
cards in your graveyard" is one `CardsInGraveyard` count whether it makes
an effect exist or a trigger fire; what differs is the instant it is asked
and what a true answer does.

**One-shot** (603.8's last sentences): a state trigger that has triggered is
in `state_triggers_armed_off: IdSet<AbilityIdentity>` until its stack
object leaves — resolution, counter, fizzle, or 603.3d's removal — and
`GameState::trigger_left_stack(identity)` at those four sites is the one
writer that re-arms it. Immortal Coil's two rulings (countered → triggers
again at once; resolved without losing → again) are the tests, on a fixture
carrying its third ability (§12), and that fixture beside Platinum Angel is
CR 104.4b's mandatory loop, which §4.9 catches. **Emperor Crocodile's
second ruling is the difference from an
intervening "if"**: a state trigger checks at the trigger only, so
`intervening_if` is `None` on a `State` condition and the matcher refuses
the pair.

**`ControlChanged` is announced by a sweep of its own** (item 13; designed
in §12's TR-4d section): for each permanent, compare
`get_effective_controller` against `PermanentState.announced_controller`;
on a difference emit the record and write the field. A field with one
meaning, the last announced controller; a "becomes" on a computed value,
detected where computed values are re-read. Its gate is
`any_control_changing` now or at the last sweep
(`control_effects_at_last_sweep`), since the summary is rebuilt as rows go,
and its points are TR-4d's rather than this check's.

**Amended 2026-10-21 (TR-6's design, read against #267's tree).** Five things
the sketch above leaves out or gets wrong.
- **Who is asked.** A state trigger reads no record, so its `record_kinds` is
  empty and `trigger_sources` never files its permanent. The check reads the
  dispatcher's own gate, all three legs of it (printed, granted, copied), through
  one reserved bit of `EventKindMask` that a `State` condition sets. A board with
  no state trigger then pays one mask test each time the check runs: after every
  dispatch, on every pass of `perform_sba_and_triggers`, and in the cleanup
  probe (the three schedules above). A granted or copied
  state trigger is found on the leg that finds a granted "whenever", which is
  `CLAUDE.md`'s rule for a new reader of the effective list. Leg 3
  (`zone_trigger_sources`) takes the bit the same way. The dispatch audit
  (§4.10) gains a state leg: every candidate's every `State` def asked with the
  gate off, and compared with the gated answer.
- **One re-arm site, not four.** Resolution (`stack.rs:118`, `:241`), fizzle
  (`:299`), countering (`resolve.rs:2370`) and CR 603.3d's removal
  (`placement.rs:201`) already share `GameState::ability_leaves_the_stack`
  (`stack.rs:264`). The re-arm is one line there, keyed on the entry's
  `TriggerOrigin` identity, and `trigger_left_stack` is not written. The sixth
  caller, an activation's rollback (`put_on_stack.rs:740`), only ever holds an
  activated ability, so it re-arms nothing. "If the object with the ability is
  still in the same zone" needs no code: the identity carries the source's
  `ObjectRef`, and a source that moved is a new object (CR 400.7) whose identity
  was never armed off.
- **Armed off when queued, not when placed.** CR 603.8's "doesn't trigger again
  until the ability has ... left the stack" holds while the trigger waits in the
  queue too. ATOM-603.8-001 asks it on the stack; every dispatch between the
  check and placement asks it of the queue.
- **What the check queues.** A `PendingTrigger` bound to no record, whose event
  roles are empty as a "when you don't" reflexive's are (§4.6). It is announced
  by `AbilityTriggered` after the check, so a tier-2 watcher sees it as it sees
  any other trigger (§4.8).
- **The predicate is the "as long as" evaluator, not a new one.** The check
  asks `condition::settled_holds` (`engine/layers/condition.rs:299`), which is
  `holds` at the walk's last layer. `holds` is what a conditional static's "as
  long as" is asked through inside the layer walk (LI-3), and `settled_holds`
  is what the replacement and restriction gathers, CR 613.11's cost effects and
  an intervening "if" ask outside it. Every one of those askers is a reader
  that already needs the answer: a static's "as long as" is asked when the
  object is read, a gather's when an event is proposed. Nothing reads a state
  trigger's condition, so the check is its asker, at the three schedules. The
  schedules are new; the evaluator is not. The leaves below are written once, in `holds`, so
  a static's "as long as you have no cards in hand" gets them too.
- **What the cards need besides the check.** `Condition::Not`, for Emperor
  Crocodile's "you control no other creatures" and Immortal Coil's "there are no
  cards in your graveyard". "Controls" is a `PlayerFact`, not a `Condition` arm,
  so the Crocodile's condition is `Not(Player { You, ControlsPermanent(creature
  and NotSource) })`. `PlayerFact::HandEmpty`, for CR 603.8's own example
  ("whenever you have no cards in hand", the board of both atoms; hellbent prints
  the same fact). And "then draw that many", read off the discards the
  resolution performed so far through `resolution_records` (§4.6), as
  `DamageDealtThisWay` reads damage, for ATOM-603.8-002's spell.

### 4.6 Delayed and reflexive (CR 603.7, 603.12; questions 1 and 14)

The registry's entries are candidates at every dispatch. A `Once` entry
that matches is removed as it queues (603.7b); if several records in one
window match it, the controller chooses which one causes it
(`ChoiceKind::ChooseDelayedTriggerEvent { delayed: DelayedTriggerId }`,
asked at the trigger because 603.7b puts the choice there, two or more
candidates only — Tatsumasa under a doubler is the corpus's board). A
`ThisTurn` entry matches every time and is dropped at cleanup with the
"until end of turn" rows. **603.7a and 513.2 need no code**: an entry
created after a `StepBegin` was performed cannot match it, because the
window is the future; a permanent entering during the end step sees no
`StepBegin { End }` until the next turn's, and a delayed "at the beginning
of the next end step" created during the end step likewise — the two
513.2 atoms assert the absence. 603.7c is the `refs` list: "exile it at the
beginning of the next end step" names an `ObjectRef { id, zone_change_epoch }`
and finds nothing if the epoch moved (CR 400.7), which is Sneak Attack's
ruling and Flickerwisp's board.

A **reflexive** trigger (603.12) is a delayed entry created as the
resolution reaches it and **checked immediately**: the window is the records
with that stamp up to `created.record`. *(Amended at TR-3b's review,
2026-10-08: this sketch named an `Effect::Reflexive { when, then }`, which
no code ever had. TR-3c builds it as `Primitive::CreateDelayedTrigger` with
`DelayedDuration::Reflexive`, its row in §12 says why.)* Its trigger event
reads what the resolution's own instructions performed: "when you do" is
the `ZoneChange { cause: Sacrificed }` the preceding `Optional` proposed; if
none matched, the entry is dropped silently. Heart-Piercer Manticore's first ruling is the shape ("goes on the
stack without a target ... a second ability triggers and you pick a
target") and its last is the count ("you can't sacrifice multiple creatures
to deal damage multiple times"). 603.12a's "one or more times" is
`OncePerEvent` over that window; its payment loop is CP-1's (§13).

**Amended 2026-09-23 (the rulings pass, G2).** The first text read the
window "since `created.record`", which contradicts both CR 603.12 — the
ability triggers "based on whether the trigger event or events occurred
earlier during the resolution of the spell or ability that created them" —
and this section's own example, whose sacrifice precedes the reflexive
entry. The window is every record the resolution has performed so far, and
it is one reader, `resolution_records(stamp)`. It is also where "this way"
and "that many" look (`trigger-survey.md` table three: 480 and 260 trigger
cards read the resolution's own events — "you gain life equal to the
damage dealt this way", "discard any number, then draw that many"); their
`AmountExpr` and `Condition` leaves come with their first cards and read
this reader, never a count kept beside it. A reflexive whose action is
making a token — Ajani, Nacatl Avenger, Generous Plunderer — needs
`CreatesToken` (§3.3), which is TR-5's; Ajani's two rulings are TR-5
fixtures, and the few such cards are not a reason to move the arm.

**As built (TR-3a, 2026-10-07).** The registry is a leg of every dispatch,
asked after the objects' and outside the audit's comparison: it has no
shortcut to check, since every entry reading a kind of the window is asked,
in creation order. Its arms run through the objects' matcher, which reads
`TriggerReferents` — "this object", "you", the owner, the host and "this
ability" — so a delayed trigger's "this creature" is its source remembered
by identity. Each record is matched as `match_def` matches an object's
ability (`match_delayed`): first arm, then the limit, then the intervening
"if".
CR 603.7b's choice is asked only between occurrences that disagree on what
the def reads (`entries_agree_on`, item 163's reading), and only with two or
more; its options name each event by its object, or by its record's number
where objects do not tell them apart. The one unbatched record that can
carry two occurrences of an arm, `AttackersDeclared`, dispatches with its
step's provider (`emit_event_with_provider`; `codebase-state.md` item 224,
closed at the review). "603.7a and 513.2 need no code" did not hold:
see §3.9's `created_in`.

**As built (TR-3b, 2026-10-08).** `resolution_records(stamp)` is
`EventWindow::resolution_records`, from a mark. The window used to flush at
every outermost batch's close, which emptied it before a later instruction
could read an earlier one's records, so it now holds a resolution's records
until the resolution ends. `Effect::Remember` is its first reader; the
reflexive check is TR-3c's.

**As built (TR-3c, 2026-10-09; "that action" at its review, 2026-10-10).**
The check is `trigger_reflexive`, through `create_delayed_trigger`. Its
window is "that action": the resolution's own records of the last
instruction before the trigger that took an action — an atom other than
making a delayed trigger, or a "you may" around one, declined or not — which
the resolution's walk keeps (`ResolutionWalk::last_action`). It is matched by
`occurrences_among`, the matcher a waiting entry's dispatch uses, without CR
603.7a's filter. "When you do"
(`ReflexiveForm::Does`) queues one trigger per matching record (603.12a), or
one for them all where its arm is "one or more"; "when you don't" (`Doesnt`)
queues one, bound to no record, when none matched. Both are queued and
announced as a dispatch queues a delayed trigger (`queue_pending`). CR 603.12
states the "doesn't" form and no printed reflexive trigger uses it — Olivia,
Crimson Bride's "when you don't control" is a state trigger (603.8), and
Heart of Bogardan's and Thought Lash's "when a player doesn't pay" are their
own triggered abilities — so it has a fixture. "When you do" after "you may
sacrifice" is `is_sacrificed`, a zone change off the battlefield caused by a
sacrifice, to any zone, since a replacement may send it elsewhere. The
window was first the whole resolution, which differs from "that action" only
when the resolution performs the arm's event twice; no printed card does,
but the CR's words are the customer, and the review closed it
(`codebase-state.md` item 241). "The first time each turn" reads the
record's own place (§3.5's amendment).

### 4.7 Triggered mana abilities (CR 605.1b, 605.4a; question 12; main item 11)

A `PendingTrigger` that `is_mana_ability` makes a mana ability, by its def
and the record that triggered it, is never queued: the
dispatcher resolves it **at once**, through the resolution path a mana
ability's effect already uses (`resolve_mana_effect`'s `ProduceMana`
proposal), with no stack object, no priority and no target (605.1b's first
criterion, checked by the derivation). CR 605.4a: "immediately after the
mana ability that triggered it" — the `ManaAdded` record's dispatch is that
instant, which is inside the CR 601.2g window when the mana was made there,
so Wild Growth's extra {G} is in the pool before the cost is paid. "Its
controller adds" is the enchanted land's controller: the effect's
`ProduceMana` names `PlayerRef::Host`'s controller, and the mana is added
to that pool whoever controls the Aura. The trace's `trigger` record says
`mana: true` and the stack never sees it.

### 4.8 The trace's sixth emit point (item 9), and `AbilityTriggered`

Two records, both behind `game.trace`: `trigger` per matcher decision —
the record index, the candidate's identity, matched or not, and which
predicate refused it (the visibility gate, the zone, the limit, the
condition, the intervening "if"); and `pending` at placement — the queue as
drained, tier by tier, the order chosen, the targets, the refusals.
`plans/traces/viewer.html` gains a summary arm for both in TR-1.

And one *performed* record: **`AbilityTriggered`**, emitted by the
dispatcher through `emit_event` for every queued trigger, after the window
it belongs to has closed (so it carries no batch and no resolution: it is a
consequence of the event, not part of it). It exists because CR 603.3b's
second tier is defined by "a trigger condition that is another ability
triggering", and Strict Proctor's ruling says that tier always goes on the
stack *after* the ability that caused it. The matcher dispatches it like
any unbatched record; a trigger watching it is tier 2 by construction
(`TriggerCondition::tier()`); a trigger watching a tier-2 trigger's
triggering is tier 2 too, ordered within the tier by APNAP and its
controller. No printed ability watches its own kind, which is necessary
and not sufficient once custom cards exist: a trigger watching abilities
triggering, whose own triggering it then watches, recurses synchronously
inside one dispatch, where TR-6's loop detector — which counts decisions —
cannot see it, so the nesting bound is the only guard for that loop.
**Amended 2026-09-22 by the owner (the TR-1 review, theme D): at the bound
the answer is CR 104.4b's draw, not an `Err`.** TR-6 settles
`GameResult::Draw` through the same settlement §4.9's detector uses, and
`DISPATCH_NESTING_LIMIT` (16 today, an `Err` until then, which since A6r
reaches the record's emitter: item 262) becomes that detector's threshold knob
rather than a second number.
`BATCH_NESTING_LIMIT` (32, `actions.rs:38`) is the same pattern but a
genuine engine invariant — CR 614.5 bounds every replacement chain — so it
stays an `Err`.

**Amended 2026-10-21 (TR-6's design, decision 5 — decided with (ii) at review).**
TR-6 builds the 2026-09-22 call's draw as written. At the bound the dispatcher
proposes §4.9's `GameEndsInADraw`, caused `GameRule::MandatoryLoop`, instead of
returning the `Err`, and its two doc comments (now `dispatch.rs:58–61` and
`game_state.rs:300–304`) lose "the engine's mistake". The bound itself stays at
16, a count the loop phase keeps until it can replace it. That phase has the
option B below; the draft's text follows as its input.

*The draft, revised at review: no second number.* The 2026-09-22 call stands:
when a tier-2 chain has no end, the answer is the draw. The review asked why a
bound is needed at all. The answer is
that `DISPATCH_NESTING_LIMIT` (16, now `dispatch.rs:62`) bounds a **call
depth**, not a loop. Each `AbilityTriggered` is dispatched inside the `dispatch`
that queued its trigger (`emit_record` → `dispatch_unbatched` → `dispatch`), so a
chain of tier-2 triggers is a chain of nested frames on the thread the game runs
on. The bound is there so that a chain with no end fails before the thread's
stack does. The deepest nesting the review's probe saw in 12,000 fuzz games is 3.
- **A — keep the recursion: two numbers** (this amendment's first version). The
  dispatcher draws at 16 and the detector at 500, both through one performer
  (§4.9, decision 4). One number would put 500 nested frames on the stack, or
  draw legal games at 16 (§4.9's table).
- **B — take the recursion out.** Step 5 of `dispatch_inner` pushes its queued
  triggers onto a work stack that the outermost dispatch drains. The drain pops
  one, emits and dispatches its `AbilityTriggered`, and pushes what that
  dispatch queued on top, which is the order the recursion visits them in.
  Nesting stops at two frames whatever the chain. A chain with no end is then a
  decision-free run the detector counts (§4.9, decision 1's fourth clause) under
  its one threshold. `DISPATCH_NESTING_LIMIT`, its `Err` and its two doc
  comments (now `dispatch.rs:58–61` and `game_state.rs:300–304`) go.

  The order matches the recursion's except where a dispatch nests inside
  another's step 4: a triggered mana ability's batch (CR 605.4a), whose tier-2
  triggers then wait for the outer dispatch's step 5. That moves their `seq`s,
  not their tier. Code: about 50 lines in `dispatch.rs`. Test: a fixture pair
  of triggers that each watch the other's triggering, drawn by the detector.

*Recommended for the loop phase: B.* It removes a number and a way to overflow
the stack, and it is what the 2026-09-22 call asked for. Under the loop phase's
principle (§4.9), an endless tier-2 chain is a run its proof reads like any
other.

`BATCH_NESTING_LIMIT` stays an `Err`, and is a different kind of number. No
legal board reaches it, since CR 614.5 bounds every replacement chain, so it is
an assertion about the engine, not a bound on play.

### 4.9 The mandatory loop (CR 104.4b, 731; P5; `backlog.md` §2.28)

The state-tracking doc's Tier 1 survives unchanged and lands here because
its first reachable board is a trigger's: Immortal Coil's third ability
with an empty graveyard beside Platinum Angel is "an involuntary infinite
loop ... the game will end in a draw" (the Coil's own ruling; the board is
a fixture, §12). `LoopDetector` on
`GameState` counts consecutive engine actions with no `DecisionProvider`
prompt of two or more options — A4e's definition of a decision, already
counted — and a run past the threshold (configurable, default 15) settles
`GameResult::Draw` through the same batch settlement CR 104.4a uses. A
decision resets it. A prompt with one legal answer, such as a priority
window where passing is all a player can do, is not a decision and does not
(corrected 2026-09-24: the count is item 138's decisions, never prompts).
Tiers 2 and 3 need the state hash item 40's discipline
makes possible and stay in `backlog.md` §2.28 with the fork harness; the voluntary
shortcut (D26) stays there too. `fuzz_games`' turn limit keeps standing in
for what Tier 1 cannot see.

**Amended 2026-10-21 (TR-6's design, decided at review): TR-6 builds the draw,
not the detector.** The paragraph above has TR-6 build Tier 1, a count of
decision-free actions that draws past a threshold. The review rejected a count
as the verdict, for three reasons:
- **Any threshold misjudges every finite run longer than itself.** That is what
  a count is, so a threshold can be defended as cheap, never as correct.
- **The boards a count confuses differ from the start.** Sanguine Bond and
  Exquisite Blood behind the opponent's Platinum Angel never stop, and with no
  Angel they stop when that opponent's life reaches 0. The Angel is on the board before the first
  iteration, and a check that reads what the loop moves and what could end it
  sees the difference; a count, which reads only how long the run has gone on,
  cannot.
- **No fuzz sitting can show a threshold safe.** The pools were built to
  exercise the engine, so their tails describe those decks, not Magic. A sitting
  can show a design wrong (the Protean Hydra below is a real card on a natural
  board), never right.

So TR-6 builds:
- **the draw** (decision 4, taken): `GameAction::GameEndsInADraw`, through the
  chokepoint;
- **the two counts the engine already has, ending the game through it,
  unchanged in number:** `MANDATORY_LOOP_CHECKS` (500 performing checks in one
  CR 704.3 repetition, `sba.rs:121`; item 263's board, Lich's Mirror's loop) and
  `DISPATCH_NESTING_LIMIT` (16 nested dispatches, §4.8, decision 5). TR-6 adds no
  number, and item 263 and `check_invariants.py:164`'s entry close.

**Loop detection is a phase of its own** (`backlog.md` §2.28 graduates; its
slot on the route is set at this review). Its principle, set at the review:
**the engine draws only when it has proved a loop cannot end.** A loop whose
iterations repeat with a constant change can be proved either way:
- Immortal Coil beside Platinum Angel changes nothing;
- Sanguine Bond and Exquisite Blood move two life totals by the same amount each
  iteration.

Comparing two iterations gives the change. Asking whether anything that reads
the changing quantities would ever answer differently gives the verdict: draw,
or play on to the iteration where something changes.

A run that never repeats cannot be decided in general. Churchill, Biderman and
Herrick ("Magic: The Gathering is Turing Complete", FUN 2021) build a
tournament-legal two-player board, every move forced, with no randomness or
hidden information, that runs a universal Turing machine. The first player wins
exactly when the machine halts, and otherwise, in the paper's words, "the game
has entered an unbreakable deterministic infinite loop, which is specified as a
draw by rule 104.4b". Only on such boards does the phase fall back to a count:
the owner expects the proof to decide "99% of cases", with a pure count "only in
the very weirdest". The phase's design starts with a census of the printed
mandatory loops by shape to measure that share. Its first test is the one TR-6
gives up: Immortal Coil's sixth ruling, the draw beside Platinum Angel. TR-6
tests the Coil's fourth and fifth rulings and the sixth's "it will immediately
trigger again".

What follows is the review's record: the probe, and decisions 1–3 as drafted,
**moved to the loop phase as its input**. Decision 4 is TR-6's; decision 5 is
§4.8's.

**The probe.** #267's merge with counters added, reset at decisions; 2,000
games a sitting at seed 12345; both pools at two seats and four, and both pools
on the Commander board (four seats, 100-card decks, 40 life). 12,000 games, all
legal and all finite. Each cell is the max (p99) of the game's longest
decision-free run. The right column is decision 1's B exactly, its first three
clauses, as the review's second probe measured it; it reads the same under
either of decision 3's reset rules:

| Sitting | Every resolution and performing check (A) | Refills (B) |
|---|---|---|
| performance, 2 seats | 19 (9) | 6 (4) |
| performance, 4 seats | **1,570** (15) | 11 (5) |
| stress, 2 seats | 19 (10) | 6 (4) |
| stress, 4 seats | 34 (13) | 14 (4) |
| Commander, performance | 44 (19) | 12 (6) |
| Commander, stress | 164 (20) | 45 (5) |

At a threshold of 15, the left column draws 139 of the 12,000 legal games; at
50, three; at 500, one. The right column draws one at 15 and none at 50. The
1,570 is seed 12587, the performance pool at four seats. A Protean Hydra at
1,569 +1/+1 counters ("Whenever a +1/+1 counter is removed from this creature,
put two +1/+1 counters on it at the beginning of the next end step") had
prevented 1,570 damage in one turn. Its 1,570 delayed triggers then resolved
one by one at the end step, with nothing to choose. The Hydra doubles each
turn, so no threshold bounds the left column in pooled play. A third reading,
the left column reset at every step's beginning as well, moved one tail (19 to
14, performance at two seats) and none of the others: the long runs are inside
one step.

**Decision 1 — what the run counts, and the threshold** (moved to the loop
phase). *Walked through at review (2026-10-21).*

*The problem, as first drafted, and the review's correction.* CR 104.4b's draw
is for a run "repeating a sequence of events with no way to stop". Sanguine Bond
("Whenever you gain life, target opponent loses that much life") and Exquisite
Blood ("Whenever an opponent loses life, you gain that much life") drain a
40-life opponent, and behind that opponent's Platinum Angel they never stop. The
draft said the two boards' first 40 iterations look alike, so Tier 1 must count
and draw past a threshold. That holds only for a count: the boards differ from
the start, and a check that reads them can tell (the amendment's head). What
follows is what a count would count, if the loop phase's fallback uses one. A
count is judged on two properties:
- **complete:** every infinite decision-free run passes the threshold, or that
  game never ends;
- **quiet:** every legal finite run stays far below it, or a legal game is
  drawn.

Its cost is the work per resolution, and what it adds to `GameState`, which
every fork copies. Today's bound is one half of A below: 500 performing
state-based checks in one CR 704.3 repetition (`MANDATORY_LOOP_CHECKS`,
`sba.rs:121`).

| | What counts | Complete | Quiet (longest legal run, 12,000 games) | Cost |
|---|---|---|---|---|
| **A** | every decision-free resolution and performing check | no: a placement loop does neither | **no**: 1,570, with no ceiling | an add per resolution |
| **B** | refills, the four clauses below | **yes**, by the argument below | **yes**: 45 | a compare per resolution, one `usize` |
| **C** | a resolution of an ability that has already resolved in the run | by object, no: a loop that makes a new object or delayed trigger each iteration never repeats one; by definition, yes | by object, yes: the Hydra's 1,570 delayed triggers are 1,570 registry entries; by definition, no: they are one definition | a set per run, which a fork copies |
| **D** | B, counting an iteration only when its records' kinds match the previous iteration's | for a loop of period one; a loop of two alternating iterations needs both kept | B's, less any refill chain whose iterations differ; Sanguine Bond's still counts | a `Vec` of kinds per iteration: an allocation per refill, and a clone cost |
| **E** | a cheap fingerprint (life totals, zone sizes, counters, stack depth) seen before in the run | no: an accumulating loop never repeats a fingerprint | no: two states with one fingerprint draw a legal game | a hash per refill |
| **F** | Tier 2 and 3, and D26's comparison: the whole game state repeated, or two iterations that match | for an exact repeat, with no threshold at all; an accumulating loop still needs one, as the cue to compare | yes | a hash over all of `GameState`; `backlog.md` §2.28 |

A, C and E each fail a property outright. D buys a little quiet for an
allocation per refill. F is the real answer, and is out of scope (§16). B is the
cheapest count with both properties.

**B, clause by clause.** A refill is:
1. **a resolution** at which the stack is at least as deep as at the run's
   previous resolution. The depth is the stack's alone: the pending queue is
   empty at every resolution, since placement comes before every priority
   grant;
2. **a performing state-based check** after the first in one CR 704.3
   repetition. Lich's Mirror's loop, item 263's board, is all checks and no
   resolutions;
3. **a placement pass** with no resolution since the previous pass. A trigger
   whose placement triggers another of its kind places forever without
   resolving: "whenever a creature becomes the target of an ability", beside a
   targeted trigger that fires on that targeting. Nothing bounds that loop
   today;
4. **a tier-2 dispatch** of an `AbilityTriggered` record whose trigger was
   itself queued from one: a trigger watching abilities triggering, set off by
   another such trigger. This clause exists if decision 5's B is taken; under
   its A the nesting bound stops that chain.

*Complete.* An infinite decision-free run does one of the four things
infinitely often. A stack's depth is a natural number and cannot fall forever,
so infinitely many resolutions include infinitely many refills. The other
clauses count every repetition after the first.

*Quiet.* A finite pile resolving counts once however deep: the Hydra's 1,570
count 1. What B still counts is a refill chain, where each resolution puts the
next trigger on the stack, and such a chain spends something each iteration: a
life total, a card, a creature. The probe measured clauses 1–3 exactly, the
right column above.

*Where B is wrong.* A refill chain that would have ended, but runs longer than
the threshold: Sanguine Bond and Exquisite Blood at two seats against an
opponent with more life than the threshold, or "whenever a creature dies,
destroy target creature" through a bigger board. The CR's answer is that the
chain ends; B draws it. A loop through the turn structure (every turn alike,
nobody able to act) does none of the four, and the turn limit stands in
(decision 3). F answers both.

*Code.* `LoopRun` (decision 2) gains the last depth. The sites are
`resolve_top_of_stack`, `check_state_based_actions_loop`,
`perform_sba_and_triggers` and, with decision 5's B, the dispatcher's tier-2
drain. The completeness argument goes in `LoopRun`'s doc comment, because
engine work that can repeat without doing one of the four needs a clause, and
that comment is where its author will look.

*The threshold, withdrawn at review.* The draft set it at 500, eleven times the
longest refill run measured (a Commander stress game's 108 Simic Ascendancy
triggers in one turn, seed 13407, turn 59, whose run counted 45). That rested
on the fuzz tails, which describe the pools' decks, and it misjudges every
finite refill chain longer than itself. In the loop phase a count is only the
fallback for a run the proof cannot decide, and B's four clauses are a
candidate for its unit: every infinite run reaches it, and no drain does.

**Decision 2 — where the count lives** (moved to the loop phase). A decision is
visible only in `ui::ask`, and its four validators (`ask.rs:162`, `:238`,
`:333`, `:398`) hold `&GameState`.
- **A — read `Diagnostics`' decision count** (the paragraph above: "already
  counted"). It adds no field. But a rule that ends the game would then read a
  measurement row, and `diagnostics.rs` promises its rows to the fixtures table
  ("reset per game", "monotonic"), not to a rule. A fork that someday resets its
  diagnostics would change its game's result.
- **B — a `LoopRun` on `GameState`:** the run's length and the last depth, in
  `Cell`s because the validators hold `&GameState`, as `Diagnostics`' do. It is
  bookkeeping the CR names nothing of, like `nesting`, and is cloned with the
  state. The validators reset it from the `only_pick(..).is_none()` they already
  count with, so the definition of a decision is shared and the counter is not.
  Decision 3 narrows which answers reset it. Clone cost: two words, no
  allocation, so floors 2 and 3 do not move at two seats or four.

*Recommended: B.* Item 40 keeps outcome-bearing state on `GameState`.
**Agreed at review (2026-10-21), for the loop phase: B.** Whatever state the
loop check keeps lives on `GameState`, and reads no measurement row.

**Decision 3 — what resets the run, and a step that never ends** (moved to the
loop phase). Two facts first.
- **Priority offers no mana abilities.** `can_begin_to_activate` refuses them
  (`mana_helpers.rs:298`), so a seat with only untapped lands is offered
  `[Pass]`, which is forced. A castable instant or a payable activated ability
  does make every priority window a decision.
- **CR 731.4–731.5 and Immortal Coil's sixth ruling** ("Unless a player
  disrupts it, the game will end in a draw") keep a player who could break a
  mandatory loop, and passes, inside it. No player can be forced to end a loop
  with an object outside it.

The options:
- **A — any decision resets the run:** any prompt with two or more legal
  answers, the definition the paragraph above borrows from the performance
  floors' unit (`codebase-state.md` item 138, where "decisions per core-second"
  is the throughput target, and `Diagnostics::record_decision`, which counts
  them). Beside a seat holding an instant, the Coil and the Angel never draw.
  The trigger is placed before every priority grant, so the stack never
  empties and the step never ends. `fuzz_games`' only guard is `max_turns`
  (`fuzz_games.rs:123`), so it hangs inside one `run_turn`, and "the turn limit
  keeps standing in" fails there.
- **B — a priority prompt answered `Pass` does not reset the run; any other
  answer does:** a cast, an activation, a target among several, an order, a
  "may". The probe: decision 1's B has the same tail either way (45). A's
  grows, 1,570 to 3,135, on the Hydra's game. Code: one reset in
  `ask_choose_priority_action` for a non-`Pass` answer; a priority prompt does
  not reset at the validator.

**The residual, either way.** A mandatory loop whose every iteration asks a
real choice resets each time, since Tier 1 cannot tell whether the choice could
break it: "target opponent" at four seats with every opponent behind a Platinum
Angel. It never draws. Tier 2's repetition test answers it (`backlog.md`
§2.28). A loop through the turn structure is the other residual: every turn
alike, nobody able to act, so none of decision 1's three counts. Tier 2's hash
at a quiescent point answers that one, and the turn limit stands in.

**The fuzz agent's stop exists already.** `RandomDecisionProvider` picks
uniformly among a window's actions that have not been rejected, and `Pass` is
always one of them. So it leaves an optional loop, TR-7's persist loop among
them, with probability at least 1/k a window, where k is the number of
actions. No agent bounds the first residual. So the harness gets a bound of its
own: a middleware that counts prompts within one step and raises the existing
`Stop` (`Game::until_stopped`) past a bound. `fuzz_games` reports it as a "stuck
in a step" row beside "Hit turn limit", never as a result. The bound is a
harness flag beside `--max-turns`, not an engine number: no rule reads it, and
a game it stops is reported, not ended. The review's second probe read the most
decisions asked in one step at 539 over the 12,000 games (the Mirrorweave game
again), with p99 at most 140 a sitting. The default sits well above that.

*Recommended: B, and the harness bound, with no new agent code.* TR-7's "a
fuzz agent that does" is the uniform pick, and TR-7 measures it on the persist
loop. **Moved with the rest at review (2026-10-21).** The reset rule is a
question about which iterations of a run are mandatory, which the loop phase's
proof asks too: CR 731.5 keeps a pass with a way out inside a mandatory loop.
The harness bound is needed once a registered card can close a loop the engine
cannot end. Until the loop phase lands no such card is registered: a pooled card
that hangs a game hangs CI's `stress` step, as item 264's token board does.

**Decision 4 — the draw goes through the chokepoint (`codebase-state.md`
item 263).** The paragraph above settles the draw "through the same batch
settlement CR 104.4a uses". It cannot. 104.4a's draw is read off performed
losses (`settle_game_result`), and a loop's draw has no performed event to be
read off. A settlement would be the direct write item 263 is about.

Proposed: `GameAction::GameEndsInADraw`, caused
`EventCause::GameRule(GameRule::MandatoryLoop)`.
- **The performer** is `end_game(GameResult::Draw)`, which announces
  `GameEvent::GameEndedInADraw`, so the log, the trace and the dev GUI say why
  the game ended. `PlayerWins`' performer is the precedent (`actions.rs:1948`).
- **No `EventPattern` arm,** as `ShuffleLibrary` and `Attach` have none: nothing
  printed replaces a draw. Platinum Angel's fourth ruling says an effect that
  makes the game a draw still works beside it, so no `is_prohibited` row
  refuses one either.
- **A draw is no loss,** so CR 603.9's "whenever a player loses the game" does
  not trigger ("unless that player leaves the game as the result of a draw").
- **A later "the game is a draw" effect** proposes the same arm with its
  effect's cause.

The state-based bound stays. It is a count the loop phase keeps for what it
cannot prove, and its branch now proposes the arm instead of writing the result,
which closes item 263 and `check_invariants.py:164`'s entry. *(Amended with
(ii): the draft had the bound go, its unit joining decision 1's count.)*

Cost: about 17 exhaustive arms, 10 on `GameAction` and 7 on `GameEvent`,
counted from `PlayerWins`' and `PlayerWon`'s. The names are for review:
`GameEndsInADraw` takes `PlayerWins`' shape, a subject and a present verb,
since "draw" alone reads as drawing a card.

*Recommended: the arm as proposed.* The alternative is the direct write.
**Taken at review (2026-10-21): a typed `GameAction` arm.** Its names are
reviewed with the build.

**The shortcut (D26) and the loop phase.** CR 731.2's procedure is
`backlog.md` §2.28's second half, with `GameNumber`. A player runs a loop and
proposes "N more times", and the others accept or shorten it. Its design is
`plans/atomic-tests/supplemental-docs/state-tracking-architecture.md`'s D26 and
`decision-provider-refactor.md` §9: run two iterations, declare a count. The
engine compares the two iterations' performed actions by structure, with
relative object references and no ids, and applies the net change N times.
Three seams with the loop phase, none of them built in TR-6:
- **The loop phase's proof is D26's comparison, run on two iterations nobody
  declared.** That design's Tier 1 only *flagged* a potential loop
  (`PotentialMandatoryLoop`). It routed a mandatory loop that accumulates to
  "fast-forward to termination or draw". Sanguine Bond and Exquisite Blood
  beside the opponent's Platinum Angel are that loop: each iteration moves two
  life totals, so no state repeats, and it never ends. Decision 1's refill rule
  is the part of the termination question that the stack alone can answer.
- **A declared shortcut is an action.** `decision-provider-refactor.md` §9.7
  makes it `PriorityAction::ProposeShortcut`, a priority answer other than
  `Pass`, so decision 3 resets the run on it with no new code.
- **The fuzz agent's leaving is CR 731.2a's "a loop that repeats a specified
  number of times", decided one window at a time.** With D26 an agent names the
  number once, through `DeclareLoopCount`.

CR 731.3's fragmented loop and 731.5's involved objects need the comparison
too, and stay in §2.28.

### 4.10 The dispatch audit — the dispatcher again, with its shortcuts off (TR-1b)

**Why it exists.** The dispatcher is an optimized answer to one question:
which triggered abilities does this window trigger? It answers through
shortcuts: a gate over record kinds, three candidate sets kept as objects
move (printed sources on the battlefield, the zone map, the zones a granting
or copying row reaches), a look-back snapshot taken only when a batch departs
an ability list's source (§4.3), and a departure's frame off its record.
Every wrong answer TR-1 has had lived there, not in a condition's
predicates: F1 asked a def where it does not function, item 167 read a
survivor's list after the event, and item 174 read a departing member's list
after an earlier member left. Every one was found by reading, because nothing
checks detection in a random game: the fuzzer's invariants are crashes, CR
117.5 and determinism, and a wrong trigger count passes all three. The audit
is that check.

**What it is.** In a run that enables it, every dispatch is answered twice by
the dispatcher's own matching loop, `match_candidates`: once over the
dispatcher's candidates, once over a set with no shortcut in it —

- every object that could carry a triggered ability, with its list now:
  every permanent and every spell, and elsewhere every object a continuous
  effect can reach or that printed one (an object no row reaches has its
  printed abilities and nothing else, and CR 604.3 keeps a CDA from adding
  one);
- and, for every batch of the window, the list each such object had before
  the batch performed: an ordinary `LookBackSnapshot`, taken at every batch
  where §4.3 takes one only when a source departs. A survivor's list from
  before answers the look-back arms of the records its batch performed; an
  object that has moved since answers them from its list then, under its
  controller then; every other arm reads the list now. A record no batch
  performed reads the list now for every arm.

The two answers are compared as multisets of triggers (source, ability, arm,
records, subject, controller), and a disagreement panics with the window and
what each side had that the other did not, which the fuzz harness reports
with the game's seed.

**What it checks, and what it cannot.** It checks the shortcuts: the gate,
the three candidate sets, when §4.3's snapshot is taken, and the departure
frames. Switched off one at a time, item 167's snapshot and item 174's frame
each panic on the forced Humility and Blood Artist board. It shares the loop
itself — CR 113.6 asked per ability, `match_def` and what it calls, and how
many triggers one ability makes of one event — so those are rules with their
own fixtures rather than the audit's: F1, a per-ability CR 113.6 bug, would
not show. It shares one reading too: which moment a record looks back to
(§4.3, the outermost batch that performed it) is CR 704.3 and 603.10
interpreted, not implemented.

**Its reads leave no trace.** The layer memo, the diagnostics and the trace
handle are saved before each audit read and restored after, so an audited
game's counters and trace are an unaudited one's: a whole-game test, and
every A/B sitting, whose counter runs are audited and whose timing rounds are
not. It costs about 2.2× the CPU per game at two seats, 2.3× at four and 2.7×
at Commander scale.

**Disagreements decided before it runs.** Item 174 was reachable on the pools,
so the audit would have fired on it; it is fixed in TR-1b's first commit.
Item 168 (a departed candidate's identity carries the post-move epoch) is a
convention the audit shares, so it does not fire. The snapshot's gaps that no
pooled card reaches — a conditional grant whose condition reads a departing
permanent, a row that arrives beside a look-back event — fire the day a pooled
card reaches them, which is the point. So does the one reading the audit makes
and the dispatcher does not: each record reads the objects as they were before
its own batch performed, where the dispatcher asks every record's LKI and
every live list of the window about every record in it (main item 175).

**The CR 305.7 route fired on 2026-09-24**, in a four-seat stress game dealt
from TR-2a's pool. Blood Moon beside Ashaya, Soul of the Wild strips every
creature Ashaya makes a land (CR 305.7). Ashaya itself is left 0/0 and dies.
Blood Artist was stripped before that death and restored after it, and the
dispatcher triggered it off the list it had after, where CR 603.10a reads the
one before. It is closed: a Layer 4 row's source is an ability-list source
(§4.3).

**Decided** by the owner on 2026-09-22, each as recommended: a runtime switch
(`fuzz_games --audit`, passed by `fuzz_ab.py`'s threaded counter runs) rather
than `debug_assertions`; every zone, libraries included, since the measured
cost did not say otherwise; item 174 fixed first, with no list of known
disagreements, since a known-disagreement list is how an audit rots; and the
dispatcher's counts as rows (`Windows past gate`, `Candidate visits`,
`Trigger matches`), the performance half of what the audit is for
correctness.

**Changed in review** (the owner, 2026-09-23). The audit was first built as a
second matcher beside the dispatcher: its own capture of every object in
every zone at every batch and every dispatch, its own pairing of each
object's two lists, its own CR 113.6 filter and fold. It came to 555 lines
against ~250 sized and 11–19× the CPU per game. The owner chose the form
above, which gives up checking the loop's rules on their own for a third of
the code and a sixth of the cost; the review also fixed the one thing the
second matcher had found in the loop, an ability matched through a
survivor's two lists triggering once per list (CR 603.2c). The first form's
sizes are in the archive, under TR-1b.

---

## 5. Placement — the stub, drained (CR 603.3)

### 5.1 The moment

`perform_sba_and_triggers` is CR 117.5 and 704.3 written out: state-based
actions until none, then triggers, then again until nothing is placed. Its
step 2 becomes `place_pending_triggers`, which also runs the state check
(§4.5) first, so a state that became true during the SBA batch is a
pending trigger before the queue drains. Every caller the stub has today is
a CR moment: the top of a priority round (117.5), after a resolution
(117.3b), after a cast or a non-mana activation (117.3c), and — through
`Game::run_turn_steps`' cleanup branch — CR 514.3a, whose "and/or any
triggered abilities are waiting" is `!pending_triggers.is_empty() ||
state_check()`. CR 502.4's untap-step triggers are held by construction
(nobody receives priority, so nothing drains) and go on the stack at the
upkeep's first grant (503.1a); 508.2b, 509.2a and 510.3a are the same
sentence at combat's steps. Eon Hub's two rulings (item 121) are tests of
exactly this: a skipped upkeep emits no `StepBegin`, and a trigger from the
untap step waits for the draw step's grant.

**The printed cards that close the two loops this moment runs** (CR 704.3's
next check, and what a resolution's events trigger), each class counted
beside its query and given a disposition, are
`plans/references/feedback-loops.md` §4 and §5.

### 5.2 APNAP over the seat list, two tiers, the ordering prompt and its elision

```
place_pending_triggers(game, dp):
  refuse every entry whose controller has left (§5.3)
  for tier in [1, 2]:                                   # CR 603.3b
    for player in seats ordered by apnap_index, in_game only:
      mine = pending[tier][player], in seq order         # process-stable default
      order = if mine.len() >= 2 && !elidable(mine):
                dp.choose_ordering(OrderTriggers { player, tier })
              else identity
      for t in mine[order]:
        modes (603.3c)  -> §5.4
        targets (603.3d) -> announce_targets; none legal -> removed, `pending` record
        push the stack object (§3.13); trace `pending`
  return placed > 0                                     # the loop's step 3
```

**APNAP is `apnap_index`** — active player first, then turn order over the
seats the game began with, departed seats skipped by `in_game` — the same
key CR 616.1's choices and CR 704.5j's legend groups already sort on, so a
four-seat game and a two-seat game are one code path. CR 405.3 ("two or more
objects on the stack at the same time ... APNAP ... that player chooses
their relative order") is this procedure's other name. CR 101.4d's restart
is not implemented, on the pipeline's precedent: placement creates no new
choice for an earlier player.

**The drain removes each entry from `pending_triggers` as it places it**,
and that is a design statement rather than a detail: the queue is then the
placement's progress record, so a `GameState` cloned at the ordering prompt
resumes by running `place_pending_triggers` again — item 40's invariant met
by construction, at the fork model's own boundary, a priority grant.

**The ordering prompt is `ChoiceKind::OrderTriggers { player, tier }`**,
subject `None` (it is about several objects, like `DeclareAttackers`), asked
through `choose_ordering` with the pending entries as `ChoiceOption`s in
`seq` order, two or more only. **Its elision is item 163's, as TR-2b
corrected it**: one player's entries give the same game in either order,
and the engine does not ask, when every one of these holds — each the
reason the prompt is asked otherwise:

- **Equal defs.** Different abilities do different things. Compared as
  defs, not ids, since the TR-1 review's theme E: two grants of one ability
  carry two ids (§3.6), and are still one ability twice.
- **Agreement on every fact the def reads** (TR-2b's decision 6), since
  "that creature" or "that much" is otherwise a different object or number,
  and "this creature" a different source. `TriggerDef::trigger_binding_reads()` walks
  the effect, the intervening "if" and CR 603.2h's gate, exhaustively, and
  names the columns: the subject, "that player", "that many", "its power"
  (the subject and the record its frame comes from), the source (the
  origin), and "this ability" (its gate and its count). A fact it does not
  read may differ: Soul Warden's two triggers for two Soldiers are one game.
  TR-1 compared the records and the subject whatever the def read, and no
  source.
- **No instance of "target".** CR 603.3d's choice is made per object as
  each goes on the stack, so the order decides who chooses against what.
- **No mode.** CR 603.3c's choice, the same way.
- **Tier 1.** A tier-2 entry's effect reads the stack it is put onto
  (CR 603.3b).

`trigger_order_cannot_change_outcome` is these conditions' conjunction and
says no more. They were written beside the predicate at first, on the
precedent of `ordering_cannot_change_outcome`, whose "expiry conditions"
are a different thing: the future code changes that break its proof, each
one a compile error. These are the predicate itself, so its reasons live
here (the owner's review of #178). A decorator's timestamp order for a
human under the toggle, and the agent's own for a bot, are `backlog.md`
§2.22's rows 8 and 9 and not the engine's.

### 5.3 CR 800.4d's second sentence (item 7)

"If a triggered ability that would be controlled by a player who has left
the game would be put onto the stack, it isn't put on the stack." One
`in_game(controller)` read at the head of `place_pending_triggers`; a refused
entry is dropped with a `pending` trace record saying why. The rule's own
example — Astral Slide's delayed trigger outliving its controller — is the
four-player fixture, on a board RE-7 can already build.

### 5.4 Modes, targets, and removal (CR 603.3c, 603.3d)

Targets are CR 601.2c through `announce_targets`, exactly the call
`activate_ability` makes, over `def.effect`'s stored instances (A4n's
walk, descending into `Effect::Triggered`); item 12's `Targeted` record is
emitted for each. "If a choice is required ... but no legal choices can be
made ... the ability is simply removed from the stack": the engine follows
that order literally — `place_one` creates the object, pushes it, announces
against it and undoes both when the announcement fails, the object having to
exist because targeting legality reads the source. It is the same game as
never creating it: nothing in between is proposed or emitted, and the writes
are an id, a timestamp and two layer-epoch bumps, none of which reaches an
outcome — while the two facts a removal could have disturbed were fixed
before it, CR 603.3a's controller at dispatch and CR 603.3b's order before
targets in the CR's own sequence, so a removed trigger consumed its slot as
it does on paper. Nothing is announced on the way out, because a trigger
removed this way is not countered (CR 701.6a is about a spell or ability on
the stack being canceled by something) and no printed trigger watches it. Modes (603.3c)
are `backlog.md` §2.7's: a `Modal` root on a trigger's effect resolves to
`Err` today and a modal trigger is unregistrable until `backlog.md` §2.7
lands; the removal rule for "no mode can be chosen" is that entry's to build on this
placement, and the two 603.3c atoms wait there with that reason (§13).

### 5.5 The controller (CR 603.3a), and what the stack object is not

The controller is locked at dispatch (`PendingTrigger.controller`) — the
player who controlled the source *when it triggered*, so a steal between
trigger and placement changes nothing (ATOM-603.3a-001), and a delayed
trigger's is 603.7d–g's. The object is created under that controller and
CR 110.2b never applies (an ability is not a spell). It is not cast from
anywhere (`cast_from: None`), it has no cost, and CR 601.2's rollback
sites never see it: the one failure placement can meet is "no legal
choices", which is a removal, not a rewind.

### 5.6 A prompt keeps what it has decided on `GameState`

**The rule (the owner, 2026-10-08, at A6k's review):** a prompt in
detection, placement or a CR 610.3 return keeps everything it has decided
and everything still owed on `GameState` while it asks, so that a clone
taken at the prompt resumes the same game by running the same code again.
That is `codebase-state.md` item 40's invariant, and the drain (§5.2) is its
shape: `pending_triggers` loses an entry only as that entry is placed, so the
queue is the placement's record of progress. Every prompt TR-4–TR-7 add
here, CR 603.3c's modes and CR 603.5's "may" among them, is written that
way.

Three prompts built before the rule break it, and they are item 233's, fixed
with item 40's violators at the first fork-based harness: CR 603.3d's targets
in `place_one`, CR 603.7b's choice of cause in `queue_delayed`, and CR
303.4f's host in `return_until`.

---

## 6. Resolution

### 6.1 The intervening "if" at both instants (CR 603.4, 608.2a), and the frame of what has left

At dispatch, `TriggerDef.intervening_if` is evaluated after the condition
matches and before the entry is queued; false means no trigger (ATOM-603.4-002).
At resolution, `resolve_taken` evaluates it again before anything else
(608.2a's place in 608.2's order — ATOM-608.2-001) and removes the object
doing nothing if it is false (-003). `Condition` is one enum for the static
"as long as", the intervening "if" and the state trigger, as
`layers-architecture.md` §13b decision 5 planned; a leaf is three edits.

**"You" is the ability's (CR 109.5; built in TR-2b).** Both instants, and a
resolving effect's own "if", evaluate through `settled_holds(condition,
game, source, locked_you)`, with the player named: the candidate's
controller at dispatch, the entry's controller locked by CR 603.3a at the
recheck, and the resolution's controller in `Effect::Conditional`. `you_for`
and `FilterPlayers::for_source` answer it before the source's frame, so a
source stolen since, or in its owner's graveyard, still reads its
controller's facts. A static ability passes `None`, and its "you" is its
source's current controller.

**The kept LKI (CR 113.7a, 608.2h; built in TR-2b).** A binding copies
its records at dispatch, so a departure after the dispatch is in no record the
entry holds. An `Lki` (§3.11) sits in a `kept_lki` list on
`PendingTrigger`, `StackEntry` and `ResolvingObject`. Placement moves the list
onto the stack entry, resolution moves it onto `resolving`, and an effect that
moves its own source adds one there.
- **One capture.** Before a batch performs, `capture_lki` frames
  every permanent its decided members take off the battlefield (item 174). It
  also frames every other mover that a pending, stacked or resolving entry
  names, from any zone: the entry's source, or `binding.subject`. On a
  player's leaving it frames the named objects that player owns. The names are
  gathered only for a batch that moves something other than a permanent, and
  on the common board they are empty or short.
- **One writer.** The two performers that move an object out of a zone,
  `perform_zone_change` and `owned_objects_leave`, call `hand_over_lki`
  before the move. It gives each entry naming the mover the frame its batch
  took, which for a permanent is the record's own `Arc`.
- **The reader.** `bound_characteristics` reads the record's frame when the
  event was the departure, the live object while its epoch holds, and the
  resolving entry's `kept_lki` after that. A miss there is a missed
  capture, and the refusal says so.
- **Rejected:** a `GameState` map keyed by `ObjectRef`, since every exit from
  the stack would have to prune it and a missed prune is the leak the
  bounded-state PR removed. Also rejected: writing at the window's close,
  since only a battlefield departure's record carries a frame.

**The frame's readers (built in TR-4a).** A condition is answered off the
board, but a triggered ability's "if" is often asked after "this" or "it" has
left it: persist asks of a creature that has just died. So the caller passes
what the condition's words refer to, `ConditionReferents`, and a static
ability passes `ConditionReferents::NONE`:
- `source_lki`, "this" as it last existed once it has left. The five leaves
  that read their source — `SourceUntapped`, `SourceTapped`, `CounterCount`
  of `ThisObject`, `SpellWasKicked` and `HostMatches` — read it when it is
  given and the live board otherwise. They had answered false once the source
  left (TR-2b's review, 2026-09-27, named three; RG added the other two), and
  a delayed trigger's "if" had read its source by id, so a source that left
  and came back answered as the new object (CR 400.7).
- `triggering_object`, "it": the triggering object with its `Lki` when the
  event was its departure or it has left since, in `bound_characteristics`'
  order. `Condition::CounterCount { object: TriggeringObject, counter, is:
  AtMost(0) }` is persist's and undying's "if it had no ... counters on it";
  the leaf's two axes, which object and how many, replaced a leaf per object
  at the review.

Four askers build it, one per instant. The dispatch takes the candidate's
frame and the record, and asks each occurrence apart with its own "it", as
resolution will (`triggers_whose_intervening_if_holds`, for an object's ability and a
delayed trigger alike): two attackers declared together are two triggers,
and a "one or more" arm's one occurrence has no "it", as its binding has
none. The CR 603.4 recheck and a resolving effect's own "if" take the
resolving entry's (`condition_objects`).
A delayed trigger takes its own `source_lki`, at the trigger and, through the
entry it queues, at resolution. A frame candidate's entry keeps its source's
`Lki` from the start (item 168), since the existence that triggered has
already gone and no hand-over reaches it. `HostMatches` reads a gone
source's host where it is, or as it left when it is the triggering object,
and a host that left otherwise has no answer, which reads as no; its filter's
"its owner" is the gone source's, off its `Lki` (CR 108.3).
`AmountExpr::SourceCounters` is the amount's half: "the number it had" (CR
122.8, 122.9), the source's counters as it last existed, put anew.

**Built in TR-4d.** The `IsCountered` look-back (CR 603.10e) reads the
countered spell's frame, which `SpellCountered` carries with `Lki.entry`
(item 253).

The two amendments below are the design record this was built from.

**Amended 2026-09-23 (the rulings pass, G1): an object that leaves after it
triggered.** The binding's frame exists only when the triggering event was
the object's departure. An enters trigger whose source is sacrificed before
it resolves has none: its entry record carries no frame, and the departure
is a later record no binding points at. CR 603.4 still rechecks, and CR
608.2h and 113.7a answer from "the object's last known information" — and
"the source can still perform the action even though it no longer exists".
So the departure writes the frame where the recheck finds it:
`capture_departure_frames` (§4.3) sets `departed: Vec<(ObjectRef,
Arc<frame>)>` on every `PendingTrigger` and `StackEntry` whose source or
bound object (`TriggerBinding.subject`) is the departing `ObjectRef`, the
frame type of the day (`EffectiveCharacteristics` until TR-4's
`LastKnownInformation`). The evaluator and §6.3's readers take an object's
facts from there once its epoch has moved. One writer, one meaning, and no
search of the log (§7). Vibrance is the case that found it, and it also
needs mana spent (recorded by type since 2026-09-23, `codebase-state.md`
item 30) and the frame's `cast` (§3.11); the common case is any "this creature deals damage equal
to its power" enters trigger answered by removal, which needs neither.

**Amended 2026-09-24 (`cr-coverage-audit.md` §4a, pass 4): from every zone.**
`capture_departure_frames` frames what leaves the battlefield, and TR-4 widens
that capture to CR 603.10a's three classes. CR 113.7a and 608.2h are scoped to
neither: they use last known information for any object gone from the zone it
was expected in, and two printed readers leave from zones no class covers.
God-Eternal Kefnet's trigger copies a revealed card, and its ruling copies
from last known information if the card leaves the hand first. Double
Vision's and Galvanic Iteration's rulings make their copy even if the spell
was countered first. So the `departed` frame is written wherever an object a
queued or stacked entry names leaves its zone: in `move_object`, gated on the
pending list or the stack naming the mover, which on the common board are
empty or short. A stack object's frame keeps its entry (§3.11), because CR
707.10 copies a spell's targets, modes, X and costs, and CV-4's `copy_of` has
no live entry to read once the spell has left. The `IsCountered` look-back
(CR 603.10e, §4.3's list) reads the same frame of the countered spell.
`codebase-state.md` item 169 carries the size.

### 6.2 "May" and "unless" (CR 603.5)

The ability goes on the stack regardless; the choice is at resolution
(ATOM-603.5-001). **Built in TR-2b** (main item 24). `Effect::Optional {
chooser, effect }` asks its chooser, `You` unless the text names another
player, through `ChoiceKind::ApplyOptionalEffect { source }`: a pick of none or
one, its subject the resolving object.

**The answer lives in the resolver's walk.** `ResolutionWalk` carries
`last_cost_answer: Option<CostAnswer>` beside the target cursor, and each
resolution makes it fresh, so a rider never sees its parent's answer.
- Each atom that takes an action answers `Does`, and a declined "may" answers
  `Doesnt`.
- A mandatory action that could not start answers `Cant`: a sacrifice with
  nothing to sacrifice, or of a permanent its player no longer controls (CR
  701.21a).
- `Condition::CostAnswer` reads it in `Effect::Conditional`, and the walk puts
  the answer back after each clause, so "if you do … if you don't …" read one
  answer.

It is never re-derived from the performed records. Wicked Guardian's ruling,
its damage prevented and the card still drawn, is ATOM-118.12-002's partial;
Standstill's board answers `Cant` (ATOM-118.12-001). CR 603.2h's gate is
written on `Does` alone (§6.4).

**What waits.** "Any player may … if a player does" asks each player in APNAP
order (CR 101.4) and is `Does` if any did; it is built with its first card.
Item 24's sized `IfYouDo { did, didnt }` is not built, since two conditionals
read one answer. "Unless [a player] pays [cost]" is a payment inside a
resolution, `cost-architecture.md` §3.10's shape and CP-1's slot; Strict
Proctor, Frost Titan and every ward wait for it (§13).

The two amendments below are the design record this was built from.

**Amended 2026-09-23 (the rulings pass, G3): the field records CR 118.12's
answer, not the event.** CR 118.12 makes the action a cost paid at
resolution, and its "If [a player] [does, doesn't, or can't]" clause
"checks whether the player chose to pay an optional cost or started to pay
a mandatory cost, regardless of what events actually occurred"; CR 118.11
keeps a modified payment paid. So `last_optional_taken` is
`last_cost_answer: Option<CostAnswer>`, `Does | Doesnt | Cant` in the
rule's words, written by the atom that takes the action and read by the
clause after it — never re-derived from the performed records. Wicked
Guardian's ruling is the board: its 2 damage prevented by protection, the
card is still drawn, where a reader asking the stream whether damage was
dealt would draw nothing. `Cant` is the mandatory form's failure —
ATOM-118.12-001's Standstill, exiled before its trigger resolves, is not
sacrificed and no one draws — and "if you can't" reads it (105 trigger
cards, `trigger-survey.md` table three). The choice names its chooser:
`Effect::Optional` carries a `PlayerRef`, `You` unless the text names
another, so "that player may ... if they do" (53) asks the right seat; "any
player may ... if a player does" asks each in APNAP order (CR 101.4) and
is `Does` if any did, built with its first card. ATOM-118.12-002 is this
paragraph's rule; its own board is Dermoplasm's morph under Gather
Specimens, which waits for Phase 8, so TR-2b covers it partially with
Wicked Guardian's.

**Amended 2026-09-24 (the owner, at TR-2's sizing): the answer lives in the
resolver's walk.** `ResolutionContext` is read-only down the effect tree, and
a field on it would mean editing its 56 literal sites. The walk already
carries one writable value for each resolution, the target cursor.
`last_cost_answer` sits beside it in a struct each resolution makes fresh. A
rider resolves separately, so it never sees its parent's answer. TR-2b builds
it with "may".

### 6.3 The bound facts, LKI, and CR 603.6's "unable to be found"

`EffectRecipient::TriggeringObject` resolves against `ObjectRef`: the
object at that id whose `zone_change_epoch` still equals the binding's. A
creature that died and was returned before the trigger resolves is a new
object and the recipient resolves to nothing — the atom "put a +1/+1 counter
on it" after a bounce (ATOM-603.6-001) and CR 603.6c's "checks for it only
in the first zone that it went to" (-001/-002), which is the same
comparison read the other way. `AmountExpr::TriggeringPower` and its
siblings read an `Lki` (§3.11): live if the object is where the event
left it, the frame otherwise — the record's when the event was the
departure, the entry's `kept_lki` when the object left afterwards
(§6.1). Nothing the effect reads is copied at
dispatch that the record does not already hold — the binding is indices
and one `Arc`.

### 6.4 "Do this only once each turn", written by the resolution (CR 603.2h)

For a def with `TriggerLimit::DoThisOnlyOnceEachTurn`, the resolver checks
`action_taken_this_turn` for the pair (the ability, its controller; §3.5)
before performing the effect: present means the
instance does nothing (Nykthos Paragon's fourth ruling — a second instance
on the stack resolves and no prompt is asked); absent means perform, then
insert only if the action was taken. A declined "may" leaves the gate open:
CR 603.2h asks whether the controller "has not yet taken the indicated
action", and Paragon's first and third rulings count only a choice to put
the counters (corrected 2026-09-24; built in TR-2b, which writes the gate
on the walk's `Does` alone). Two Paragons are two
identities and act twice (second ruling).

### 6.5 CR 603.7h's count, and the copy (S3, CR 707.10b)

`AbilityResolved` advances `GameState.resolutions_this_turn[identity]`.
The count is keyed by the ability, not by a player's row. Ashling the
Pilgrim's ruling says it doesn't matter who controlled the ability, and a
per-seat count would restart on a steal (TR-2a, 2026-09-24). The delayed form
("when this ability has resolved for the third time this turn") reads the
same count through `AbilityResolves { identity: ThisAbility }` (TR-3) plus a
count condition. The instance field is ignored by the count. A copy
of an ability (CV-4) carries the original's identity on its stack entry
and advances the same key — CR 707.10b's "the same ability" — and that
sentence is what this document owes CV-4.

**"If this is the Nth time this ability has resolved this turn"** — Omnath,
Locus of Creation's three branches, Ashling the Pilgrim's third — is not a
`TriggerLimit` but a branch inside the effect,
`Effect::Conditional(Condition::ResolvedThisTurn(n), ..)`, read at
resolution against the same field. Because the record that advances the
count is emitted as the resolution's last step (CR 608.2n), the leaf reads
count + 1 == n: Ashling's ruling counts the times the ability "has already
resolved", and Omnath's says it counts resolutions and not triggers and
has no effect past the third. Three edits for the leaf, shipped in TR-2a
with the count; the resolving identity rides on `ResolvingObject`.

### 6.6 CR 121.2c — each player, in APNAP order (S2; item 122)

Decided for item 122's first shape, generalized: **`EffectRecipient::EachPlayer(PlayerSet)`**,
a recipient any player-subject primitive accepts, resolved by the atom's
resolver as one proposal per player in `apnap_index` order over in-game
seats. "Each player draws a card" is `Atom(DrawCards(1), EachPlayer(All))`;
Alms Collector's rider is `EachPlayer` over a set naming you and the affected
player; CR 121.2c's "the active player performs all of their draws first"
is the loop, and CR 121.2d's shared-team-turns leg is a third arm on the
set when Phase 9 wants it. Rejected: an `Effect::Simultaneous` arm, because
it would be a second ordering rule beside the batch's for one reading, and
the recipient is what CR 608.2c's "in the order written" already varies
over. The customer is "whenever you draw a card" — the rule's first
gameplay reader — with Alms Collector already registered as the two-player
producer and Temple Bell as the plain one.

**As built (TR-2a, 2026-09-24; item 122 closed).** The recipient resolves to
the seats still in the game, in APNAP order, and passes them to the
primitive. `DrawCards` performs one instruction per player. `DealDamage`
performs one event with a member for each player, first needed by a
magecraft fixture's "each opponent".

The recipient is `EffectRecipient::EachOf(PlayerGroup)` (#186's review): the
players standing in a `PlayerSet` relation to "you", together with the
players the effect's context names (`NamedPlayers`). Alms Collector's rider
is `PlayerGroup::you_and_that_player()`: "that player" is the player its first
instance names, which for a rider is the replaced event's subject, and no
`PlayerSet` can say it, since a set answers membership from two ids. A new
printed phrase is a new `NamedPlayers` arm: Zurzoth's "you and those players"
is `You` with the players its trigger names.

Every other primitive refuses the recipient by name until a card needs
it. Item 94's per-player replacement rows are refused in
`CreateReplacement`. The "whenever you draw a card" reader is Psychosis
Crawler, in TR-2b.

---

## 7. The trackers — what is materialized, and what is never derived

"Not derived live" cuts both ways (`CLAUDE.md`; item 42): a condition that
scans even a short window of events is deriving CR state, and a stored
field must mean one thing. Every fact this design reads from the past is a
field with one writer:

| Fact | Field | Writer | Readers |
|---|---|---|---|
| "this turn" quantities, "last turn", "your last turn", "this game" | `PlayerHistory`'s two rows, total and snapshot (§3.10) | the dispatcher, record by record | `Condition::ThisTurn/LastTurn/SinceYourLastTurn/ThisGame`, `FirstTimeEachTurn` |
| the action was taken this turn (603.2h) | `action_taken_this_turn`, a set of `(AbilityIdentity, PlayerId)` — "its source's controller" (§3.5) | the resolution | the dispatcher, the resolution |
| the ability triggered this turn ("only once each turn") | `triggered_this_turn: IdSet<AbilityIdentity>` | the dispatcher | the dispatcher |
| a state trigger is on the stack (603.8) | `state_triggers_armed_off: IdSet<AbilityIdentity>` | the dispatcher (arm off), `trigger_left_stack` (re-arm) | the state check |
| resolutions per ability per turn (603.7h) | `GameState.resolutions_this_turn`, keyed by the ability (§6.5) | the history's writer, off `AbilityResolved` | `Condition::ResolvedThisTurn(n)` |
| the controller the stream last announced (item 13) | `PermanentState.announced_controller` | placement, the state check's sweep | the sweep |
| who cast this permanent, from which zone, and with what mana (400.7d; main items 9 and 30) | `PermanentState.cast: Option<CastFacts { by, from, mana_spent }>`, `None` for a copy of a spell (CR 707.10) | the entry performer, off `ResolvingObject.cast`, which resolution builds from the stack entry; `mana_spent` is written onto the entry at CR 601.2h | `EntersBattlefield { was_cast }`, "if you cast it", Coal Stoker's "from your hand", Prized Amalgam's "from your graveyard" |
| its cost decisions — kicked, bargained, evoked (707.10; main item 30) | `PermanentState.cost_choices: CostChoices { additional, alternative }`, kept by a copy of the spell and the token it becomes, and by its `Lki` once it leaves (§3.11) | the entry performer, off `ResolvingObject.cost_choices` | `Condition::SpellWasKicked`, "if it was kicked", "if its evoke cost was paid" |
| the object a delayed trigger refers to (603.7c) | `DelayedTrigger.refs: Vec<ObjectRef>` | the producer, from the records its instruction performed (§3.9) | the delayed check, the resolution |
| the answer to a cost paid at resolution (118.12: does, doesn't, can't) | `ResolutionWalk.last_cost_answer`, beside the target cursor (§6.2) | the atom that takes the action (`Does`), a declined "may" (`Doesnt`), a sacrifice that could not start (`Cant`) | `Condition::CostAnswer` in the clause after it, and CR 603.2h's gate (§6.2, §6.4) |
| an object's last known information after it left, for an entry that names it (113.7a, 608.2h) | `kept_lki` on `PendingTrigger`, `StackEntry` and `ResolvingObject` | `hand_over_lki`, with the frame `capture_lki` took | `bound_characteristics`, for §6.3's readers; the recheck's bound-fact form is TR-4's (§6.1) |
| when a delayed trigger was created (603.7a, 513.2) | `DelayedTrigger.created` | the producer | the reflexive window; nothing else needs it (§4.6) |
| which extra turn "that turn" is | `ExtraTurnId` on `turn_queue` entries and `GameState.current_turn_origin` | `Primitive::ExtraTurn`, `begin_turn` | `StepBegins { whose: Turn(id) }` |
| the trigger's event, subject, amount, frame | `TriggerBinding` (the matched records, copied whole; the matched event; the subject's epoch) | the dispatcher | the resolution, through the arm's projections |

The pending queue, the delayed registry, the histories and the four sets
are `GameState` fields, cloned with a fork. The window (`EventWindow`) is
read at dispatch and never later, since the outermost dispatch flushes it
(§4.1). A resolution that wants "what happened" reads its binding's records.

---

## 8. Engine interaction points — counted against the tree

| Site | Today | Change | Phase |
|---|---|---|---|
| `GameState::emit_event` (`state/trace.rs:485`) | one door, writes the trace's `event` record and the log | after the write: if the stamp has no batch, dispatch this record | TR-1 |
| `execute_actions` (`engine/actions.rs:647`) and `execute_actions_new_batch` (`:742`) | open, decide, perform, riders, close | after the riders of the *outermost* call: dispatch the window; `execute_actions_new_batch` dispatches its own | TR-1 |
| `perform_sba_and_triggers` (`engine/priority.rs`) | the stub | the state check, then `place_pending_triggers`; returns whether anything was placed | TR-1 |
| `Game::run_turn_steps`' cleanup branch (`state/game.rs`) | `check_state_based_actions` as the 514.3a probe | probe becomes SBAs-would-perform OR triggers-waiting OR state-check | TR-1 |
| `put_on_stack.rs::activate_ability` (`:437`) | builds the `StackEntry` with `is_spell: false` | `trigger: None`; the placement path builds its twin; `Targeted`, then `AbilityActivated`, once the costs are paid (#231) | TR-1, TR-5 |
| `stack.rs::resolve_taken` (`:120–230`) | resolves, announces `AbilityResolved` | the intervening "if" first (608.2a); `trigger_left_stack` for a state trigger; the binding threaded into `ResolutionContext` | TR-1, TR-6 |
| `stack.rs::handle_fizzle`, `resolve.rs`' `CounterAbility` arm, 603.3d's removal | remove the object | call `trigger_left_stack` | TR-6 |
| `resolve.rs` (`:220–240`) | `Conditional`, `Optional` → `Err` | `Conditional` resolves (its `Condition` through `settled_holds`); `Optional` asks; `Triggered` is refused outside a trigger's placement (a spell cannot carry one); `Reflexive` creates and checks | TR-1, TR-2, TR-3 |
| `oracle/characteristics.rs::get_effective_abilities` | three index readers | a fourth reader, the dispatcher, keying each def by its id (a granted instance's names its grant, §3.6) | TR-1 |
| `zone_function::functioning_zones` | six of fourteen subrules | the `Triggered` arm (113.6k, derived) | TR-1 |
| `register_static_effects` / `cleanup_zone_state` / `place_on_battlefield` | maintain the replacement gate sets | maintain `trigger_sources` and `zone_trigger_sources` beside them | TR-1 |
| `RegistryScopeSummary` | nine fields | `unattributed_trigger_zones` | TR-1 |
| `engine/turns.rs::begin_step` / `begin_phase` | propose with `player` | the record carries it (item 10) | TR-1 |
| `actions.rs`' `DealDamage`, `LoseLife` performers | drop `is_combat`, `cause` | carry them (item 10) | TR-1 |
| `mana.rs::activate_mana_ability` → `resolve_mana_effect` | activated mana abilities | the same path for a triggered one, entered from dispatch | TR-1 |
| `engine/zones.rs::perform_zone_change`, `leaving.rs::owned_objects_leave` | capture `EffectiveCharacteristics` for a battlefield departure | capture the widened `Lki`; widen the condition to the three classes, the third for a visible object only | TR-4 |
| `engine/combat/steps.rs` (`:86`, `:177`) | attackers, per-defender blockers | defenders on the record; one blockers record | TR-5 |
| `replacement/pipeline.rs::apply_replacements` | sums a subject group's prevented damage | announce `DamagePrevented` per recipient (`apply_rewrite`'s prevention legs report the amount) | #260 |
| `actions.rs` `EnterBattlefield` performer | announces the entry | one `CountersChanged` per entry row after it | TR-5 |
| `sba.rs`, the `Attach` performer, the zone-change performers | `EquipmentDetached` at two sites | `Unattached` from `announce_unattached`, called by the `Attach` and `Unattach` arms and the two leave performers; CR 704.5n, 704.5p and 704.5d as members of the state-based batch | TR-4b |
| `sba.rs` (`:484`–`:503`), CR 704.5q | writes both counter kinds directly, announces `CountersAnnihilated` | two `RemoveCounters` proposals in the state-based batch; the variant deleted | TR-5 |
| `resolve.rs` `Primitive::GainControl` | writes the Layer 2 row | unchanged — the sweep detects it | TR-4 |
| `ui/choice_types.rs`, `ui/ask.rs`, `ui/cli.rs`, `trace_records.rs`, `sba.rs` | five exhaustive `ChoiceKind` matches in `src`, four test files | one arm each per new kind (§9) | per phase |
| `ui/display.rs::format_event` | 34 arms | −3, +4, one renamed | TR-1, TR-4, TR-5 |
| `fuzz_games` | reads the record's resolution (`EventRecord::resolution`) for casts | a `Triggers placed` row and an `AbilityTriggered` count in `--dump-events` | TR-1 |

---

## 9. DecisionProvider surface

Every kind names its subject at birth (A4j) and is asked with two or more
options (`CLAUDE.md`); each is an arm in the five `src` matches above.

| `ChoiceKind` | Method | Subject | Rule | Phase |
|---|---|---|---|---|
| `OrderTriggers { player, tier }` | `choose_ordering` | `None` (several objects) | 603.3b; elided per item 163 | TR-1 |
| `ApplyOptionalEffect { source }` | `pick_n` (yes/no) | the resolving object | 603.5; main item 24 | TR-2b |
| `ChooseDelayedTriggerEvent { delayed }` | `pick_n` | the delayed trigger's source | 603.7b | TR-3 |
| `SelectRecipients` (existing) at placement | `pick_n` | the trigger's stack object | 603.3d → 601.2c | TR-1 |

The 603.3c mode prompt is `backlog.md` §2.7's and arrives with it. Nothing here adds a
trait method (`roadmap-v2.md` §9's watch item): the four methods suffice.

---

## 10. N players, and the permanent versus its components

**Every ordering is APNAP over the seat list**, through `apnap_index` and
`in_game`: placement (§5.2), `EachOf` recipients (§6.6), the delayed
trigger's simultaneous choice (the controller's, one player). Every "you"
resolves through a `PlayerId`: the controller locked at dispatch, the
frame's controller for a look-back, 603.7d–g's for a delayed trigger, and
`PlayerRef::Each` for "each player's upkeep". The histories are per seat
(`Vec<PlayerHistory>`), the gates key on identities that name no seat, and
800.4d is one read. A player who leaves takes their pending triggers with
them at the next placement and their delayed triggers stay registered and
are refused there too — Astral Slide's example.

**Written on today's single-component `PermanentState`, before CV-7 (CR
729).** Which reads key on the permanent and which on a component, so the
merging phase knows what it inherits: the identity's `source` is the
permanent — CR 729.2a gives a merged permanent one set of characteristics
(its topmost component's, or what the merging effect says: mutate's is
every component's abilities), so the effective list is the permanent's
and so is every identity read off it, and a component's own ordinal is not
a thing this design knows; `cast`, `announced_controller`, the gates, the
counters in `Status` and the histories key on the permanent;
`zone_change_epoch` is the permanent's, since CR 729.2c makes a merged
permanent "the same object that it was before". **One read is a
component's**: when a merged permanent leaves the battlefield "one
permanent leaves" and each component is put into its zone (CR 729.3), and
an effect that can find the new object "finds all of those objects"
(729.3c) — so a `TriggerBinding.subject` for "return it to the battlefield"
names *N* new objects, and an `Lki`, one frame, the permanent's, must
let CV-7 say which component's card each lookup means. That is CV-7's to
design; this document names it and keys nothing on it.

---

## 11. Performance — the sweep, the gate, the memo, and what each PR predicts

**The cost model.** A dispatch is: four set probes — a probe being one
hash-set `is_empty` or `contains`, nanoseconds and no allocation — for the
gate; if any is
non-empty, one `get_effective_abilities` per candidate (a memo hit for
every object the batch did not touch; one board pass for the rest, which
the SBA check after the batch shares), one `matches` per triggered def per
record in the window, one `settled_holds` per intervening "if" that
reached it. About 313 batches and ~700 records a game today, ~16
permanents a board, and — on the pools as they are — zero trigger sources,
so the pools measure the four probes and nothing else.

**The lever, built 2026-09-22** (the TR-1 review, theme C, after the reading
below asked for it): `trigger_sources` carries a per-source `EventKindMask`,
so a window of one kind visits only the objects whose printed conditions read
it. The window's kinds are OR-ed once; the sources whose masks intersect them
are selected, and only those are ordered (review round 1 — the first cut
ordered the whole battlefield and then probed each permanent's mask); a
window whose kinds no source reads returns at the gate. It narrows the battlefield leg and nothing
else — the granted and copied legs and the records' LKI read a list no
registration saw, so they keep their whole-list walk, and the zone map is
keyed by ability rather than by kind and keeps its `is_empty` probe. An
object whose every trigger is on itself moving is off the zone map since
TR-4c (§4.2's amendment), asked only as a newcomer.
`TriggerEvent::reads` is now written *from* the mask's table rather than
beside it, because a second table that had to agree with it is the bug this
lever would otherwise have shipped with.

### 11.1 The reading that asked for it, and what it bought

**The sitting, 2026-09-20 (before) and 2026-09-22 (after)** — `fuzz_games`
with an `Instant` around `GameState::dispatch` at the outer depth and three
diagnostics rows beside it, which is a throwaway build both times because §3
refuses to store a timer. The boards are `--require "Soul Warden,Blood
Artist,Wild Growth" --copies N`, which is what `--copies` exists for; at 8 it
is 24 of 36 nonland slots. `performance`, 200 games, seed 12345,
`--threads 1`, medians of three interleaved rounds for the milliseconds, and
the counters are exact.

| copies | seats | dispatch ms/game | share of CPU | past the gate | candidate visits | matches |
|---|---|---|---|---|---|---|
| 0 | 2 | 0.443 → **0.114** | 6% → **2%** | 243 → **25** | 297 → **30** | 2.7 → 2.7 |
| 8 | 2 | 2.878 → **1.209** | 20% → **10%** | 1,252 → **189** | 10,131 → **707** | 66.1 → 66.1 |
| 0 | 4 | 1.734 → **0.500** | 7% → **2%** | 660 → **71** | 888 → **106** | 6.1 → 6.1 |
| 8 | 4 | 10.963 → **4.583** | 21% → **10%** | 2,913 → **421** | 32,311 → **2,150** | 183.2 → 183.2 |

**Matches and `Triggers placed` are unchanged on every board**, which is the
claim that matters: the mask skipped nothing that could have matched. What it
skipped is what `match_def` would have refused with `Refusal::TriggerCondition`
after paying for a layer walk — the driver the reading named, fewer than 1%
of visits matching, and a gate that asked "is any source present" rather
than "does any source read this".

Whole-game CPU on the same sitting: 14.07 → 12.55 ms at eight copies and two
seats (**−11%**), 52.10 → 46.13 at four seats (**−11%**), and −0.4% / −2.7%
on the shipped pool, which is inside the sitting's spread. **The before-arm
numbers reproduce the 2026-09-20 reading's count columns exactly** — 243 /
297 / 2.7 at zero copies, 1,252 / 10,131 / 66.1 at eight — so the shipped
`--copies` is the retired `--stuff` probe's equal and the two readings are
one sitting.

**What the A/B says, and the one row of the gate that could not be met.**
`fuzz_ab.py` against `main`, both pools, two seats and four: every gameplay
counter, `Triggers placed`, `Decisions`, `Replacement gathers` and
`Restriction queries` are **IDENTICAL**, and so is an arm built at F1 plus
the flatten alone — byte-identical outside `=== Timing ===` on all four
combinations. The mask's arm moves six rows and only six: `Layer walks`,
`Board walks`, `Memo hits`, `Layer frames`, `Frames/walk`, `Dependency
checks`, all **down** (372 → 366, 64,786 → 64,524, 4,861 → 4,748 on
`performance` at two seats). Those are the cost-model rows
(`state/diagnostics.rs`), and a lever whose whole purpose is to stop asking a
source cannot leave them where they were — "IDENTICAL on every counter" is
unmeetable by construction for this change, and the F1-only arm is what
carries that claim instead.

### 11.2 The same probe at Commander scale, and what it says is next

Taken on review round 1's head (2026-09-22) to answer "is this enough for
v1?", with the probe widened to time `replacement::gather` and
`restriction::is_prohibited` too and to count each one's calls past its own
gate. Commander scale is four seats, 100-card decks, 40 life
(`fuzz_ab.py`'s documented board); `performance`, 200 games, seed 12345,
`--threads 1`, medians of three rounds; counts exact.

| board | CPU ms/game | dispatch | replacement gather | restriction check | triggers placed |
|---|---|---|---|---|---|
| 2 seats, 60 cards, shipped | 7.3 | 0.12 ms (2%) | 1.05 ms (14%) | 0.17 ms (2%) | 1.5 |
| 2 seats, 60 cards, 8 copies | 13.4 | 1.28 ms (10%) | 1.90 ms (14%) | 0.28 ms (2%) | 47.5 |
| Commander scale, shipped | 43.0 | 0.86 ms (2%) | 6.63 ms (15%) | 1.08 ms (3%) | 4.7 |
| Commander scale, 4 copies | 52.7 | 3.69 ms (7%) | 8.14 ms (15%) | 1.30 ms (2%) | 32.3 |
| Commander scale, 8 copies | 57.2 | 4.98 ms (9%) | 8.99 ms (16%) | 1.38 ms (2%) | 80.4 |

| board | past each gate, dispatch / gather / restriction | of all calls, gather / restriction | dispatch candidate visits / matches |
|---|---|---|---|
| 2 seats, 60 cards, shipped | 25 / 1,017 / 72 | 1,133 / 1,135 | 30 / 2.7 |
| 2 seats, 60 cards, 8 copies | 189 / 1,317 / 108 | 1,761 / 1,763 | 707 / 66.1 |
| Commander scale, shipped | 106 / 3,461 / 352 | 3,487 / 3,493 | 141 / 8.3 |
| Commander scale, 4 copies | 360 / 3,945 / 398 | 4,017 / 4,023 | 938 / 50.0 |
| Commander scale, 8 copies | 448 / 4,319 / 430 | 4,452 / 4,459 | 1,680 / 111.9 |

**What it says.** There is no absolute rate to be "enough" against — §3.1's
ratchet is the owner's deliberate choice while the pools understate
Commander — so this reads what is left, not whether it suffices. **The
dispatcher is 2% of CPU at Commander scale on the shipped pool and 7–9% with
12–24 trigger cards per deck**, where it was a fifth of CPU on the 60-card
eight-copy boards before the lever. What
still reaches it is dominated by `ManaAdded` and `ZoneChange` windows
(187–205 and 169–238 a game at Commander scale with the three forced; 67 and
38 on the shipped pool): Wild Growth reads every mana add and Blood Artist
every zone change, and the kind cannot tell a death from a draw or Wild
Growth's own land from another. Sub-kind keys — a zone
change's from and to, a mana watcher's host — are the next dispatcher lever,
and at 5–7% of visits matching they are worth less than the one beside it.

**The replacement gather is the bigger lever, and it has the dispatcher's
old shape.** It is 14–16% of CPU on every board, flat in trigger density, and
it passes its gate on **90–99% of calls** — 3,461 of 3,487 at Commander
scale — because its gate asks "is any replacement source present" and twelve
pooled cards carry one, a tapland among them. Past the gate it orders the
whole battlefield and asks every source whatever the proposal. A per-source
mask over `GameAction` kinds, selected source first, is this lever again
(`EventPattern` has one arm per `GameAction` variant, so the one-table rule
carries over). It is `replacement-architecture.md`'s to design and measure,
not this doc's. The restriction check is 2–3% and passes its gate on 6–10%
of calls; nothing here asks for a lever there.

The three sweeps together are at most about a quarter of the CPU; the rest
is outside this probe, and §3's instruction-count profile is the instrument
for ranking it.

**Since TR-1b the probe's count columns are rows** (§4.10, decision 4):
`Windows past gate`, `Candidate visits` and `Trigger matches` print in every
sitting. And the instruction-count profile has its first reading at this
scale — `fuzz-record.md`'s TR-1b block, taken with `plans/profile/`: the
dispatcher is 1.8% of 95.96 G instructions, and the replacement gather 17.7%
inclusive, which agrees with the probe above.

**A/B predictions, per phase**, in `engineering-practices.md` §3.1's terms —
three arms where a pool changes (`main`, the engine with pools unchanged,
shipped), two seats and four, and the budget is 2.5 points of CPU per
decision at identical counters:

| Phase | Engine arm, pools unchanged | Shipped arm | Why |
|---|---|---|---|
| TR-1 | every counter `IDENTICAL`; CPU per decision inside the budget | `differ` on both pools; a `fuzz-record.md` block | the gate is empty on the old pools; the new pool has three trigger sources and a `Triggers placed` row |
| TR-2 | `IDENTICAL` on `performance`; `differ` on `stress` | `differ` | the histories are advanced on every game (a cost, no counter); Alms Collector is in `stress` and its rider's draw order changes (§6.6) |
| TR-2a, as measured | every gameplay row `IDENTICAL` on both pools; `Layer walks` +5% (a spell's types read as it is cast) and `Candidate visits` up (a cast's window has a kind now); the dumps differ by the rider's order and lifelink's gains | `differ`; a `fuzz-record.md` block | the row above said the histories move no counter, and they move two; the rider's order moved one four-seat `stress` game's dump, not a counter (`fuzz-record.md`, TR-2a) |
| TR-2b, as measured | every gameplay row `IDENTICAL` on both pools at two seats and four, and all 800 dumps identical game by game; `Memo hits` −1 to −9. The elision arm moves 27 of 800 games, each first at an `OrderTriggers` prompt | `differ`; a `fuzz-record.md` block | the row said no other counter would move, and `Memo hits` fell: an edict's choice reads its filter only for the chooser's own permanents (`fuzz-record.md`, TR-2b) |
| TR-3 | `IDENTICAL` both pools | `differ` | the registry is empty on the old pools |
| TR-4 | `Layer walks` up by the widened captures on `stress`, `Memo hits` up on both (the control sweep runs while Act of Treason's row lives); every gameplay counter `IDENTICAL` | `differ` | the sweep is gated on `any_control_changing`; captures gate on `from`/`to` |
| TR-5 | `IDENTICAL` counters; `--dump-events` gains `DamagePrevented` and entry `CountersChanged` lines (`Targeted`'s came with #231, `DamagePrevented`'s with #260) | `differ` | records announced, no decision moved |
| TR-6 | `IDENTICAL` both pools | `differ` | no state trigger on the old pools; Tier 1 counts decisions and settles nothing on them |

Each phase's `fuzz-record.md` block re-records both pools when its
`PERFORMANCE_POOL` entry lands, and the reachability rows (`--require`)
name each new card's trigger count per 200 games. A stream-moving finding
inside a phase is its own PR, as §9 of the practices requires.

---

## 12. Sizing and the phase plan — TR-1 to TR-7

Sized against the tree on 2026-09-18 (`engineering-practices.md` §4: count
first) and re-counted on 2026-09-24 (the last subsection). Each PR carries
at least one registered consumer of what it builds, and each closes against
`specdb owed` for the atoms §13 assigns it. The order is the dependency order:
- TR-1 is the spine every later phase reads.
- TR-2's histories are what TR-3's "this turn" durations and TR-5's
  `FirstTimeEachTurn` read.
- TR-3 builds `ReturnToBattlefield`, which TR-4's persist and Rancor need.
  A return makes a new object, so **TR-3 waited for CV-2, then CV-1b with
  `codebase-state.md` item 10 (CR 400.7)**, both landed: a move now ends every
  reference made before it (`copy-effects-architecture.md` §7d).
- TR-4 widens the frame TR-5's combat shapes never read. TR-5a needs only
  TR-2b, so it may move up to just after it.
- TR-6's loop detector counts the decisions every earlier phase adds.
- TR-7 is last because its capstone loop reads TR-4's frame, TR-5's
  counters and TR-6's loop handling.

**Between TR-3b and TR-4a, `codebase-state.md` item 176's zone-change
record design** (the owner, 2026-09-24): what was done, who did it, which replacement redirected
it, the object the move made (item 177) and the moment each fact is taken
at (item 175), designed and reviewed before code. TR-4 widens that record,
and no earlier phase's card reads the parts it settles. **Designed
2026-10-10** in `replacement-architecture.md` §2b, and built as Phase RH
(RH-1, RH-2, RH-3) before A6m and TR-4a. TR-4a then widens `Lki` and changes nothing
else on the record.

**Between A6o and A6m, A6p, the records' contract** (§3.12a): the five
questions asked of every record, the binding and the frame. It filed items
251–260 and re-counted TR-4 and TR-5, which split into four sub-phases each.

### TR-1 — the spine: dispatch, the queue, placement, the stack object — ✅ landed 2026-09-19

**What shipped.** `types/triggers.rs` (the def, the condition, fourteen
`TriggerEvent` arms with the four projections, the binding, the queue's entry)
and `engine/triggers/` — `dispatch.rs` at the two doors with the three-leg
gate, the frames' look-back, CR 605.4a's immediate resolution and the
`trigger` record; `placement.rs` with CR 800.4d's refusal, the two tiers over
`apnap_index`, `OrderTriggers` and item 163's elision, CR 603.3d through
`announce_targets`, the stack object and the `pending` record; `binding.rs`
reading the bound facts back through the arm. `Effect::Triggered`,
`EffectRecipient::{TriggeringObject, TriggeringPlayer}`,
`AmountExpr::TriggeringAmount`, `StackEntry.trigger`,
`ResolutionContext.trigger`, `AbilityIdentity`'s two fields,
`PermanentState.cast`, `EventId`, item 10's three fields, item 18's three
deletions, `AbilityTriggered`, CR 113.6k derived in `functioning_zones`, the
`Triggers placed` row. Five cards; Soul Warden, Blood Artist and Wild Growth
pooled (91 → 94). Fifty tests, §13's TR-1 row clean.

**What moved on the way in** — the sizing's `ObjectFilter::NotSource` is
`NotSource`; `Attacks` and `GainsLife` shipped narrow because §13 owed their
atoms here; leg 3 is swept with leg 4 because they are one map; the gate has
a sixth probe (the departure frames); the binding carries the def. A
look-back arm on a surviving permanent reads its post-event list — main item
167, closed by the review's theme E (§4.3). The archive has the sizing table
and the nine notes.

**Measured** (`fuzz-record.md`, the TR-1 block): the probe found no dispatch
passing the gate on the old pools; the engine arm `IDENTICAL` on every counter
on both pools at two seats and four, +1.0% and +1.5% CPU per decision; the
shipped arm differs, with `Triggers placed` 1.5 and 4.2 on `performance`.

→ `plans/archive/triggers-architecture-landed.md`, "TR-1"; the trace page
`plans/traces/tr-1-a-trigger-is-matched-at-the-close.html`.

### TR-1b — the dispatch audit (~550) — ✅ landed 2026-09-22

**What shipped.** Main item 174's fix: each departing permanent framed before
its batch performs, a destruction's nested move reading the outer batch's
frame (§4.3). `engine/triggers/audit.rs`: the dispatcher's own matching loop,
`match_candidates`, run a second time over a candidate set with no shortcut
in it, the comparison and its panic, and reads that leave no trace (§4.10).
`enable_dispatch_audit`, `fuzz_games --audit`, and `fuzz_ab.py` auditing its
counter runs. The dispatcher's rows, `Windows past gate`, `Candidate visits`
and `Trigger matches`. `plans/profile/`'s callgrind scripts, the board their
argument. Thirteen tests: ten in `phase_tr1b_integration_test.rs` (item 174 in
both batch orders, CR 603.2c across a survivor's two lists, the audit over
each candidate set, the whole-game invisibility test, the rows) and three
unit tests of the comparison.

**What moved on the way in.** The departure frames became memo reads where
the capture was one uncached board walk per departure, which is the whole of
the measured saving. The owner's review replaced the first audit, a second
matcher at 555 lines and 11–19× the CPU, with the shortcuts-off form, and
fixed the one bug that matcher had found in the shared loop (CR 603.2c); one
reading is filed as main item 175.

**Measured** (`fuzz-record.md`, the TR-1b block and its review round):
zero disagreements on every audited sitting, both pools at two seats and
four, Commander scale, and the forced boards; the audit shown to bite with
item 167's snapshot off and item 174's capture off; the audit 2.2–2.7× the
CPU per game. Item 174's fix changes one forced game's answer in 200 at each
seat count and no shipped game's play. The review round is `IDENTICAL` to the
first head on every row. The first Commander-scale callgrind reading: 95.96 G
instructions, −3.97% against `main`.

→ `plans/archive/triggers-architecture-landed.md`, "TR-1b".

### TR-2a — the histories, the gates, and each player — ✅ landed 2026-09-24

**What shipped.** `PlayerState.history` (§3.10 as built): a `TurnSummary` per
player per turn, advanced by the dispatcher record by record before the gate,
with `TurnOrdinals` for "the first time each turn" and four `Condition` leaves
over a `HistoryCount`. The three turn-scoped sets (§3.5, §6.4, §6.5): CR
603.2h's keyed by the ability and its controller, "triggers only once each
turn" by the ability, and CR 603.7h's count by the ability, which
`Condition::ResolvedThisTurn(n)` reads. The arms `LosesLife` (item 171) and
`CastsSpell`. `EffectRecipient::ThisObject`, found by identity (CR 400.7);
`EachOf(PlayerGroup)` in APNAP order (§6.6, item 122); `AmountExpr::TriggeringPower` and `TriggeringToughness` (item 172). A
one-shot over a filter fixes its affected set (CR 611.2c). Four cards:
Paladin of Atonement, Vengeful Warchief, Elvish Warmaster and Temple Bell.
Warchief and Warmaster are pooled (94 → 96), and all four are in `stress`
(171 → 175). Thirty-seven tests; §13's TR-2a row is clean.

**What moved on the way in.** TR-2 split at its re-count (3,115–3,530
against 2,080–2,575), and "each player" came forward from TR-2b so item 122
closed here. `engineering-practices.md` §2b, the call-site naming rule, and
its renames. The resolution count moved off the controller's row;
`AbilityResolves` moved to TR-3. Three fixes rode in: lifelink gains once per
source per batch (CR 702.15e); CR 305.7's Layer 4 route to an ability list,
which the audit found (§4.10); Elvish Archers' and Alms Collector's printed
stats. It landed at +2,875 in code and tests, over the band; the owner kept
"each player" in on the condition that the coding was done.

**Measured** (`fuzz-record.md`, the TR-2a block). The four engine arms play
every gameplay row as `main` does on both pools at two seats and four. Game by
game, the rider's APNAP order moves one `stress` game in 800; lifelink's gain
moves within its batch in 76–139 games of 200 and merges in 2–16; the Layer 4
fix moves none. CPU per decision +1.3% at four seats over five rounds (+2.9%
over three). The four cards ×4 place 6.3 and 13.4 triggers a game, and a
thousand-game audited smoke on every board agrees on 5.65 million dispatches.

→ `plans/archive/triggers-architecture-landed.md`, "TR-2a" (the plan as
sized, the split, and why there is no trace page).

### TR-2b — "may", CR 118.12's answer, and the `departed` frames — ✅ landed 2026-09-26

**What shipped.** Six "your" conditions folded into `Condition::Player {
whose, fact }`. `EventKindMask` at a width that follows `EventKind`, and the
arms `DrawsCard` and `ShufflesLibrary` (a move to the hand that is not a draw
fires nothing, CR 121.5). `Effect::Optional` with its chooser, CR 118.12's
answer in the resolver's walk, `Condition::CostAnswer`, and CR 603.2h's gate
written on `Does` alone (§6.2, §6.4; main item 24). A player's choice at
resolution as a recipient, `EffectRecipient::ChosenBy`, with `event_for` under
the six object verbs and `Sacrifice` on Destroy's grammar. `AddCounters` over
a filter, a hand's count in the layer walk, and each opponent's life loss. The
`departed` frames, one capture and one writer, and CR 109.5's "you" named by
the asker (§6.1; main item 169). The elision over what a def reads, with the
source row (§5.2; item 163's elision half). Three cards: Nykthos Paragon,
Psychosis Crawler and Cosi's Trickster. Crawler and the Trickster are pooled
(96 → 98), and all three are in `stress` (175 → 178). Twenty-six tests;
§13's TR-2b row is clean.

**What moved on the way in.** It landed at +2,430 in code and tests against
the count's 2,165–2,355, inside the band. The owner's review moved the choice
out of `Sacrifice` into a recipient every verb takes, put all six object verbs
on `event_for`, made the mask's width follow `EventKind`, and left Soul
Shatter's "greatest mana value" for its card. The close-out found a redundant
CR 101.2 query: the verb re-checked a permanent its choice had just admitted.

**Measured** (`fuzz-record.md`, the TR-2b block). The engine arm plays every
gameplay row as `main` does on both pools at two seats and four, and all 800
dumps are identical game by game. `Memo hits` fall by 1–9, since an edict's
choice reads its filter only for the chooser's own permanents. CPU per
decision −0.03% (callgrind). The elision arm moves 27 of 800 games, each first
at an `OrderTriggers` prompt: 26 of one Soul Warden it no longer asks, one of
two Warchiefs it now does. The shipped arm places 2.2 and 5.0 triggers a game
on `performance`; Crawler places 0.7 of them at two seats.

→ `plans/archive/triggers-architecture-landed.md`, "TR-2b" (the design as
reviewed, what landing and the code review changed, and no trace page).

### TR-3a — the delayed-trigger registry, with Final Fortune and Blessed Wine — ✅ landed 2026-10-07

*Body evicted 2026-10-07 to `plans/archive/triggers-architecture-landed.md`,
"TR-3a", with the TR-3 plan as sized.*

**What shipped.** `engine/triggers/delayed.rs`: the registry, its one door
`register_delayed_trigger`, and its leg of every dispatch through the
objects' arm matcher, which reads `TriggerReferents` (§4.6).
`Primitive::CreateDelayedTrigger` with CR 603.7d–f's provenance off the
resolution and 107.3n's X; `TriggerOrigin::Delayed` on `AbilityId::delayed`
(§3.8); CR 603.7b's choice, `ChooseDelayedTriggerEvent`, asked only between
causes that differ; `ThisTurn` ended at cleanup; the extra turn's id on the
queue and on `BeginTurn`, which "that turn" reads, and `NextTurn` for "the
next turn's"; a source's departure record and last frame (`source_left_at`,
`source_frame`) and the creation filter (§3.9's as-built list); `TriggerEvent::AbilityResolves` for CR 603.7h, and `AmountExpr::X` at
resolution. Item 222, the named copy and grant rows' leg, as its own commit.
Two cards, neither pooled: Final Fortune and Blessed Wine (182 → 184
registered). Twenty-seven tests; §13's fourteen TR-3a atoms are covered and
`owed` is clean for them; the review closed item 224 and added `source_frame`.

**What moved on the way in.** The owner added Blessed Wine beside Final
Fortune: every "that turn" card makes its caster lose, and Final Fortune
ended 29 of 200 two-seat stress games at its extra turn. CR 603.7a needed
code after all: a rider runs between the members of one simultaneous event,
so an entry skips its creating batch. The Tatsumasa and Flickerwisp boards
wait for TR-3b's `refs`; 603.7b-002 and 603.7e-001 are covered on the rule's
own boards. `GameEvent` grew from 64 to 72 bytes. It landed at +2,278 in code
and tests (code +1,205, tests +1,073) against the re-count's 1,600–1,980,
inside the band, and the review brought it to +2,489 (code +1,325, tests
+1,164), 11 lines inside the band's ceiling.

**Measured** (`fuzz-record.md`, the TR-3a block). The item 222 and registry
arms play every gameplay row as `main` does on both pools at two seats and
four, and the audit agrees on every dispatch. `Candidate visits` fall from
797.4 to 44.8 a game at two seats and from 2,090.8 to 140.3 at four.
Instructions per decision: item 222 −1.21% against `main`, the registry
−0.1% against item 222.

→ `plans/archive/triggers-architecture-landed.md`, "TR-3a".

### TR-3b — the returns and "until", with Flickerwisp and Banishing Light — ✅ landed 2026-10-08

*Body evicted 2026-10-08 to `plans/archive/triggers-architecture-landed.md`,
"TR-3b", with the plan as sized and the gap hunt that split it.*

**What shipped.** Item 223's row half: the ten sites that write a
resolution's registry row read `ResolutionContext::effect_source`, and the
battlefield sweep ends only the rows that end with their source (CR 611.3b,
611.2b), the look-back gate with it; items 223 and 95 closed.
`Effect::Remember`, §3.9's `refs` as #234's review decided it, read back off
the resolution's own records, which the window now holds until the
resolution ends. `Primitive::ReturnToBattlefield` with `ReturnUnder`, and CR
303.4f/g through the existing non-targeting `SelectRecipients`. CR 610.3:
`Primitive::ExileUntil`, its event any `TriggerEvent`, with 610.3a/b's check
for a leaving, and `GameState::until_returns`, made at a dispatch as one batch
(610.3d). CR 111.8. Item 226. Flickerwisp and Banishing Light registered and
pooled (185 → 187, 104 → 106). Thirty-six tests; §13's six CR 610.3 atoms are
covered.

**What moved on the way in.** The gap hunt found ten facilities no row
listed, and the owner split the phase at the design review: the reflexive
half is TR-3c. Three corrections to the plan: the window flushed at every
batch's close, so a resolution could not read its own records; 610.3d is one
batch, which the window alone did not give; and the return is no triggered
ability, so it is not a `DelayedDuration`. `ReturnToHand` moved to TR-4 with
Rancor. The review generalized the until-event and took CR 111.8 in. +2,235
in code and tests (code +1,090, tests +1,145).

**Measured** (`fuzz-record.md`, the TR-3b block). The engine arm plays every
gameplay row as `main` does on both pools at two seats and four, and the
audit agrees on every dispatch. One cost row moved, by the look-back gate's
narrowing. Instructions per decision +0.04%.

→ `plans/archive/triggers-architecture-landed.md`, "TR-3b".

### TR-3c — the reflexive trigger, item 225 and item 99 — ✅ landed 2026-10-09

*Body evicted 2026-10-09 to `plans/archive/triggers-architecture-landed.md`,
"TR-3c", with the plan as sized and the gap hunt.*

**What shipped.** CR 603.12's reflexive trigger as `DelayedDuration::Reflexive`,
`Does` or `Doesnt`, through the registry's one door, checked as it is made
against what its resolution performed and never waiting (§3.9, §4.6; no
`Reflexive` origin, §3.8). Item 229: a resolution's filters ask with its
effect's source. Item 225: a source of damage chosen and matched by its
existence, the frame of a departure and of damage a `DepartedFrame`, CR
400.7c's re-point. Item 103: `targeting::matched` at the six sites that
swallowed a filter's `Err`, and a frame answering for a source the store has
lost. Item 99's first half and its resolving-object gap: `damage_sources`,
CR 609.7a's one enumeration, with `ChoiceOption::LastKnown`; its command-zone
half is B2's. Cornered Crook registered, not pooled (187 → 188). Twenty-four
tests; ATOM-603.12-001 covered.

**What moved on the way in.** The gap hunt found twelve facilities the row
did not name, the departure record's own frame among them: a dies trigger
names its source by the card it became, so only that frame knows which
creature deals its damage. The "doesn't" form has no printed reflexive
customer, and the owner chose to build it with a fixture. The review
(2026-10-10) closed item 241 ("that action") and the "first time" read it
had excused, typed the filter's refusals, put "another" in costs, and made
one rule of a stack object's LKI. +1,626 in code and tests before it (code
+663, tests +963), against the hunt's 1,250–1,500.

**Measured** (`fuzz-record.md`, the TR-3c block). The engine arm plays
every `performance` game as `main` does at two seats, and two of 200 apart at
four: a missed "dies" trigger on `main`, where the creature's owner lost in
the same state-based check, fixed by item 103's frame half and pinned by a
test. The `stress` games apart are CR 609.7a's widened choice. Every dispatch
agreed; instructions per decision +0.17%.

→ `plans/archive/triggers-architecture-landed.md`, "TR-3c".

### TR-4a — the frame's facts and their readers — ✅ landed 2026-10-14

*Body evicted 2026-10-14 to `plans/archive/triggers-architecture-landed.md`,
"TR-4a", with the plan as sized.*

**What shipped.** `Lki` as one body behind one `Arc`: the existence, its
frame, its owner, its cost decisions and, as a permanent, its tapped status,
counters and attachments both ways, with the readers on the type (§3.11).
`ConditionReferents` carries "this" and "it" as they last existed into the
evaluator at the trigger, the CR 603.4 recheck, an effect's own "if" and a
delayed trigger's, so the five live-source leaves read the frame (§6.1);
`CounterCount` over "this" or "it", `AmountExpr::SourceCounters` (CR 122.8,
122.9), `ReturnToBattlefield` with counters, and `persist()` and `undying()`
as authoring words. `DelayedTrigger.source_lki`; `owner_now_or` retired;
the filter table's owner off a held `Lki`. Items 14, 168, 251 and 252
closed. Strangleroot Geist registered and pooled, Demonic Vigor and Compost
registered (188 → 191, 106 → 107). Thirty-two tests; ATOM-603.6e-002,
ATOM-122.8-001 and ATOM-122.9-001 are covered.

**What moved on the way in.** `cast`, `damage_marked` and
`controller_since_turn` stayed out of the `Lki` with no reader in TR-4a to
TR-4d, and `entry` is TR-4d's (§3.11). Item 251 has a resolution half:
`subject_lki` answers only a look-back arm. Kitchen Finks is unregistered
until hybrid payment (CP-1), so its seven rulings are tested on fixtures,
each on both keywords. The model pass (§9) found three readings that
already disagreed in what the branch wrote — "it" per record against per
occurrence, a resolving spell's cost decisions, and "its owner" of a gone
source, a reachable panic — fixed and shown failing first.
+1,870 in code and tests (code +901, tests +969), inside the band and
under the 2,120–2,440 sizing.

**Measured** (`fuzz-record.md`, the TR-4a block). The engine arm plays every
row outside timing as `main` does, cost rows included, on both pools at two
seats and four, and the audit agrees on every dispatch. Instructions per
decision +0.17%.

→ `plans/archive/triggers-architecture-landed.md`, "TR-4a".

### TR-4b — attachment's records — ✅ landed 2026-10-14

*Body evicted 2026-10-14 to `plans/archive/triggers-architecture-landed.md`,
"TR-4b", with the plan as sized.*

**What shipped.** `Unattached { attachment, former_host, route }` replaced
`EquipmentDetached`: both ends `ObjectRef`s, the route one of CR 701.3d's
four (`UnattachRoute`), the cause on the stamp, and `announce_unattached`
the one emitter. The `Attach` arm announces the unattach from the old host
ahead of `Attached`, and the two leave performers what their move dropped,
after their own record. CR 704.5n, 704.5p and 704.5d are members of the CR
704.3 batch (`GameAction::Unattach`, `GameAction::TokenCeasesToExist`),
decided against the board before the check. An Aura carries its host on the
entry (`EnterMods.attached_to`), and the entry's performer attaches it (item
228); `Primitive::Attach` finds its Equipment by identity (item 178). The
arms: `BecomesAttached`, read after the event, and `BecomesUnattached`,
looking back, each end as it was. Grafted Wargear and Bramble Elemental
registered, Bramble Elemental pooled (191 → 193, 107 → 108). Thirty-five tests;
ATOM-603.10c-001, -002, -003 and ATOM-603.2e-002 are covered.

**What moved on the way in.** CR 704.7's collapse keyed members on their
subject alone, which drops an indestructible animated Equipment's unattach
for good; the key now says whether a member unattaches, and the unattaches
follow the moves, so a subject's group decides under the move's cause.
CR 603.10c's "existence of those abilities" reaches an unattach: an attached
ability-list source's move snapshots the lists (§4.3). The 704.5n gather
asks CR 301.6 of a Fortification, and 704.5p CR 310.9 of a battle. The
Wargear distorts `stress` without ending games, so Bramble Elemental, the
benign consumer beside it, is the phase's pooled card (the owner, at #252's
review). +1,650 in code and tests (code ~+600, tests ~+1,050), inside the
band and the 1,150–1,850 sizing.

**Measured** (`fuzz-record.md`, the TR-4b block). The engine arm plays every
gameplay row as `main` does, on both pools at two seats and four, and the
audit agrees on every dispatch. Instructions per decision −1.69%: CR
704.5n/p decided before the batch read frames the check already holds.
Bramble Elemental pooled at the review, and both tables re-recorded with it.

→ `plans/archive/triggers-architecture-landed.md`, "TR-4b".

### TR-4c — the other two look-back classes and the zone statement — ✅ landed 2026-10-15

*Body evicted 2026-10-15 to `plans/archive/triggers-architecture-landed.md`,
"TR-4c", with the plan as sized.*

**What shipped.** CR 603.10a's second and third classes carry their frame:
`zone_change_looks_back(from, to)` is the one statement, read by the capture,
the performers' hand-over and `TriggerEvent::looks_back`; leaving the game
counts only from the battlefield (§3.11). An intervening "if" that
states a zone is where its ability works (CR 113.6b, item 173). A trigger on
the card itself moving outside those classes is asked where the move left it
(CR 603.10, 702.29c; item 249, §4.2). CR 603.2f is asked when a candidate's
list is read (§4.2). A live candidate the window made answers no look-back
arm (item 259), and an Aura's trigger on its host leaving finds the card the
Aura became (CR 400.7f, item 260; §4.3). Items 15, 173, 249, 259 and 260
closed. Bridge from Below, Guerrilla Tactics, Angelic Destiny and Rancor
registered, Bridge from Below pooled (193 → 197, 108 → 109). Thirty tests;
ATOM-603.6e-001 and ATOM-400.7f-001 are covered.

**What moved on the way in.** Three of the brief's recommendations went the
other way on the CR's text: item 249 asks a discard trigger where the card
went, not off a frame; S1 asks a frame where it was; item 259 takes the whole
window as one event. The review fixed three wrong answers, each failing
first: a departing player's graveyard framed against Erebos's Titan's third
ruling, an exiled Aura found as if in the graveyard, and hidden zones left out
of a discard trigger's zones (Library of Leng), and built item 266 at the
owner's call: a card whose every trigger is on itself moving is asked only
as a newcomer. Filed: item 264 (Primal Vigor copies no game finishes;
`GameNumber`), 265 (a cast from a graveyard carries no frame). +1,547 in code
and tests (code about +670, tests about +875), inside the band.

**Measured** (`fuzz-record.md`, the TR-4c block). The engine arm plays every
gameplay row as `main` does, on both pools at two seats and four, and the
audit agrees on every dispatch. Instructions per decision +0.06% after the
review (+0.39% before it, when a departing player's graveyard was framed).

→ `plans/archive/triggers-architecture-landed.md`, "TR-4c".

### TR-4d — control, countering, loss, and the records' missing facts — ✅ landed 2026-10-15

*Body evicted 2026-10-15 to `plans/archive/triggers-architecture-landed.md`,
"TR-4d", with the plan as designed and its two decisions.*

**What shipped.** CR 603.10's three remaining look-backs. A change of control
is announced by a sweep (`announce_control_changes`, `engine/triggers/control.rs`):
each permanent's controller against `PermanentState.announced_controller`, at
the head of an outermost batch, at each state-based pass and in the cleanup
step after the expiry, one sweep's changes one event, caused by
`GameRule::ControlChanges`. It looks back off the frame the last sweep read,
kept while a control-changing effect exists and kept again before a
resolution's row, an expiry and the first control row (decision 1, the
owner's). `ControlChanges` reads it, and for an arm naming no one gaining
control a battlefield departure (Khârn the Betrayer's first ruling). A
countered spell's record carries its frame with `Lki.entry` (item 253), which
`IsCountered` reads, "a spell you've cast" off `StackEntry::caster`.
`PlayerLoses` reads a loss and looks back at the loser's departures (CR 603.9,
603.10f). A mill a replacement keeps in its library is a `SameZoneMove` (item
131), and an effect finds what it moved in any public zone it went to and
never a hidden one (item 258; decision 2, settled by Rescue from the
Underworld's fourth ruling). Items 13, 131, 253 and 258 closed. Multani's
Presence and Ray of Command registered, Ray of Command pooled (197 → 199,
109 → 110). Twenty-eight tests; ATOM-603.10d-001, ATOM-603.10e-001 and
ATOM-603.9-001 are covered.

**What moved on the way in.** The re-read found three defects: the sweep's
gate missed the change back (the registry's summary is rebuilt as rows go),
nothing kept a frame before the first steal, and a `Before` candidate kept
today's controller. A zone-change arm had no "that many". TR-3b's Rest in
Peace test pinned the opposite of Rescue from the Underworld's ruling, and
flipped. +1,613 in code and tests (code +742, tests +871), inside the band.

**Measured** (`fuzz-record.md`, the TR-4d block). Engine against `main`, every gameplay row
identical on both pools at two seats and four, and the audit agreeing on
every dispatch; instructions per decision +0.32%. The 1,000-game audited
smoke: no error, panic or turn-limit game, every dispatch agreed.

→ `plans/archive/triggers-architecture-landed.md`, "TR-4d".

### TR-4 — the frame, the look-back classes and their records — TR-4a to TR-4d (6,005–8,080)

**Re-counted and split in four on 2026-10-12 (A6p, §3.12a)**, one read-only
agent per half of the 2026-09-24 split, card by card against the tree after
RH. Each sub-phase carries a consumer, and each is sized as code at 1.5–2×
plus tests. TR-4b, TR-4c and TR-4d each build on TR-4a's `Lki`, and none of
the three needs another.

| Sub-phase | Pieces | Cards, fixtures and atoms | Code (×1.5–2) | Tests | Total |
|---|---|---|---|---|---|
| **TR-4a — the frame's facts and their readers** — ✅ landed 2026-10-14, above | → `plans/archive/triggers-architecture-landed.md`, "TR-4a" | | | | +1,870 |
| **TR-4b — attachment's records** — ✅ landed 2026-10-14, above | → `plans/archive/triggers-architecture-landed.md`, "TR-4b" | | | | +1,650 |
| **TR-4c — the other two look-back classes and the zone statement** — ✅ landed 2026-10-15, above | → `plans/archive/triggers-architecture-landed.md`, "TR-4c" | | | | +1,547 |
| **TR-4d — control, countering, loss, and the records' missing facts** — ✅ landed 2026-10-15, above | → `plans/archive/triggers-architecture-landed.md`, "TR-4d" | | | | +1,613 |
| *TR-4d as sized (CR 603.10d, 603.10e, 603.10f, 603.9), kept for the record* | `ControlChanged` from the sweep, with the frame before, a `GameRule` cause and the earlier moment (§3.12a), and `announced_controller`, with sweep points of its own rather than TR-6's; the arms `ControlChanges`, `IsCountered`, `PlayerLoses`; items 253 (with `Lki.entry`), 131 and 258; items 254 and 261 went to A6r, before TR-4 (the owner, 2026-10-13) | Multani's Presence; a fixture over the registered Act of Treason, shaped like Khârn the Betrayer; fixtures for items 253, 131 and 258; ATOM-603.10d-001, ATOM-603.10e-001 and ATOM-603.9-001 | ~610 (920–1,220) | ~770 | 1,690–1,990 |

**Prerequisite if TR-4 registers Ichorid or Bloodghast:** both return
themselves from a graveyard, which CR 113.6m places in that zone and
`zone_function` does not derive (Ichorid's intervening "if" is also main
item 173's 113.6b statement).

### TR-5a — the binding's roles — ✅ landed 2026-10-16

*Body evicted 2026-10-16 to `plans/archive/triggers-architecture-landed.md`,
"TR-5a", with the plan as designed and its four decisions.*

**What shipped.** The binding keeps its triggering events
(`TriggeringEvent`), each its record and its `TriggeringEventObjects`: every
object the event is about, by the part it played, as the existence the
dispatch found (§3.4's amendment). Each reader names its `EventRole` (then
`Participant`), as `EffectRecipient::Triggering(EventRole::DamageSource)`,
since CR 608.2c leaves the antecedent of "that creature" to the text, and a
registry test checks each named event role against its condition's arms.
One `record_frame` reads a departure's frame and a damage source that left
before dealing its damage (CR 608.2h), at the trigger and at resolution.
`ObjectFilter::WithCounters`, and `Condition::CounterCount`, count one kind
or every kind, off the board or the `Lki`. Item 267:
a change of control removes the permanent from combat (CR 506.4), shown
failing first. Items 250, 255 and 267 closed. Sphere Grid registered and
pooled (199 → 200, 110 → 111). Seventeen tests; TR-5a owed no atom.

**What moved on the way in.** The count crossed the band, so TR-5a split
again: TR-5e is the attack declaration (decision 4). Decision 2 was first
ruled for the condition, then reversed to the instruction once the choices
were counted from the records: "that player" alone has two or more
candidates on almost every record. Item 268 keeps "that player" for its first
two-candidate reader. The close-out's game-by-game attribution found the
elision comparing every participant, which asked an ordering prompt `main`
skips; it now compares the ones the def reads, shown failing first. The
owner's code review renamed the types for their call sites and merged "that
creature" and "them" into one reader. +1,585 in code and tests (code +961,
tests +624), against the 1,380–1,600 sizing.

**Measured** (`fuzz-record.md`, the TR-5a block). Engine against `main`: the
four-seat tables identical, and three games in 400 at two seats apart, each
first differing at a Ray of Command steal in combat (item 267). Every
dispatch agreed, in the sitting and the 1,000-game smoke. Instructions per
decision +0.02%.

→ `plans/archive/triggers-architecture-landed.md`, "TR-5a".

### TR-5e — the attack declaration — ✅ landed 2026-10-16

*Body evicted 2026-10-16 to `plans/archive/triggers-architecture-landed.md`,
"TR-5e", with the plan as designed, its decisions 5–10 and what the build
corrected.*

**What shipped.** The declaration's record names what each attacker attacks
(item 11's attackers half). `TriggerEvent::Attacks` reads CR 508.3a–e and
506.5 as fields: the attacker, an optional `DefenderFilter`, `alone`, and
`per`, one triggering event for each attacker or for each player,
planeswalker or battle attacked. A defender's triggering event holds its
attackers (decision 8), and the registry refuses a reader of one of them as
one object. `EffectRecipient::AttackedBy` is "the player or planeswalker it's
attacking", read as CR 608.2h reads the creature, and `Lki` keeps the
attack. "Defending player" names its creature in all three player
vocabularies: `PlayerRef`, `EffectRecipient` and `Condition`.
`PlayerRef::AnyDefendingPlayer` is decision 10's predicate. Hellrider is
registered and pooled (200 → 201, 111 → 112). Seventeen tests;
ATOM-508.2a-001 covered.

**What moved on the way in.** Review split TR-5f off at CR 508.4 (decision
9). The build found decision 6's "defending player" in three player
vocabularies rather than one, and a player reader that finds nobody falling
to the controller; both fixed. Items 270 and 271 carry what TR-5f and RS-2
owe; item 11 stays open for its blockers half, TR-5c's. +1,286 in code and
tests (code +727, tests +559), against 1,465–1,680.

**Measured** (`fuzz-record.md`, the TR-5e block). Engine against `main`:
every gameplay row identical, both pools, two seats and four, and every
dispatch agreed. Instructions per decision +0.05%. Hellrider moved `stress`'s
average turns by +0.7 at two seats and −1.9 at four.

→ `plans/archive/triggers-architecture-landed.md`, "TR-5e".

### TR-5f — creatures put onto the battlefield attacking, and removal from combat — ✅ landed 2026-10-17

*Body evicted 2026-10-17 to `plans/archive/triggers-architecture-landed.md`,
"TR-5f", with the plan as designed, its decisions 11–16 and what the build
corrected.*

**What shipped.** A creature can be put onto the battlefield attacking
(item 128): "tapped and attacking" is the creating effect's word, read into
the one entry, which asks its controller with two or more candidates
(`ChooseEnteringDefender`) and writes the attack when CR 506.3 allows, and
such a creature never "attacked" (CR 508.4). Removal from combat is a record,
`RemovedFromCombat`, at every cause that leaves the permanent on the
battlefield, the end of combat among them (CR 506.4, 511.3). A game-wide
history keeps the combat's rows, which "defending player" reads once its
creature has left combat (item 270) and "attacked this turn" reads. One
settlement answers what the layers' output changes, control and a
combatant that stops being a creature, at each outermost batch's close as
well as TR-4d's points. The log leaves a rule's sweep implicit unless a
trigger reads it. Hanweir Garrison (pooled) and Relentless Assault are
registered (201 → 203, 112 → 113). Thirty-six integration tests and three unit
tests.

**What moved on the way in.** The design's probe found the type-change
cause in today's games, so it was built rather than deferred. The build
found the settlement's gate reading too little and RE-10 ignoring "after
this main phase"; the review, the return door missing "tapped" and the memo
serving stale frames to a static over a life total, a tapped status, combat
or the histories (items 187 and 199, closed). All fixed; item 272 carries
"for each opponent". +2,547/−286 in code and tests (code +1,374/−247, tests
+1,170/−35, the dev GUI +3/−4), against 1,770–1,995: past it by the review.

**Measured** (`fuzz-record.md`, TR-5f's block). The engine arm plays
`main`'s games but the one predicted, a Mirrorweave whose attackers become a
land and leave combat. +0.77% callgrind instructions per decision, within
the budget and above the predicted +0.5%: the type half's frame reads after
each layer-epoch move in a combat (+1.7% layer frames on the budget's
board), and `GameAction`'s 8 bytes once the entry's attack was boxed.

→ `plans/archive/triggers-architecture-landed.md`, "TR-5f".

### TR-5b — counters, and one history struct — ✅ landed 2026-10-17

*Body evicted 2026-10-17 to `plans/archive/triggers-architecture-landed.md`,
"TR-5b", with the plan as designed, its decisions 1–4 and what the build
corrected.*

**What shipped.** One history struct: the players' rows in `GameHistory`,
each "since your last turn" snapshot shared behind an `Arc`. A put and a
removal are two records, `CountersPutOn` and `CountersRemovedFrom`, each
naming the player who did it where the rules name one and the count it
left; an entry announces its counters (CR 122.6). Two trigger arms read
them once per counter, once per object (`Multiplicity::OncePerSubject`) or
once per event, and CR 122.7's "the Nth counter" is the ability's own "if",
`Condition::CounterCountReached`. CR 704.5q is two removals in the
state-based batch, and `CountersAnnihilated` is gone. An entry template
reads its spell's X (CR 107.3m). Simic Ascendancy and Protean Hydra are
registered and pooled (203 → 205, 113 → 115). Items 6 and 'Before Triggered
abilities' 17 closed. ATOM-122.7-001, ATOM-107.3m-001, ATOM-704.5q-001 and
-002 and ATOM-704.8-001 covered.

**What moved on the way in.** Review moved CR 122.7's threshold off the arm,
gave a removal its remover (Vampire Hexmage), and shared the snapshot so the
history costs a clone fewer allocations than before. The build found
that a permanent's controller removes its shield and stun counters, as the
shield's damage half already said. The close-out found Protean Hydra uncastable: the gate
refuses `{X}` until MA-2, which moved up to the next PR. +2,103 in code and
tests (code +946, tests +1,157), against 1,630–1,850.

**Measured** (`fuzz-record.md`, TR-5b's block). The engine arm plays `main`'s
games on every table. +0.11% instructions per decision. The history commit
read IDENTICAL and cut the worst clone from 41 to 38 allocations.

→ `plans/archive/triggers-architecture-landed.md`, "TR-5b".

### TR-5c — blocks, targeting and activation — ✅ landed 2026-10-18

*Body evicted 2026-10-18 to `plans/archive/triggers-architecture-landed.md`,
"TR-5c", with the plan as designed, its one decision, pieces 11 and 12 as
TR-5d's design starts from them, and what the build corrected.*

**What shipped.** The declaration of blockers is one `BlockersDeclared` per
step, written also when nobody blocks, naming its unblocked attackers (CR
509.1h, 509.3g), and each block is a row on the combat history. "This turn"
and "this combat" are `ObjectFilter::ThisTurn` and `ThisCombat` over an
`ObjectFact` (attacked, blocked, was blocked, entered), and
`AttackedThisTurn` folded into them. The block arm reads CR 509.3a–g as
fields, with 509.3e's count and 506.5's `alone`, and "that creature" of
"blocks or becomes blocked by" is `EventRole::OtherCombatant`.
`BecomesTarget` reads #231's `Targeted`, an object or a player, one window
per spell or ability, valiant's first time each turn off the turn's
targeting rows, and `Primitive::Counter` counters what it is handed.
`ActivatesAbility` reads `AbilityActivated`, which a mana ability's
activation writes too (CR 605.3a), and CR 605.1b makes a trigger on one
that adds mana a mana ability. `EffectRecipient::Group` gives one
instruction several objects (CR 608.2f). `Attacks`' `per` is `occurrence`.
Loyal Sentry and Cephalid Aristocrat are registered and pooled (206 → 208,
116 → 118). Items 11 and 12 closed.

**What moved on the way in.** Review moved excess damage, the "this turn"
damage facts and item 275 to TR-5d, built valiant, the mana ability's
activation and the other creature in a block here, and kept "blocks alone",
which CR 506.5 defines. The build gave the record whether its ability is a
mana ability, left a mana ability's activation out of the log, and found
that the save reports a combat or targeting row rather than writing it.

**Measured** (`fuzz-record.md`, TR-5c's block). The engine arm plays
`main`'s games on every table; +0.15% instructions per decision; stress
distortion within the deal's noise. The model pass found two readings
written in one place and missed in another, both fixed, and eight copies.

**What it leaves.** Items 278–283 (`codebase-state.md`, "Found by TR-5c").

→ `plans/archive/triggers-architecture-landed.md`, "TR-5c".

### TR-5h — one history of what happened — ✅ landed 2026-10-19

*Body evicted 2026-10-19 to `plans/archive/triggers-architecture-landed.md`,
"TR-5h", with the plan as designed, its measurement gate, and what the build
corrected.*

**What shipped.** One log of the game's events, kept whole
(`state::history_log`): a row per event a card can ask about later, each with
its object's frame as the event left it, in chunks of sixteen behind `Arc`s,
so a clone copies one pointer. Every history reader reads it (§3.10): the
four span conditions over a `HistoryCount`, which gains `of`, a filter over
each row's frame; `AmountExpr::History`, the same count as a number (item
280); `ObjectFilter::ThisTurn` and `ThisCombat`, with the damage facts
`WasDealtDamage` and `DealtDamageTo` (item 279's damage half); a record's
place in its turn; the targeting limit; a removed creature's defending
player. The layer memo's gate keys a read by kind of row, the turn number and
the combat count, noted inside each lookup. `TurnSummary`, the seats × seats
snapshot, `CombatHistory`, `TargetingHistory` and `SpellsCastOfType` are
gone. The scenario states a count as a row and names each cast by its card.

**What moved on the way in.** The gate's draw-row trim was measured and
reverted with the owner's approval, held memory unchanged. Frames see their
own window's rows. The count's comparison moved into the condition so an
amount shares it. The scenario names casts only; the other facts stay
counts.

**Measured** (`fuzz-record.md`, TR-5h's block). The engine arm plays
`main`'s games on every table; +1.37% instructions per decision, past the
predicted +0.3% to +1%, most of it the entry rows' frames. Floor 3 2.2 KB
lower per clone; 2.7–4.6 KB held a turn. The model pass found three faults,
each fixed and shown failing first, and one copy.

**What it leaves.** Items 286 and 287 (`codebase-state.md`, "Found by
TR-5h"); item 279's other facts, each with its first card.

→ `plans/archive/triggers-architecture-landed.md`, "TR-5h".

### TR-5d — prevention and excess damage — ✅ landed 2026-10-19

*Body evicted 2026-10-19 to `plans/archive/triggers-architecture-landed.md`,
"TR-5d", with the plan as designed and what the build and its review
corrected.*

**What shipped.** `TriggerEvent::DamageIsPrevented` reads `DamagePrevented`
at its batch's close: one trigger each time a prevention effect is applied
to one recipient's damage, however many sources (CR 615.13). Excess damage
(CR 120.10) is decided with the batch: before any member performs, each
creature or planeswalker it damages gets the damage still lethal to it, and
each `DealDamage` takes its share, so a permanent's records sum to the
event's excess. The damage arm's `excess` reads it once per permanent, and
`ObjectFact::WasDealtExcessDamage` reads it off the damage row. Item 275:
`SOURCE` compares the source by identity and what each entry carries of it,
and `TriggerBindingReads::X` reads X. At review: the damage and excess a
resolution dealt "this way" (item 289), and the frames a control change
and an unattach carry, read where they apply (item 290). Selfless Squire
and Goblin Negotiation are registered and pooled (208 → 210, 118 → 120).
Items 16, 275, 289 and 290 closed.

**What moved on the way in.** `occurrence_subject` names an `EventSubject`.
A history row grew from 104 to 112 bytes. The "dealt this way" family the
design counted at 4 is 24 cards. Review asked the phase's loose ends built
rather than filed: two were, and item 288 needs CR 607.2h's linked
abilities and the prevented damage's sources, so it went to A6s.

**Measured** (`fuzz-record.md`, TR-5d's block). The arm without item 275
plays `main`'s games on every table at +0.13% instructions per decision;
item 275 moves only games where Protean Hydra's delayed triggers fire
together. The two cards' stress distortion is small. TR-5h's held-memory
gate is asserted over every game together (the owner, 2026-10-20): 3,923 B
a turn.

**What it leaves.** Item 288, at A6s. TR-5g followed it.

→ `plans/archive/triggers-architecture-landed.md`, "TR-5d".

### TR-5g — the multiplier, tokens and scry — ✅ landed 2026-10-20

*Body evicted 2026-10-20 to `plans/archive/triggers-architecture-landed.md`,
"TR-5g", with the plan as designed and what the build corrected.*

**What shipped.** CR 603.2d's count: `Effect::TriggerMultiplier`, a static
shape with no layer rows, gated by a printed source set and two summary
flags, as a cost effect is. Of each match the dispatch asks how many more
times it triggers; a multiplier adds when it is on the board the matched
arm reads, the source is a permanent it names as the dispatch read it, and
a triggering record is the event it names, read with its own referents and
frames. Each time is its own `PendingTrigger`; a triggered mana ability
resolves as many times; a delayed or reflexive trigger is never counted.
The audit counts again over every candidate. `TokenCreated` carries the
definition its creation made the token from, after any replacement, and
`CreatesToken` reads it; `Scries` reads `Scried`. `TargetCount::UpToAmount`
is CR 601.2c's count of targets the text defines, read as the announcement
starts. Panharmonicon is registered and pooled (210 → 211, 120 → 121);
Gandalf the White, Ajani, Nacatl Avenger and Elrond, Master of Healing are
fixtures.

**What moved on the way in.** The resolution's `CountOf`, which Ajani's
damage needs, had never been built. A creation is the only maker of a
token's entry, which two fixtures had built by hand. The model pass found
four defects in the new code, each fixed (`duplicate-inventory.md` §3.10).

**Measured** (`fuzz-record.md`, TR-5g's block). The engine arm plays
`main`'s games on every table, both pools, two seats and four, at +0.18%
instructions per decision. The shipped arm moves every game and the
audit agrees on every dispatch, 1,000 games a table included; Panharmonicon
moved `stress` games by about a turn either way. `history_held_test` reads
3,877 B a turn. One Commander game ended with 31 Sphere Grid triggers
waiting, an allocation each (82 against floor 2's 64), and cloned at 77 at a
28-trigger ordering prompt: B10's batch of triggers, not the multiplier. A
finished game now drops waiting triggers (CR 104.1), and reads 50.

**What it leaves.** B10's design is next (the owner), then its build CL-1 (the owner, 2026-10-21), then TR-6.

→ `plans/archive/triggers-architecture-landed.md`, "TR-5g".

### TR-5 — the binding's roles, counters, combat and targeting, prevention and the multiplier — TR-5a to TR-5g (8,690–10,010)

**Re-counted and split in four on 2026-10-12 (A6p)**, as TR-4 was. TR-5c
reads TR-5a's roles; the others are independent of one another. Damage an
ability deals is dealt by its source permanent since TR-3b (item 223), so it
left the row. **TR-5a split in two at its design (2026-10-16), above**: TR-5a,
the binding's roles, then TR-5e, the attack declaration. **TR-5e split in
two at its review (2026-10-16, decision 9)**: the declaration, then TR-5f,
creatures put onto the battlefield attacking and removal from combat, whose
rows TR-5c's block triggers read. The route is TR-5a, TR-5e, TR-5f, TR-5b,
then MA-2 (`mana-architecture.md` §6, for TR-5b's Protean Hydra), then TR-5c,
TR-5h, TR-5d, TR-5g. **TR-5d split in two at its design (2026-10-18),
above**: prevention and excess damage, then TR-5g, the multiplier, tokens
and scry. At its review (2026-10-19) the history became a phase of its own
ahead of both, TR-5h.

| Sub-phase | Pieces | Cards, fixtures and atoms | Code (×1.5–2) | Tests | Total |
|---|---|---|---|---|---|
| **TR-5a — the binding's roles** — ✅ landed 2026-10-16, above | → `plans/archive/triggers-architecture-landed.md`, "TR-5a" | | | | +1,454 |
| **TR-5e — the attack declaration** — ✅ landed 2026-10-16, above | → `plans/archive/triggers-architecture-landed.md`, "TR-5e" | | | | +1,286 |
| **TR-5f — creatures put onto the battlefield attacking, and removal from combat** — ✅ landed 2026-10-17, above | → `plans/archive/triggers-architecture-landed.md`, "TR-5f" | | | | +2,547 |
| *TR-5a as sized on 2026-10-12, kept for the record* | items 250 and 255 as one facility: an occurrence keeps both of its record's objects as `ObjectRef`s at the dispatch, a recipient for the counterpart, the damage arm's subject its source; item 11's attackers with their defenders, and `AttackShape`'s seven readings over them; item 128, with CR 508.4's defender choice; `ObjectFilter::WithCounters` | Hellrider; Sphere Grid; ATOM-508.2a-001 and CR 508.3a–e's readings | ~700 (1,050–1,400) | ~990 | 2,040–2,390 |
| **TR-5b — counters, and one history struct** — ✅ landed 2026-10-17, above | → `plans/archive/triggers-architecture-landed.md`, "TR-5b" | | | | +2,103 |
| **TR-5c — blocks, targeting and activation** — ✅ landed 2026-10-18, above | → `plans/archive/triggers-architecture-landed.md`, "TR-5c" | | | | +2,630 |
| **TR-5h — one history of what happened** — ✅ landed 2026-10-19, above | → `plans/archive/triggers-architecture-landed.md`, "TR-5h" | | | | +2,058 |
| **TR-5d — prevention and excess damage** (CR 615.13, 120.10) — designed above | item 16's arm over `DamagePrevented`; excess damage per recipient, decided with the batch, and on TR-5h's damage row; item 275 | Selfless Squire; ATOM-120.10-001 | ~240 (360–480) | ~520 | 880–1,000 |
| **TR-5g — the multiplier, tokens and scry** — ✅ landed 2026-10-20, above | → `plans/archive/triggers-architecture-landed.md`, "TR-5g" | | | | +1,781 |
| *TR-5d as sized at TR-5c's review (2026-10-18), kept for the record* | item 16's arm over `DamagePrevented { target, amount, by }`, whose record came with #260; excess damage per recipient over the batch, and the "this turn" damage facts ("was dealt damage", "dealt damage to"), moved from TR-5c at its review (TR-5c's piece 11 and part (a)); item 275, moved with them; `CreatesToken` with the definition `TokenCreated` carries; `Scries`; `Effect::TriggerMultiplier` with its three legs and the triggered mana path | Selfless Squire, Panharmonicon (seven of its nine rulings; two are about linked abilities); Gandalf the White's multiplier against item 251's "from anywhere" arm on one board (its ninth ruling); Ajani, Nacatl Avenger's and Elrond's fixtures; ATOM-603.2d-001, ATOM-120.10-001 | ~610 (915–1,220) | ~1,080 | 1,995–2,300 |

### TR-6 — state triggers, the draw, and the rule-owned arm (1,305–1,720)

**Re-counted 2026-10-21, at its design** (§4.5, §4.8 and §4.9's amendments),
card by card against #267's tree, then **cut at the review to (ii)**: loop
detection moved to a phase of its own (§4.9's head), and TR-6 builds the draw
and the state triggers. Code at 1.5–2× plus tests, the band's measure since RH:

| Piece | ~additions |
|---|---|
| the state check: the gate's reserved bit on all three legs and on leg 3, the sweep through `settled_holds`, a trigger queued with no record and announced by `AbilityTriggered`, its trace record, the armed-off set and its one re-arm (§4.5); the three schedules; the dispatch audit's state leg | ~255 |
| `Condition::Not`; `PlayerFact::HandEmpty`; "then draw that many" over the resolution's discards | ~85 |
| the draw: `GameAction::GameEndsInADraw` and its event, the performer, `GameRule::MandatoryLoop`, about 17 exhaustive arms; the two counts the engine has proposing it, `MANDATORY_LOOP_CHECKS`' branch (item 263) and `DISPATCH_NESTING_LIMIT`'s, with the latter's two doc comments (§4.8, §4.9, decisions 4 and 5) | ~100 |
| `TriggerOrigin::Rule` with `InherentAbility` uninhabited | ~20 |
| cards and fixtures: **Emperor Crocodile**, registered after its stress distortion is measured; the Coil's third ability, a fixture beside the registered Platinum Angel; CR 603.8's example permanent and its discard spell; a flicker and an ability counter for the rulings' boards (Adaptive Shimmerer, registered, is the second Crocodile ruling's arrival); a pair of triggers that each watch the other's triggering, for the nesting bound | ~110 |
| **code**, ~570 as sized | **855–1,140** |
| tests, 10: the 2 atoms; the Crocodile's 2 rulings and one cast from hand; the Coil's fourth and fifth rulings and its sixth's "it will immediately trigger again", the Angel registered; Lich's Mirror's loop drawn through the new arm (item 263); the nesting bound drawn through it, beside Platinum Angel (its fourth ruling: an effect that makes the game a draw still works) | **450–580** |
| **code and tests** | **1,305–1,720** |
| docs: these amendments, with `backlog.md` §2.28's pointer (about 460); the close-out — the stub and archive, the record block, the items, the 37 deferred Phase 7 atoms given their owners in the session files (13 are ticketed NEW, which is what `owed` reads), `trigger-survey.py` deleted | ~820 |

At the band's floor, and under it at its low end. Nothing is folded in to fill
it. The 2026-09-24 row missed these facilities, a count inside the 3–11 every
earlier re-count found:
- the gate's bit, and the audit's state leg;
- `PlayerFact::HandEmpty`;
- the "that many" leaf;
- the exhaustive arms;
- the fixtures for the rulings' boards.

Its tests come to more than its row's ~300, as before. What (ii) moved out:
the loop detector and its counting sites, the resets, the harness bound, the
tier-2 work stack, and seven tests, among them the Coil's draw beside Platinum
Angel. They go with §4.9's record to the loop phase.

*As sized on 2026-09-24, kept for the record:*

| Piece | ~additions |
|---|---|
| the state check at its three schedules, `state_triggers_armed_off`, `trigger_left_stack` at four sites, `TriggerCondition::State` matched; `Condition::Not` (Emperor Crocodile's "no other creatures" is its first card — LI-3's rule for when the enum grows); `LoopDetector` (Tier 1) on `GameState` reading A4e's decision counter, `GameResult::Draw` through the settlement; the same draw at `DISPATCH_NESTING_LIMIT`, which becomes the threshold knob (§4.8), with its two doc comments (`dispatch.rs:40`, `game_state.rs:214`); `TriggerOrigin::Rule` declared with `InherentAbility` empty | ~320 |
| cards: **Emperor Crocodile** (603.8, its two rulings), pooled. Immortal Coil is the CR 104.4b board — its third ability beside Platinum Angel is "an involuntary infinite loop ... the game will end in a draw" by its own ruling — and it is a **fixture**, not a registration: its first ability's `Cost::ExileFromGraveyard` is a validation and payment stub today (`engine/costs.rs`), and a card wearing a real name with a dead ability is what `engineering-practices.md` §3 forbids | ~120 |
| tests, 9: §13's 2 TR-6 atoms; Emperor Crocodile's two rulings; the Coil's three trigger rulings on the fixture; the draw at threshold, with Platinum Angel registered; the draw at the nesting bound (§4.8). The ratchet's reading at the phase's close (§9 of the practices) is a reading, not a test | ~300 |
| docs, the close-out, §16's deferrals re-read, ledger, record | ~400 |

### TR-7 — the persist loop, the track's capstone

**Re-scoped 2026-10-13 (the owner, at #246's review).** The track ends with a
loop built only from what TR-4 to TR-6 build. The Krark-Clan Ironworks loop it
replaces moved to `roadmap-v2.md` B13, a capstone across triggers, mana and
permission. Its graveyard targets, metalcraft's activation restriction and CR
732.1's reverse-all belong to later rows, and it exercised little of TR-4 to
TR-6: no control change, combat, counters or state trigger.

**The loop.** Four cards, each playable whole by then:
- Grumgully, the Generous: "Each other non-Human creature you control enters
  with an additional +1/+1 counter on it";
- Wingrattle Scarecrow: "This creature has persist as long as you control a
  black creature";
- Viscera Seer: "Sacrifice a creature: Scry 1", black;
- Blood Artist, registered: "Whenever this creature or another creature dies,
  target player loses 1 life and you gain 1 life", black.

Sacrifice the Scarecrow. Persist returns it with a -1/-1 counter, and
Grumgully's +1/+1 counter enters with it. CR 704.5q annihilates the pair, so
the new object has no counters and persists again. Blood Artist drains on
every death.

What one loop exercises:
- **Persist's intervening "if"** read off the frame at both instants (TR-4a;
  CR 603.4, 603.10a; the Scarecrow's second ruling).
- **A conditionally granted triggered ability**, asked of a departed object's
  frame: persist exists as long as a black creature does (§4.2's leg 2).
- **Each return is a new object** (CR 400.7; its fifth ruling).
- **An entry with two counter rows**, announced (item 17), and their
  annihilation as two removals in the state-based batch (TR-5b). That includes
  the seventh ruling's board, where the counters arrive and the creature dies in
  one check.
- **The ordering** of persist and Blood Artist's trigger at each death (CR
  603.3b), with item 163's ordering decorator.
- **An optional loop** its player must be able to stop, and a fuzz agent that
  does (TR-6, CR 731.1b, §4.9).

Persist itself is undying's mirror, built beside it in TR-4a (Kitchen Finks'
rulings are its tests). Mikaeus, the Unhallowed's undying lord was the first
pick. It was set aside because its intimidate is a flag nothing reads until
RS-3a (`cant-effects-architecture.md`), and a registered card may not carry a
dead ability (`engineering-practices.md` §3). Not sized: its brief re-counts it
against the tree TR-6 leaves.

### Re-counted 2026-09-24 — the splits, and why the plan keeps missing

One read-only agent per phase checked every clause of every named card
against the code. The counts below replace the 2026-09-22 and 2026-09-23
re-counts. Figures are code plus tests, the band `engineering-practices.md`
§4 sets:

| Phase | Was | Re-count | Split into |
|---|---|---|---|
| TR-2b | 1,500–1,700 | 1,720–2,180 | no split |
| TR-3 | 2,070–2,540 | 2,800–3,600 | **TR-3a**: the delayed registry, with Final Fortune, 1,600–1,980. **TR-3b**: reflexive triggers, the returns and "until", with Flickerwisp, Cornered Crook and Banishing Light, 1,410–1,640, and 1,650–1,880 with item 223's row half and item 225 (2026-10-08) |
| items 176/177 | — | 800–1,050 | their own PR, between TR-3b and TR-4a. **Re-sized 2026-10-10** as Phase RH, with item 175, item 248 and A6l's L1–L7 added: RH-1 ~1,530 and RH-2 ~1,620 as sized (`replacement-architecture.md` §2b.10); **re-split 2026-10-12** into RH-2 ~845 (the batch's members, item 248) and RH-3 ~1,160 (the record) |
| TR-4 | 2,000–2,460 | 2,640–3,460 | **TR-4a**: the frame, 1,360–1,620. **TR-4b**: the records, 1,430–1,790 |
| TR-5 | 2,195–2,710 | 3,080–4,350 | **TR-5a**: combat, targeting, ability damage and excess damage, 1,200–2,050. **TR-5b**: counters, prevention, the multiplier, tokens and scry, 1,870–2,380 |
| TR-6 | 740–1,030 | 935–1,360 | no split |
| TR-7 | — | 1,020–1,420 | not yet |

Each half keeps the cards that read its facilities, so each half still
carries a consumer.

**Why the plan keeps missing.** The agents found the same two causes in every
phase. First, tests run about twice the row's estimate. Second, every phase's
cards hit 3–11 facilities its row never listed, such as a sacrifice by a
chooser, a kind mask past sixteen kinds, or a source-relative "another". So
the card-by-card gap hunt is now a sizing step (`engineering-practices.md`
§4), not a review finding.

**Build once.** Four facilities recur across phases, so each is built whole
where it first appears:
- a complete `Primitive::Sacrifice` (the source itself, a named object, or a
  chooser over a filter), in TR-2b;
- `EventKindMask` widened past `u16`, in TR-2b;
- "another target" (targeting refused `NotSource`), built by CV-1b for
  Cryptoplasm (`copy-effects-architecture.md` §7d), so TR-3b no longer owes it;
- damage an ability deals, dealt by its source permanent (CR 113.7a), built
  with item 223 in TR-3b.

**Design corrections the re-count made.**
- TR-2b's CR 603.2h writer records the action only when it was taken (§6.4).
- Simic Ascendancy triggers once per creature, not once per window (§3.3,
  §4.4).
- TR-6's loop detector counts decisions as item 138 defines them, never
  prompts (§4.9).
- The Ironworks loop is TR-7's test, not TR-2's (§17). Since 2026-10-13 it
  is `roadmap-v2.md` B13's, and TR-7 is the persist loop.

### Re-counted 2026-10-12 (A6p) — TR-4 and TR-5, each in four

TR-4 and TR-5 build on the records §3.12a reads, and their sizes dated from
2026-09-24, before items 249–260 and before RH changed the record. The method
was the one above, run again by four read-only agents, one per half of the
2026-09-24 split. Each read every clause and ruling of its cards against the
tree after RH, and each was given the landed phases' actuals to calibrate
against:

| Phase | Code as sized | Code added | Tests added |
|---|---|---|---|
| RH-1 | ~780 | 1,569 | 1,245 |
| RH-2 | ~390 | 314 | 341 |
| RH-3 | ~460 | 553 | 726 |

The totals below are code at 1.5–2× plus tests, the band check
`replacement-architecture.md` §2b.10 adopted.

| Half (2026-09-24) | Was | Re-count | Split into |
|---|---|---|---|
| TR-4a, the frame | 1,360–1,620 | 3,390–3,860 | **TR-4a**, the frame's facts and their readers, 2,120–2,440; **TR-4c**, the other two look-back classes and the zone statement, 1,560–1,750 |
| TR-4b, the records | 1,430–1,790 | 2,380–4,060 | **TR-4b**, attachment's records, 1,150–1,850; **TR-4d**, control, countering, loss and the records' missing facts, 1,175–2,040 since items 254 and 261 went to A6r |
| TR-5a, combat, targeting, ability damage, excess damage | 1,200–2,050 | 3,960–4,640 | **TR-5a**, the binding's roles and the attack declaration, 2,040–2,390; **TR-5c**, blocks, targeting, activation and excess damage, 1,920–2,250 |
| TR-5b, counters, prevention, the multiplier, tokens, scry | 1,870–2,380 | 2,980–3,415 | **TR-5b**, counters, 1,510–1,715; **TR-5d**, prevention, the multiplier, tokens and scry, 1,470–1,700 |

Each half keeps its old code where it keeps the old half's subject, and the
new halves take the next codes, in route order.

**Why it grew, about twofold again.**
- **The multiplier.** The 2026-09-24 figures summed the rows as sized. Since
  RH the band is read with code at 1.5–2×, since code has run 0.8–2.0× its
  sizing (the table above).
- **The items filed since.** 249, 250 and §3.12a's 251–260, with item 131
  moved in.
- **The facilities no row listed**, found card by card as the last re-count
  predicted. Among them:
  - X in an entry template (Protean Hydra; RC-5 built only `SourcePower`);
  - CR 704.5q's two removals, which the state-based batch's per-subject dedupe
    would collapse;
  - no blockers record when nobody blocks;
  - `ReturnToBattlefield` with no entry counters (undying);
  - the dispatch audit's snapshot, which keeps no host and no status;
  - the re-attach route of becoming unattached;
  - the exhaustive matches a new record kind meets (about fourteen).
- **Tests**, at about twice their rows again, with every registered card's
  rulings tested.

**What left the rows.**
- Damage an ability deals, dealt by its source, is built (item 223, TR-3b).
- `Primitive::ReturnToHand` is built.
- Rancor's trigger is on its own move (CR 400.7e), which RH-3 built and
  tested word for word. Rancor rides in TR-4c for its registration.
- Golgari Brownscale's dredge does not fit `Rewrite`: dredge also needs a
  library-size condition and a CR 616.1 choice among dredges. So the class is
  a fixture, and the card stays in §15.

**The route** (`roadmap-v2.md` A6d): A6q's gates and A6r's fixes, then TR-4a,
TR-4b, TR-4c, TR-4d, TR-5a, TR-5b, TR-5c, TR-5d, then TR-6 and TR-7. Item 6 is seventeen PRs where the 2026-09-24
count made it twelve.

---

## 13. Testing — the atoms this owes, by phase

The 133 Phase 7 atoms, read out of `spec.sqlite` on 2026-09-18. Each phase
closes with `python plans/specdb.py owed --phase "Phase 7"` clean for its
rows, annotated at write time (`// COVERS:` / `// COVERS-PARTIAL:`); Phase 7
joins `SHIPPED_PHASES` at TR-6's close. Six partials become full where
their trigger half lands.

| Phase | Atoms (rule ids) | Count |
|---|---|---|
| **TR-1** | 117.2a-001; 500.6-001; 502.4-001; 503.1a-001; COMP-UNTAP-TRIGGER-UPKEEP-001; 508.1m-001; 511.2-001; 405.3-001, -002; 603.2-001; 603.2b-001; 603.2c-001; 603.2e-001 (fixture); 603.2f-001; 603.2g-001; 603.3-001; 603.3a-001; 603.3b-001, -002 (the tier, by fixture; Strict Proctor's card waits for CP-1); 603.3d-001; 603.4-001, -002, -003; 603.6-001; 603.6a-001; 603.6b-001, -002; 603.6c-001, -002; 603.10a-001, -002 (partial → full); 605.1b-001; 605.4a-001; 605.5a-001; 106.12a-001 (partial → full); 119.9-001, -002; 113.9-003; 608.2-001; 608.2a-001; 608.2k-001; 614.6-001, 614.8-002, 615.6-001 (partial → full); COMP-CLEANUP-RELOOP-001; 800.4d-001 (partial → full) | 46 — 44 of them Phase 7; ATOM-603.10a-001 carries no phase in the corpus and ATOM-800.4d-001 is Phase 9's, and both are claimed here because their trigger half is this phase's |
| **TR-2a** | 603.1b-001 (fixture); 603.2h-001; 603.2h-002 (partial — its board is Nykthos Paragon's "may"); 608.2h-001; 608.2p-001 (fixture); and 121.2c-001, a Phase 8 atom item 122's rule closes | 6 — 5 of them Phase 7 |
| **TR-2b** | 603.2h-002 (partial → full); 603.5-001; 118.12-001; 121.5-001 (partial → full); 118.12-002 (partial — a Phase 8 atom whose rule §6.2's amendment builds; its Dermoplasm board waits for morph) | 5 — 4 of them Phase 7, one shared with TR-2a |
| **TR-3a** | 603.7-001; 603.7a-001; 603.7b-001, -002; 603.7c-001; 603.7d-001; 603.7e-001; 603.7f-001; 603.7g-001 (fixture); 603.7h-001; 107.3n-001; 513.2-001, -002; and 400.7-001, whose whole board is a delayed trigger, from the deferred row | 14 — 13 of TR-3's 20, and CV-1b's partial |
| **TR-3b** | 610.3-001; 610.3a-001; 610.3b-001; 610.3c-001, -002; 610.3d-001 | 6 |
| **TR-3c** | 603.12-001 | 1 |
| **RH-3** (`replacement-architecture.md` §2b.5) | 400.7e-001, -002, moved from TR-4 on 2026-10-10, and from RH-2 to RH-3 on 2026-10-12: the record carries the object the move made | 2 |
| **TR-4** | TR-4a: 603.6e-002; 122.8-001; 122.9-001. TR-4b: 603.10c-001, -002, -003; 603.2e-002. TR-4c: 603.6e-001; 400.7f-001. TR-4d: 603.10d-001; 603.10e-001; 603.9-001 (2026-10-12, §12) | 12 |
| **TR-5** | TR-5e: 508.2a-001, moved from TR-5a at its design (2026-10-16). TR-5b: 122.7-001. TR-5d: 120.10-001, moved from TR-5c at its review (2026-10-18). TR-5g: 603.2d-001 (2026-10-12, §12), split from TR-5d at its design (2026-10-18) | 4 |
| **TR-6** | 603.8-001, -002 | 2 |
| **Deferred, with the rule that lets each wait** | 603.2a-001 (needs an "activated abilities can't be activated" restriction — RS-2's); 603.3c-001, -002 and 700.2b-001 (modes — `backlog.md` §2.7, on §5.4's placement); 607.2c-001, 607.2h-001 (linked — §2.2); 603.12a-001 and 605.3a-002 (a cost paid at resolution — CP-1, which also unlocks 702.21a-001, -002 (ward = TR-5's event + CP-1's "unless"), Strict Proctor and Frost Titan); 111.13-001, 112.2-002, 700.2g-001, 707.10b-001, 707.5-002, BOUNDARY-707.7-001, BOUNDARY-707.9g-001 (copies — CV-2, CV-4, with §6.5's sentence); 208.2b-001, -002 (copiable values from an entry choice — CV); 610.5-001, -002 (a granted keyword at cast — §2.1's convoke); 611.2e-001, 611.3d-001, -002 (their owners: 611.3d is §2.3's foretell); 115.9a-001 ("with N targets" — a filter over `chosen_targets`, Phase 8 with its first card); 701.43d-001 (exert — §2.5); 701.66a-001, -002 and 702.176a-003 (earthbend, impending — Phase 8); 724.1-001, 724.2-001, -002, COMP-MONARCH-COMBAT-001, 724.3-001, 724.5-001, 725.1-001, 725.2-002, 725.3-001 (designations — Phase 9, on §3.8's arm); 608.2d-001 (stays partial: choices at resolution are §2.7's and CP-1's); 608.2j-001 (a characteristic read — ALREADY-IMPL's, re-filed at TR-6's close) | 40 |

Ninety-three of the 133 Phase 7 atoms are owed across the six phases and RH-3 (plus
the two from outside the phase TR-1 claims, 121.2c-001, which TR-2a claims,
and the one TR-2b claims in part),
forty are deferred with an owner each; 93 + 40 = 133, no atom listed twice and none unlisted — checked
against `spec.sqlite` on 2026-09-18 (400.7-001 moved from the deferrals to
TR-3a on 2026-10-07), and worth re-checking the same way at each close. The deferrals are re-read at item 6's close audit
(`engineering-practices.md` §9 pass 1) and any whose owner has landed by
then is claimed there.

---

## 14. The survey's sixteen questions, answered

1. **CR 513.2 and "next".** Falls out: the window is the future, so a
   permanent entering during the end step and a delayed trigger created
   during it both wait for the next `StepBegin { End }`, which is next
   turn's. `DelayedTrigger.created` exists for the reflexive window and
   "this turn", not for this. TR-3 asserts both atoms.
2. **608.2b and `SpellFizzled`.** `IsCountered` reads `SpellCountered` and
   `AbilityCountered` and never `SpellFizzled`; Multani's Presence's ruling
   (2018-04-27) says so in as many words and calls it a change from the
   previous rules. TR-4.
3. **Life loss per source or per batch.** Per record: CR 120.3a makes each
   source's damage its own loss, `LoseLife` is one proposal per source, and
   the gain side's ruling (Nykthos Paragon: "each creature with lifelink
   dealing combat damage causes a separate life-gaining event") is the
   printed shape. "Whenever you lose life" (Vilis) triggers once per
   attacker; "for the first time each turn" (Vengeful Warchief) reads the
   history. TR-2a.
4. **122.6's entry counters.** A `CountersChanged` per entry row, announced
   after the entry inside its batch, never proposed. §3.12 item 17. TR-5.
5. **Unattach's routes.** Four, all the event (CR 701.3d lists them): an
   unattach, a move to another object, the attachment leaving and the host
   leaving. One record through one emitter, the route on the record, and an
   Aura put into the graveyard by 704.5m becomes unattached as it leaves,
   which its own zone change's performer announces. Grafted Wargear's ruling
   enumerates the routes. Built in TR-4b.
6. **Triggered abilities with no source.** `TriggerOrigin::Rule(InherentAbility)`
   — an arm beside the object's identity, never a fake source on it — built
   with the designation that owns it (the monarch, Phase 9). TR-6 declares
   the arm empty.
7. **When a state trigger is re-checked.** Three schedules (§4.5): every
   dispatch, every `perform_sba_and_triggers` iteration, the cleanup probe.
   "A dispatch when nothing was performed" is the second schedule, and
   702.131d is the epoch. TR-6.
8. **603.6a and the batch boundary.** Match at the close of the batch,
   every record, against the settled board (§4.1). The refinement of the
   brief's "at `emit_event`, once per record", with the rule that forced it.
   TR-1.
9. **The entry's `from`.** Joined from the same object's zone change or
   token creation in the window; no field on the entry record. TR-1.
10. **Which copies are cast.** Tier D (707.10) puts a copy on the stack and
    emits no `SpellCast`; tier E (707.12) casts through `cast_spell` and
    does. Ashling, Flame Dancer's rulings agree ("copies ... in a zone other
    than the stack ... will not ... trigger; casting the copy will"). The
    copy track builds the doors; this document states which each takes.
11. **One spell, one permanent, two instances of "target".** Once per spell
    or ability: Frost Titan's ruling ("each spell ... that Frost Titan
    becomes a target of"), Diffusion Sliver's the same; CR 115.9a's
    per-instance count is for "with N targets", a different question.
    `Targeted` is one record per (spell, target) with `instances` on it for
    whoever needs the count. TR-5.
12. **605.1b's stackless mana trigger.** Resolved at dispatch, never queued
    (§4.7). TR-1.
13. **"From anywhere" without a frame.** `from: None` is not on 603.10's
    list, so it reads the board after; Guile's two boards are the test
    (§4.3). TR-1.
14. **Delayed triggers' identity and provenance.** §3.9: who registers (a
    resolution, a rider, later a special action), what it records (source
    as of creation, controller, instant, refs by epoch, X), what its object
    reference is (`ObjectRef`). TR-3.
15. **Game-scoped lookback.** A scope on a counter, the histories kept whole
    game, the pregame sweep deferred until measured, the commander tax as
    the first game-scoped reader (§3.10). TR-2a.
16. **Two once-per-turn gates.** Two `TriggerLimit` arms, two sets, two
    writers — the dispatcher for "triggers only once", the resolution for
    "do this only once" — read at the instants the rulings name (§3.5). TR-2a.

---

## 15. Findings and open questions

Recorded here at authoring; a finding that becomes a code item moves to
`codebase-state.md` under the phase that found it.

1. **The candidate order is observable in one place only.** The
   `OrderTriggers` prompt lists a player's triggers in `seq` order, which is
   dispatch order, which is window order, which is
   `battlefield_ids_ordered` order for a batch of entries — process-stable
   end to end. A `HashMap` reaching the queue would be a determinism bug;
   `trigger_sources` is an `IdMap`, iterated to select the gate's readers
   and never for their order: the readers are sorted on
   `PermanentState::timestamp`, the key `battlefield_ids_ordered` sorts on,
   so the candidate order is the whole-battlefield walk's.
2. **The `AbilityTriggered` record and CR 603.2d.** A multiplied trigger
   queues N entries and emits N records, so Strict Proctor under
   Panharmonicon triggers twice, which is the CR's answer (each instance is
   "a triggered ability [that] trigger[ed]"). Worth a test; not a doubt.
3. **The window's `mark` and a rewound cast.** `// CAST-ROLLBACK:` moves a
   card silently and `announce_zone_change` records the move at 601.2i, so
   no record of a failed cast ever reaches a window; nothing to do, noted
   because the matcher is the first reader that would have cared.
4. **`ControlChanged.lki` is the frame before** (decided 2026-10-12,
   §3.12a). The sweep sees the new controller, and the prior frame is only
   in the memo's stale entry, a cache and not a record. This item once said
   the difference showed only when the change also removed an ability that
   conditioned on control. It shows on the controller itself. "When you lose
   control of this" (9 cards, Khârn the Betrayer among them) reads the
   controller the object had, and the frame after makes the new controller
   the trigger's (CR 603.3a), so it never matches. So the sweep keeps a frame
   per permanent while `any_control_changing` is set, and TR-4d prices it.
   TR-4d's re-read (§12) found that this keeps nothing for the first change
   on a board with no control row; its decision 1 is the fix.
5. **Golgari Brownscale's dredge** does not fit `Rewrite` (re-counted
   2026-10-12): it needs a library-size condition and a CR 616.1 choice among
   dredges as well. So TR-4c's class is a fixture, and the card waits here
   for dredge.
6. **Sneak Attack** waited for an indefinite haste, which CV-1b's prune
   made expressible (a `Duration::Indefinite` row ends when its subject
   moves); its ruling is a TR-3 fixture board.
7. **The pregame sweep** over the registry is a measurement TR-2 takes
   and records (histories advanced per record, ~40 fields, ~1,000 records
   a game); the decision to prune waits for the number. **Measured
   2026-09-24 (TR-2a):** the each arm adds the histories and nothing a
   pooled card reads. It read +0.1% CPU per decision at four seats over
   five rounds (+2.1% over three), and `Layer walks` +5%. No prune: the
   number does not ask for one.
8. **`ResolutionContext`'s rider pair** (item 137) gains no third `Option`:
   the binding is one field, and the two rider numbers stay as they are.
9. **CR 400.7e/f's "find the new object"** through a merged permanent is
   CV-7's (§10).
10. **The visibility predicate's `face_down` half** is exact today (no
    face-down permanent exists that is not a `PermanentState` flag) and
    becomes `backlog.md` §2.9's the day a reveal does.
11. **Two `Cost` arms are stubs and decide two cards.** `Cost::Discard` and
    `Cost::ExileFromGraveyard` return `Err` from both validation and payment
    (`engine/costs.rs`), which is why Immortal Coil is a fixture (§12, TR-6)
    and Anurid Brushhopper was not TR-3's delayed consumer. Neither is this
    phase's to build — a cost is `cost-architecture.md`'s — and both are
    recorded there when the first card wants one.
12. **`backlog.md` §2.2 (linked abilities, CR 607) is unscheduled and
    shares one piece with this design.** None of TR-3's four consumers is a
    linked ability — Banishing Light's "until" is CR 610.3, the template
    printed to avoid the old O-Ring linkage; Flickerwisp's "that card" is
    603.7c's object reference — but `ObjectRef` (an object remembered by id
    and epoch) is the per-ability memory 607.2c's "the exiled card" needs,
    and §2.2 should reuse it rather than mint a second one. Miracle's
    603.11 shape waits there (§16).
13. **Prompts inside a resolution or a batch are not fork points, and this
    design adds none.** Placement's prompts run at a priority grant with
    the queue on `GameState` (§5.2); a vote or a CR 616.1 choice keeps its
    continuation on the native stack (item 40's two violators), so the fork
    model as decided answers them with a policy inside a rollout, in APNAP
    order as the engine asks. Branching a search on one needs the resumable
    resolution, Phase 10's, and nothing here makes it harder.
14. **The card-authoring builder waits for its first caller** (the TR-1
    review, theme B, 2026-09-20). `cards::authoring::triggers` gives the
    printed shapes their words — `dies`, `enters`,
    `leaves_the_battlefield`, `at_beginning_of(step, Whose)` — and
    `CountableEvent::once_per_event` for CR 603.2c. The two methods the
    design also sketched, `.caused_by(ZoneChangeCause)` and
    `.owned_by(PlayerRef)`, are **not written**: no card among TR-1's five
    prints either, and none of TR-2's seven does. Two of those seven are
    zone-change triggers, Paladin of Atonement's "dies" and Elvish
    Warmaster's "enter", and neither names a cause or an owner. They are
    one method each on `CountableEvent`, whose only constructors already
    build an arm carrying both fields, so the first
    card that prints "whenever a creature an opponent controls is put into
    a graveyard from the battlefield" adds them in the PR that adds the
    card. Until then a card that asks writes the arm out — which is what
    the one fixture that asks (a discard, `cause: Some(Discarded)`) does,
    and it reads correctly, because its fields are `Some`: it was `None`
    standing for "any" that the review objected to, not `Some`.
15. **The kept LKI serves activated abilities too, and no reader
    reads theirs yet** (the rulings pass, 2026-09-23). CR 113.7a names
    activated and triggered abilities alike: "{T}: this creature deals
    damage equal to its power" with its source sacrificed in response reads
    the source's last known information, and ATOM-113.7a-001 (ALREADY-IMPL)
    proves only that the ability still resolves. TR-2b's writer hands the
    frame to every stack entry naming its source, an activated ability's
    included, so the reader is a leaf away; which reader, and whether today's
    `SourcePower` already answers it, is the first activated card's question.

---

## 16. Explicitly out of scope

- **Modes on a triggered ability** (603.3c) — `backlog.md` §2.7, which
  builds on §5.4's placement.
- **"Unless [a player] pays"** — `cost-architecture.md` §3.10, CP-1; ward
  (702.21a), Strict Proctor, Frost Titan register there, on TR-5's event.
- **Linked abilities** (603.11, 607) — `backlog.md` §2.2; miracle's
  "reveal ... as you draw it" is its first trigger customer.
- **The monarch, the initiative, rad counters** (724, 725, 727) — Phase 9's
  designations, on `TriggerOrigin::Rule`.
- **Phasing's qualifier** on CR 603.6c (item 6) — with phasing, which
  arrives second.
- **The per-viewer visibility query** (S1's body) — `backlog.md` §2.9, B4.
- **Loop detection Tiers 2 and 3, and the voluntary shortcut** — `backlog.md` §2.28,
  with the state hash and the fork harness.
- **CR 707.10b's ability copies** — CV-4, with §6.5's sentence.
- **Extra steps** (`backlog.md` §2.17's step half; Obeka) and **step-scoped
  durations** (§2.12 there) — their entries; the hooks exist.
- **"As though" effects** (609.4) — B8.
- **Bounding the in-state event log** (item 42's window) — nothing here
  reads past one batch, so the window's size is a fork-cost question, not a
  rules one.
- **Special actions that create delayed triggers** (603.7g) — `backlog.md` §2.8, with a
  fixture here.

---

## 17. Documents this owes, and what it changed on the way in

- `CLAUDE.md`'s architecture row gains `triggers` (one entry, the row's own
  rule); the critical path is unchanged and says nothing about progress.
- `roadmap-v2.md` row A6 carries this document's date and the six phases;
  §8's sizing row for item 6 reads six.
- `codebase-state.md` "Before Triggered abilities" items 1, 3, 6, 7, 9 and
  10–18 each point at their TR phase in one line; main items 9 (`PermanentState.cast`),
  11 (605.1b), 122 (S2), 149 (S3), 163 (the elision) and 121 (Eon Hub) are
  cited above and get their pointer when their phase lands.
- `copy-effects-architecture.md` owes nothing new; §6.5 is the sentence
  CV-4's §4.4 asked this document for.
- `cost-architecture.md` §3.11's Ironworks loop was TR-7's integration test,
  and since 2026-10-13 it is `roadmap-v2.md` B13's. Every step but the
  triggers exists. Step 6's "with lesser mana value" compares the target with
  the artifact that triggered Scrap Trawler, as it last existed on the
  battlefield (its second ruling), not with the trigger's source. So the leaf
  reads the binding's subject's last known information, where this line once
  planned `ObjectFilter::ManaValueLessThanSource`.
- `plans/glossary.md` gained *dispatch*, *window*, *binding*, *tier* and
  *probe* with TR-1 (2026-09-19), and `check_glossary.py --suggest` at each
  close is the gate on the rest.
- `plans/references/trigger-survey.py` deletes with the survey when TR-6
  closes, per its own docstring.
