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
> builds on and does not reopen; `cost-architecture.md` §3.11 is the
> integration test.

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
`ObjectFilter`, `Zone`, `ZoneChangeCause`, `CounterType`, `StepType` — the
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
rather than a field. The turn summary's field list is the CR's "this turn"
quantities as of today's cards, and a new lookback quantity is a field plus
an update arm, authored with the card. Modes on a trigger (603.3c), linked
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
│  stamped)      │   │  advance the turn summaries  │   │ pending_triggers:   │   │  false,      │
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
| **Reads** | the window's records, the live board, the LKI frames the records carry, the turn summaries, the delayed registry | the pending queue, the seat list, the board for targets and modes |
| **Writes** | `pending_triggers`, the turn summaries, the two once-per-turn gates, `AbilityTriggered` records, the trace's `trigger` record | the stack, `AbilityTriggered`-shaped nothing (placement announces no event), the trace's `pending` record |
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
`is_mana_ability(def)` (item 11) reads a `Triggered` def too: no target, a
condition that is `ManaAdded` or an activation or resolution of a mana
ability, and a `ProduceMana` in the effect — CR 605.1b's three criteria,
derived, never a tag.

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
`intervening_if` over the turn summary (§3.10): the ability triggers on any
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

Within an arm, `subject` is one of three readings of "which object": `This`
(CR 603.6a's "when [this object] enters", read against the ability's own
source), `Filter(ObjectFilter)` ("whenever a creature you control ..."), or
`Any`. "Another" is `Filter(And(filter, Not(Self)))` — the leaf
`ObjectFilter::NotSource` is the one new filter leaf this phase adds, a
sibling of A4i's `OtherThanInstance`. "Whose" is `PlayerRef` (`You`,
`Opponent`, `Player(_)`, and `Each` for "each player's", "a player") and
`None` for "any". `Option` on a field means the arm does not ask.

| `GameEvent` variant | `TriggerEvent` arm and its fields | Look-back? | Lands |
|---|---|---|---|
| `ZoneChange` | `ZoneChange { subject, from: Option<Zone>, to: Option<Zone>, cause: Option<ZoneChangeCause>, owner: Option<PlayerRef>, multiplicity }` — dies is `from: Battlefield, to: Graveyard`; "leaves the battlefield" `from: Battlefield, to: None`; sacrificed/discarded/milled/exiled/countered by `cause`; "from anywhere" `from: None` | iff `from == Some(Battlefield)`, `from == Some(Graveyard)`, or `to ∈ {Hand, Library}` from a zone all players can see (CR 603.10a's three classes, the third written about visibility; §4.3) | TR-1 (battlefield departures), TR-4 (the other two classes, item 15) |
| `Tapped` / `Untapped` | `BecomesTapped { subject }` / `BecomesUntapped { subject }` — transition-only by the record's own contract (603.2e) | no | TR-1 (fixture), Phase 8 (card) |
| `CardDrawn` | `DrawsCard { player: Option<PlayerRef>, multiplicity }` — never a library-to-hand `ZoneChange` (121.5) | no | TR-2 |
| `ManaAdded` | `ManaAdded { source: Option<ObjectFilter>, tapped_for_mana: Option<bool>, mana: Option<ManaType> }` — CR 106.12a's "tapped for mana" reads `tapped_for_mana` | no | TR-1 |
| `DamageDealt` | `DamageDealt { source: Option<SourcePattern>, recipient: DamageRecipient, combat: Option<bool>, multiplicity }` — "is dealt damage", "deals damage", "deals combat damage to a player"; `combat` is item 10's field | no | TR-1 |
| `PhaseBegin` / `StepBegin` / `TurnBegin` | `PhaseBegins { phase, whose }` / `StepBegins { step, whose }` / `TurnBegins { whose }` — "your upkeep", "each upkeep", "the monarch's end step"; `whose` reads item 10's `player` | no | TR-1 |
| `PermanentEnteredBattlefield` | `EntersBattlefield { subject, controller: Option<PlayerRef>, from: Option<Zone>, cast: Option<bool>, multiplicity }` — `from` and `cast` are joined from the same object's `ZoneChange` or `TokenCreated` in the window (§4.4; question 9) | no (603.6a reads the board after, with 603.6b's effects applied) | TR-1 |
| `LifeChanged` | `GainsLife { player, multiplicity }` / `LosesLife { player, cause: Option<LifeLossCause>, multiplicity }` — two arms for one record because the sign decides which printed family reads it; per record (question 3) | no | TR-2 |
| `AttackersDeclared` | `Attacks { shape: AttackShape, attacker: Option<ObjectFilter>, attacking_player: Option<PlayerRef>, defender: Option<DefenderRef> }` — CR 508.3a–e's five shapes as one enum: `Creature`, `CreatureAgainst`, `PlayerIsAttacked`, `PlayerAttacksWith`, `PlayerAttacks`, `PlayerAttacksPlayer`, `Alone`; reads item 11's defender | no; 508.2a's snapshot is the dispatch instant | TR-5 |
| `BlockersDeclared` | `Blocks { shape: BlockShape, .. }` — 509.3a–d and 509.3g's five readings of one pair list: `Blocks`, `BlocksACreature`, `BecomesBlocked`, `BecomesBlockedBy`, `AttacksAndIsntBlocked` | no | TR-5 |
| `SpellCast` | `CastsSpell { caster: Option<PlayerRef>, spell: Option<ObjectFilter>, from: Option<Zone> }` — the stack object is live at dispatch (types, colors, mana value through the layer walk; `cast_from` off its entry) | no | TR-2 |
| `AbilityActivated` | `ActivatesAbility { controller, source: Option<ObjectFilter>, loyalty: Option<bool> }` | no | TR-5 |
| `AbilityResolved` | `AbilityResolves { identity: IdentityRef }` — `This` for CR 603.7h's delayed form, read with the resolution count (§6.5) | no | TR-3 |
| `SpellCountered` / `AbilityCountered` | `IsCountered { subject: CounteredRef }` — **never `SpellFizzled`** (question 2; Multani's Presence's ruling) | yes (603.10e) | TR-4 |
| `SpellFizzled` | **no arm.** CR 608.2b's "doesn't resolve" is not "countered" under the baseline CR, and no printed trigger reads it | — | — |
| `PlayerLost` | `PlayerLoses { player, reason: Option<LossReason> }` — 603.9's "unless as the result of a draw": a draw is `GameResult::Draw`, not a `PlayerLost` | yes (603.10f) | TR-4 |
| `PlayerWon` | **no arm** — no printed trigger; recorded so the projection stays one-to-one | — | — |
| `Scried` | `Scries { player }` — Elrond's X is `TriggerBinding.amount = looked_at` | no | TR-5 (fixture) |
| `LibraryShuffled` | `ShufflesLibrary { player }` — two printed watchers, Cosi's Trickster and Psychic Surgery ("whenever an opponent shuffles their library"; the survey's 168 on this row is the substring over-count); one record per shuffle, an empty or one-card library included, and never for cascade's random bottom (Cosi's Trickster's rulings) | no | TR-2 |
| `CountersChanged` | `CountersPutOn { subject, kind: Option<CounterType>, by: Option<PlayerRef>, nth: Option<u32>, multiplicity }` / `CountersRemovedFrom { .. }` — the sign splits the arm as it does life; `nth` is CR 122.7's before/after read live (count now minus `added`); **an occurrence is a counter, not a record** — Protean Hydra's ruling: several +1/+1 counters removed at once trigger "whenever a +1/+1 counter is removed" that many times, so `PerOccurrence` on these arms multiplies by the count and `OncePerEvent` is Simic Ascendancy's "one or more" | no | TR-5 |
| `CountersAnnihilated` | **no arm, and the variant goes** — CR 704.5q's annihilation *is* a removal of counters: Protean Hydra's ruling has a -1/-1 counter meeting a +1/+1 counter trigger "whenever a +1/+1 counter is removed". Today the state-based sweep writes both kinds directly and announces this variant (Deferred Migrations item 6's counter half, `sba.rs:484`); TR-5 routes it through two `RemoveCounters` proposals in the state-based batch, so the annihilation is two `CountersChanged` records the removal arm reads, and the variant is deleted with item 18's three | — | TR-5 |
| `Attached` | `BecomesAttached { attachment: Option<ObjectFilter>, host: Option<ObjectFilter> }` — transition-only (701.3b), so re-equipping the same creature announces nothing (603.2e-002) | no | TR-4 |
| `EquipmentDetached` → **`Unattached`** | `BecomesUnattached { attachment, former_host }` — one record for CR 701.3d's three routes (question 5), replacing `EquipmentDetached` | yes (603.10c): the frame is the attachment's, with `attached_to` from item 14 | TR-4 |
| `LeftTheGame` | folded into `ZoneChange`'s leaves-the-battlefield reading: a `LeftTheGame` from the battlefield matches `from: Battlefield, to: None` and nothing narrower, which is CR 603.6c's own sentence; the phased-in qualifier is item 6's and waits for phasing | yes (the record carries the frame since RE-7) | TR-1 |
| `TokenCreated` | `CreatesToken { owner, zone: Option<Zone>, kind: Option<TokenKind>, multiplicity }` — keyed here and never on `is_token` at entry (item 8; CR 111.13) | no | TR-5 (fixture) |
| `TokenCeasedToExist` | **no arm** — CR 704.5d names no trigger event, and what cards observe is the *absence*: Flickerwisp's ruling has an exiled token "cease to exist and won't return", which is a delayed trigger's `ObjectRef` finding nothing (§3.9), not an event to match | — | — |
| `StateBasedActionPerformed` | **no arm** | — | — |
| *new* `Targeted` (item 12) | `BecomesTarget { subject: TargetRef (object or player), by: Option<TargetingFilter> (a spell, an ability, "an opponent controls", "an Aura spell"), first_time_each_turn }` — once per spell or ability, however many instances (question 11) | no | TR-5 |
| *new* `ControlChanged` (item 13) | `ControlChanges { subject, from: Option<PlayerRef>, to: Option<PlayerRef> }` — "loses control", "an opponent gains control of a permanent you own" | yes (603.10d), with the caveat in §15 item 4 | TR-4 |
| *new* `DamagePrevented` (item 16) | `DamageIsPrevented { target: DamageRecipient, multiplicity }` — one record per prevention applied per subject group (CR 615.13) | no | TR-5 |
| `AbilityTriggered` (new, §4.8) | `AbilityTriggers { caused_by: Option<TriggerEvent> (Strict Proctor's "a permanent entering causes"), of: Option<ObjectFilter> }` — **the arm that is CR 603.3b's second tier**: `TriggerCondition::tier()` reads it | no | TR-1 (the event and the tier), TR-5 (the arm's card) |

