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
> integration test (TR-7).

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
| `ZoneChange` | `ZoneChange { subject, from: Option<Zone>, to: Option<Zone>, cause: Option<ZoneChangeCause>, owner: Option<PlayerRef>, multiplicity }` — dies is `from: Battlefield, to: Graveyard`; "leaves the battlefield" `from: Battlefield, to: None`; sacrificed/discarded/milled/countered by `cause`, which a redirect keeps (CR 614.6, `codebase-state.md` item 176); exiled by `to: Exile`, since CR 701.13a's act is its destination; "from anywhere" `from: None` | iff `from == Some(Battlefield)`, `from == Some(Graveyard)`, or `to ∈ {Hand, Library}` from a zone all players can see (CR 603.10a's three classes, the third written about visibility; §4.3) | TR-1 (battlefield departures), TR-4 (the other two classes, item 15) |
| `Tapped` / `Untapped` | `BecomesTapped { subject }` / `BecomesUntapped { subject }` — transition-only by the record's own contract (603.2e) | no | TR-1 (fixture), Phase 8 (card) |
| `CardDrawn` | `DrawsCard { player: Option<PlayerRef>, multiplicity }` — never a library-to-hand `ZoneChange` (121.5) | no | TR-2b |
| `ManaAdded` | `ManaAdded { source: Option<ObjectFilter>, tapped_for_mana: Option<bool>, mana: Option<ManaType> }` — CR 106.12a's "tapped for mana" reads `tapped_for_mana` | no | TR-1 |
| `DamageDealt` | `DamageDealt { source: Option<SourcePattern>, recipient: DamageRecipient, combat: Option<bool>, multiplicity }` — "is dealt damage", "deals damage", "deals combat damage to a player"; `combat` is item 10's field | no | TR-1 |
| `PhaseBegin` / `StepBegin` / `TurnBegin` | `PhaseBegins { phase, whose }` / `StepBegins { step, whose }` / `TurnBegins { whose }` — "your upkeep", "each upkeep", "the monarch's end step"; `whose` reads item 10's `player` | no | TR-1 |
| `PermanentEnteredBattlefield` | `EntersBattlefield { subject, controller: Option<PlayerRef>, from: Option<Zone>, was_cast: Option<bool>, multiplicity }` — `from` and `was_cast` are joined from the same object's `ZoneChange` or `TokenCreated` in the window (§4.4; question 9) | no (603.6a reads the board after, with 603.6b's effects applied) | TR-1 |
| `LifeChanged` | `GainsLife { player, multiplicity }` / `LosesLife { player, multiplicity }` — two arms for one record because the sign decides which printed family reads it; per record (question 3). `cause` waits for its one customer, CR 727's rad counters | no | TR-1, TR-2a |
| `AttackersDeclared` | `Attacks { shape: AttackShape, attacker: Option<ObjectFilter>, attacking_player: Option<PlayerRef>, defender: Option<DefenderRef> }` — CR 508.3a–e's five shapes as one enum: `Creature`, `CreatureAgainst`, `PlayerIsAttacked`, `PlayerAttacksWith`, `PlayerAttacks`, `PlayerAttacksPlayer`, `Alone`; reads item 11's defender | no; 508.2a's snapshot is the dispatch instant | TR-5 |
| `BlockersDeclared` | `Blocks { shape: BlockShape, .. }` — 509.3a–d and 509.3g's five readings of one pair list: `Blocks`, `BlocksACreature`, `BecomesBlocked`, `BecomesBlockedBy`, `AttacksAndIsntBlocked` | no | TR-5 |
| `SpellCast` | `CastsSpell { caster: Option<PlayerRef>, spell: Option<ObjectFilter> }` — the stack object is live at dispatch (types, colors, mana value through the layer walk). `from` waits for casting from a zone other than the hand (`backlog.md` §2.3) | no | TR-2a |
| `AbilityActivated` | `ActivatesAbility { controller, source: Option<ObjectFilter>, loyalty: Option<bool> }` | no | TR-5 |
| `AbilityResolved` | `AbilityResolves { identity: IdentityRef }` — `ThisAbility` for CR 603.7h's delayed form, read with the resolution count (§6.5) | no | TR-3 |
| `SpellCountered` / `AbilityCountered` | `IsCountered { subject: CounteredRef }` — **never `SpellFizzled`** (question 2; Multani's Presence's ruling) | yes (603.10e) | TR-4 |
| `SpellFizzled` | **no arm.** CR 608.2b's "doesn't resolve" is not "countered" under the baseline CR, and no printed trigger reads it | — | — |
| `PlayerLost` | `PlayerLoses { player, reason: Option<LossReason> }` — 603.9's "unless as the result of a draw": a draw is `GameResult::Draw`, not a `PlayerLost` | yes (603.10f) | TR-4 |
| `PlayerWon` | **no arm** — no printed trigger; recorded so the projection stays one-to-one | — | — |
| `Scried` | `Scries { player }` — Elrond's X is `TriggerBinding.amount = looked_at` | no | TR-5 (fixture) |
| `LibraryShuffled` | `ShufflesLibrary { player }` — two printed watchers, Cosi's Trickster and Psychic Surgery ("whenever an opponent shuffles their library"; the survey's 168 on this row is the substring over-count); one record per shuffle, an empty or one-card library included, and never for cascade's random bottom (Cosi's Trickster's rulings) | no | TR-2b |
| `CountersChanged` | `CountersPutOn { subject, kind: Option<CounterType>, by: Option<PlayerRef>, nth: Option<u32>, multiplicity }` / `CountersRemovedFrom { .. }` — the sign splits the arm as it does life; `nth` is CR 122.7's before/after read live (count now minus `added`); **an occurrence is a counter, not a record** — Protean Hydra's ruling: several +1/+1 counters removed at once trigger "whenever a +1/+1 counter is removed" that many times, so `PerOccurrence` on these arms multiplies by the count. Simic Ascendancy's "one or more ... on a creature" is once per creature, §4.4's third multiplicity (corrected 2026-09-24; it read `OncePerEvent`) | no | TR-5 |
| `CountersAnnihilated` | **no arm, and the variant goes** — CR 704.5q's annihilation *is* a removal of counters: Protean Hydra's ruling has a -1/-1 counter meeting a +1/+1 counter trigger "whenever a +1/+1 counter is removed". Today the state-based sweep writes both kinds directly and announces this variant (Deferred Migrations item 6's counter half, `sba.rs:484`); TR-5 routes it through two `RemoveCounters` proposals in the state-based batch, so the annihilation is two `CountersChanged` records the removal arm reads, and the variant is deleted with item 18's three | — | TR-5 |
| `Attached` | `BecomesAttached { attachment: Option<ObjectFilter>, host: Option<ObjectFilter> }` — transition-only (701.3b), so re-equipping the same creature announces nothing (603.2e-002) | no | TR-4 |
| `EquipmentDetached` → **`Unattached`** | `BecomesUnattached { attachment, former_host }` — one record for CR 701.3d's three routes (question 5), replacing `EquipmentDetached` | yes (603.10c): the frame is the attachment's, with `attached_to` from item 14 | TR-4 |
| `LeftTheGame` | folded into `ZoneChange`'s leaves-the-battlefield reading: a `LeftTheGame` from the battlefield matches `from: Battlefield, to: None` and nothing narrower, which is CR 603.6c's own sentence; the phased-in qualifier is item 6's and waits for phasing | yes (the record carries the frame since RE-7) | TR-1 |
| `TokenCreated` | `CreatesToken { owner, zone: Option<Zone>, kind: Option<TokenKind>, multiplicity }` — keyed here and never on `is_token` at entry (item 8; CR 111.13). **Amended 2026-09-23 (the rulings pass, G2):** a reflexive "when you do" on a creation asks whether each token is of the definition the instruction named — Ajani, Nacatl Avenger's ruling triggers once per Cat Warrior under Doubling Season and not at all for the Angels Divine Visitation makes instead — so `kind` matches the instructed definition, not a token type | no | TR-5 (fixture) |
| `TokenCeasedToExist` | **no arm** — CR 704.5d names no trigger event, and what cards observe is the *absence*: Flickerwisp's ruling has an exiled token "cease to exist and won't return", which is a delayed trigger's `ObjectRef` finding nothing (§3.9), not an event to match | — | — |
| `StateBasedActionPerformed` | **no arm** | — | — |
| *new* `Targeted` (item 12) | `BecomesTarget { subject: TargetRef (object or player), by: Option<TargetingFilter> (a spell, an ability, "an opponent controls", "an Aura spell"), first_time_each_turn }` — once per spell or ability, however many instances (question 11) | no | TR-5 |
| *new* `ControlChanged` (item 13) | `ControlChanges { subject, from: Option<PlayerRef>, to: Option<PlayerRef> }` — "loses control", "an opponent gains control of a permanent you own" | yes (603.10d), with the caveat in §15 item 4 | TR-4 |
| *new* `DamagePrevented` (item 16) | `DamageIsPrevented { target: DamageRecipient, multiplicity }` — one record per prevention applied per subject group (CR 615.13) | no | TR-5 |
| `AbilityTriggered` (new, §4.8) | `AbilityTriggers { caused_by: Option<TriggerEvent> (Strict Proctor's "a permanent entering causes"), source: Option<ObjectFilter> }` — **the arm that is CR 603.3b's second tier**: `TriggerCondition::tier()` reads it | no | TR-1 (the event and the tier), TR-5 (the arm's card) |

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