Thirty-four variants, four deleted (three never emitted, and
`CountersAnnihilated` once CR 704.5q proposes), four added (three from the
survey's gaps and `AbilityTriggered`), one renamed: **thirty-four performed
kinds after TR-5, thirty with an arm and four with none, each of the four
named above with the reason.** Two kinds carry two arms each — life and
counters, where a signed field splits two printed families — which is the
one place the projection is not one-to-one, and it is a property of the
record, not a second axis. `PhaseType` and `StepType` are
the engine's own; "at end of combat" is `StepBegins { step: EndCombat }`
(511.2) and "at end of turn" is `StepBegins { step: End }` (513.1a), which
is why the three `*End` variants die.

### 3.4 Subjects, "you", and the bound facts — `TriggerBinding`

A trigger's effect refers back to its event: "that creature", "that player",
"that many", "it" in "return it to the battlefield", "damage equal to that
creature's power". CR 608.2k says the reference survives characteristic
changes; CR 603.6 says a zone-change trigger looks for the object in the
zone it moved to and finds nothing if it left; CR 113.7a and 608.2h say
information about an object that is gone is its last known information.
The engine's answer is a struct that **points at the records and copies
nothing they hold**, filled at dispatch and carried on the `PendingTrigger`
and then the `StackEntry`:

```rust
pub struct TriggerBinding {
    /// The records that matched: one for a `PerOccurrence` trigger, every
    /// matching record of the window for a `OncePerEvent` one. By stable
    /// id, never by `Vec` index — `EventSeq` is the log's monotonic
    /// sequence number (today the index; the trace's `seq` already), and
    /// `EventLog::record(seq)` is the read. Their `EventStamp` is what the
    /// reflexive check and CR 603.7h's "this ability" read.
    pub records: Vec<EventSeq>,
    /// Which of the condition's events matched — the `TriggerEvent` whose
    /// projections (below) say what "that object", "that player" and
    /// "that many" are for these records.
    pub event: EventIndex,
    /// The one fact a record does not carry and the resolution needs: the
    /// subject's `zone_change_epoch` at dispatch (CR 400.7 — a later move
    /// makes it a new object the reference cannot find;
    /// `codebase-state.md` item 10's field). `None` for an event about no
    /// object.
    pub object: Option<ObjectRef>,
    /// The ability whose triggering this is (CR 603.3b's second tier), or
    /// for a reflexive trigger, the action within the resolution.
    pub triggered: Option<TriggerSeq>,
}
```

**The projections live on the arm, and they are exhaustive matches.**
`TriggerEvent` carries four methods — `subject_of(&GameEvent) ->
Option<ObjectId>`, `player_of(..) -> Option<PlayerId>`, `amount_of(..) ->
Option<u64>`, `occurrences_of(..) -> u32` — each a `match` over the arms
with no wildcard, so a new arm must say what its "that many" is (damage
dealt, cards looked at, counters put on, damage prevented) or fail to
compile, and must say what an occurrence is (a record for most kinds; a
counter for the counter arms, per Protean Hydra's ruling; an attacker for
the attack shapes). The readers resolve through them at resolution:
`EffectRecipient::TriggeringObject` is `object` checked against the live
object's epoch — CR 603.6's "unable to be found" and CR 400.7 in one
comparison; `EffectRecipient::TriggeringPlayer` is `player_of` on the
records; `AmountExpr::TriggeringAmount` is `amount_of` summed over them
(Simic Ascendancy's "that many" across a batch); and
`AmountExpr::TriggeringPower` reads the live object when it is where the
event left it and the record's `lki` frame otherwise (CR 608.2h). All
three leaves ship in TR-1 because the type that carries them opens there.

**Why no stored amount.** A single `Option<u64>` on the binding would mean
a different field per arm with nothing forcing a new arm to declare which,
and a number copied at dispatch is a second copy of a fact the record
already materializes. Pointing at the record keeps one copy, and the frame
comes with it. The cost is one constraint on item 42's future window: **no
record a pending or stacked trigger references may be evicted** — the
window's floor is the oldest referenced `EventSeq`, which a trigger holds
for at most the few batches between its dispatch and its resolution.

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
    DoOnceEachTurn,
    /// "This ability triggers only once each turn" — no rule of its own in
    /// the baseline CR (CR 702.179d's speed is the one use), so the earliest
    /// printing's ruling defines it: Elvish Warmaster's, "once the triggered
    /// ability has triggered once during a turn, it can't trigger again,
    /// even if [it] is still on the stack, has been countered, or has
    /// otherwise left the stack". A triggered gate: written by the
    /// DISPATCHER as it queues; once per source object per instance; once
    /// for a batch; read before CR 603.2d's multiplier applies, so
    /// Panharmonicon cannot double it.
    OnceEachTurn,
    /// "Whenever [event] for the first time each turn" (Kira, Vengeful
    /// Warchief): not a gate on the ability but a predicate on the event —
    /// this record is the first of its kind in the player's turn summary
    /// (§3.10). Written by nobody; read at dispatch, record by record.
    FirstTimeEachTurn,
}
```

The two gate sets live on `GameState` beside the turn summaries:
`action_taken_this_turn` and `triggered_this_turn`, each an `IdSet` keyed by
the full identity (§3.6) so a bounced and replayed permanent — a new object,
CR 400.7 — starts clean, and both cleared by the `BeginTurn` performer.
"Each turn" is the game's turn, not the controller's.

### 3.6 `AbilityIdentity` gains `instance` and the object's epoch (S3)

```rust
pub struct AbilityIdentity {
    pub source: ObjectId,
    /// CR 400.7 — which existence of `source` this is. Stamped by
    /// `move_object` today (item 10's field); two activations across a
    /// bounce are two abilities' worth of counting.
    pub zone_change_epoch: u64,
    pub ability: AbilityId,
    /// The k-th instance of `ability` on `source`, in effective-list order.
    /// A4g made an `AbilityId` per definition, so two sources granting one
    /// ability put it on an object twice under one id (item 149). For a
    /// mana ability nothing tells the two apart in outcome; for a TRIGGERED
    /// one the outcome differs — Diffusion Sliver's ruling: the abilities
    /// Slivers grant "are cumulative", so a Sliver under two Diffusion
    /// Slivers triggers twice and the opponent pays twice — and a
    /// dispatcher keyed on `(source, ability)` alone would fold the two
    /// into one trigger. Each instance triggers, is placed, and is its own
    /// gate; CR 603.7h counts the ABILITY (§6.5), so the counters key on
    /// the pair and ignore this field.
    pub instance: u32,
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
    /// The def, shared: the source's effective list is an `Arc<Vec<..>>`
    /// and the def is cloned out of it once, here, so a Humility that
    /// lands between triggering and placement cannot un-trigger it
    /// (CR 113.7a — "exists on the stack independently of its source"
    /// begins at the trigger for everything but the text's own reading).
    pub def: Arc<TriggerDef>,
    /// What `GameObject::new` needs to build the stack object — the name and
    /// display the source's card gives it, as `activate_ability` clones the
    /// source's `card_data` today. Held here, behind an `Arc`, because the
    /// source may be gone by placement: a dies trigger's source is in the
    /// graveyard and a `LeftTheGame` source is not in the object map at all.
    /// CR 603.3 gives the object "the text of the ability that created it,
    /// and no other characteristics", and nothing reads this card's types.
    pub source_card: Arc<CardData>,
    pub binding: TriggerBinding,
    /// CR 603.3b's tier: 1 unless the condition is another ability triggering.
    pub tier: TriggerTier,
    /// CR 605.1b — resolved at dispatch, never queued; here for the record.
    pub mana: bool,
    /// CR 603.8's one-shot: a state trigger stays armed-off until its stack
    /// object leaves (§4.5).
    pub state: bool,
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
    /// that has begun does not "back up"), and the reflexive window.
    pub created: Instant { turn: u32, record: usize },
    /// CR 603.7b — once, or a stated duration.
    pub duration: DelayedDuration,   // Once | ThisTurn | UntilEvent(..)
    /// CR 603.7c — the objects it refers to, by id and epoch (CR 400.7).
    pub refs: Vec<ObjectRef>,
    /// CR 107.3n — X, inherited from the creating spell when unstated.
    pub x: Option<u64>,
    /// CR 603.12 — `Some(stamp)` for a reflexive trigger: checked at once
    /// against the records that carry this stamp since `created.record`.
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

### 3.10 `TurnSummary`, `PlayerHistory`, and the game scope (item 42; P2–P4; question 15)

```rust
/// One player's turn, materialized: every "this turn" quantity the CR or a
/// registered card reads, one field each, one meaning each — never a scan
/// of the log. Advanced at dispatch, record by record (§4.1).
pub struct TurnSummary {
    pub turn: u32,
    pub spells_cast: u32,
    pub creature_spells_cast: u32,
    pub noncreature_spells_cast: u32,
    pub cards_drawn: u32,
    pub life_lost: u64,           // total
    pub life_loss_events: u32,    // "for the first time each turn"
    pub life_gained: u64,
    pub life_gain_events: u32,
    pub lands_played: u32,        // moves off PlayerState with the first reader
    pub creatures_died: u32,      // permanents this player controlled that died
    pub attacked: bool,
    pub damage_dealt_to_players: u64,
    pub counters_put: u32,
    pub abilities_resolved: HashMap<(ObjectId, u64, AbilityId), u32>,  // CR 603.7h, §6.5
}

/// Every turn of the game, for every player — the survey's recommendation
/// taken: a few dozen counters per player per turn is kilobytes at
/// Commander scale, "this game" becomes a fold, "last turn" an index, and
/// "since the beginning of your last turn" a range. Indexed by turn
/// number: "last turn" (Paladin of Atonement — its ruling: whether you
/// lost life last turn, whoever's turn it was) is `turns[turn - 1]`, "your
/// last turn" (CR 730's day/night, Arboria, Concert Kaboomist) is the row
/// for the turn before `last_turn_began[player]`, which `own_turns` keeps.
pub struct PlayerHistory {
    pub turns: Vec<TurnSummary>,
    pub own_turns: Vec<u32>,
}
// on GameState: history: Vec<PlayerHistory>, one per seat
```

**Three decisions.** *Whole game, not two turns*: the brief's recommendation,
because the pregame sweep that would prune it (the state-tracking doc's
`RelevantEffects`) is an optimization over a static property of the
registry and pays only if measured — deferred until a reading says it
should, with the fallback the doc already names (conjure, wishes: track
everything). *A game-scoped quantity is a scope on a counter, not a window
on the log*: Approach of the Second Sun's casts and CR 903.8's commander tax
are folds over `turns`, and the tax — `cost-architecture.md` §3.8, waiting
on designation — becomes the first game-scoped reader, a field
`commander_casts_from_command_zone` on the summary the day B2 lands. *A
quantity no field anticipates is a field plus an update arm, authored with
its card* — the postscript's rule, restated here so it is this document's
too. The `Condition` leaves that read it are `ThisTurn(TurnFact, Cmp)`,
`LastTurn(..)`, `SinceYourLastTurn(..)`, `ThisGame(..)`, three edits each
(the variant, the `holds` arm, the `condition_reads` arm, which for a
summary read is "nothing" — no frame is read).

### 3.11 `LastKnownInformation` — the LKI frame, widened (items 14, 15)

CR 603.10 says "the appearance of objects immediately prior to the
event"; CR 113.7a and 608.2h call the same fact "last known information",
and the tree has called the field `lki` since RA — so the type takes the
rules' name for the fact rather than for the look. Persist, undying, an
Aura's 603.6e trigger and the four "becomes unattached" Equipment each
read a **status**, which `EffectiveCharacteristics` deliberately does not
carry (it is the layer walk's output). So the frame is a pair:

```rust
pub struct LastKnownInformation {
    pub characteristics: EffectiveCharacteristics,
    /// `None` for an object that was not a permanent. CR 110.5's four
    /// statuses plus the two facts the four printed families read.
    pub status: Option<Status>,
}
pub struct Status {
    pub tapped: bool,
    pub counters: HashMap<CounterType, u32>,
    pub attached_to: Option<ObjectId>,
    pub damage_marked: u32,
    pub controller_since_turn: u32,
}
```

`ZoneChange.lki`, `LeftTheGame.lki` and the new `ControlChanged.lki` carry
`Option<Box<LastKnownInformation>>` in place of `Option<Box<EffectiveCharacteristics>>`
— three field sites, seven literal sites — and the capture in
`perform_zone_change` and `owned_objects_leave` copies the status off
`PermanentState` beside the uncached walk it already makes (~15 lines, item
14's size). **The capture condition widens to CR 603.10a's three classes**
(item 15): `from == Battlefield` (today), `from == Graveyard`, and `to ∈
{Hand, Library}` from a zone all players could see — CR 603.10a's own
words for the third class; a Brainstorm put-back from a hand was never a
shared fact, so it has nothing to look back at and stays `None`. **The
reader methods live on the type** — `power()`, `had_counters(kind)`,
`host()`, `controller()` — and `TriggerBinding`'s readers answer live when
the object is still where the event left it and from the frame otherwise
(CR 608.2h, 113.7a); there is no second view type. Item 3's "three ad-hoc reads in `sba.rs`" are two existence probes
today (`sba.rs:327`, `:425`, both `?` ahead of a subtype read) and not
frames; the item closes on the reader existing, not on those lines.

### 3.12 The stream's records — items 10 to 18, decided

Each of the survey's nine gaps, absorbed or declined:

| Item | Decision | Phase |
|---|---|---|
| **10** — three performers drop a proposal field | absorbed as three fields: `StepBegin.player` and `PhaseBegin.player` (`GameAction::BeginStep`/`BeginPhase` already carry it; the survey's 1,167 + 976 + 313 + 391 cards read "whose"); `DamageDealt.is_combat` (804 cards, and not live-derivable — a trigger resolving during the combat damage step deals noncombat damage); `LifeChanged.cause: LifeLossCause` (one card, CR 727.1a, and the field is free). Literal sites: `StepBegin {` 5 in `src` and 12 in `tests`, `DamageDealt {` 10 and 7, `LifeChanged {` 10 and 9 | TR-1 |
| **11** — `AttackersDeclared` carries no defender | absorbed: `attackers: Vec<(ObjectId, AttackTarget)>` at the one emit site; `BlockersDeclared` becomes **one record per declaration step** carrying every defending player's pairs, because 509.3g's "attacks and isn't blocked" is a fact about the whole declaration and several defenders declare in turn | TR-5 |
| **12** — no event announces a target being chosen | absorbed: `GameEvent::Targeted { target: TargetRef, by: ObjectId, controller: PlayerId, instances: u32 }`, one record per (spell or ability, distinct target), emitted where CR 601.2c's announcement completes — `cast_spell` (ahead of `SpellCast`, both at 601.2i), `activate_ability`, and placement's 603.3d | TR-5 |
| **13** — no event announces a control change | absorbed as **a sweep, not a write hook**: control is a computed value and "gains control" is a "becomes" on the walk's output, so the dispatcher's state check (§4.5) compares every permanent's effective controller against `PermanentState.announced_controller` — a materialized field with one meaning, the controller the stream last announced — gated on `RegistryScopeSummary.any_control_changing`, and emits `ControlChanged { object, from, to, lki }` per change. Every route lands there: the registry write, its expiry at cleanup, a stripped granting ability, a conditional row flipping | TR-4 |
| **14** — the frame carries no status | absorbed: `LastKnownInformation` (§3.11) | TR-4 |
| **15** — the frame is captured for battlefield departures only | absorbed: the three classes (§3.11) | TR-4 |
| **16** — no event for a prevention applying | absorbed: `GameEvent::DamagePrevented { source, target, amount, by: ReplacementInstanceId }`, emitted by `apply_rewrite`'s prevention leg once per prevention instance per subject group (CR 615.13's "each time a prevention effect is applied to one or more simultaneous damage events"); inside the batch, ahead of the `DamageDealt` it reduced. Not a performer emitting — the application is the event | TR-5 |
| **17** — entry counters announce nothing | absorbed, the survey's question 4 decided: the `EnterBattlefield` performer announces one `CountersChanged` per `EntryCounters` row after `PermanentEnteredBattlefield`, inside the batch, with a new `by: Option<PlayerId>` on the record (CR 122.6a's putter, which the proposal has and the record dropped). Announced, never proposed, so CR 614.16's doublers see nothing twice; CR 122.7's "before" reads the live count minus `added`, which for an entry is zero | TR-5 |
| **18** — three variants never emitted | absorbed: `PhaseEnd`, `StepEnd`, `TurnEnd` deleted with their `format_event` arms. No printed trigger reads an end (511.2, 513.1a), and an arm the stream cannot carry misleads the reader. `CountersAnnihilated` is the fourth deletion, in TR-5, once CR 704.5q proposes (§3.3) | TR-1 |

Plus one record this design adds for its own use: **`AbilityTriggered {
seq, identity: TriggerOrigin, controller, caused_by: usize }`**, the event
CR 603.3b's second tier watches (§4.8). And one renamed: `EquipmentDetached`
becomes `Unattached { attachment, former_host, cause: UnattachCause }` with
three emitters behind one `announce_unattached` (question 5).

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
`records_from(mark)` with `mark` taken at the outermost `open_batch` — the
suffix `EventLog` was built to hand the matcher (item 42: "the trigger
matcher's suffix as the window's only in-state consumer").

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
each: the turn summaries advance (§3.10), then the record is matched. So
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
   `This` moving from zone Z functions in Z ("when this card is put into a
   graveyard from anywhere" functions everywhere it can be; Guerrilla
   Tactics' discard functions in the hand); a condition about other
   objects functions on the battlefield unless 113.6b states otherwise.
4. **The record's own subject**, wherever it is now — CR 603.10's default
   is "objects that exist immediately after an event", and a card
   discarded into a graveyard exists there with its printed abilities.
5. **The delayed registry** (§4.6) and **rule-owned abilities** (§3.8, none
   built).

**The gate, three legs on each of two sets**, mirroring `gather` exactly
because `CLAUDE.md` says a new reader of the effective list is dead on every
board a gate skips: `trigger_sources: IdSet<ObjectId>` — permanents that
*printed* a triggered ability or a trigger multiplier, inserted by
`place_on_battlefield`, removed by `cleanup_zone_state`, over-approximating
in one direction only; `RegistryScopeSummary.granted_trigger_zones` (Layer
6); `copied_trigger_zones` (a copy, `copy-effects-architecture.md` §4.7's
leg); and the zone set above. `puts_a_triggered_ability(def)` is the
predicate beside `puts_a_replacement_ability`. A dispatch on a board where
every set is empty and the delayed registry is empty returns after five
probes — the whole of what today's pools pay.

**S1 — visibility (CR 603.2f).** "The object with that triggered ability is
at no time visible to all players" is asked per candidate, at the dispatch
instant, of the object as the event left it: `oracle::visible_to_all(game,
id)`, whose body is `zone.is_public() && !face_down` today and whose body
is `backlog.md` §2.9's to replace. A frame candidate is exempt — it was a
permanent, which is visible — and a library card asked about a creature
entering is refused by it (ATOM-603.2f-001). The 603-2f postscript's
reading is adopted: the instant is after the event, not the object's
history, so Coercion's reveal before the discard does not count and Future
Sight's revealed top card does. `Zone::is_public()` gets its first caller
here and `backlog.md` §2.9 is not pulled forward.

### 4.3 Look-back (CR 603.10), and "from anywhere" (question 13)

A condition looks back — reads the frame for the ability's existence and
the object's appearance — iff it is on CR 603.10's list, and the list is a
property of the arm's fields, never a flag a card sets:

- `ZoneChange { from: Some(Battlefield) }` — leaves-the-battlefield, dies
  (603.10a's first class, 603.6c);
- `ZoneChange { from: Some(Graveyard) }` — leaves a graveyard (second class);
- `ZoneChange { to: Some(Hand) | Some(Library) }` — put into a hand or
  library from a public zone (third class);
- `BecomesUnattached` (603.10c), `ControlChanges` (603.10d), `IsCountered`
  (603.10e), `PlayerLoses` (603.10f). 603.10b (phases out) waits for
  phasing; 603.10g (planeswalks) is out of scope.

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

### 4.4 Occurrence: per record, per window, and the multiplier

- **`PerOccurrence`** (the default): one trigger per occurrence, and the
  arm's `occurrences_of` says what an occurrence is — a record for most
  kinds, a counter for the counter arms (Protean Hydra's ruling: several
  removed at once trigger that many times), an attacker for the attack
  shapes. A wipe of three lands is three `ZoneChange`s in one batch and
  three triggers (ATOM-603.2c-001); Soul Warden entering beside two
  creatures triggers twice (its ruling).
- **`OncePerEvent`** ("one or more"): one trigger per window in which any
  record matches; the binding holds every matching record,
  `TriggeringAmount` is `amount_of` summed over them, and `object` is `None`.
  CR 603.2c's boundary is the `BatchId`, which the envelope was built to
  carry (`events/event.rs` says so in as many words).
- **The join** for `EntersBattlefield { from, cast }`: the entry record and
  the same object's `ZoneChange` or `TokenCreated` are both in the window;
  the matcher reads `from` off the zone change and "cast" off
  `PermanentState.cast` (main item 9, CR 400.7d — who cast it and from
  which zone, two facts the permanent keeps, recorded by the entry
  performer off the stack entry the proposal's zone change came from;
  Coal Stoker reads "from your hand", Prized Amalgam "from your
  graveyard"), and a token has neither. No field is added to the entry record (question 9).
- **CR 603.2d's count modifier** is a static ability, discovered off the
  effective list at dispatch behind the same gate: `Effect::TriggerMultiplier
  (TriggerMultiplierDef { caused_by: TriggerEvent, of: ObjectFilter,
  additional: u32 })`. Panharmonicon's rulings are the edges: only the
  object's own triggered abilities, never 603.6d's entry statics or a
  replacement, never a delayed or reflexive trigger it creates; two
  Panharmonicons make three, not four (additive); each instance is its own
  `PendingTrigger` with its own `seq`, choices and targets. The gate
  `OnceEachTurn` is read before the multiplier, so a capped ability triggers
  once under Panharmonicon (the survey's judge-literature note); `DoOnceEachTurn`
  is read after, so both doubled instances go on the stack and one acts.

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

The same pass is where **`ControlChanged` is announced** (item 13): for each
permanent, when `any_control_changing`, compare `get_effective_controller`
against `PermanentState.announced_controller`; on a difference emit the
record and write the field. A field with one meaning, the last announced
controller; a "becomes" on a computed value, detected where computed values
are re-read.

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

A **reflexive** trigger (603.12) is a delayed entry with `reflexive:
Some(stamp)`, created by `Effect::Reflexive { when: ReflexiveEvent, then }`
as the resolution reaches it, and **checked immediately**: the window is
the records with that stamp since `created.record`. `ReflexiveEvent` is a
`TriggerEvent` restricted to what the resolution's own instructions can
perform — "when you do" is the `ZoneChange { cause: Sacrificed }` the
preceding `Optional` proposed; if none matched, the entry is dropped
silently. Heart-Piercer Manticore's first ruling is the shape ("goes on the
stack without a target ... a second ability triggers and you pick a
target") and its last is the count ("you can't sacrifice multiple creatures
to deal damage multiple times"). 603.12a's "one or more times" is
`OncePerEvent` over that window; its payment loop is CP-1's (§13).

### 4.7 Triggered mana abilities (CR 605.1b, 605.4a; question 12; main item 11)

A `PendingTrigger` whose def `is_mana_ability` is never queued: the
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
controller. The recursion is bounded by the abilities present — no
registered or printed ability watches its own kind — and a depth past
`BATCH_NESTING_LIMIT` is an `Err`, the engine's mistake, not a rules
answer.

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
decision resets it. Tiers 2 and 3 need the state hash item 40's discipline
makes possible and stay in `backlog.md` §2.28 with the fork harness; the voluntary
shortcut (D26) stays there too. `fuzz_games`' turn limit keeps standing in
for what Tier 1 cannot see.

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
`seq` order, two or more only. **Its elision is item 163's, inherited and
not re-derived**: two entries that are instances of one ability under one
controller with no targets and no modes and identical bindings give the
same game in either order, so the engine does not ask — with the expiry
conditions written beside the predicate the way `ordering_cannot_change_outcome`
carries item 47's: a binding that differs, a target, a mode, or an effect
that reads the stack (a tier-2 trigger) reopens the prompt. A decorator's
timestamp order for a human under the toggle, and the agent's own for a
bot, are `backlog.md` §2.22's rows 8 and 9 and not the engine's.

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
made ... the ability is simply removed from the stack": the engine never
creates the object — the outcome is the same and there is no stack to
remove it from — and announces nothing, because a trigger removed this way
is not countered (CR 701.6a is about a spell or ability on the stack being
canceled by something) and no printed trigger watches it. Modes (603.3c)
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

---

## 6. Resolution

### 6.1 The intervening "if" at both instants (CR 603.4, 608.2a)

At dispatch, `TriggerDef.intervening_if` is evaluated after the condition
matches and before the entry is queued; false means no trigger (ATOM-603.4-002).
At resolution, `resolve_taken` evaluates it again before anything else
(608.2a's place in 608.2's order — ATOM-608.2-001) and removes the object
doing nothing if it is false (-003). The evaluator is `settled_holds` for a
condition about the board, and a `TriggerContext` variant of it for a
condition about the bound facts — persist's "if it had no -1/-1 counters
on it" reads the record's `lki` frame at both instants, since the
creature is in the graveyard at both. `Condition` is one enum for the
static "as long as", the intervening "if" and the state trigger, as
`layers-architecture.md` §13b decision 5 planned; a leaf is three edits.

### 6.2 "May" and "unless" (CR 603.5)

The ability goes on the stack regardless; the choice is at resolution.
`Effect::Optional(inner)` resolves through `ChoiceKind::OptionalEffect {
source }` — a yes/no with two `ChoiceOption`s, subject the resolving
object, asked once per resolution (main item 24's yes/no ask, built here
with its first trigger customer). "If you do" is a `Conditional` on the
optional's outcome, which `Effect::Optional` reports as a bool the
resolver threads to the next atom — item 24's "no vocabulary" closed by a
field on the resolution context, `last_optional_taken`. "Unless [a player]
pays [cost]" is a payment inside a resolution, `cost-architecture.md`
§3.10's shape and CP-1's slot; Strict Proctor, Frost Titan and every ward
wait for it (§13).

### 6.3 The bound facts, LKI, and CR 603.6's "unable to be found"

`EffectRecipient::TriggeringObject` resolves against `ObjectRef`: the
object at that id whose `zone_change_epoch` still equals the binding's. A
creature that died and was returned before the trigger resolves is a new
object and the recipient resolves to nothing — the atom "put a +1/+1 counter
on it" after a bounce (ATOM-603.6-001) and CR 603.6c's "checks for it only
in the first zone that it went to" (-001/-002), which is the same
comparison read the other way. `AmountExpr::TriggeringPower` and its
siblings read `LastKnownInformation` (§3.11): live if the object is where the event
left it, the frame otherwise. Nothing the effect reads is copied at
dispatch that the record does not already hold — the binding is indices
and one `Arc`.

### 6.4 "Do this only once each turn", written by the resolution (CR 603.2h)

For a def with `TriggerLimit::DoOnceEachTurn`, the resolver checks
`action_taken_this_turn` before performing the effect: present means the
instance does nothing (Nykthos Paragon's fourth ruling — a second instance
on the stack resolves and no prompt is asked); absent means perform, then
insert. Two Paragons are two identities and act twice (second ruling).

### 6.5 CR 603.7h's count, and the copy (S3, CR 707.10b)

`AbilityResolved` advances `TurnSummary.abilities_resolved[(source,
epoch, ability)]` — the pair, per Ashling the Pilgrim's ruling, and the
delayed form ("when this ability has resolved for the third time this
turn") reads the same field through `AbilityResolves { identity: This }`
plus a count condition. The instance field is ignored by the count. A copy
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
has no effect past the third. Three edits for the leaf, in TR-2 with the
count.

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

---

## 7. The trackers — what is materialized, and what is never derived

"Not derived live" cuts both ways (`CLAUDE.md`; item 42): a condition that
scans even a short window of events is deriving CR state, and a stored
field must mean one thing. Every fact this design reads from the past is a
field with one writer:

| Fact | Field | Writer | Readers |
|---|---|---|---|
| "this turn" quantities, "last turn", "your last turn", "this game" | `PlayerHistory.turns[..]` (§3.10) | the dispatcher, record by record | `Condition::ThisTurn/LastTurn/SinceYourLastTurn/ThisGame`, `FirstTimeEachTurn` |
| the action was taken this turn (603.2h) | `action_taken_this_turn: IdSet<AbilityIdentity>` | the resolution | the dispatcher, the resolution |
| the ability triggered this turn ("only once each turn") | `triggered_this_turn: IdSet<AbilityIdentity>` | the dispatcher | the dispatcher |
| a state trigger is on the stack (603.8) | `state_triggers_armed_off: IdSet<AbilityIdentity>` | the dispatcher (arm off), `trigger_left_stack` (re-arm) | the state check |
| resolutions per ability per turn (603.7h) | `TurnSummary.abilities_resolved` | the dispatcher, off `AbilityResolved` | the count condition |
| the controller the stream last announced (item 13) | `PermanentState.announced_controller` | placement, the state check's sweep | the sweep |
| who cast this permanent, and from which zone (400.7d; main item 9) | `PermanentState.cast: Option<CastFacts { by: PlayerId, from: Zone }>` | the entry performer, off the stack entry (`controller`, `cast_from`) the proposal's zone change came from | `EntersBattlefield { cast }`, "if you cast it", Coal Stoker's "from your hand", Prized Amalgam's "from your graveyard" |
| the object a delayed trigger refers to (603.7c) | `DelayedTrigger.refs: Vec<ObjectRef>` | the producer | the delayed check, the resolution |
| when a delayed trigger was created (603.7a, 513.2) | `DelayedTrigger.created` | the producer | the reflexive window; nothing else needs it (§4.6) |
| which extra turn "that turn" is | `ExtraTurnId` on `turn_queue` entries and `GameState.current_turn_origin` | `Primitive::ExtraTurn`, `begin_turn` | `StepBegins { whose: Turn(id) }` |
| the trigger's event, subject, amount, frame | `TriggerBinding` (record ids, the matched arm, the subject's epoch — nothing the records hold) | the dispatcher | the resolution, through the arm's projections |

The pending queue, the delayed registry, the histories and the four sets
are `GameState` fields, cloned with a fork. The window (`records_from`) is
read at dispatch and never later: a resolution that wants "what happened"
reads its binding, not the log. Item 42's bounded window stays open and
unblocked by this — nothing here reads further back than one batch.

---

## 8. Engine interaction points — counted against the tree

| Site | Today | Change | Phase |
|---|---|---|---|
| `GameState::emit_event` (`state/trace.rs:485`) | one door, writes the trace's `event` record and the log | after the write: if the stamp has no batch, dispatch this record | TR-1 |
| `execute_actions` (`engine/actions.rs:647`) and `execute_actions_new_batch` (`:742`) | open, decide, perform, riders, close | after the riders of the *outermost* call: dispatch the window; `execute_actions_new_batch` dispatches its own | TR-1 |
| `perform_sba_and_triggers` (`engine/priority.rs`) | the stub | the state check, then `place_pending_triggers`; returns whether anything was placed | TR-1 |
| `Game::run_turn_steps`' cleanup branch (`state/game.rs`) | `check_state_based_actions` as the 514.3a probe | probe becomes SBAs-would-perform OR triggers-waiting OR state-check | TR-1 |
| `put_on_stack.rs::activate_ability` (`:437`) | builds the `StackEntry` with `is_spell: false` | `trigger: None`; the placement path builds its twin; `Targeted` emitted after `announce_targets` | TR-1, TR-5 |
| `stack.rs::resolve_taken` (`:120–230`) | resolves, announces `AbilityResolved` | the intervening "if" first (608.2a); `trigger_left_stack` for a state trigger; the binding threaded into `ResolutionContext` | TR-1, TR-6 |
| `stack.rs::handle_fizzle`, `resolve.rs`' `CounterAbility` arm, 603.3d's removal | remove the object | call `trigger_left_stack` | TR-6 |
| `resolve.rs` (`:220–240`) | `Conditional`, `Optional` → `Err` | `Conditional` resolves (its `Condition` through `settled_holds`); `Optional` asks; `Triggered` is refused outside a trigger's placement (a spell cannot carry one); `Reflexive` creates and checks | TR-1, TR-2, TR-3 |
| `oracle/characteristics.rs::get_effective_abilities` | three index readers | a fourth reader, the dispatcher, indexing by `instance` ordinal | TR-1 |
| `zone_function::functioning_zones` | six of fourteen subrules | the `Triggered` arm (113.6k, derived) | TR-1 |
| `register_static_effects` / `cleanup_zone_state` / `place_on_battlefield` | maintain the replacement gate sets | maintain `trigger_sources` and `zone_trigger_sources` beside them | TR-1 |
| `RegistryScopeSummary` | nine fields | `granted_trigger_zones`, `copied_trigger_zones` | TR-1 |
| `engine/turns.rs::begin_step` / `begin_phase` | propose with `player` | the record carries it (item 10) | TR-1 |
| `actions.rs`' `DealDamage`, `LoseLife` performers | drop `is_combat`, `cause` | carry them (item 10) | TR-1 |
| `mana.rs::activate_mana_ability` → `resolve_mana_effect` | activated mana abilities | the same path for a triggered one, entered from dispatch | TR-1 |
| `engine/zones.rs::perform_zone_change`, `leaving.rs::owned_objects_leave` | capture `EffectiveCharacteristics` for a battlefield departure | capture `LastKnownInformation`; widen the condition to the three classes, the third for a visible object only | TR-4 |
| `engine/combat/steps.rs` (`:86`, `:177`) | attackers, per-defender blockers | defenders on the record; one blockers record | TR-5 |
| `replacement/pipeline.rs::apply_rewrite` | prevention leg | announce `DamagePrevented` | TR-5 |
| `actions.rs` `EnterBattlefield` performer | announces the entry | one `CountersChanged` per entry row after it | TR-5 |
| `sba.rs` (`:439`, `:480`), `GameState::attach`, the zone-change performers | `EquipmentDetached` at two sites | `announce_unattached` at three | TR-4 |
| `sba.rs` (`:484`–`:503`), CR 704.5q | writes both counter kinds directly, announces `CountersAnnihilated` | two `RemoveCounters` proposals in the state-based batch; the variant deleted | TR-5 |
| `resolve.rs` `Primitive::GainControl` | writes the Layer 2 row | unchanged — the sweep detects it | TR-4 |
| `ui/choice_types.rs`, `ui/ask.rs`, `ui/cli.rs`, `trace_records.rs`, `sba.rs` | five exhaustive `ChoiceKind` matches in `src`, four test files | one arm each per new kind (§9) | per phase |
| `ui/display.rs::format_event` | 34 arms | −3, +4, one renamed | TR-1, TR-4, TR-5 |
| `fuzz_games` | reads `EventStamp::resolution` for casts | a `Triggers placed` row and an `AbilityTriggered` count in `--dump-events` | TR-1 |

---

## 9. DecisionProvider surface

Every kind names its subject at birth (A4j) and is asked with two or more
options (`CLAUDE.md`); each is an arm in the five `src` matches above.

| `ChoiceKind` | Method | Subject | Rule | Phase |
|---|---|---|---|---|
| `OrderTriggers { player, tier }` | `choose_ordering` | `None` (several objects) | 603.3b; elided per item 163 | TR-1 |
| `OptionalEffect { source }` | `pick_n` (yes/no) | the resolving object | 603.5; main item 24 | TR-2 |
| `ChooseDelayedTriggerEvent { delayed }` | `pick_n` | the delayed trigger's source | 603.7b | TR-3 |
| `SelectRecipients` (existing) at placement | `pick_n` | the trigger's stack object | 603.3d → 601.2c | TR-1 |

The 603.3c mode prompt is `backlog.md` §2.7's and arrives with it. Nothing here adds a
trait method (`roadmap-v2.md` §9's watch item): the four methods suffice.

---

## 10. N players, and the permanent versus its components

**Every ordering is APNAP over the seat list**, through `apnap_index` and
`in_game`: placement (§5.2), `EachPlayer` recipients (§6.6), the delayed
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
every component's abilities), so the effective list is the permanent's,
`instance` indexes that list, and a component's own ordinal is not a thing
this design knows; `cast`, `announced_controller`, the gates, the
counters in `Status` and the histories key on the permanent;
`zone_change_epoch` is the permanent's, since CR 729.2c makes a merged
permanent "the same object that it was before". **One read is a
component's**: when a merged permanent leaves the battlefield "one
permanent leaves" and each component is put into its zone (CR 729.3), and
an effect that can find the new object "finds all of those objects"
(729.3c) — so a `TriggerBinding.object` for "return it to the battlefield"
names *N* new objects, and `LastKnownInformation`, one frame, the permanent's, must
let CV-7 say which component's card each lookup means. That is CV-7's to
design; this document names it and keys nothing on it.

---

## 11. Performance — the sweep, the gate, the memo, and what each PR predicts

**The cost model.** A dispatch is: five set probes — a probe being one
hash-set `is_empty` or `contains`, nanoseconds and no allocation — for the
gate; if any is
non-empty, one `get_effective_abilities` per candidate (a memo hit for
every object the batch did not touch; one board pass for the rest, which
the SBA check after the batch shares), one `matches` per triggered def per
record in the window, one `settled_holds` per intervening "if" that
reached it. About 313 batches and ~700 records a game today, ~16
permanents a board, and — on the pools as they are — zero trigger sources,
so the pools measure the five probes and nothing else. **The lever**,
pre-approved and not built until a reading asks: a per-source
`EventKindMask` on `trigger_sources` so a record of one kind visits only
the objects whose conditions read it.

**A/B predictions, per phase**, in `engineering-practices.md` §3.1's terms —
three arms where a pool changes (`main`, the engine with pools unchanged,
shipped), two seats and four, and the budget is 2.5 points of CPU per
decision at identical counters:

| Phase | Engine arm, pools unchanged | Shipped arm | Why |
|---|---|---|---|
| TR-1 | every counter `IDENTICAL`; CPU per decision inside the budget | `differ` on both pools; a `fuzz-record.md` block | the gate is empty on the old pools; the new pool has three trigger sources and a `Triggers placed` row |
| TR-2 | `IDENTICAL` on `performance`; `differ` on `stress` | `differ` | the histories are advanced on every game (a cost, no counter); Alms Collector is in `stress` and its rider's draw order changes (§6.6) |
| TR-3 | `IDENTICAL` both pools | `differ` | the registry is empty on the old pools |
| TR-4 | `Layer walks` up by the widened captures on `stress`, `Memo hits` up on both (the control sweep runs while Act of Treason's row lives); every gameplay counter `IDENTICAL` | `differ` | the sweep is gated on `any_control_changing`; captures gate on `from`/`to` |
| TR-5 | `IDENTICAL` counters; `--dump-events` gains `Targeted`, `DamagePrevented` and entry `CountersChanged` lines | `differ` | records announced, no decision moved |
| TR-6 | `IDENTICAL` both pools | `differ` | no state trigger on the old pools; Tier 1 counts prompts and settles nothing on them |

Each phase's `fuzz-record.md` block re-records both pools when its
`PERFORMANCE_POOL` entry lands, and the reachability rows (`--require`)
name each new card's trigger count per 200 games. A stream-moving finding
inside a phase is its own PR, as §9 of the practices requires.

---

## 12. Sizing and the phase plan — TR-1 to TR-6

Sized against the tree on 2026-09-18 (`engineering-practices.md` §4: count
first). Six PRs at the top of row A6's 4–6; each carries at least one
registered consumer of what it builds; each closes against `specdb owed`
for the atoms §13 assigns it. The order is the dependency order: TR-1 is
the spine every later phase reads; TR-2's histories are what TR-3's
"this turn" durations and TR-5's `FirstTimeEachTurn` read; TR-3 builds
`ReturnToBattlefield`, which TR-4's persist and Rancor need; TR-4 widens
the frame TR-5's combat shapes never read; TR-6 is last because the loop
detector reads every prompt the earlier phases add.

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
`PermanentState.cast`, `EventSeq`, item 10's three fields, item 18's three
deletions, `AbilityTriggered`, CR 113.6k derived in `functioning_zones`, the
`Triggers placed` row. Five cards; Soul Warden, Blood Artist and Wild Growth
pooled (91 → 94). Fifty tests, §13's TR-1 row clean.

**What moved on the way in** — the sizing's `ObjectFilter::NotSource` is
`EachOther`; `Attacks` and `GainsLife` shipped narrow because §13 owed their
atoms here; leg 3 is swept with leg 4 because they are one map; the gate has
a sixth probe (the departure frames); the binding carries the def. A
look-back arm on a surviving permanent reads its post-event list — main item
167. The archive has the sizing table and the nine notes.

**Measured** (`fuzz-record.md`, the TR-1 block): the probe found no dispatch
passing the gate on the old pools; the engine arm `IDENTICAL` on every counter
on both pools at two seats and four, +1.0% and +1.5% CPU per decision; the
shipped arm differs, with `Triggers placed` 1.5 and 4.2 on `performance`.

→ `plans/archive/triggers-architecture-landed.md`, "TR-1"; the trace page
`plans/traces/tr-1-a-trigger-is-matched-at-the-close.html`.

### TR-2 — the histories, the gates, "may", and each player (~2,000)

| Piece | ~additions |
|---|---|
| `TurnSummary`, `PlayerHistory`, `own_turns`, the record-by-record advance, the four `Condition` leaves (three edits each), `FirstTimeEachTurn`, the two gate sets and their two writers, `TriggerLimit` on the def, the 603.7h count off `AbilityResolved` and its condition, the arms `DrawsCard`, `GainsLife`, `LosesLife`, `CastsSpell`, `AbilityResolves`, `ShufflesLibrary`; `Condition::ResolvedThisTurn(n)` (§6.5) | ~500 |
| `Effect::Optional` with `OptionalEffect`, `last_optional_taken` for "if you do" (main item 24), 118.12's cost-object check | ~120 |
| `EffectRecipient::EachPlayer(PlayerSet)` in APNAP order (S2, item 122); Alms Collector's rider re-encoded | ~80 |
| cards: **Paladin of Atonement** (last turn, whoever's; `AmountExpr::TriggeringToughness` off the frame), **Vengeful Warchief** ("for the first time each turn"), **Elvish Warmaster** ("one or more", "triggers only once each turn"), **Nykthos Paragon** (603.2h, "may", "that many" on each creature), **Psychosis Crawler** (draws, each opponent, a CDA), **Temple Bell** (each player draws), **Cosi's Trickster** ("whenever an opponent shuffles", "may"; its three rulings); Warchief, Warmaster, Crawler and Trickster pooled (the histories, the gate, `EachPlayer`, the shuffle arm) | ~400 |
| tests: §13's TR-2 atoms; Nykthos Paragon's six rulings as six tests; Ashling the Pilgrim's count as a fixture (the card needs two amount leaves and waits); 603.1b's fixture in Avatar Aang's shape; 121.2c against Alms Collector; the pregame-sweep question measured (a probe, recorded) | ~700 |
| docs, ledger, record | ~260 |

### TR-3 — delayed, reflexive, and "until" (~1,900)

| Piece | ~additions |
|---|---|
| the registry, `DelayedTrigger`, `DelayedSource`, `DelayedDuration`, `ObjectRef`, `Instant`, `ExtraTurnId` on `turn_queue` and `current_turn_origin`, `Primitive::CreateDelayedTrigger`, provenance from `ResolutionContext` and from a rider, 107.3n's X, `ChooseDelayedTriggerEvent`, cleanup expiry of `ThisTurn`; `Effect::Reflexive` and the immediate check; `UntilEvent` resolved at dispatch (610.3) | ~520 |
| `Primitive::ReturnToBattlefield` and `ReturnToHand` made real over `change_zone` / `EnterBattlefield` (the stub arm at `resolve.rs:1417`), with 610.3c's owner's control; a source-relative "another" for a sacrifice chooser | ~120 |
| cards: **Final Fortune** (603.7d, a named extra turn; its ruling — a skipped extra turn loses nothing — is the `ExtraTurnId` test), **Flickerwisp** (603.7e from a triggered ability, 603.7c through exile, CR 400.7; its second ruling is 513.2's sibling), **Cornered Crook** (603.12: `Optional` then reflexive, any target — Heart-Piercer Manticore prints the same shape with an LKI power read and cannot register whole, since embalm is `backlog.md` §2.3's and CV's), **Banishing Light** (610.3's until-return, no stack; its ruling that an Aura or Equipment on the exiled permanent falls off is CR 400.7's, and "leaves before the trigger resolves, nothing is exiled" is 610.3a); Flickerwisp and Banishing Light pooled (the registry, the until path) | ~320 |
| tests: §13's TR-3 atoms; Tatsumasa's simultaneous choice as a fixture; 513.2 both ways; 603.7f through a rider fixture; 603.7g's fixture; Heart-Piercer Manticore's four trigger rulings as fixtures (the LKI power read); Sneak Attack's ruling as a fixture board (the card waits for an indefinite haste, CV-1b) | ~650 |
| docs, ledger, record | ~250 |

### TR-4 — the look-back list, the frame, unattach, control (~2,200)

| Piece | ~additions |
|---|---|
| `LastKnownInformation` and `Status` (3 field sites, 7 literals), the widened capture (item 15) gated on prior visibility for the third class, the reader methods on the type, `Unattached` with `announce_unattached` at three performers (7 `EquipmentDetached` sites), `ControlChanged` from the sweep with `announced_controller`, the arms `BecomesAttached`, `BecomesUnattached`, `ControlChanges`, `IsCountered`, `PlayerLoses`, `ZoneChange`'s other two classes, `Condition::TriggeringObjectHadCounters` | ~480 |
| cards: **Grafted Wargear** (603.10c, item 14's host, "sacrifice that permanent"), **Strangleroot Geist** (undying as an `AbilityDef` — quadrant ③ — with 702.93a's intervening "if" off `Status.counters`, `ReturnToBattlefield` with an entry counter; Kitchen Finks prints persist, the mirror, and is not registered because its hybrid cost would have to be misspelled — Mirrorweave's precedent, `codebase-state.md`'s CV-1 status), **Rancor** (603.6e/400.7f, an Aura's own dies-trigger, `ReturnToHand`), **Multani's Presence** (603.10e — `SpellCountered`, never `SpellFizzled`), **Golgari Brownscale** (603.10a's third class; registered whole if dredge — a draw replacement functioning from the graveyard, which LK and RF make expressible — fits the band, else the atom's fixture and the card in §15); a 603.10d fixture over Act of Treason's steal; Strangleroot Geist and Grafted Wargear pooled | ~420 |
| tests: §13's TR-4 atoms; Kitchen Finks' eight persist rulings as undying's tests (the same shape with the counter's sign flipped); Grafted Wargear's three; Guile's two boards with Yixlid Jailer for the second class (its rulings name both cards); 122.8 and 122.9 off the frame | ~700 |
| docs, ledger, record; trace page decided at close (the look-back reads changed) | ~300 |

### TR-5 — combat's shapes, targeting, counters, prevention, the multiplier (~2,200)

| Piece | ~additions |
|---|---|
| `AttackShape` and `BlockShape` over item 11's records (one blockers record per declaration), `Targeted` at three emit sites, `DamagePrevented` from the prevention leg, entry `CountersChanged` with `by`, the arms `Attacks`, `Blocks`, `BecomesTarget`, `DamageIsPrevented`, `CountersPutOn`/`RemovedFrom` with `nth` and per-counter occurrences, `ActivatesAbility`, `CreatesToken`, `Scries`; `Effect::TriggerMultiplier` behind the gate; CR 704.5q routed through two `RemoveCounters` proposals in the state-based batch (Deferred Migrations item 6's counter half; `CountersAnnihilated` deleted) | ~560 |
| cards: **Hellrider** (508.3a's defender), **Loyal Sentry** (509.3b), **Cephalid Aristocrat** (item 12, mills), **Simic Ascendancy** (122.6 entry counters, "one or more", 603.4 at upkeep, `WinGame`), **Selfless Squire** (item 16 — its second ruling: any prevention, not only its own), **Panharmonicon** (603.2d, its ten rulings), **Protean Hydra** (CR 704.5q's removal is a removal — its six rulings; X entry counters through RC-5's dynamic amount, RD's rider for "remove that many", TR-3's delayed trigger, so the routing's fixture if any of the three refuses it); Hellrider, Simic Ascendancy and Panharmonicon pooled | ~440 |
| tests: §13's TR-5 atoms; 508.4's "put onto the battlefield attacking never attacked" as a fixture over item 128's field; 509.3a–g's seven readings; Frost Titan's once-per-spell as a fixture (the card waits for "unless pays"); Panharmonicon's edges | ~750 |
| docs, ledger, record | ~250 |

### TR-6 — state triggers, the loop, and the rule-owned arm (~1,300)

| Piece | ~additions |
|---|---|
| the state check at its three schedules, `state_triggers_armed_off`, `trigger_left_stack` at four sites, `TriggerCondition::State` matched; `Condition::Not` (Emperor Crocodile's "no other creatures" is its first card — LI-3's rule for when the enum grows); `LoopDetector` (Tier 1) on `GameState` reading A4e's decision counter, `GameResult::Draw` through the settlement; `TriggerOrigin::Rule` declared with `InherentAbility` empty | ~300 |
| cards: **Emperor Crocodile** (603.8, its two rulings), pooled. Immortal Coil is the CR 104.4b board — its third ability beside Platinum Angel is "an involuntary infinite loop ... the game will end in a draw" by its own ruling — and it is a **fixture**, not a registration: its first ability's `Cost::ExileFromGraveyard` is a validation and payment stub today (`engine/costs.rs`), and a card wearing a real name with a dead ability is what `engineering-practices.md` §3 forbids | ~120 |
| tests: §13's TR-6 atoms; the Coil's three trigger rulings on the fixture; the draw at threshold, with Platinum Angel registered; the ratchet's reading at the phase's close (§9 of the practices) | ~450 |
| docs, the close-out, §16's deferrals re-read, ledger, record | ~400 |

**Not five and not seven.** TR-1 and TR-2 sum past the band and their
consumers differ in kind (an event matcher against a history); TR-3's
registry is the one algorithm a review should read alone; TR-4 and TR-5
are two different seams — the frame and the record — each with its own
literal sweep; TR-6 is small on purpose, because the state check and the
loop detector are the two things most likely to surprise a measurement and
are cheapest to revert alone. Every number above is a starting point: the
phase re-counts against the tree before it starts, and A4n's rider is the
precedent for a count being wrong by a factor.

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
| **TR-2** | 603.1b-001 (fixture); 603.2h-001, -002; 603.5-001; 118.12-001; 121.5-001 (partial → full); 608.2h-001; 608.2p-001 | 8 |
| **TR-3** | 603.7-001; 603.7a-001; 603.7b-001, -002; 603.7c-001; 603.7d-001; 603.7e-001; 603.7f-001; 603.7g-001 (fixture); 603.7h-001; 603.12-001; 107.3n-001; 513.2-001, -002; 610.3-001; 610.3a-001; 610.3b-001; 610.3c-001, -002; 610.3d-001 | 20 |
| **TR-4** | 603.6e-001, -002; 400.7e-001, -002; 400.7f-001; 603.10c-001, -002, -003; 603.10d-001; 603.10e-001; 603.9-001; 603.2e-002; 122.8-001; 122.9-001 | 14 |
| **TR-5** | 508.2a-001; 603.2d-001; 122.7-001; 120.10-001 | 4 |
| **TR-6** | 603.8-001, -002 | 2 |
| **Deferred, with the rule that lets each wait** | 603.2a-001 (needs an "activated abilities can't be activated" restriction — RS-2's); 603.3c-001, -002 and 700.2b-001 (modes — `backlog.md` §2.7, on §5.4's placement); 607.2c-001, 607.2h-001 (linked — §2.2); 603.12a-001 and 605.3a-002 (a cost paid at resolution — CP-1, which also unlocks 702.21a-001, -002 (ward = TR-5's event + CP-1's "unless"), Strict Proctor and Frost Titan); 400.7-001 (the rule itself — CV-1b); 111.13-001, 112.2-002, 700.2g-001, 707.10b-001, 707.5-002, BOUNDARY-707.7-001, BOUNDARY-707.9g-001 (copies — CV-2, CV-4, with §6.5's sentence); 208.2b-001, -002 (copiable values from an entry choice — CV); 610.5-001, -002 (a granted keyword at cast — §2.1's convoke); 611.2e-001, 611.3d-001, -002 (their owners: 611.3d is §2.3's foretell); 115.9a-001 ("with N targets" — a filter over `chosen_targets`, Phase 8 with its first card); 701.43d-001 (exert — §2.5); 701.66a-001, -002 and 702.176a-003 (earthbend, impending — Phase 8); 724.1-001, 724.2-001, -002, COMP-MONARCH-COMBAT-001, 724.3-001, 724.5-001, 725.1-001, 725.2-002, 725.3-001 (designations — Phase 9, on §3.8's arm); 608.2d-001 (stays partial: choices at resolution are §2.7's and CP-1's); 608.2j-001 (a characteristic read — ALREADY-IMPL's, re-filed at TR-6's close) | 41 |

Ninety-two of the 133 Phase 7 atoms are owed across the six phases (plus
the two from outside the phase TR-1 claims), forty-one are deferred with an
owner each; 92 + 41 = 133, no atom listed twice and none unlisted — checked
against `spec.sqlite` on 2026-09-18, and worth re-checking the same way
at each close. The deferrals are re-read at TR-6's close-out
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
   history. TR-2.
4. **122.6's entry counters.** A `CountersChanged` per entry row, announced
   after the entry inside its batch, never proposed. §3.12 item 17. TR-5.
5. **Unattach's three routes.** All three are the event (CR 701.3d lists
   them), one record through one emitter, and an Aura put into the graveyard
   by 704.5m becomes unattached as it leaves, which is its own zone change's
   announcement. Grafted Wargear's ruling enumerates the routes. TR-4.
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
    the first game-scoped reader (§3.10). TR-2.
16. **Two once-per-turn gates.** Two `TriggerLimit` arms, two sets, two
    writers — the dispatcher for "triggers only once", the resolution for
    "do this only once" — read at the instants the rulings name (§3.5). TR-2.

---

## 15. Findings and open questions

Recorded here at authoring; a finding that becomes a code item moves to
`codebase-state.md` under the phase that found it.

1. **The candidate order is observable in one place only.** The
   `OrderTriggers` prompt lists a player's triggers in `seq` order, which is
   dispatch order, which is window order, which is
   `battlefield_ids_ordered` order for a batch of entries — process-stable
   end to end. A `HashMap` reaching the queue would be a determinism bug;
   `trigger_sources` is an `IdSet` and is never iterated for order.
2. **The `AbilityTriggered` record and CR 603.2d.** A multiplied trigger
   queues N entries and emits N records, so Strict Proctor under
   Panharmonicon triggers twice, which is the CR's answer (each instance is
   "a triggered ability [that] trigger[ed]"). Worth a test; not a doubt.
3. **The window's `mark` and a rewound cast.** `// CAST-ROLLBACK:` moves a
   card silently and `announce_zone_change` records the move at 601.2i, so
   no record of a failed cast ever reaches a window; nothing to do, noted
   because the matcher is the first reader that would have cared.
4. **`ControlChanged.lki` is the frame *after*, not before.** The sweep
   sees the new controller; the prior frame is only in the memo's stale
   entry, which is a cache and not a record. The difference is observable
   only when the control change also removes an ability that
   conditioned on control — no registered card does. Recorded rather than
   built; if a card needs it, the sweep captures a frame per permanent
   while `any_control_changing` is set, which is the cost to weigh then.
5. **Golgari Brownscale's dredge** may or may not fit TR-4's band; the
   atom's fixture is the floor and the card is the ceiling.
6. **Sneak Attack** waits for an indefinite haste (CV-1b's
   `Duration::Indefinite` prune); its ruling is a TR-3 fixture board.
7. **The pregame sweep** over the registry is a measurement TR-2 takes
   and records (histories advanced per record, ~40 fields, ~1,000 records
   a game); the decision to prune waits for the number.
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
- `cost-architecture.md` §3.11's Ironworks loop is TR-2's integration test
  — every step but the triggers exists, and step 6's "with lesser mana
  value" is the one selection leaf this document adds when Scrap Trawler
  registers (a graveyard target compared against the trigger's source:
  `ObjectFilter::ManaValueLessThanSource`, three edits).
- `plans/glossary.md` gained *dispatch*, *window*, *binding*, *tier* and
  *probe* with TR-1 (2026-09-19), and `check_glossary.py --suggest` at each
  close is the gate on the rest.
- `plans/references/trigger-survey.py` deletes with the survey when TR-6
  closes, per its own docstring.