**Not one enum with `EventPattern`** (the TR-1 review, 2026-09-22). The two
watch different streams: an `EventPattern` reads a *proposal*, before it
happens and rewritable by the closed `Rewrite` algebra; a `TriggerEvent` arm
reads a *performed* record, after, with frames, look-back and multiplicity.
Ten of TR-1's twelve arms have a sibling in shape (`Attacks` and
`AbilityTriggers` have none), and the vocabulary is already shared at the
leaf: `ObjectFilter`, `PlayerRef`, `Condition`, `ZoneChangeCause`. What is
worth sharing is a *field struct* — a `ZoneChangePattern { from, to, cause }`
both `ZoneChange` arms hold, and `SourcePattern` for damage where the trigger
arm has a `TriggerSubject` — and TR-4's widening of `ZoneChange` is the
moment. Never a shared enum. Layers watch nothing.

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
    /// `OncePerEvent` one. Each carries its `EventSeq`, the stream's
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

**The projections live on the arm, and they are exhaustive matches.**
`TriggerEvent` carries four methods — `subject_of(&GameEvent) ->
Option<ObjectId>`, `player_of(..) -> Option<PlayerId>`, `amount_of(..) ->
Option<u64>`, `occurrences_of(..) -> u32` — each a `match` over the arms
with no wildcard, so a new arm must say what its "that many" is (damage
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
record's `lki` frame when the event was the departure, the entry's
`departed` frame when the object left afterwards (§6.1's amendment). All
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
    /// this record is the first of its kind in the player's turn summary
    /// (§3.10). Written by nobody; read at dispatch, record by record.
    FirstTimeEachTurn,
}
```

The two gate sets live on `GameState` beside the turn summaries:
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
not on the event. The proposal carries `extra: Option<ExtraTurnId>`, the
turn's `EventPattern` arm reads it, and the turn that begins takes it from the
proposal. "During that turn" (Alchemist's Gambit, Kang the Conqueror) is a
duration on the same id. ~20–30 lines, with this id or with the first of the
four cards.

### 3.10 `TurnSummary`, `PlayerHistory`, and the game scope (item 42; P2–P4; question 15)

```rust
/// One player's turn, materialized: every "this turn" quantity the CR or a
/// registered card reads, one count each, one meaning each — never a scan
/// of the log. Advanced at dispatch, record by record (§4.1). As built in
/// TR-2a and its review: the fact is the key, and each `TurnFact` variant's
/// name says whose row it is counted on.
pub struct TurnSummary {
    counts: [u64; TurnFact::COUNT],  // one slot per fact; spells per card type
}
// TurnFact: SpellsCast, SpellsCastOfType(CardType), CardsDrawn, LifeGained,
// LifeGainEvents, LifeLost, LifeLossEvents, DamageTaken (dealt to this
// player: bloodthirst), ControlledCreaturesDied (controlled as it died:
// morbid sums the rows), AttackersDeclared (raid is at least one)

/// One player's history, bounded by the table and never by the turn count
/// (the bounded-state PR, `codebase-state.md` item 179). "This turn" and
/// "last turn" (Paladin of Atonement: whether you lost life last turn,
/// whoever's turn it was) are two rows that move along as a new turn is
/// counted on; "this game" is a running total; "since your last turn" (CR
/// 730's day/night, Arboria, Concert Kaboomist) is every player's total
/// now less their total as your last turn ended, taken as the next turn
/// began. Your last turn is your most recent to have ended.
pub struct PlayerHistory {
    turn: u32,                              // the turn `this_turn` counts
    this_turn: TurnSummary,
    last_turn: TurnSummary,                 // turn - 1's
    this_game: TurnSummary,
    own_turn: Option<u32>,                  // the turn this player last began
    at_your_last_turn: Vec<TurnSummary>,    // by PlayerId: O(seats²) in all
}
// on PlayerState: history: PlayerHistory
```

**Three decisions.** *Whole game, not two turns*: the brief's recommendation,
because the pregame sweep that would prune it (the state-tracking doc's
`RelevantEffects`) is an optimization over a static property of the
registry and pays only if measured — deferred until a reading says it
should, with the fallback the doc already names (conjure, wishes: track
everything). **The reading came 2026-09-25 and the bounded-state PR acted on
it (item 179):** no reader needed the whole-game rows, so every fact is still
tracked for every player, and only the turns stopped being kept one row each.
*A game-scoped quantity is a scope on a counter, not a window on the log*:
Approach of the Second Sun's casts and CR 903.8's commander tax are
`this_game` counts, and the tax — `cost-architecture.md` §3.8, waiting
on designation — becomes the first game-scoped reader, a field
`commander_casts_from_command_zone` on the summary the day B2 lands. *A
quantity no field anticipates is a field plus an update arm, authored with
its card* — the postscript's rule, restated here so it is this document's
too. The `Condition` leaves that read it are `ThisTurn(TurnFact, Cmp)`,
`LastTurn(..)`, `SinceYourLastTurn(..)`, `ThisGame(..)`, three edits each
(the variant, the `holds` arm, the `condition_reads` arm, which for a
summary read is "nothing" — no frame is read).

**A departed player's counts stay readable (CR 800.4i).** The rule says that
"if an effect requires information from the game about actions players have
taken, the effect can find actions that were taken by a player who has left
the game." `GameState.players` never shrinks, so a departed player's history
keeps its counts. A read over every player (`PlayerSet::Everyone`) sums them,
"an opponent" includes them, and the snapshot "since your last turn"
subtracts holds their totals too. No test pins this yet: session 10 deferred
CR 800.4i to Phase 9 with no atom.

**As built (TR-2a, 2026-09-24).** The owner had each field named at the
sizing for the side it counts, chosen from the cards that read it:
- `damage_taken` is damage dealt *to* the player, which bloodthirst reads.
- `controlled_creatures_died` is counted on the row of whoever controlled the
  creature as it died, and morbid sums the rows.
- `attackers_declared` is a count, and raid reads it as at least one.

Three sketched fields are not in the struct:
- `counters_put` waits for TR-5's `by` on the record.
- `lands_played` stays on `PlayerState`.
- CR 603.7h's count is not a row. It lives on `GameState`, keyed by the
  ability (§6.5).

A leaf takes a `HistoryCount { whose: PlayerSet, fact: TurnFact, is:
CountIs }`. Its `condition_reads` arm declares the source's controller,
because "whose" resolves against "you". CR 103.5's opening hands are drawn
before the first turn, so `Game::setup` clears the rows they wrote.

**Amended 2026-09-24 (`cr-coverage-audit.md` §4a, pass 3): a count "before
it" is taken at its record.** Storm copies a spell "for each other spell that
was cast before it this turn" (CR 702.40a). That covers 33 cards (`kw:storm`)
plus Thousand-Year Storm's "cast before it this turn". The trigger resolves
later, and a spell cast in response advances `spells_cast` without being
before it. So a leaf that reads the live summary at resolution counts too
many.

The count is every player's casts this turn, taken at the storm spell's
`SpellCast` record. The dispatcher advances the summaries record by record,
so the count is known there, and it has to travel with the trigger. The
smallest form is `SpellCast` carrying the turn's ordinal. It is authored with
the first storm card, like any other field in the summary.

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
    /// CR 400.7d's facts the permanent kept about the spell it was
    /// (`PermanentState.cast`, main item 9). Amended 2026-09-23 (the
    /// rulings pass, G1): an intervening "if" about them is rechecked after
    /// the permanent may have left (CR 603.4), and CR 113.7a answers from
    /// last known information — Vibrance evoked, its sacrifice ordered
    /// first, still deals 3 damage if {R}{R} was spent to cast it.
    pub cast: Option<CastFacts>,
    /// The spell's cost decisions: kicked, bargained, evoked
    /// (`PermanentState.cost_choices`). Amended 2026-09-24
    /// (`cr-coverage-audit.md` §4a, pass 2). PR #181 moved them out of
    /// `cast` a day after the field above was written, because CR 707.10
    /// copies them to a copy that was never cast. Without this field, an "if
    /// it was kicked" rechecked after the permanent left reads not kicked, and
    /// so does the token a copy of a kicked spell became, whose `cast` is
    /// `None`.
    pub cost_choices: CostChoices,
    /// For a stack object, its entry as it left the stack: what CR 707.10
    /// copies (targets, modes, X, the costs) and who controlled it, for CR
    /// 603.10e's look-back. `None` for anything else. Amended 2026-09-24
    /// (`cr-coverage-audit.md` §4a, pass 4): a copy trigger still copies a
    /// spell countered before it resolves (Double Vision's ruling), and
    /// CV-4's `copy_of` clones a live entry, of which there is none by then.
    pub entry: Option<Box<StackEntry>>,
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
(CR 608.2h, 113.7a); there is no second view type. TR-2a built the first
such reader, `bound_characteristics`, over today's frame. It answers from the
record's frame when the matched event was the subject's departure, and from
the live object otherwise. It answers nothing when neither holds, for example
an enters trigger whose creature was sacrificed in response; TR-2b's
`departed` frames answer that case (§6.1). Item 3's "three ad-hoc reads in `sba.rs`" are two existence probes
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
`records_since(mark)`, with `mark` the stream's sequence number at the
outermost `open_batch`.

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
   `ThisObject` moving from zone Z functions in Z ("when this card is put into a
   graveyard from anywhere" functions everywhere it can be; Guerrilla
   Tactics' discard functions in the hand); a condition about other
   objects functions on the battlefield unless 113.6b states otherwise.
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
zone map's, and waits for main item 173.

**A source that leaves in the same event** (main item 174, TR-1b's first
commit). A departure record's frame is taken at the same point, between
deciding and performing, for the same
reason: taken as each member moved, a later member's frame showed an earlier
one gone — Blood Artist framed after Humility left had its ability back, and
an artifact March of the Machines animated, framed after March left, was no
creature. So the batch frames every permanent its decided members take off
the battlefield before any of them performs — every permanent when a player
leaves, since CR 800.4a's fourth clause decides its exiles only after its
first two — and the move reads that frame (`capture_departure_frames`). A
destruction's move is a nested batch, and the permanent keeps the frame the
outer batch took: the event it is part of. Both halves are fixtures, each in
both batch orders.

### 4.4 Multiplicity: per record, per window, and the multiplier

CR 603.2c's two answers, and the `multiplicity` field is which one an arm
gives. The *occurrence* it counts is `occurrences_of`'s, below; the word is
the project's, and `plans/glossary.md` carries it.

- **`PerOccurrence`** (the default): one trigger per occurrence, and the
  arm's `occurrences_of` says what an occurrence is — a record for most
  kinds, a counter for the counter arms (Protean Hydra's ruling: several
  removed at once trigger that many times), an attacker for the attack
  shapes. A wipe of three lands is three `ZoneChange`s in one batch and
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
  three occurrences (CR 603.2c). Added 2026-09-24; TR-5b builds it.
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
  (TriggerMultiplierDef { caused_by: TriggerEvent, source: ObjectFilter,
  additional: u32 })`. Panharmonicon's rulings are the edges: only the
  object's own triggered abilities, never 603.6d's entry statics or a
  replacement, never a delayed or reflexive trigger it creates; two
  Panharmonicons make three, not four (additive); each instance is its own
  `PendingTrigger` with its own `seq`, choices and targets. The gate
  `TriggersOnlyOnceEachTurn` is read before the multiplier, so a capped ability triggers
  once under Panharmonicon (the survey's judge-literature note); `DoThisOnlyOnceEachTurn`
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
the records with that stamp up to `created.record`. `ReflexiveEvent` is a
`TriggerEvent` restricted to what the resolution's own instructions can
perform — "when you do" is the `ZoneChange { cause: Sacrificed }` the
preceding `Optional` proposed; if none matched, the entry is dropped
silently. Heart-Piercer Manticore's first ruling is the shape ("goes on the
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
controller. No printed ability watches its own kind, which is necessary
and not sufficient once custom cards exist: a trigger watching abilities
triggering, whose own triggering it then watches, recurses synchronously
inside one dispatch, where TR-6's loop detector — which counts decisions —
cannot see it, so the nesting bound is the only guard for that loop.
**Amended 2026-09-22 by the owner (the TR-1 review, theme D): at the bound
the answer is CR 104.4b's draw, not an `Err`.** TR-6 settles
`GameResult::Draw` through the same settlement §4.9's detector uses, and
`DISPATCH_NESTING_LIMIT` (16 today, `dispatch.rs:44`, an `Err` until then)
becomes that detector's threshold knob rather than a second number.
`BATCH_NESTING_LIMIT` (32, `actions.rs:38`) is the same pattern but a
genuine engine invariant — CR 614.5 bounds every replacement chain — so it
stays an `Err`.

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
its own batch performed, where the dispatcher asks every departure frame and
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
not re-derived**: one player's entries give the same game in either order,
and the engine does not ask, when every one of these holds — each the
reason the prompt is asked otherwise:

- **Equal defs.** Different abilities do different things. Compared as
  defs, not ids, since the TR-1 review's theme E: two grants of one ability
  carry two ids (§3.6), and are still one ability twice.
- **Identical bindings** — the same records and the same subject — since
  "that creature" or "that much" is otherwise a different object or number.
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
siblings read `LastKnownInformation` (§3.11): live if the object is where the event
left it, the frame otherwise — the record's when the event was the
departure, the entry's `departed` frame when the object left afterwards
(§6.1's amendment). Nothing the effect reads is copied at
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
the counters (corrected 2026-09-24; TR-2b's writer). Two Paragons are two
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
| its cost decisions — kicked, bargained, evoked (707.10; main item 30) | `PermanentState.cost_choices: CostChoices { additional, alternative }`, kept by a copy of the spell and the token it becomes, and by `LastKnownInformation` once it leaves (§3.11) | the entry performer, off `ResolvingObject.cost_choices` | `Condition::SpellWasKicked`, "if it was kicked", "if its evoke cost was paid" |
| the object a delayed trigger refers to (603.7c) | `DelayedTrigger.refs: Vec<ObjectRef>` | the producer, from the records its instruction performed (§3.9) | the delayed check, the resolution |
| the answer to a cost paid at resolution (118.12: does, doesn't, can't) | `last_cost_answer`, carried by the resolver's walk beside the target cursor (§6.2) | the atom that takes the action | the "if" clause after it (§6.2) |
| an object's last known information after it left, for an entry that names it (113.7a, 608.2h) | `PendingTrigger.departed`, `StackEntry.departed` | `capture_departure_frames` | the intervening "if" recheck, §6.3's readers (§6.1) |
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
| `put_on_stack.rs::activate_ability` (`:437`) | builds the `StackEntry` with `is_spell: false` | `trigger: None`; the placement path builds its twin; `Targeted` emitted after `announce_targets` | TR-1, TR-5 |
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
names *N* new objects, and `LastKnownInformation`, one frame, the permanent's, must
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
else — the granted and copied legs and the departure frames read a list no
registration saw, so they keep their whole-list walk, and the zone map is
keyed by ability rather than by kind and keeps its `is_empty` probe.
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
| TR-2b | every gameplay row `IDENTICAL`, with TR-1's predicate; the new predicate `differ`s on an arm of its own, the cards still unregistered (§12, TR-2b) | `differ` | the elision is the one change to what the engine asks, in both directions |
| TR-3 | `IDENTICAL` both pools | `differ` | the registry is empty on the old pools |
| TR-4 | `Layer walks` up by the widened captures on `stress`, `Memo hits` up on both (the control sweep runs while Act of Treason's row lives); every gameplay counter `IDENTICAL` | `differ` | the sweep is gated on `any_control_changing`; captures gate on `from`/`to` |
| TR-5 | `IDENTICAL` counters; `--dump-events` gains `Targeted`, `DamagePrevented` and entry `CountersChanged` lines | `differ` | records announced, no decision moved |
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
  A return makes a new object, so **TR-3 waits for CV-2, then CV-1b with
  `codebase-state.md` item 10 (CR 400.7)**.
- TR-4 widens the frame TR-5's combat shapes never read. TR-5a needs only
  TR-2b, so it may move up to just after it.
- TR-6's loop detector counts the decisions every earlier phase adds.
- TR-7 is last because the Ironworks loop reads all of them.

**Between TR-3b and TR-4a, `codebase-state.md` item 176's zone-change
record design** (the owner, 2026-09-24): what was done, who did it, which replacement redirected
it, the object the move made (item 177) and the moment each fact is taken
at (item 175), designed and reviewed before code. TR-4 widens that record,
and no earlier phase's card reads the parts it settles.

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

### TR-2b — "may", CR 118.12's answer, and the `departed` frames (1,980–2,170)

Split from TR-2 on 2026-09-24; it builds on TR-2a's gates and histories.
**Designed 2026-09-26**, against the tree after the bounded-state PR, #190 and
LL, and reviewed by the owner before code.

**The pieces, re-counted against the tree** by §12's method: 37 lines a test,
and the largest code row at ×1.9 for the top end.

| Piece | code | tests |
|---|---|---|
| The fold: six `Condition` leaves become one (decision 1) | ~130 | the leaves' unit tests, rewritten |
| `EventKindMask` at `u64` (decision 5); the arms `DrawsCard` and `ShufflesLibrary`, their projections, matching and authoring words | ~80 | 1: ATOM-121.5-001 made full — a move to the hand without "draw" fires no draw trigger |
| `Effect::Optional` with its chooser, the walk's answer, `Condition::CostAnswer`, `OptionalEffect`, and CR 603.2h's writer reading the answer (decision 2) | ~210 | 3: ATOM-603.5-001; ATOM-118.12-002's partial, a "may" whose damage is prevented still answering `Does`; "if you do" and "if you don't" reading one answer |
| `Sacrifice(Sacrificed)` (decision 4) | ~95 | 4: ATOM-118.12-001 on Standstill's board, as a fixture; the "if you can't" fixture; a stolen source answering `Cant`; "sacrifice that creature" |
| `AddCounters` over a filter; `CountOf(CardsInHand)` in the layer walk; `EachOf` over `LoseLife`, which Crawler's "each opponent loses 1 life" needs and the row did not list | ~40 | through the cards' tests |
| The `departed` frames, one capture and one writer, and CR 109.5's "you" at both instants and in a resolving "if" (decision 3) | ~140 | 5: an enters trigger's "if" and power read after its source is sacrificed in response; the recheck of a stolen source; a frame from the stack, from a hand, and from an effect that moved its own source |
| Item 163's predicate over the facts a def reads (decision 6) | ~120 | 3: the binding-read board, the source row, equal and unequal amounts; the three migrated tests are edits |
| **Nykthos Paragon**, **Psychosis Crawler**, **Cosi's Trickster**; Crawler and Trickster pooled (175 → 178 registered, 96 → 98 pooled) | ~105 | 11: Paragon's six rulings (the fourth is ATOM-603.2h-002, made full), Trickster's three, Crawler's one, and Crawler cast from hand with exact mana under `ManaWindowStop` |
| **Total** | ~920, top end ~1,110 | 27, ~1,060 with the edits |

**1,980–2,170 in code and tests**, inside §4's band, against the
2026-09-24 count's 1,720–2,180. Docs add ~500 more: this section,
archived at landing, with the stub, the record and the items. What the count
moved:
- `EachOf` refuses `LoseLife` by name today, and Crawler needs it.
- The fold is six leaves, not five: `CardInYourGraveyard` says "your" too.
- The predicate compares the source when the def reads it (decision 6).
- Standstill and Wicked Guardian stay fixtures. Standstill's "each of that
  player's opponents" is a group relative to the bound player, which
  `PlayerGroup` cannot say. Wicked Guardian's "another creature you control"
  is chosen at resolution (CR 608.2d), where the engine announces a `Choose`
  at placement, and its "another" is TR-3b's.

**Decision 1 — the fold: one variant, `whose` and a fact.** Six leaves each
read one fact about one player and carry the player in their name:

```rust
Condition::Player { whose: PlayerSet, fact: PlayerFact }
pub enum PlayerFact {
    ControlsPermanent(ObjectFilter), // "you control a Forest"
    LifeAtLeast(AmountExpr),         // "you have 40 or more life"
    LifeAtMost(AmountExpr),          // "you have 5 or less life"
    LibraryEmpty,                    // "your library has no cards in it"
    CardInGraveyard(ObjectFilter),   // "a red card in your graveyard"
}
// Kird Ape:   Condition::Player { whose: PlayerSet::You, fact: PlayerFact::ControlsPermanent(forest) }
// Bloodghast: Condition::Player { whose: PlayerSet::Opponents, fact: PlayerFact::LifeAtMost(AmountExpr::Fixed(10)) }
```

- **It holds when any player `whose` names meets the fact**, over the
  players still in the game, as `EachOf` and `resolve_putter` already read a
  set. "An opponent controls a creature" asks whether one opponent does, and
  `You` is one player.
- **It quantifies and does not sum.** `HistoryCount` sums its rows, and has
  to: "a creature died this turn" is every row. A sum would read "an opponent
  controls three artifacts" across two opponents, and a life total is not a
  count. The same sum misreads "an opponent lost 3 life this turn" at four
  seats. No registered card asks it, and when one does it takes the same
  quantifier.
- **Maintainability.** One arm in `holds`, `condition_reads` and
  `zone_function`'s match in place of six. A new player fact is one
  `PlayerFact` arm, and "each opponent" is a quantifier added with its card.
- **The first commit changes no behavior**, since each old leaf is exactly
  one pair. The scans are the same scans. §2b's rule 8 cites three of the
  old names as its examples and changes with them.

**Decision 2 — "may": its chooser, and CR 118.12's answer.**

```rust
Effect::Optional { chooser: PlayerRef, effect: Box<Effect> }
pub enum CostAnswer { Does, Doesnt, Cant }
Condition::CostAnswer(CostAnswer)   // "if you do", "if you don't", "if you can't"
struct ResolutionWalk { cursor: usize, last_cost_answer: Option<CostAnswer> }
```

- **The walk.** `resolve_effect_at`'s `cursor: &mut usize` becomes
  `walk: &mut ResolutionWalk`. Each resolution makes a fresh one, so a rider
  never reads its parent's answer (§6.2's amendment).
- **The chooser** is a `PlayerRef` (§6.2), resolved as `AddCounters`' `by`
  is: `You` is the controller, `Opponent` a player target or the only
  opponent. "That player may" (53 cards) is the bound player, which
  `PlayerRef` cannot name; it gets an arm with its first card.
- **The prompt** is `OptionalEffect { source }` (§9): yes or no, asked
  whenever the optional is reached.
- **The answer's writers.**
  - An atom writes `Does` as it performs: CR 118.12's "started to pay".
  - An atom that cannot start writes `Cant`. In TR-2b that is `Sacrifice`
    (decision 4) alone. Another primitive gets its "can't" with its first "if
    you can't" card.
  - `Optional` writes `Doesnt` when declined. When accepted it keeps the
    action's answer, except that `Cant` becomes `Doesnt`: CR 118.3 lets no
    player pay a cost they can't, so "you may sacrifice a creature; if you
    don't, …" with no creature is "you don't".
  - The yes-or-no is asked even when the action cannot start. Sparing it is a
    per-primitive pre-check, `backlog.md` §2.22's rule 1, and no TR-2b card
    needs one: Paragon's and Trickster's counters always start.
  - A clause's own atoms don't answer for the action before it. The walk
    restores the answer after a `Conditional`, so "if you do … if you don't …"
    reads one answer.
- **The reader is a leaf, not a second combinator.** CR 118.12 calls the
  clause an "if", and `Conditional` is the tree's "if". A leaf also composes
  under `All`.
  - The walk's `Conditional` arm answers `CostAnswer` itself, inside `All`
    too, and hands every other leaf to the evaluator.
  - `holds` treats `CostAnswer` as it treats `ModeChosen`: a static context
    has no answer.
  - Item 24's sized `IfYouDo { did, didnt }` is the combinator not built.
- **CR 603.2h's writer reads the resolution's last answer** and records the
  action only on `Does` (§6.4). So Paragon's declined "may" leaves the gate
  open, its first and third rulings. A prevented action still answers `Does`,
  Wicked Guardian's third ruling, because the answer is the choice and never
  the stream.

**Decision 3 — the `departed` frame, after the bounded-state PR.** A binding
copies its records at dispatch, and the window is flushed after it. So a
departure after the dispatch is in no record the entry holds. Its frame has to
reach the entry as the object leaves.

- **The frame and where it lives.**
  `DepartedFrame { object: ObjectRef, frame: Arc<EffectiveCharacteristics> }`
  sits in a `departed` list on `PendingTrigger`, on `StackEntry` and on
  `ResolvingObject`. `StackEntry` means every entry: CR 113.7a names
  activated abilities too (§15 item 15).
  - Placement moves the list onto the stack entry.
  - Resolution moves it onto `resolving`. Resolution takes the entry off the
    stack, and an effect can move its own source mid-resolution (CR 608.2h's
    "the effect has moved it").
  - TR-4a swaps the frame for `LastKnownInformation`. Its `cost_choices` and a
    stack object's `entry` are item 169's other two amendments (§3.11).
- **One capture, one writer.** Item 174's pass already frames every permanent
  a batch's decided members take off the battlefield, before any of them
  performs. It now also frames any other mover that a pending, stacked or
  resolving entry names (its source, or `binding.subject`), from any zone.
  - The performer that takes a mover's frame hands it to each entry naming
    the mover, so for a battlefield departure the entry holds the record's
    own `Arc`.
  - The gate is a scan of those entries, which on the common board are empty
    or short.
  - Rejected: a `GameState` map keyed by `ObjectRef`. Every exit an entry has
    from the stack would have to prune it, and a missed prune is the slow leak
    the bounded-state PR removed.
  - Rejected: writing at the window's close. Only a battlefield departure's
    record carries a frame.
- **The readers.**
  - `bound_characteristics` reads the record's frame when the event was the
    departure, the live object while its epoch holds, and the entry's
    `departed` frame after that. TR-2a's refusal then marks a missing
    capture.
  - The intervening "if", at both instants, and a resolving effect's own "if"
    read CR 109.5's "you" as the ability's. `settled_holds` gains a form that
    takes the player, and `you_for` and `FilterPlayers::for_source` answer it
    before the source's frame. The player is the candidate's controller at
    dispatch and the entry's locked controller at the recheck (CR 603.3a). In
    `Effect::Conditional` it is the resolution's controller. A static ability
    keeps its source's current controller.
- **When the source is gone.** `SourceUntapped`, `SpellWasKicked` and
  `HostMatches` still read the live object, and answer false once it has
  left. Their last known answer is a status or a cost decision. Those are
  §3.11's fields, and TR-4a points the leaves at the frame when it builds the
  type.
- **Item 169 closes here.** Its base, the widening and "from every zone" land
  in TR-2b. The frame's cost decisions and a stack object's entry are §3.11's
  fields, which TR-4a's row builds.

**Decision 4 — `Sacrifice`: the recipient is who sacrifices, and the payload
what.**

```rust
Primitive::Sacrifice(Sacrificed)
pub enum Sacrificed {
    ThisObject,        // "sacrifice this enchantment" (Standstill)
    TriggeringObject,  // "sacrifice that permanent" (Grafted Wargear, TR-4a)
    Chosen { filter: SelectionFilter, amount: AmountExpr }, // "target player sacrifices a creature" (Diabolic Edict)
}
```

- **The recipient keeps one meaning** in every form: CR 701.21a's "its
  controller". It is `Controller` for "sacrifice this enchantment", and a
  target player or `EachOf` for an edict.
- **When it answers `Cant`.** A named permanent is sacrificed only while the
  recipient controls it and no "can't" forbids the sacrifice (CR 701.21a,
  101.2). Otherwise the atom answers `Cant`, whether the permanent was exiled
  (Standstill, ATOM-118.12-001) or stolen. `Chosen` answers `Cant` when
  nothing can be chosen.
- **One spelling per concept (§2b).** The arms take `EffectRecipient`'s names
  for the same objects and resolve through the same readers, `this_object`
  and `bound_object`.
- **Later arms.** TR-3a adds one for a delayed trigger's remembered object
  ("sacrifice it at the beginning of the next end step"). TR-3b adds
  "another" to `Chosen`'s filter.
- **Rejected: `Destroy`'s grammar**, with the recipient naming the object.
  The recipient would then mean a player for an edict and an object for "it",
  and the named form would lose who sacrifices. `Primitive::Sacrifice`'s doc
  already warns against that for the edict.
- **Diabolic Edict** becomes `Chosen { filter: Creature, amount: Fixed(1) }`
  and plays the same game.

**Decision 5 — the mask: `u64`.** The fourteen kinds with `CardDrawn` and
`LibraryShuffled` fill `u16`. §3.3's table reaches thirty kinds with an arm by
TR-5. `u32` holds those with two to spare, and Phase 8's arms (discard,
surveil, cycling, …) would pass it. `u64` holds every `GameEvent` variant
twice over: thirty-two today, thirty-five after TR-5. The cost is six bytes on
each of the few `trigger_sources` entries, and the same instructions for each
OR. A compile-time assertion on the last kind's bit makes the next overflow a
compile error rather than a wrapped shift.

**Decision 6 — the elision's predicate: equal on every fact the def reads.**
Four of §5.2's conditions stay: equal defs, no instance of "target", no mode,
tier 1. "Identical bindings" becomes this: the entries agree on every fact the
def reads.

`TriggerDef::bound_reads()` walks the effect and the intervening "if". It
matches exhaustively over `Effect`, `EffectRecipient`, `AmountExpr` and
`Condition`, and over `Primitive` for its amounts, so a new leaf cannot
compile until it says what it reads.

| The def reads | Compared across the entries |
|---|---|
| "that object" (`TriggeringObject`) | `binding.subject` |
| "that player" (`TriggeringPlayer`) | `bound_player` |
| "that many" (`TriggeringAmount`) | `bound_amount` |
| "its power", "its toughness" | the subject, and the record its frame comes from |
| "this object" (`ThisObject`, `SourceInZone`, `SourceUntapped`, `HostMatches`, `SpellWasKicked`, `Attach`) | `origin` |
| "this ability" (the CR 603.2h gate, `ResolvedThisTurn`) | each identity's gate and count |

A fact the def does not read may differ. Two landfall triggers whose effect
ignores the land go on the stack unasked, and so do Soul Warden's two triggers
for Raise the Alarm's two Soldiers. Two Paragons with both gates open
(ruling 2) still go unasked. Their states are equal, so either order is the
same game.

**The source row corrects TR-1's predicate.** §5.2 compares bindings because
"that creature" is otherwise a different object. By the same argument, "this
creature" is a different object when the sources differ, and TR-1 compares
no source. Two Vengeful Warchiefs triggering on one life loss each put a
counter on themselves. Which one has its counter is on the board in the
window between the two resolutions, and a response can use it, so the two
orders are two games. `main` elides that choice, and the new predicate asks
it. So the random agent's stream moves in both directions.

**The migration, sized by running it.** Both predicates were patched in as a
throwaway and the whole suite run. Under each, 1,760 tests passed and the
same three failed, all in `phase_tr1_integration_test.rs`. **Three of the six
scripted `OrderTriggers` expectations move:**
- `soul_warden_entering_beside_two_creatures_triggers_for_each_of_them` and
  `an_artifact_dying_in_the_wipe_still_sees_the_creatures_die` each gain 1
  life and read nothing bound. They are now placed unasked.
- `identical_triggers_with_different_bindings_are_asked_their_order` has the
  prompt as its instrument, so it becomes decision 6's binding-read board.
- Blood Artist's two tests, which have targets, and the two different defs
  are asked as before.

**The A/B, predicted before any arm runs.** `fuzz_ab.py` on both pools at two
seats and four, 200 games at seed 12345. The counter runs are audited, and
timing is 3 × 200 on `performance`. There are four arms:
- **`main`**: #191's merge, `5fdf342`.
- **engine**: TR-2b with TR-1's predicate, and the three cards unregistered.
- **elision**: the engine arm plus the new predicate, with the cards still
  unregistered.
- **shipped**: the cards registered and two of them pooled.

The elision arm is the brief's third arm. It needs its own because pooling
Crawler and Trickster changes every deck, and §3 never reads a number across a
pool change. The shipped arm is that pool change, recorded and not budgeted,
as TR-2a's was.

| | predicted |
|---|---|
| engine vs `main` | Every gameplay row `IDENTICAL` on both pools at both seat counts, and the dumps identical game by game. `Windows past gate` and `Candidate visits` identical, or up where a zone-map or granted trigger exists: a draw's or a shuffle's window now has a kind, as a cast's did in TR-2a (+0.6 visits at four seats). Every other row identical, since only a mover outside the battlefield that an entry names costs a new frame, and no pooled card makes one. CPU per decision inside the 2.5-point budget |
| elision vs engine | `differ`, and **each diverging game's first difference is an `OrderTriggers` prompt that one arm asks and the other does not**. Only the engine arm asks Soul Warden's triggers for two or more creatures entering in one batch (Raise the Alarm, doubled by Parallel Lives). Only the elision arm asks two Vengeful Warchiefs of one player on one first life loss, and on `stress` two Paladins of Atonement at an upkeep. Every other game is identical |
| shipped | A pool change. `Triggers placed` rises with Crawler's draws, one trigger per draw per Crawler. Trickster's trigger is rare: on `performance` its only shuffle is an opponent's Darksteel Colossus shuffled back in, which `--require` reads |
| audit | agrees on every arm |

**The commits.** Each commit carries its own tests and re-measures the band.
1. The fold, A6b's first commit.
2. The mask and the two arms.
3. "May" and CR 118.12's answer.
4. `Sacrifice`.
5. The three small facilities.
6. The `departed` frames and "you". This head is the engine arm.
7. The predicate and the migration. This head is the elision arm.
8. The cards, registered and pooled. This head is the shipped arm.
9. The record.

§5.2, §6.1, §6.2, §7 and §11 are rewritten by the commits that build their
pieces.

### TR-3 — delayed, reflexive, and "until" — TR-3a and TR-3b (2,800–3,600)

| Piece | ~additions |
|---|---|
| the registry, `DelayedTrigger`, `DelayedSource`, `DelayedDuration`, `ObjectRef`, `Instant`, `ExtraTurnId` on `turn_queue` and `current_turn_origin`, `Primitive::CreateDelayedTrigger`, provenance from `ResolutionContext` and from a rider, 107.3n's X, `ChooseDelayedTriggerEvent`, cleanup expiry of `ThisTurn`; `Effect::Reflexive` and the immediate check; `UntilEvent` resolved at dispatch (610.3) | ~520 |
| `Primitive::ReturnToBattlefield` and `ReturnToHand` made real over `change_zone` / `EnterBattlefield` (the stub arm at `resolve.rs:1417`), with 610.3c's owner's control; a source-relative "another" for a sacrifice chooser | ~120 |
| cards: **Final Fortune** (603.7d, a named extra turn; its ruling — a skipped extra turn loses nothing — is the `ExtraTurnId` test), **Flickerwisp** (603.7e from a triggered ability, 603.7c through exile, CR 400.7; its second ruling is 513.2's sibling), **Cornered Crook** (603.12: `Optional` then reflexive, any target — Heart-Piercer Manticore prints the same shape with an LKI power read and cannot register whole, since embalm is `backlog.md` §2.3's and CV's), **Banishing Light** (610.3's until-return, no stack; its ruling that an Aura or Equipment on the exiled permanent falls off is CR 400.7's, and "leaves before the trigger resolves, nothing is exiled" is 610.3a); Flickerwisp and Banishing Light pooled (the registry, the until path) | ~320 |
| tests, 30: §13's 20 TR-3 atoms (513.2 both ways, 603.7f through a rider fixture and 603.7g's fixture among them); Heart-Piercer Manticore's four trigger rulings as fixtures (the LKI power read); Tatsumasa's simultaneous choice as a fixture; Sneak Attack's ruling as a fixture board (the card waits for an indefinite haste, CV-1b); the three card rulings above — Final Fortune's, Flickerwisp's second, Banishing Light's Aura; `refs` under Parallel Lives, each token made exiled (§3.9's amendment) | ~1,110 |
| docs, ledger, record | ~250 |

**Split 2026-09-24**, by the re-count below. **TR-3a** is the delayed
registry, with Final Fortune. **TR-3b** is reflexive triggers, the returns
and "until" (610.3), with Flickerwisp, Cornered Crook and Banishing Light.

### TR-4 — the look-back list, the frame, unattach, control — TR-4a and TR-4b (2,640–3,460)

| Piece | ~additions |
|---|---|
| `LastKnownInformation` and `Status` (3 field sites, 7 literals), its `cast` (§3.11's amendment), item 173's zone statement read off the intervening "if" (scheduled here 2026-09-23), the widened capture (item 15) gated on prior visibility for the third class, the reader methods on the type, `Unattached` with `announce_unattached` at three performers (7 `EquipmentDetached` sites), `ControlChanged` from the sweep with `announced_controller`, the arms `BecomesAttached`, `BecomesUnattached`, `ControlChanges`, `IsCountered`, `PlayerLoses`, `ZoneChange`'s other two classes, `Condition::TriggeringObjectHadCounters` | ~510 |
| cards: **Grafted Wargear** (603.10c, item 14's host, "sacrifice that permanent"), **Strangleroot Geist** (undying as an `AbilityDef` — quadrant ③ — with 702.93a's intervening "if" off `Status.counters`, `ReturnToBattlefield` with an entry counter; Kitchen Finks prints persist, the mirror, and is not registered because its hybrid cost would have to be misspelled — Mirrorweave's precedent, `codebase-state.md`'s CV-1 status), **Rancor** (603.6e/400.7f, an Aura's own dies-trigger, `ReturnToHand`), **Multani's Presence** (603.10e — `SpellCountered`, never `SpellFizzled`), **Golgari Brownscale** (603.10a's third class; registered whole if dredge — a draw replacement functioning from the graveyard, which LK and RF make expressible — fits the band, else the atom's fixture and the card in §15); a 603.10d fixture over Act of Treason's steal; Strangleroot Geist and Grafted Wargear pooled | ~420 |
| tests, 29: §13's 14 TR-4 atoms (122.8 and 122.9 off the frame among them); Kitchen Finks' eight persist rulings as undying's tests (the same shape with the counter's sign flipped); Grafted Wargear's three; Guile's two boards with Yixlid Jailer for the second class (its rulings name both cards); item 173's Bridge from Below in a graveyard, and the Jailer fixture that follows it | ~1,070 |
| docs, ledger, record; trace page decided at close (the look-back reads changed) | ~300 |

**Split 2026-09-24**, by the re-count below: **TR-4a** is the frame and
**TR-4b** the records.

**Prerequisite if TR-4 registers Ichorid or Bloodghast:** both return
themselves from a graveyard, which CR 113.6m places in that zone and
`zone_function` does not derive (Ichorid's intervening "if" is also main
item 173's 113.6b statement).

### TR-5 — combat's shapes, targeting, counters, prevention, the multiplier — TR-5a and TR-5b (3,080–4,350)

| Piece | ~additions |
|---|---|
| `AttackShape` and `BlockShape` over item 11's records (one blockers record per declaration), `Targeted` at three emit sites, `DamagePrevented` from the prevention leg, entry `CountersChanged` with `by`, the arms `Attacks`, `Blocks`, `BecomesTarget`, `DamageIsPrevented`, `CountersPutOn`/`RemovedFrom` with `nth` and per-counter occurrences, `ActivatesAbility`, `CreatesToken` (its `kind` the instructed definition, §3.3's amendment), `Scries`; `Effect::TriggerMultiplier` behind the gate; CR 704.5q routed through two `RemoveCounters` proposals in the state-based batch (Deferred Migrations item 6's counter half; `CountersAnnihilated` deleted) | ~575 |
| cards: **Hellrider** (508.3a's defender), **Loyal Sentry** (509.3b), **Cephalid Aristocrat** (item 12, mills), **Simic Ascendancy** (122.6 entry counters, "one or more", 603.4 at upkeep, `WinGame`), **Selfless Squire** (item 16 — its second ruling: any prevention, not only its own), **Panharmonicon** (603.2d, its ten rulings), **Protean Hydra** (CR 704.5q's removal is a removal — its six rulings; X entry counters through RC-5's dynamic amount, RD's rider for "remove that many", TR-3's delayed trigger, so the routing's fixture if any of the three refuses it); Hellrider, Simic Ascendancy and Panharmonicon pooled | ~440 |
| tests, 32: §13's 4 TR-5 atoms; 509.3a–g's seven readings; Panharmonicon's ten rulings (its edges); Protean Hydra's six; Selfless Squire's second; 508.4's "put onto the battlefield attacking never attacked" as a fixture over item 128's field; Frost Titan's once-per-spell as a fixture (the card waits for "unless pays"); Ajani, Nacatl Avenger's two reflexive rulings as fixtures, under the registered Doubling Season and Divine Visitation (§3.3's amendment) | ~1,180 |
| docs, ledger, record | ~250 |

**Split 2026-09-24**, by the re-count below. **TR-5a** is combat,
targeting, ability damage and excess damage. **TR-5b** is counters,
prevention, the multiplier, tokens and scry.

### TR-6 — state triggers, the loop, and the rule-owned arm (935–1,360)

| Piece | ~additions |
|---|---|
| the state check at its three schedules, `state_triggers_armed_off`, `trigger_left_stack` at four sites, `TriggerCondition::State` matched; `Condition::Not` (Emperor Crocodile's "no other creatures" is its first card — LI-3's rule for when the enum grows); `LoopDetector` (Tier 1) on `GameState` reading A4e's decision counter, `GameResult::Draw` through the settlement; the same draw at `DISPATCH_NESTING_LIMIT`, which becomes the threshold knob (§4.8), with its two doc comments (`dispatch.rs:40`, `game_state.rs:214`); `TriggerOrigin::Rule` declared with `InherentAbility` empty | ~320 |
| cards: **Emperor Crocodile** (603.8, its two rulings), pooled. Immortal Coil is the CR 104.4b board — its third ability beside Platinum Angel is "an involuntary infinite loop ... the game will end in a draw" by its own ruling — and it is a **fixture**, not a registration: its first ability's `Cost::ExileFromGraveyard` is a validation and payment stub today (`engine/costs.rs`), and a card wearing a real name with a dead ability is what `engineering-practices.md` §3 forbids | ~120 |
| tests, 9: §13's 2 TR-6 atoms; Emperor Crocodile's two rulings; the Coil's three trigger rulings on the fixture; the draw at threshold, with Platinum Angel registered; the draw at the nesting bound (§4.8). The ratchet's reading at the phase's close (§9 of the practices) is a reading, not a test | ~300 |
| docs, the close-out, §16's deferrals re-read, ledger, record | ~400 |

### TR-7 — the Krark-Clan Ironworks loop, the track's showcase (1,020–1,420)

`cost-architecture.md` §3.11's judged loop, walked step by step as item 6's
integration test. It moved here from TR-2 on 2026-09-24: the loop's triggers
are the easy part. The re-count found five facilities that no earlier row
lists:
- graveyard targets, for Scrap Trawler's and Myr Retriever's returns (with
  §17's `ManaValueLessThanSource`);
- any-color mana, for Mox Opal and Chromatic Sphere (`backlog.md` §2.19);
- metalcraft, an activation restriction with a count condition (§2.8);
- a draw inside a mana ability's effect (step 5);
- a mana window inside a mana ability's activation (step 3).

Not split yet. Its brief re-counts it against the tree TR-6 leaves.

### Re-counted 2026-09-24 — the splits, and why the plan keeps missing

One read-only agent per phase checked every clause of every named card
against the code. The counts below replace the 2026-09-22 and 2026-09-23
re-counts. Figures are code plus tests, the band `engineering-practices.md`
§4 sets:

| Phase | Was | Re-count | Split into |
|---|---|---|---|
| TR-2b | 1,500–1,700 | 1,720–2,180 | no split |
| TR-3 | 2,070–2,540 | 2,800–3,600 | **TR-3a**: the delayed registry, with Final Fortune, 1,600–1,980. **TR-3b**: reflexive triggers, the returns and "until", with Flickerwisp, Cornered Crook and Banishing Light, 1,410–1,640 |
| items 176/177 | — | 800–1,050 | their own PR, between TR-3b and TR-4a |
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
- "another target" (targeting refuses `NotSource`), by TR-3b;
- damage an ability deals, dealt by its source permanent (CR 113.7a), by
  TR-5a.

**Design corrections the re-count made.**
- TR-2b's CR 603.2h writer records the action only when it was taken (§6.4).
- Simic Ascendancy triggers once per creature, not once per window (§3.3,
  §4.4).
- TR-6's loop detector counts decisions as item 138 defines them, never
  prompts (§4.9).
- The Ironworks loop is TR-7's test, not TR-2's (§17).

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
| **TR-3** | 603.7-001; 603.7a-001; 603.7b-001, -002; 603.7c-001; 603.7d-001; 603.7e-001; 603.7f-001; 603.7g-001 (fixture); 603.7h-001; 603.12-001; 107.3n-001; 513.2-001, -002; 610.3-001; 610.3a-001; 610.3b-001; 610.3c-001, -002; 610.3d-001 | 20 |
| **TR-4** | 603.6e-001, -002; 400.7e-001, -002; 400.7f-001; 603.10c-001, -002, -003; 603.10d-001; 603.10e-001; 603.9-001; 603.2e-002; 122.8-001; 122.9-001 | 14 |
| **TR-5** | 508.2a-001; 603.2d-001; 122.7-001; 120.10-001 | 4 |
| **TR-6** | 603.8-001, -002 | 2 |
| **Deferred, with the rule that lets each wait** | 603.2a-001 (needs an "activated abilities can't be activated" restriction — RS-2's); 603.3c-001, -002 and 700.2b-001 (modes — `backlog.md` §2.7, on §5.4's placement); 607.2c-001, 607.2h-001 (linked — §2.2); 603.12a-001 and 605.3a-002 (a cost paid at resolution — CP-1, which also unlocks 702.21a-001, -002 (ward = TR-5's event + CP-1's "unless"), Strict Proctor and Frost Titan); 400.7-001 (the rule itself — CV-1b); 111.13-001, 112.2-002, 700.2g-001, 707.10b-001, 707.5-002, BOUNDARY-707.7-001, BOUNDARY-707.9g-001 (copies — CV-2, CV-4, with §6.5's sentence); 208.2b-001, -002 (copiable values from an entry choice — CV); 610.5-001, -002 (a granted keyword at cast — §2.1's convoke); 611.2e-001, 611.3d-001, -002 (their owners: 611.3d is §2.3's foretell); 115.9a-001 ("with N targets" — a filter over `chosen_targets`, Phase 8 with its first card); 701.43d-001 (exert — §2.5); 701.66a-001, -002 and 702.176a-003 (earthbend, impending — Phase 8); 724.1-001, 724.2-001, -002, COMP-MONARCH-COMBAT-001, 724.3-001, 724.5-001, 725.1-001, 725.2-002, 725.3-001 (designations — Phase 9, on §3.8's arm); 608.2d-001 (stays partial: choices at resolution are §2.7's and CP-1's); 608.2j-001 (a characteristic read — ALREADY-IMPL's, re-filed at TR-6's close) | 41 |

Ninety-two of the 133 Phase 7 atoms are owed across the six phases (plus
the two from outside the phase TR-1 claims, 121.2c-001, which TR-2a claims,
and the one TR-2b claims in part),
forty-one are deferred with an owner each; 92 + 41 = 133, no atom listed twice and none unlisted — checked
against `spec.sqlite` on 2026-09-18, and worth re-checking the same way
at each close. The deferrals are re-read at item 6's close audit
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
15. **The `departed` frames serve activated abilities too, and nothing here
    builds that half** (the rulings pass, 2026-09-23). CR 113.7a names
    activated and triggered abilities alike: "{T}: this creature deals
    damage equal to its power" with its source sacrificed in response reads
    the source's last known information, and ATOM-113.7a-001 (ALREADY-IMPL)
    proves only that the ability still resolves. §6.1's field is written
    for every stack entry whose source departs, so an activated ability's
    reader is a leaf away; which reader, and whether today's `SourcePower`
    already answers it, is the first activated card's question.

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
- `cost-architecture.md` §3.11's Ironworks loop is TR-7's integration test
  — every step but the triggers exists, and step 6's "with lesser mana
  value" is the one selection leaf this document adds when Scrap Trawler
  registers (a graveyard target compared against the trigger's source:
  `ObjectFilter::ManaValueLessThanSource`, three edits).
- `plans/glossary.md` gained *dispatch*, *window*, *binding*, *tier* and
  *probe* with TR-1 (2026-09-19), and `check_glossary.py --suggest` at each
  close is the gate on the rest.
- `plans/references/trigger-survey.py` deletes with the survey when TR-6
  closes, per its own docstring.
