# Replacement & Prevention Effects — CR 614–616

> **Status:** design, authored 2026-08-24; revised 2026-08-25 after a
> grounded-in-code audit — §4.1a (`then` timing, researched), §4.2 (applied-set
> scope corrected), RA items 11–12 (life-mutation routing), RD's CR 120.3
> decomposition, and the RA split note. No code written yet.
> **Authority:** type shapes, the pipeline algorithm, the event vocabulary, and
> the phase sequencing for CR 614/615/616. Where this contradicts
> `codebase-state.md`, that file wins on *what exists*; this file wins on *what
> is being built*. `CLAUDE.md` → "Critical path to v1" still owns the ordering
> of this phase against the others.
> **Companion:** `layers-architecture.md` is the model for this document and the
> owner of everything CR 613. Read §5.2 (acyclicity) and §9 (hypothetical check)
> before touching §5 here — the look-ahead frame shares its read-side seam,
> though not its perturbation (§5).

---

## 0. The budget — why this stays tight

Written in answer to a review question worth keeping at the top of the file:
*what stops this sprawling when we start populating cards?* The design makes
four commitments, each of which is falsifiable and at least one of which has
already been tested against the whole card pool.

**One. A card is data, not code.** The success condition for RB onward is that
adding a replacement-effect card touches `src/cards/*.rs` and nothing else. A
card that needs an engine branch is a design failure and should be treated as
one at review time, not absorbed. The `then: Option<Effect>` field exists
precisely so per-card variety ("and its controller taps it", "and you gain that
much life", "then that creature explores") lands in the effect tree the engine
already has rather than in a new enum arm.

**Two. There is exactly one growth axis, and it is not card-shaped.** A
replacement can only replace an event the engine **proposes**, so the set of
replaceable event kinds is the set of `GameAction` variants — a question about
`perform_action`, not about 30,000 cards (§8a). `EventPattern` is a mechanical
projection of that set. `Rewrite` is closed. Growth arrives as *new mutations*,
and mutations are bounded by the engine plus CR 701's enumerated keyword-action
list, both finite and both readable.

**Three. The completeness claim was tested, not asserted.** All 561 printed
cards matching `o:/would.*instead/` were pulled and every clause classified
(§3.2c). 549 of 574 clauses watch an event kind already in the vocabulary; 14
want a new `GameAction`; 11 are CR 701 keyword actions; **zero want a sixth
`Rewrite` arm.** That is the evidence that the pressure lands on the axis this
design chose to absorb it on. Re-run the pass if a future phase changes the
algebra — `plans/references/replacement-census.py` re-runs the clause pass and
the answer is a number.

**Three-and-a-half. The types have a measured ceiling.** §8b sizes them from
both ends — CR 701's 67 enumerated keyword actions from the top, and what cards
actually watch from the bottom. `GameAction` projects to ~16 at end of RE and
~30 for a Commander-viable pool, against a derived ceiling in the low 40s;
`Rewrite` stays at 5. For calibration, `Primitive` is already 36 variants and
`GameEvent` 28, and neither causes trouble. The single largest structural saving
is that `ZoneChange { cause }` absorbs eight keyword actions and ~1,500
trigger-watching cards into one arm.

**Four. Performance has one designated lever and a measurement gate.** The
pipeline sits on `execute_action`, so it is hot by construction. §8 commits to
building it straight, measuring with `fuzz_games --games 200 --seed 12345`, and
recording the number here — the discipline the layer phases used — with an
answer-preserving event-kind bitmask as the only pre-approved optimization and
semantics-assuming shortcuts ruled out in advance (`can_change_abilities()` is
the cautionary tale). §11 item 5 prices the two hypothetical-frame call sites
separately and closes that question on measured numbers rather than taste.

**The axis this absorbs breadth on.** Card variety is real and unbounded, and
§8c separates where it lands: event *kinds* are a closed enum, event *predicates*
are a composed grammar, and *results* reuse the `Effect` tree. Nothing in the
design has a variant per card, which is the only route to the "thousands of
entries" the review worried about. §8c works six deliberately awkward cards
through end to end and prices the total at three already-budgeted variants and
two struct fields.

**What this does *not* claim.** The event vocabulary is incomplete and known to
be (§8a names eight missing kinds). ETB look-ahead has genuinely
counter-intuitive rulings (§5a works the worst one through). The
`AmountRewrite` sub-enum has one identified pressure point (§3.2c). And the one
risk that measurement cannot close today is named rather than waved at: whether
the predicate grammar stays a grammar instead of becoming a per-card DSL (§8c's
last section, with three guards and a 5% escape-hatch budget). Those are
budgeted, not hidden — a plan whose risks are named and sized is the thing that
resists sprawl; a plan that claims none is the thing that produces it.

---

## 1. Verdict — is this the right next step?

**Yes, and the prerequisites are met.** Recorded 2026-08-24, so a later session does not re-litigate it.

→ The prerequisites, why not triggers or the CR 613.8 cluster first, the honest cost of going first, and "Scope, measured"'s 88-of-124 atom split and card table: `plans/archive/replacement-architecture-landed.md`, "1. Verdict" (evicted 2026-09-15).

---

## 2. The event spine

One vocabulary, two hooks. This is the shape the fork decision fixed
(`state-tracking-architecture.md` → Resolution postscript), written out for
this phase:

```
 caller builds        CR 614/615/616         mutation           CR 603
 a proposal           rewrites it            happens            watches it
┌──────────────┐     ┌──────────────┐      ┌──────────┐      ┌──────────────┐
│  GameAction  │ ──► │ apply_repl.. │ ───► │ perform  │ ───► │  GameEvent   │
│  (proposed)  │     │  (this doc)  │      │ _action  │      │ (performed)  │
└──────────────┘     └──────────────┘      └──────────┘      └──────────────┘
                            │                                        │
                            │ may emit sub-actions (616.1g)          │ LKI frame
                            └──► recurse                             │ cause
                                                                     │ batch id
                                                                     │ resolution ctx
```

Three invariants fall out, and each is load-bearing:

1. **A `GameAction` is a proposal and is never authoritative.** Nothing may read
   it as a record of what happened. `execute_action` returns what was actually
   performed, because callers need it (CR 121.2a's "draw that many cards" is a
   return value, not a log query).
2. **A `GameEvent` is a performed record and is emitted only from inside the
   chokepoint.** Ad-hoc emission is what produced the activation-invisibility
   gap (`codebase-state.md`, Before Triggers item 2). Phase RA closes it.
3. **`GameAction` carries semantics the state delta cannot.** "Destroyed" is not
   "moved to graveyard" (CR 701.8b); "drawn" is not "put into hand" (CR 121.5);
   "sacrificed" is not "destroyed". These are `cause` fields on the action, set
   by the caller who knows, not inferred by the performer.

---

## 2a. As built — every type Phase RB shipped, in one page

**Why this exists.** Twelve types landed in one PR with no summary of what they
are, and the review that followed recorded "I lost the mental model" as a real
cost of the size rather than a personal failing. This section is the map. The
sections below are the argument for each shape; this one is only what shipped
and where it lives.

**Read the Growth column as a contract.** *Closed* means a new arm is a claim
that the CR permits an operation the list omits, and should arrive with the rule
number that says so. *Grows* means adding an arm is a normal diff — but on the
stated axis only. Everything is matched exhaustively and nothing is
`#[non_exhaustive]`, so adding an arm fails to compile at every reader, which is
the enforcement.

### The shipped types

| Type | Where | What it is | Growth |
|---|---|---|---|
| `GameAction` | `engine/actions.rs` | The proposal vocabulary — what a caller asks for, before CR 614 sees it | **grows**, one variant per replaceable event kind (§8a) |
| `ActionContext` | `engine/actions.rs` | `{ dp, resolution }`, threaded to every mutation. `dp` is how the pipeline reaches CR 616.1's prompt | grows only if a new ambient input appears |
| `ZoneChangeCause` | `types/zones.rs` | Why an object moved. No catchall, by design — `(from, to)` cannot tell a sacrifice from a destruction | **grows**, one per distinct reason a mover can name |
| `DestructionSource` | `types/zones.rs` | Which of CR 701.8b's two routes destroyed it; lowers to a `ZoneChangeCause` | closed by CR 701.8b (effect, lethal damage, deathtouch) |
| `ReplacementDef` | `types/replacement.rs` | One replacement effect as data, nine fields in declaration order: `pattern` + `affected_objects` + `rewrite` + `then` + `class` + `uses` + `is_regeneration` + `exempt_from_614_5` + `optional` | grows by *field*, rarely; per-mechanic variety goes in `then` |
| `EventPattern` | `types/replacement.rs` | Which events an effect watches. Ships 6 arms: `DealDamage`, `ZoneChange`, `Untap`, `Tap`, `Destroy`, `CounterChange` | **grows on one axis only** — an arm per `GameAction` variant (§3.2a) |
| `DestructionSourcePattern` | `types/replacement.rs` | The `EventPattern::Destroy` filter over `DestructionSource` | tracks `DestructionSource` |
| `Rewrite` | `types/replacement.rs` | What the effect does to the event. **Ships 2 arms, not §3.2b's 5**: `Prevent` and `Instead(GameActionTemplate)` | **closed** — `Amount`, redirection and the rest land with the phase that can apply them |
| `GameActionTemplate` | `types/replacement.rs` | The substitute an `Instead` produces, as a template over the incoming event. Ships `ZoneChangeTo` (three RB customers — the finality counter, CR 903.9b, and Kalitas's exile) and `RemoveCountersFromAffected` (two — the shield and stun counters) | **grows per card** — this is the unbounded arm's payload, bounded by "must produce a `GameAction` the engine already proposes" |
| `ReplacementClass` | `types/replacement.rs` | CR 616.1a–e's forced-choice buckets, `Ord` in the rule's own order | **closed** — all five ship; only `Other` has a producer |
| `Uses` | `types/replacement.rs` | `Static` or `Once`. `CounterBacked` did not survive contact with the CR (§3.2) | **closed** — `Shield(u64)` is CR 615.7's and lands with RD |
| `ReplacementInstanceId` | `engine/replacement/instance.rs` | CR 614.5's identity key: `Registered` / `StaticAbility` / `Counter` / `GameRule` | **grows**, one arm per §3.3 gather source |
| `GameRuleReplacement` | `engine/replacement/instance.rs` | A replacement belonging to no object's text. CR 903.9b is the only member | grows with the rules that behave as effects |
| `ReplacementInstance` | `engine/replacement/instance.rs` | One applicable effect, gathered as a snapshot — the loop mutates state between iterations, so a borrow could not survive a pass | — |
| `CounterEffectKind` | `engine/replacement/gather.rs` | Which of CR 122.1c's *two* effects a shield counter is. Two effects, one counter, two CR 614.5 identities | grows with CR 122.1's replacement-shaped counters |
| `EventSubject` | `engine/replacement/gather.rs` | What a proposed event is *about* — an object or a player. Named for the event because `ObjectSet` already answers the other question, which objects an *effect* applies to | closed by what an event can be about |
| `Rider` | `engine/replacement/pipeline.rs` | A queued `then`, resolved by the caller after the event is performed. CR 615.5 when a *prevention* effect queued it; CR 614.1a/614.6 otherwise — the rest of an "instead" is part of the modified event | — |
| `ReplacementEffectId` / `RegisteredReplacementEffect` / `ReplacementEffectRegistry` | `state/replacement_effects.rs` | The registry for replacements a *resolution* created, with CR 614.3 durations. Static abilities are **not** here — they are read off the effective ability list | — |

### One action, end to end

The path every mutation takes, with the file that owns each step:

```
caller
  │  builds a GameAction — the proposal, never authoritative
  ▼
GameState::execute_action / execute_actions            engine/actions.rs
  │  a batch opens one BatchId; a nested call joins it (CR 120.3f)
  │
  ├─ phase 1: DECIDE, in APNAP order of chooser (CR 616.1 + 101.4)
  │    └─ apply_replacements(action)                   replacement/pipeline.rs
  │         ├─ never_happens? → CR 614.7a/120.8/119.10, no event to replace
  │         ├─ is_blocked?  → CR 614.17 "can't" wins, event dropped (101.2)
  │         ├─ gather()     → §3.3's sources           replacement/gather.rs
  │         │    static abilities off the EFFECTIVE ability list, never a
  │         │    registry — which is what lets Humility strip one for free
  │         ├─ filter to the applied-set-eligible (CR 614.5)
  │         ├─ must_choose_among() → CR 616.1a–e's first non-empty step
  │         ├─ 0 candidates → done.  1 → apply it, no CR 616.1 prompt.
  │         │  2+ → DecisionProvider::choose (CR 616.1's ordering prompt)
  │         ├─ if `optional`: ask_apply_optional_replacement (CR 614.1a) —
  │         │  a second, independent prompt, and one candidate is enough
  │         ├─ apply_rewrite() → a new GameAction, or None (CR 614.6)
  │         ├─ push `then` onto riders (615.5 / 614.1a) — unconditional (615.12)
  │         ├─ exempt from CR 614.5? check_exempt_terminates — the argument
         │  the applied set does not supply. **The loop has no cap**
         └─ loop (CR 616.1f) with the applied set carried
  │
  ├─ phase 2: PERFORM, in batch order
  │    └─ perform_action(decided)                      engine/actions.rs
  │         the ONLY writer. `EnterBattlefield` is the zone change onto the
  │         battlefield: its performer moves, announces, builds the entity.
  │         `announce_zone_change` is the one ZoneChange emitter — three
  │         callers, each of which performed the move it announces
  │
  └─ phase 3: RIDERS, after the event (615.5 / 614.1a), fresh applied set
       └─ resolve_rider()                              engine/actions.rs
```

Four properties of that picture are load-bearing and easy to break:

1. **The decide/perform split is CR 704.3**, not an optimization. A batch
   decides every member against *one* board. One write is deliberately inside
   deciding: `consume_use` removes a spent `Uses::Once` row, because the next
   member must not be offered a shield the previous one already spent.
2. **`gather` is the only place effect existence is decided**, and it asks the
   effective ability list. `GameState::replacement_ability_sources` gates the
   sweep and is a *hint* — a new gather source that is not added to the gate is
   silently dead on every board the gate skips.
3. **Riders run after, never mid-loop.** During the loop nothing has happened
   yet, so a rider run inside it runs before the event it rides on.
4. **The replaceable event is the outermost proposal** (§11 item 20). A
   performer may nest a proposal only when the outer event is real whether
   or not the nested one survives replacement. Entering fails that test —
   entering *is* the zone change — so `EnterBattlefield` is the only proposal
   for it and no `ZoneChange { to: Battlefield }` exists; casting fails it
   too, so CR 601.2a's move is silent until 601.2i (RC-4b).

---

## 3. Type surface

### 3.1 `GameAction` — the replaceable-event vocabulary

Today: `DealDamage`, `DrawCard`, `GainLife`, `LoseLife`, `ZoneChange`, `Untap`,
`Tap`. The full target vocabulary, with the phase that adds each:

| Variant | CR | Phase | Note |
|---|---|---|---|
| `DealDamage` | 614.2, 615, 609.7 | exists → RD | already routed; combat + spells |
| `ZoneChange { cause }` | 400.6 | RA | `cause` is new and required |
| `Destroy { object, source }` | 701.8b, 614.8 | RB | **outer** event; performs an inner `ZoneChange{cause: Destroyed}` |
| `Untap` / `Tap` | 122.1d, 603.2e | RA (events) → RB | currently silent; stun counters need `Untap` replaceable |
| `AddCounters { target, kind, n, source }` | 122.1, 614.16 | RB | |
| `EnterBattlefield { object, controller, mods }` | 614.1c/d, 614.12, 110.5b | RC | **not** `ZoneChange{to: Battlefield}` — carries the *how* |
| `DrawCards { player, n }` (outer) | 121.2a, 616.1g | RE | contains N `DrawCard` inner events |
| `GainLife` / `LoseLife` | 119.10 | exists → RE | |
| `CreateTokens { defs: Vec<TokenDef>, controller }` | 614.16 | RE | **`Vec`, not `(def, n)`** — Academy Manufactor's "one of each", Chatterfang's "those tokens plus that many Squirrels", Divine Visitation's substitution (§3.2c) |
| `BeginStep` / `BeginPhase` / `BeginTurn` | 614.10 | RE | skips replace these — RE-1, proposed by a turn queue `advance_turn` drains, which is also CR 500.7's extra turns (`backlog.md` §2.17) |
| `Scry { player, n }` | 701.22 | RE | RE-8, with `Primitive::Scry`; Eligeth replaces it |
| `ProduceMana` | 106.6a | RE | RE-9; one proposal from the two silent writers `mana.rs` and `resolve.rs` hold today |
| `PlayerLoses { player, reason }` / `PlayerWins { player }` | 104.2b, 104.3e, 704.5a–c, 704.7 | RE | added by §8a's audit (2026-08-24); RE-6, where the four SBA loops become batch members |
| `CreateTokenIn { object, zone }` | 111, 704.5d | RE | RE-4: the substituted form of a token's entry — an *appearance*, not a move — and the second deliberate no-`EventPattern` variant after `Attach` |
| `ShuffleLibrary { player }` | 701.24a | RF | the third no-`EventPattern` variant — nothing printed says "would shuffle … instead" — and the first in-game writer of a library's order; proposed by `Primitive::ShuffleLibrary`, the rider of Darksteel Colossus's "shuffle it into its owner's library" (CR 701.24c) |

`ZoneChangeCause` is the semantic carrier that makes CR 701.8b answerable:

```rust
// Derived from call sites, not researched from the card pool — see §11.
// **No catchall.** Every mover names its reason.
pub enum ZoneChangeCause {
    // --- effects (CR 701), one per object-moving `Primitive` ---
    Destroyed,       // 701.8b way 1 — an effect using the word "destroy"
    Sacrificed,      // 701.21 — NOT destruction
    Exiled,          // 701.13
    Discarded,       // 701.9 — includes the CR 514.1 cleanup discard
    Milled,          // 701.17
    Returned,        // "return to hand" / "return to the battlefield"
    PutIntoLibrary,  // top / bottom / shuffled-in; *position* is a field, not a cause

    // --- state-based actions (CR 704.5) ---
    DestroyedBySba,  // 704.5g lethal damage + 704.5h deathtouch = 701.8b ways 2 and 3
    ZeroToughness,   // 704.5f — NOT destruction
    ZeroLoyalty,     // 704.5i
    LegendRule,      // 704.5j
    AuraSba,         // 704.5m — 704.5n only unattaches, it does not move

    // --- the stack ---
    Cast,            // hand (or elsewhere) → stack
    Resolved,        // 608.2n (instant/sorcery → graveyard), 608.3a/c (permanent spell → battlefield)
    Countered,       // 701.6
    Fizzled,         // 608.2b — every target illegal; does not resolve

    // --- turn structure and special actions ---
    Drawn,           // 121.5 makes this trigger-visibly distinct from "put into hand"
    PlayedAsLand,    // 305.1 / 505.6b
}
```

Three rules for `cause`, all learned the hard way elsewhere in this tree:

- **The caller sets it.** `Primitive::Sacrifice` knows it is sacrificing;
  `perform_action` cannot recover that from `(from, to)`.
- **Nothing may branch on `cause` outside the replacement pipeline and the
  trigger matcher.** It is not a general-purpose tag; a third reader is a third
  place for it to drift.
- **No catchall variant.** No `Other`, no `Unknown`, no `#[non_exhaustive]`.
  This is the whole of what makes the enum cheap to extend later — see §11's
  "the one that blocks". A site with no honest reason to give is a site whose
  reason nobody worked out.

Type-specific death events (`CreatureDied`, `PlaneswalkerDied`,
`LegendRuleSacrificed`) become display sugar in Phase RA and stop being trigger
keys — matching keys on the zone-change event plus its LKI frame, so a
multi-type permanent matches every applicable trigger from one event.

### 3.2 `ReplacementDef`

A replacement effect is neither a `ContinuousEffect` nor a `Primitive`. It does
not apply in a layer, so it cannot be a registry row; it does not run at
resolution, so it is not an `Effect::Atom`. It gets its own type, reached
through the `Effect` variant that `types/effects.rs` has been reserving:

```rust
// types/effects.rs -- replaces the commented-out `ApplyReplacement` line
pub enum Effect {
    // ...
    /// CR 614/615. On a *static* ability, this ability generates a replacement
    /// effect and produces no layer rows (`register_static_effects` skips it,
    /// without tripping the loud-lowering assert). On a *resolving* spell or
    /// ability, this registers a shield in the replacement registry.
    Replacement(Box<ReplacementDef>),
}

pub struct ReplacementDef {
    /// Which proposed events this watches (CR 614.1, 615.1). See 3.2a.
    pub pattern: EventPattern,
    /// Which objects/players it shields -- CR 614.1's "they act like shields
    /// around whatever they're affecting". Reuses the layer system's
    /// `ObjectSet`; `SourceOnly` vs `Filter` is exactly CR 614.12's "affects
    /// only that permanent (as opposed to a general subset of permanents that
    /// includes it)".
    pub affected: ObjectSet,
    /// How it rewrites a matching event. See 3.2b -- a closed algebra, not an
    /// open taxonomy.
    pub rewrite: Rewrite,
    /// The "and also" half: CR 615.5's "the rest of the effect takes place
    /// immediately afterward", CR 701.19a's tap-and-remove-from-combat,
    /// CR 122.1c's counter removal. **This is the existing `Effect` tree** --
    /// no new vocabulary, and it is where per-mechanic variety goes.
    ///
    /// **Timing contract (§4.1a):** queued when this replacement is applied,
    /// resolved immediately AFTER the final modified event is performed --
    /// never mid-loop. Unconditional once queued (CR 615.12), fresh lineage.
    pub then: Option<Effect>,
    /// CR 616.1a-d -- which forced-choice bucket this falls in.
    pub class: ReplacementClass,
    /// How many times it can fire (CR 615.7 shields, 701.19a "next time").
    pub uses: Uses,
    /// CR 903.9b is an explicit exception to CR 614.5 and is the only one in
    /// the rules. Default `false`.
    pub exempt_from_614_5: bool,
    /// "you **may** ... instead" -- Retriever Phoenix, Library of Leng, and 14
    /// others (Scryfall 2026-08-24). The affected player is asked before the
    /// effect is applied, and declining does not consume a `Uses`. Found by the
    /// 3.2c classification pass, not by reading the CR, which is the point of
    /// running it.
    pub optional: bool,
}

/// CR 616.1a-e. `Other` is 616.1e -- free choice.
pub enum ReplacementClass { SelfReplacement, ControlChanging, CopyOnEnter, BackFaceUp, Other }

pub enum Uses {
    /// CR 614.1a static abilities, 615.10, 701.19b -- every time, forever.
    Static,
    /// CR 701.19a regeneration shield, CR 615.8 "next time [source] would deal
    /// damage" -- one application, then the effect is gone.
    Once,
    /// CR 615.7 -- "prevent the next N damage"; each point prevented decrements.
    Shield(u64),
}
```

**`Uses::CounterBacked` did not survive contact with the CR, and RB removed it.**
It was specified here as "applying removes one counter; the effect exists while
at least one remains". CR 122.1c and 122.1d state their effects verbatim, and in
both the counter removal is the *substituted event* — "instead remove a stun
counter from it" — or the CR 615.5 rider, never bookkeeping. Modelling it as a
use would have written `PermanentState.counters` from inside `consume_use`,
which is exactly the invisible-to-CR-614 write the chokepoint invariant exists
to prevent. Existence is asked at gather time instead ("does this permanent have
at least one such counter"), which is where CR 614.4 wants it asked. The CR's
"one *or more* counters create **a single** replacement effect" is handled by
the instance key — `Counter(ObjectId, CounterType, half)` — not by a count.

`Shield(u64)` survives and belongs to RD. RB ships `Uses { Static, Once }`.

```rust
// (shipped)
pub enum Uses { Static, Once }
```

**Two new open-ended enums was one too many; the count is now one.** `Rewrite`
is new and closed. `EventPattern` is a mechanical projection of `GameAction`,
not an independent taxonomy. The per-mechanic variety that would otherwise
inflate a second enum goes into `then: Option<Effect>` — the tree the engine
already has. Both growth contracts are stated below and are meant to be
enforced in review, the way the layer system's "registry membership is not
effect existence" is.

### 3.2a `EventPattern` — one arm per `GameAction` variant, and no other axis

`EventPattern` is a predicate over a proposed `GameAction`. It is data rather
than a closure for the same reason `ObjectFilter` is: closures cannot be
compared, cloned cheaply, or inspected by the loop detector.

**Growth contract: exactly one arm per `GameAction` variant, and it grows on no
other axis.** It is a projection of §3.1's table. If a card needs a pattern
`EventPattern` cannot express, the missing thing is a `GameAction` variant or a
field on one, and that is where the fix goes — because a replacement effect can
only watch for an event the engine actually proposes (§8a). A change that adds
an `EventPattern` arm without a corresponding `GameAction` change is the smell
this contract exists to catch.

Within an arm, constraints on the event's fields reuse existing vocabulary —
`ObjectFilter`, `PlayerRef`, `ZoneChangeCause`, `CardType` — rather than
inventing per-mechanic predicates. "If a **red source you control** would deal
damage to an opponent or a permanent an opponent controls" (Torbran, Thane of
Red Fell) is `And(ByColor(Red), ByController(You))` on the source, and on the
target it is **both** of CR 614.1's halves: `ObjectSet::Filter {
ByController(Opponent) }` for the permanent and `PlayerSet::Opponents` for the
player, unioned by `set_affects`. Every leaf already existed, which is what
RD-3 confirmed when it built them (§9, RD-3 as landed, decision 1).

**Naming only the object half, as this paragraph did on 2026-09-09, understates
the card and the type.** "An opponent" is not an `ObjectFilter` question at
all — RD-1's `affected_players` is what answers it, and without that field
Torbran would add 2 to damage dealt to an opponent's creatures and nothing to
the opponent, which is half a card. The same union is why `combat: None`
covers *direct* damage as well as combat: neither field constrains how the
damage arrives.

**Battles are the case where "controls" and "defends" come apart, and
`ObjectFilter` has a leaf for only the first.** CR 310.8 gives every battle a
*protector*, chosen as it enters, and 310.8b makes a Siege attackable by its
own controller — so a Siege you control while an opponent protects it is
**not** "a permanent an opponent controls", and Torbran correctly adds nothing
to damage dealt to it. That is the card's own word, not an approximation. What
battles will need is a second relation this type cannot express — "a battle an
opponent protects" — and it belongs to whoever builds CR 310 (`backlog.md`
§2.23), not to `SourcePattern`.

**The paraphrase this paragraph carried until 2026-09-09 was of a card that
does not exist.** Daunting Defender says "If a source would deal damage to a
Cleric creature you control", with no colour clause: it is CR 615.10's example
for *partial prevention*, and its predicate is entirely on the target side.
Torbran is the two-sided one.

### 3.2b `Rewrite` — a closed algebra with a checkable completeness claim

The right question about `Rewrite` is not "will it grow" but "is its
completeness checkable". It is, because CR 614 and 615 enumerate what a
replacement effect may do to an event, and the list is short:

```rust
pub enum Rewrite {
    /// CR 614.6 / 615.6 -- the event does not happen. "Prevent that damage",
    /// "skip", "instead do nothing".
    Prevent,
    /// CR 614.5's doublers, CR 615.7's partial prevention, CR 122.6a's
    /// "enters with N more". Scales or offsets the event's numeric field.
    Amount(AmountRewrite),
    /// CR 614.9 redirection, CR 616.1b control-changing. Changes who or what
    /// the event is about, leaving its kind alone.
    Retarget(RetargetSpec),
    /// CR 614.1c/d -- modify the *parameters* of an entering permanent
    /// (tapped, counters, controller, copy-of) without changing the event.
    EnterWith(EnterMods),
    /// CR 614.1a's general "instead" -- replace the event with a different
    /// proposed action. The escape hatch, and the only unbounded arm.
    Instead(GameActionTemplate),
}
```

**Only `Instead` is unbounded, and that is the CR's own shape.** CR 614.1a says
replacement effects "use the word *instead* to indicate what events will be
replaced with other events", so arbitrary event-for-event substitution is a
rule, not a design gap — and it costs nothing, because the substitute is a
`GameAction`, a vocabulary that already has to exist.

The other four arms exist because those are the cases where the replacement is
*not* a substitution, and flattening them into `Instead` would lose information
the pipeline needs:

- `Amount` has to **compose**. CR 614.5's worked example — two doublers turning
  2 damage into 8, "not just 4, and not an infinite amount" — is only
  expressible if the second doubler sees the first one's output as a number.
- `Retarget` has to survive CR 614.9's destination check: "if one of those
  permanents is no longer on the battlefield when the damage would be
  redirected … the effect does nothing." An `Instead` carrying a baked-in
  target cannot re-check that at application time.
- `EnterWith` has to **accumulate across CR 616.1f iterations** while the
  permanent does not yet exist, and be readable by the CR 614.12 look-ahead
  (§5) as "replacement effects that have already modified how it enters".
- `Prevent` is distinguishable from an `Instead` that produces nothing because
  CR 615.13 lets triggers fire on damage *being prevented*, and CR 615.12
  ("prevention effects are still applied … those effects won't prevent any
  damage, but any additional effects they have will take place") needs the
  engine to know a prevention was attempted.

A sixth arm is a claim that CR 614/615 permits an operation this list omits. It
should arrive with the rule number that says so; absent one, it belongs in
`Instead` or in `then`.

**`Instead` carries a template, not a constant.** Several cards build the
replacement out of the event they are replacing: Chatterfang ("those tokens
*plus that many* 1/1 Squirrels"), Divine Visitation ("*that many* 4/4 Angels"),
Rain of Gore ("loses *that much* life instead"), Academy Manufactor ("instead
create one of each"). So `GameActionTemplate` is a `GameAction` whose fields may
reference the incoming event's fields, the same way `AmountExpr` references
resolution context today. This is what keeps those four cards out of the engine
and in `src/cards/`.

### 3.2c Evidence: the algebra checked against every printed "would ... instead"

The completeness claim above is the kind that deserves testing rather than
asserting, so it was tested. Every card whose oracle text matches
`o:/would.*instead/ -is:funny` was pulled from Scryfall (2026-08-24) and each
matching clause classified — **561 cards, 574 clauses.** The buckets are by
*event kind watched*, because that is the axis that decides whether the design
sprawls. Reproduce with:

```bash
python plans/references/replacement-census.py
```

| | clauses | share |
|---|---|---|
| Watches an event kind already in §3.1's `GameAction` table | **549** | 95.6% |
| Needs a new `GameAction` variant | 14 | 2.4% |
| Residual — all CR 701 keyword actions (below) | 11 | 1.9% |
| **Needs a sixth `Rewrite` arm** | **0** | **0%** |

Distribution of the 549: `ZoneChange` 227, `DealDamage` 153, `DrawCard` 46,
`AddCounters` 35, `CreateTokens` 33, `GainLife` 21, `EnterBattlefield` 15,
`LoseLife` 8, `BeginStep`/skip 6, `ProduceMana` 5.

**The result to take from this is not "zero" — it is *where* the pressure went.**
It went entirely onto the `GameAction` vocabulary, which §8a already names as
the growth axis and which is bounded by the engine's own mutations plus CR 701,
not by the card pool. It did not go onto `Rewrite`, and it did not go onto
per-card engine branches.

Three cards from the review, worked through, because they are the ones that
looked like they would break it:

- **Twinflame Tyrant vs. Bloodletter of Aclazotz.** The subtle difference is
  real and it is *not* a `Rewrite` difference: Twinflame watches `DealDamage`,
  Bloodletter watches `LoseLife` "during your turn". Both are
  `Amount(Multiplier(2))`. Two famously-confusable cards, one arm, two
  `EventPattern`s — which is the taxonomy working, not straining.
- **Aether Revolt / Artist's Talent** ("as long as a permanent left the
  battlefield this turn … plus 2 instead"). The condition is not a rewrite. A
  conditional static ability's effect *exists* only while the condition holds,
  which is asked at gather time (§3.3 source 1) exactly as CR 614.4 wants —
  the same question `static_ability_still_exists` already answers for layers.
  `Amount(Plus(2))`, gated.
- **Academy Manufactor** ("if you would create a Clue, Food, or Treasure token,
  instead create one of each"). `Instead`, and it forces one data-shape
  decision: **`CreateTokens` must carry `Vec<TokenDef>`, not `(TokenDef, n)`**,
  or Manufactor, Chatterfang and Divine Visitation each need a special case.
  Recorded in §3.1.

The one genuine pressure point the pass found is **`AmountRewrite`, not
`Rewrite`**: the Ali from Cairo family ("damage that would reduce your life
total to less than 1 reduces it to 1 instead", 8 cards) is a *clamp against
player state*, not a scale or an offset. It is one `AmountRewrite` variant with
8 customers and a clear rule shape, and it is named here so it is budgeted
rather than discovered.

### 3.2d Multiplicity lives in a field, not in a set of events

```rust
/// CR 614.6 -- either the event happens in modified form, or it never happens.
pub type ReplacementOutcome = Option<GameAction>;
```

**Read that as a claim about the rewrite's arity, not about how many cards get
drawn.** A replacement absolutely can multiply what happens — Teferi's Ageless
Insight turns one draw into two cards in hand, and pretending otherwise would be
silly. The claim is narrower and load-bearing: *the multiplicity is carried in a
**field of one event**, never in a set of events*, so `Rewrite` maps one
`GameAction` to at most one `GameAction`.

That is not an encoding trick to dodge fan-out. `DrawCards { n }` is an object
the rules themselves talk about, and Alms Collector cannot be written without it.
Its printed ruling gives the test verbatim:

> "To determine whether a player is instructed to draw multiple **once** or
> instructed **multiple times to draw one card**, count how many times the word
> 'draw' is used. Alms Collector's replacement effect watches for one 'draw'
> that instructs a player to draw multiple cards."

So "draw two cards" is one event with `n = 2`, and two separate "draw a card"
instructions are two events. Teferi's says "draw two cards instead" — one
"draw" — so its output is a single `DrawCards { n: 2 }`. `Instead(DrawCards{2})`
is the faithful encoding, and `Split([DrawCard, DrawCard])` would be the wrong
one: it skips the instruction level, and an opponent's Alms Collector would then
have nothing to match.

That the instruction-level event is visible to *other* replacements is also
printed, and it is CR 616.2 ("a replacement effect can become applicable as the
result of another replacement effect that modifies the event"):

> "Once a replacement effect has been applied to an event, it can't be applied
> again to the resulting events. For example, once Alms Collector's replacement
> effect has modified the effect of a player's Divination, **Thought Reflection
> can double that player's resulting card draw** without Alms Collector's
> replacement effect applying again."

**Where heterogeneous multiplicity goes.** When a replacement really does cause
two unlike things — Alms Collector's "you *and* that player each draw a card",
Notion Thief's "that player skips that draw *and* you draw a card" — the split is
between the replaced event and the `then` half, not between two rewrite outputs.
Notion Thief is `Prevent` + `then:` you draw one; its ruling confirms the
boundary is real, since "that opponent still discards a card" if the original
instruction was draw-then-discard. Homogeneous multiplicity is a count field;
heterogeneous multiplicity is `then`. Between them, nothing needs a fan-out.

**Corrected 2026-09-11 (RE's sizing): Notion Thief is not the `then` case.**
Its 2018-03-16 ruling walks two Thieves in a two-player game — the drawing
player applies one, "then the player whose Notion Thief's effect was chosen
repeats this process among the remaining", each "applied to the card draw only
once", and "it really will be that player who draws a card". A rider's draw is
a fresh proposal with a fresh applied set, so two Thieves as `Prevent` +
`then` trade the draw forever. The Thief's draw is **the same event with a new
subject** — CR 614.5's "modified events that may replace that event" — and so
`Instead(DrawCards { n: 1, player: Some(You) })`, keeping the lineage. Alms
Collector is **half** the `then` case, which the correction above got wrong and
RE-2's own tests caught two paragraphs later.

**Corrected again 2026-09-11 (building RE-2): a rider does not carry the
lineage, so whatever has to carry it belongs in the rewrite.** Alms Collector
was filed as `Prevent` plus two riders on the reading that "you and that player
each draw a card" is two unlike things. Its second ruling refuses that: *"once
Alms Collector's replacement effect has modified the effect of a player's
Divination, Thought Reflection can double that player's resulting card draw
**without Alms Collector's replacement effect applying again**."* CR 614.5's own
words are the test — an effect gets one opportunity to affect "an event **or any
modified events that may replace that event**" — and only the rewrite's output
is such a modified event. As riders the board is not a wrong number but an
infinite loop: the rider's one draw is doubled back to two, Alms applies again,
and the two effects trade cards until CR 104.4b calls the game a draw. *(Corrected
at RE-4's review, 2026-09-13: that loop was the engine's, not the rules' —
a rider's proposals carry the replaced event's applied set now, §11 item 77,
so both encodings terminate; `Instead` stays because the drawing player's one
draw is the same event as the instruction it replaced.)* Measured
as a stack overflow the first time the test ran.

**The rule, and it is the one both "and" cards needed.** Split a printed
"instead X and Y" by asking which half CR 614.5 has to cover; that half is the
rewrite, and only what is left over is `then`. Notion Thief's is the draw with a
new subject; Alms Collector's is the same draw with a smaller count, which is
this section's own homogeneous case hiding inside a heterogeneous sentence. Each
card's own ruling names the loop the other filing produces, and neither card's
remainder needs the lineage: "you draw a card" is a different player's draw and
nothing watches it twice. **What this does not yet answer** is a card whose
second half needs the lineage *and* cannot be folded into the rewrite — one
event, two subjects, both under CR 614.5. Nothing in RE prints it; the day one
does, `ReplacementOutcome`'s arity is the thing that has to move, not `then`'s
timing. §9, RE decision 1; §11 item 53.

**Honest note on why `Split` went.** It was first removed because the question
"does a fanned-out branch inherit the CR 614.5 applied-set" had no answer. The
lineage rule below now answers that question either way, so that reason has
expired. `Option` survives on faithfulness — the instruction-level event is real
and `Split` would erase it — not on dodging the hard case.

This was drafted with a third `Split(Vec<GameAction>)` variant justified by
Doubling Season. **That was wrong, and the correction matters structurally.**
Doubling Season reads "If an effect would create one or more tokens under your
control, it creates twice that many of those tokens **instead**" (verified on
Scryfall, 2026-08-24) — one token-creation event whose *count* changes, which
is `Rewrite::Amount`. Its second ability does the same for counters.

A first revision of this section claimed "nothing in CR 614 turns one event into
several", and **that was too strong** — flagged in review with Teferi's Ageless
Insight, which is right. `Rewrite` yields at most one action, but *performing*
one can produce several events: CR 121.2 says an instruction to draw N is
carried out as N individual draws. So the question dropping `Split` appeared to
retire — does a derived event inherit the CR 614.5 applied-set? — is alive,
load-bearing, and has a determinate answer.

**The rule: the applied-set follows an event's *lineage*. Decomposition
continues a lineage; containment starts a new one.**

Two printed cards pin it down, and they disagree about which case they are:

- **Teferi's Ageless Insight** — "If you would draw a card … draw two cards
  instead." The printed ruling: with two copies "each card that player would
  draw after the first will result in **four** cards being drawn. If they
  control three, they draw **eight**." Trace it: `DrawCard` → T1 →
  `DrawCards{2}`, applied `{T1}`; the performer decomposes into two `DrawCard`
  events, **each inheriting `{T1}`**; each meets T2 → `DrawCards{2}` → four
  draws inheriting `{T1,T2}`; T3 makes eight. Exactly 2ⁿ. **Without
  inheritance, T1 re-applies to its own output and the game hangs** — so this is
  not a nicety, it is the termination argument. The two draws are 614.5's
  "modified events that may replace that event": same lineage.
- **CR 616.1g's own example** — Doubling Season creates two Voice of All tokens,
  and the rule says "the effects of the two Voice of All tokens may be applied
  in either order", i.e. each token's ETB replacement is a fresh choice. A
  token's entering is a *consequence* of the creation event, not a modified form
  of it: new lineage.

The discriminator is whether the derived event is the same kind of thing as its
parent. `DrawCards{2}` → 2 × `DrawCard` is one event expressed at finer grain.
`CreateTokens` → `EnterBattlefield` is a different event that the first one
caused. `perform_action` knows which it is emitting, so the lineage tag rides on
the call rather than being inferred.

**Doubling Season is the contrast test, and it reaches 2ⁿ by the other route.**
Its printed ruling — "two Doubling Seasons … four times the original number,
three … eight times" — is `Amount(Multiplier(2))` composing *within a single event's*
CR 616.1f loop, no decomposition anywhere. Two mechanisms, same arithmetic. Both
belong in the suite: `test_two_teferis_draw_four_not_infinity` and
`test_two_doubling_seasons_quadruple`, and the first one hangs rather than fails
if the lineage rule is wrong, so give it a bounded iteration guard.

`then` resolves through the existing `resolve_effect` path, not new machinery —
but **after** the pipeline returns, never inside `Rewrite::apply` (§4.1a).
`Rewrite::apply` itself still needs game state and a `DecisionProvider` for
`GameActionTemplate` evaluation and optional-effect prompts.

### 3.3 Where replacement effects come from

Five sources, and the pipeline gathers from all five. Getting this list wrong is
the failure mode that shows up as a card silently doing nothing:

1. **Static abilities of permanents.** Discovered by sweeping
   `battlefield_ids_ordered()` and reading each object's **effective** ability
   list (`oracle::characteristics::get_effective_abilities`). Not a registry
   scan. This is not a shortcut — it is what makes Humility and Blood Moon
   strip a replacement ability for free, and it is CR 614.4's "must exist before
   the event" asked at the one instant that matters.
2. **Static abilities functioning in other zones** (CR 113.6). The same read
   off a different candidate list — **RF, 2026-09-16** (§9): the objects
   whose printed ability functions where they are
   (`GameState::zone_replacement_ability_sources`), plus the zones a Layer 6
   grant or a copy row can reach, in CR 613.7d timestamp order, each ability
   asked `zone_function::functions_in` of the zone its object is in. Not a
   walk of the zones: a library is ~60 objects a seat and a gather runs
   ~2,300 times a game.
3. **Continuous effects with a duration, from resolutions.** CR 614.3 — "Prevent
   all damage that would be dealt this turn". These go in the replacement
   registry with a `Duration`, expiring through the same cleanup/turn-start
   hooks `ContinuousEffectRegistry` already uses.
4. **Shields from resolutions.** CR 615.7/615.8, CR 701.19a regeneration. Also
   the registry, with `Uses::Once` or `Uses::NextDamage(n)` (named `Shield(n)`
   until RD's design check; §9 says why it moved, and `plans/glossary.md`
   under *shield* separates the three things that word names).
5. **Counters.** CR 122.1c (shield), 122.1d (stun), 122.1h (finality). These
   come from the *counter*, not from any ability — nothing on the card says so.
   Synthesized during the sweep from `PermanentState.counters`.

Source 5 is why Phase RB can ship a working pipeline with **zero new card-text
machinery**: three counter types, 164 printed cards, and they exercise untap
replacement, destroy replacement, damage prevention, and zone-change replacement
between them.

### 3.4 `ReplacementEffectRegistry`

Sources 3 and 4 need storage. Model it on `ContinuousEffectRegistry`
(`state/continuous_effects.rs`) — same duration-expiry hooks, same
`remove_by_source`, same "recompute the summary by full walk rather than
maintaining incremental counters" discipline, and for the same stated reason:
a drifting counter shows up as a silently skipped effect.

It differs in two ways, both because replacement effects are not layered:

- **No `Layer`, no timestamp ordering.** CR 616.1 orders by *player choice*, not
  by timestamp. There is no analogue of `effects_in_layer`.
- **No per-layer existence re-check.** CR 604.2's re-check exists because a
  layer walk asks the same question nine times. A replacement effect is asked
  once, at the instant the event is proposed, which is when CR 614.4 wants it
  asked. Source 1's discovery-from-effective-abilities gives the same guarantee
  structurally.

---

## 4. The pipeline

### 4.1 The CR 616.1 loop

`Rewrite` maps one `GameAction` to at most one `GameAction` — multiplicity rides
in a *field* of the event, not in a set of them (§3.2d). Performing that event
can still produce several: `perform_action` proposes the derived events, and each
re-enters here carrying the parent's applied-set if it is a **decomposition**
(`DrawCards{2}` → two `DrawCard`s) and a fresh one if it is a **contained** event
of a different kind (`CreateTokens` → `EnterBattlefield`). That parameter is the
whole of §3.2d's lineage rule, and the reason two Teferi's Ageless Insights draw
four cards instead of hanging.

```
fn apply_replacements(game, action, ctx, inherited, riders) -> Option<GameAction>
    # `riders` collects every applied effect's `then` half, in application
    # order. The caller resolves them AFTER performing the returned event
    # (and even when the return is None) -- §4.1a.
    applied:  HashSet<ReplacementInstanceId> = inherited  # 3.2d lineage
    declined: HashSet<ReplacementInstanceId> = {}        # nothing is exempt
    ev = action

    loop:                                                # CR 616.1f
        cands = gather(game, ev)                         # 3.3, five sources
                  .filter(applies_to(ev))                # watches() && affects()
                  .filter(|c| c.exempt_from_614_5        # CR 903.9b
                              || !applied.contains(c.instance))   # CR 614.5
                  .filter(|c| !declined.contains(c.instance))     # see below
        if cands.is_empty(): return Some(ev)

        choosable = must_choose_among(cands)             # CR 616.1a -> b -> c -> d -> e
        chooser = affected_chooser(game, ev)             # CR 616.1 / 400.6
        chosen  = if bucket.len() == 1 { bucket[0] }
                  else { ask_choose_replacement(dp, chooser, bucket) }

        if chosen.optional && !ask_apply(dp, chooser, chosen):
            applied.insert(chosen.instance)              # opportunity taken (CR 614.5)
            declined.insert(chosen.instance)             # ... and it is final
            continue                                     # ... but no `consume_use`
        if !chosen.exempt_from_614_5: applied.insert(chosen.instance)
        chosen.consume_use(game)                         # Uses::Once (Shield: RD)

        if chosen.then is Some(t): riders.push(t)        # queued, NOT resolved (§4.1a)
        match chosen.rewrite.apply(game, ev, ctx)?:      # Rewrite only
            None     => return None                      # CR 614.6 -- never happens
                                                         # (queued riders still run)
            Some(e)  => ev = e                           # CR 616.1f -- re-gather
```

Six things this encodes, each with its rule:

- **CR 614.4** — `gather` runs against live state at the moment of proposal.
  There is no "go back in time" path because there is no other place to ask.
- **CR 614.5** — the `applied` set is keyed on the *effect instance*, and it
  follows the event through every modification *and into its decomposition*,
  which is what 614.5's "an event or any modified events that may replace that
  event" describes (§3.2d). `exempt_from_614_5` exists for exactly one rule
  (903.9b) and must not grow a second user without a CR cite.
- **CR 616.1a–e** — `must_choose_among` returns the first non-empty step
  and only that class; 616.1e is the fallthrough.
- **CR 616.1f** — the loop re-gathers after every application, so an effect made
  newly applicable by the modification is picked up (CR 616.2).
- **CR 616.1g** — nothing here sequences outer against inner, and nothing needs
  to. A derived event does not exist until the outer one is *performed*, and
  this function returns before that happens — so "the second effect can't be
  chosen until after the first effect has been chosen" is a consequence of the
  call order rather than a rule the loop enforces. Doubling Season's count is
  fixed before either Voice of All token has an ETB event to replace (fresh
  lineage, so both tokens choose freely — 616.1g's own example); CR 121.2a's
  "draw N" is fixed before any individual draw exists (same lineage, so a draw
  replacement that already fired does not fire again).
- **CR 614.6/614.7** — `None` drops the event, and CR 614.7a's zero damage
  never reaches a candidate: `never_happens` is asked at the top of every
  iteration, ahead of even the "can't" check, because there is no event there to
  forbid. It is re-asked per iteration rather than once, since CR 616.1f can
  rewrite an event into one. **This check used to live in `perform_action`,
  which runs after this function** — so a 0-damage proposal traversed the whole
  loop, and `EventPattern::DealDamage` carries no amount constraint, so a shield
  counter's prevention half applied to it and its CR 615.5 rider spent a counter
  on an event 120.8 says never happened (`rb-review.md` H1). Life gain is here
  on CR 119.10's own words; life loss and 0-count counter changes have no such
  rule and keep their no-op guards in `perform_action`.

**Declining an optional replacement marks it applied but does not consume a
use.** Both halves are load-bearing. Marking it applied is CR 614.5's "one
opportunity" — being offered and refusing *is* the opportunity — and without it
the `continue` re-gathers the same candidate forever, which is a hang rather
than a wrong answer. Not consuming a use is what leaves Retriever Phoenix's
ability and a regeneration shield intact for the *next* event. Static-ability
optionals (Library of Leng) have no use to consume either way.

**The applied set alone is not enough, and RB found that out by hanging.**
CR 903.9b is `exempt_from_614_5`, which means the applied set does not filter
it — that is the whole of the exception — *and* it is optional. Put those two
together with the decline path above and the loop re-offers the same declined
choice forever: the mark is there and the filter ignores it. A hang, not a wrong
answer, which is the worst shape of bug and exactly the one this section warned
about for the other reason. Declining is therefore tracked in a **second set**
that nothing is exempt from. The two are genuinely different questions:
CR 614.5 is about *applying* more than once and 903.9b's exception is to that,
while a decline is a final answer about this event and no rule exempts anything
from it. `tests/phase_rb_integration_test.rs::test_declining_903_9b_terminates_despite_the_614_5_exemption`
is the regression.

One caveat
recorded at audit (2026-08-25): mark-on-decline is a *reading* of CR 614.5, not
a cited ruling — the corner is a declined optional whose event a later
replacement then modifies into something the player now wants to replace after
all. Check printed rulings when the first optional card lands; if one
contradicts this, it changes the loop, not the types.

**`ask_choose_replacement` is called only when `bucket.len() >= 2`.** That is
CR-correct (there is no choice to make with one candidate), it is what keeps the
existing `ScriptedDecisionProvider` tests green rather than drowning them in
unexpected prompts, and it is what loop-detection Tier 1 counts as "forced".

### 4.1a When `then` runs — and the other "then", which never enters the pipeline

Researched 2026-08-25, because the first draft resolved `then` inside
`chosen.apply`, mid-loop — under-specified on sequencing and on survival. Two
different "then"s have to be separated first, because they answer to different
rules.

**One: the rider on a replacement effect** — Kalitas's "and create a 2/2
Zombie", Notion Thief's "and you draw a card", regeneration's
tap-and-remove-from-combat. This is `ReplacementDef.then`, and CR 615.5 states
its timing outright: the prevention takes place at the time the original event
would have happened, and the rest of the effect takes place *immediately
afterward*. Three consequences, each load-bearing:

- **Queue at application, resolve after the performed event.** During the
  CR 616.1f loop nothing has happened yet — the loop is deciding what the event
  *is*. A rider resolved mid-loop runs before the event it rides on, which is
  observably wrong the moment triggers land: Kalitas's Zombie would enter the
  battlefield before the creature it replaced has left it, and the LKI frame
  order inverts. So `execute_action` performs the surviving event, emits its
  `GameEvent`, then resolves the queued riders in application order.
- **Unconditional once queued.** CR 615.12: prevention effects applied to
  unpreventable damage prevent nothing, "but any additional effects they have
  will take place." A rider belongs to the *application* of its replacement,
  not to the survival of the event — a later replacement in the same loop
  further modifying or even dropping the event does not un-queue an earlier
  rider.
- **A rider carries the replaced event's applied set** — corrected at RE-4's
  review (2026-09-13, §11 item 77); until then this bullet said the opposite.
  A rider's actions are the *rest of the replacement's effect*, which CR 614.5
  names in as many words — "any modified events that may replace that event"
  — and Alms Collector's ruling applies to its "you draw a card" half exactly
  as to the halved draw: an effect already applied "can't be applied again to
  the resulting events". So `Rider::lineage` is the group's final applied set,
  and the rider's proposals start from it. Both directions still hold: Kalitas
  plus Doubling Season makes two Zombies, because the Season never applied to
  the death; and a second Reflection on the far side of a Collector doubles
  the rider's draw once, because it had not applied yet. What a rider does
  *not* hand down is its lineage to events nested *inside* its proposals — an
  entry inside a creation it makes — which are contained and start fresh.
  The fresh-set reading was RE-2's, and it was not theoretical: two
  Reflections and two Collectors across two seats handed one draw back and
  forth until the stack overflowed (RE-4's A/B, seed 12523).

**Two: "A, then B" in card text** — Goggles of Night: "Whenever equipped
creature deals combat damage to a player, scry 1, then draw a card." This
"then" is CR 608.2c instruction sequencing inside a resolution and **never
reaches the pipeline as a unit**: each instruction proposes its own events, one
at a time, and a replacement rewriting instruction A's event touches nothing
about instruction B.

Worked through, because it pins two behaviors at once — Goggles of Night
triggers while you control Eligeth, Crossroads Augur ("If you would scry a
number of cards, draw that many cards instead"; mandatory, no "may"; the `Scry`
event kind is on §8a's known-missing list):

1. Instruction 1 proposes `Scry { n: 1 }`. Eligeth rewrites it —
   `Instead(DrawCards { n: 1 })`, an `Instead` that **changes the event's
   kind**, which the arm permits. The draw happens *at that point in the
   resolution* (CR 614.6 — the modified event occurs in the original's place).
2. Instruction 2 proposes `DrawCards { n: 1 }` as its own event, and resolves
   normally.

Net: **draw two cards, zero scrys.** "Whenever you scry" triggers never fire
(CR 614.6 — a replaced event never happens); draw-watchers see two separate
one-card instructions, neither of which is "draw two or more" (§3.2d's Alms
Collector granularity). The substituted draw keeps the original event's
applied-set — that is the loop's own mechanics (`ev = e` under one `applied`
set, kind change or not), so Eligeth cannot re-apply, while a draw-watching
replacement such as Teferi's Ageless Insight applies fresh and doubles it —
CR 616.2's Alms-Collector-then-Thought-Reflection shape with the event kind
having changed in between.

If B references A's outcome ("…then put one of the cards you looked at into
your hand"), CR 614.6's second sentence answers it: a modified event may
contain instructions that can't be carried out, and an impossible instruction
is simply ignored. B does as much as it can against what actually happened.

**What this costs the engine: nothing new.** `Effect::Sequence` already
resolves atoms in order and each atom already proposes independently. What it
*pins* is that `Instead` may change the event kind with the applied-set carried
across, and that `apply_replacements` returns riders instead of resolving them.
Tests in §10.

### 4.2 Batches and simultaneity

Three rules need an event *set*, not an event:

- **CR 704.3** — "performs all applicable state-based actions **simultaneously
  as a single event**". `engine/sba.rs` performed them one `change_zone` at a
  time, and worse, evaluated 704.5g's conditions only after 704.5f's moves had
  happened.
- **CR 704.7** — "if multiple state-based actions would have the same result at
  the same time, a single replacement effect will replace all of them."
  Unreachable without the batch.
- **CR 615.7** — "if damage would be dealt … by two or more applicable sources
  at the same time, the … controller chooses which damage the shield prevents."
  `apply_combat_damage` looped one assignment at a time, so this choice could
  not exist.

So `execute_action` gains a batch sibling (shipped RA-3):

```rust
pub fn execute_actions(&mut self, batch: Vec<GameAction>, ctx: &ActionContext)
    -> Result<(), String>;
```

**The return type is `()` rather than the `Vec<GameAction>` this section first
specified.** The performed-action vector has no consumer in RA and the project
does not add one speculatively; RB restores it the moment `apply_replacements`
gives it a customer. Nothing else about the signature moved.

The batch shares one batch id (which lands on every emitted `GameEvent`,
satisfying the CR 603.2c "one or more" trigger requirement that Phase 7
will need). `execute_action` becomes `execute_actions(vec![action])`.

**A nested `execute_actions` joins the enclosing batch rather than opening its
own** (decided in RA-3, and it is a rules point, not an implementation
convenience). CR 120.3f makes lifelink's life gain a *result of* the damage, and
CR 120.4c/d process the results and then let the one damage event occur — so the
gain is not merely simultaneous with the damage, it is part of the same event.
Lifelink proposes from inside `perform_action(DealDamage)`, and a second batch id
there would tell a CR 603.2c trigger that two events happened. RD's CR 120.3
decomposition is the same shape.

**One exception, and it is the converse of that argument** (RC-5, 2026-09-03).
CR 614.13's auxiliary zone changes are *not* a result of the entry: they are
performed while the entry is still being decided, in phase 1, before the entry
event exists at all. Joining would put a devoured creature's death and the entry
it paid for into one CR 603.2c event. `execute_actions_new_batch` opens a
genuinely fresh id; it has one caller, tagged `// AUXILIARY-MOVE:`, and a second
caller needs the rule that says its events are not a result of the enclosing one.

**A fresh id is right and one-per-application is wrong, corrected on review the
same day.** Thunder-Thrash Elder's own ruling (2008-10-01) says "all creatures
devoured this way are sacrificed at the same time" when several devour creatures
enter together — so the target is one batch per *entry event*, not one per
`apply_auxiliary_move` call. That is a **deferral**, not a relabelling: the moves
would have to be collected across phase 1 and performed once, which collides
with counting what was performed (§10's finding 5). Unreachable from the
registered pool, sized as `codebase-state.md` item 61.

**Each batch member keeps its own `applied` set.** A first draft had the batch
share one, and that is wrong: CR 614.5 is per *event*, and batch members are
separate events. Kalitas's own ruling pins it — when Kalitas dies at the same
time as several opponent creatures, every one of those cards is exiled and each
makes a Zombie: one static replacement, applied once *per death*. Under a
shared set the first death would consume Kalitas's application and the rest
would go to the graveyard. (Wrath of God under Leyline of the Void is the same
shape: all five zone changes get replaced, not one.)

**Corrected by RD-2 (2026-09-09): the set is per `(batch, subject)`, not per
member.** Kalitas's ruling holds either way — N deaths are N subjects — and it
was CR 122.1c's ruling the per-member shape got wrong: two blockers hitting one
creature with two shield counters spent two, where "only one shield counter is
removed". `execute_batch_inner` now buckets the APNAP-ordered members by
`subject_of` and `apply_replacements` runs one CR 616.1 loop per group, with one
applied set and one chooser; an instance applies to every member of the group
it applies to, its rider is queued once with the members' amounts summed, its
use spent once. The batch, not the turn, is the scope — CR 510.4's two combat
damage steps are two batches and spend two counters. §9's RD decision 3 and §11
items 15 and 24 carry the argument; `plans/traces/rd-2-a-decision-is-per-subject.html`
walks the boards read by read.

What a shared set was reaching for is **CR 704.7**, and that rule is a
*same-result collapse*, not a cross-member share: multiple state-based actions
with the same result at the same time (a player who would lose the game for
both life and poison) merge into **one** event before the pipeline runs, and
that one event has one applied-set. Implement 704.7 as a dedupe step on the
batch, upstream of `apply_replacements`.

`Uses` needs no batch special-casing either way: `consume_use` writes game
state, so a regeneration shield spent on one group of a batch is correctly gone
when the next group asks (CR 701.19a — one shield, one destruction replaced).
Since RD-2 it is spent *after* the application, by what the application did
(RD decision 7): `Once` when the rewrite took effect, `NextDamage` by the
damage prevented.

Callers that must batch: `apply_combat_damage` (CR 510.2), the SBA sweep
(704.3), **the untap step (CR 502.1 — "all the permanents untap
simultaneously", which this list originally missed)**, and **a spell's actions
over several objects (CR 608.2f — "in most cases, each such action is processed
simultaneously"), which is what a board wipe is**. All four batch as of
2026-08-26; `Primitive::Destroy` was the last loop and was converted in review,
since RA-3's ticket named only two of them.

That last one is where the pipeline earns the batch most visibly. Kalitas,
Traitor of Ghet applies once *per death* in a wipe — CR 614.5 is per event and
batch members are separate events — but the CR 616.1 loop can only reach that
answer if the deaths arrive as one proposal set. A loop of `execute_action`
hands it N unrelated events and the question never comes up.

**The dedupe is where CR 704.7 lives, and RA-3 implemented it as one action per
object.** Two state-based actions that would put the same permanent into the
same graveyard at the same time have the same *result*, so they are one event
with one applied-set. The first condition in CR order names the cause: a creature
that is both a duplicate legend and dead to lethal damage was destroyed
(704.5g), not put away by the legend rule. Note this is the *within-batch*
collapse; §4.2's opening paragraph on per-member applied sets is about distinct
events and still holds.

### 4.3 N players, from day one

Per `CLAUDE.md`: write it N-player-shaped or pay to retrofit it.

- **CR 616.1's chooser** is "the affected object's controller (or its owner if
  it has no controller) or the affected player" — a lookup, already N-safe.
- **CR 616.1 + 101.4** — "if two or more players have to make these choices at
  the same time, choices are made in APNAP order". A batch whose events affect
  several players produces several choosers; they are ordered
  active-player-first, then turn order. `PlayerId` vectors, never a `bool`.
- **CR 101.4d** — if a nonactive player's choice forces an earlier player to
  choose again, APNAP restarts for all outstanding choices. Implement the
  restart; do not assume one pass.

### 4.4 What is *not* a replacement effect

> **Owned elsewhere as of 2026-08-27.** `plans/cant-effects-architecture.md` is
> the authority for "can't" effects: CR 614.17 is one of six enforcement points
> it measures, and the two bullets below are the only ones that touch this
> pipeline. Everything else a "can't" does happens before `execute_action` is
> ever reached.

CR 614.17: **"can't" effects follow similar rules but are not replacement
effects.** They are checked before the pipeline and they win (CR 101.2). Two
consequences for this design:

- Indestructible (CR 702.12b) is a "can't", not a replacement. The check that
  lives in `Primitive::Destroy` today moves into the `Destroy` action's
  performer, ahead of the pipeline — not into a `ReplacementDef`.
- **CR 614.17c** — an event that can't happen is replaceable *only* by a
  self-replacement effect that changes the event's type. So the pipeline needs a
  `blocked: bool` on the proposal, and when set, `gather` returns only
  `ReplacementClass::SelfReplacement` candidates.

---

## 5. The look-ahead frame (CR 614.12, 614.17d)

This is the single largest technical risk in the phase, and the reason Phase RC
is split in two.

CR 614.12 says: to decide which ETB replacements apply and how, check the
permanent's characteristics **as it would exist on the battlefield**, taking
into account (1) replacements that already modified how it enters, (2)
continuous effects from *its own* static abilities that would apply to it once
on the battlefield, and (3) continuous effects that already exist and would
apply to it. CR 614.17d says the same for ETB "can't" effects.

The engine cannot answer that today, and it is not one gap but three:

| Clause | Blocker in `compute.rs` |
|---|---|
| (1) pending mods | Nothing carries them; `place_on_battlefield` is where "tapped" would be decided, after the fact |
| (2) own statics | `register_static_effects` runs *inside* `place_on_battlefield`, so the object's own rows do not exist yet |
| (3) existing effects | `effect_applies_to` hard-requires `game.battlefield.contains_key(&id)` for `ObjectSet::Filter` — an entering object matches no filter-based effect at all |

The fix is a **hypothetical overlay**: a read-side indirection through which
`compute_to_ceiling` computes the entering object as a permanent under the
proposed controller — with the registry rows its own statics would generate, and
with the pending mods applied.

**But the overlay is a hypothetical about one object's characteristics, not
about the game state, and getting that boundary wrong is the failure this
section exists to prevent.** See §5a. (An earlier draft framed this as
*"as a permanent" is narrower than "on the battlefield"*. That was wrong and is
struck — CR 110.1 makes the two coextensive, so no boundary can run between
them. The real boundary is applicability vs. enumeration.)

```rust
/// CR 614.12 / 614.17d — `id`'s characteristics as it *would exist* on the
/// battlefield under `controller`, with `pending` already applied.
pub fn compute_as_entering(
    game: &GameState,
    id: ObjectId,
    controller: PlayerId,
    pending: &EnterMods,
) -> Option<EffectiveCharacteristics>;
```

**Three notes that decide how to build it.**

- **Self does not mean self-affecting.** CR 614.12's own Orb of Dreams example:
  a permanent's *replacement* effect applies to itself only if it "affects only
  that permanent", i.e. `ObjectSet::SourceOnly` — a filter-based one
  ("Permanents enter tapped") does not. But clause (2) puts **no such
  restriction on the characteristics computation**: an entering creature with
  "Creatures you control get +1/+1" does get its own anthem in the look-ahead
  frame. The self-only test belongs to the replacement's applicability, not to
  the frame. Two different questions, one rule number.

- **Build a read-side view, not a `GameState` clone.** `compute.rs` reaches for
  concrete state in more places than any audit of it counted — five on
  2026-08-24, eight on 2026-09-02, and **nine when RC-4 was built**: the seed's
  controller and CR 302.6 clock, the entity's counters, the registry slice,
  the zone gate, `base_controller`'s four probes, and the two `objects` reads
  the filter leaves make for tokenness and ownership. The count kept drifting
  because it was the wrong thing to count. What a hypothetical has to perturb
  is *kinds* of read, and there are four: the seed, the counters, the rows and
  the gate. The `objects` reads never change — the entering object exists in
  the store — and `base_controller` is perturbed only through the seed. RC-4
  routed exactly those four through one accessor pair on `FrameCache`
  (`entity` and `rows_in_layer`; `engine/layers/lookahead.rs`) and touched
  nothing else in the walk. A `GameState` clone would work here on budget
  grounds — §11 item 5 prices both call sites — but is the wrong instrument:
  it duplicates the object store and `GameState.rng`, the latter against the
  determinism doctrine outright, and it produces a second live copy of every
  `ObjectId`, which is a v4 UUID and therefore *aliased* rather than
  distinguishable.

- **CR 613.8 shares the seam, not the overlay** (corrected 2026-08-24 in
  review). The first draft claimed the two were "the same machinery". They are
  not, and the difference is worth stating because it changes what gets built:

  | | CR 614.12 look-ahead | CR 613.8 step-4 hypothetical |
  |---|---|---|
  | What is perturbed | battlefield membership, controller, the entering object's own registry rows, pending `EnterMods` | one more `EffectModification` applied to a frame |
  | What is re-evaluated | the whole layer walk for one object | `object_matches_filter(A.filter, chars, …)` |
  | Cost of the perturbation | game-state-shaped | `EffectiveCharacteristics` clone — measured 0.37 → 0.27 µs over N=10–80, i.e. flat (`layers-architecture.md` §12) |
  | Frequency | once per entering permanent | up to O(effects²) per layer, inside a per-permanent walk |

  613.8's check is **frame-level**. "Recompute A's `affected_objects` with B applied" is
  a clone of `chars`, one `EffectModification` applied to it, and one
  `object_matches_filter` call — it never asks whether an object is on the
  battlefield differently than it already is. It does not need the overlay and
  must not be built on one, because a game-state-shaped perturbation inside an
  O(effects²) loop inside a per-permanent walk is the cubic this project has
  spent two phases avoiding.

  What 613.8 *does* inherit is the accessor pair, so it never has to re-plumb
  `compute.rs`. And the seam has to be general on principle, which is the review
  question that prompted this correction: dependency (CR 613.8a) can be created
  or destroyed by a control change, a counter, an ability grant, a duration
  expiry, or a CR 305.7 strip — entering the battlefield is one cause among
  many. So the accessor is parameterized by *what it returns*, not by "is this
  object entering"; the ETB case is one caller supplying one perturbation.

### 5a. Visible to filters, invisible to counts (the Thassa boundary)

Review raised the Theros gods, correctly, as the case where this gets
counter-intuitive. It does, and the printed rulings are unambiguous. Thassa, God
of the Sea reads "As long as your devotion to blue is less than five, Thassa
isn't a creature", and the two rulings that matter say opposite-looking things:

> "As a God enters the battlefield, your devotion to its color will determine
> whether any replacement effects that affect creatures entering the battlefield
> apply to that God. **Because replacement effects are considered before the God
> is on the battlefield, the mana symbols in its mana cost won't be counted when
> determining this.**"

> "When a God enters the battlefield, your devotion to its color (**including
> the mana symbols in the mana cost of the God itself**) will determine if a
> creature entered the battlefield or not for abilities that trigger whenever a
> creature enters the battlefield."

So with Authority of the Consuls out and devotion at four, an entering Thassa is
**not** a creature for the ETB-replacement check (she does not count her own
`{U}`), enters untapped — and *is* a creature a moment later for the trigger
check. Same permanent, two answers, one instant apart.

**This is not a judge fudge, and it does not need special-casing.** CR 614.12
asks for "the characteristics of the permanent as it would exist on the
battlefield" — the characteristics *of the object*. Devotion is a property of
the **player**, computed from the permanents they control, and the entering
object is not yet one of them. The gods' own type-changing ability is consulted
(that is clause 2 working); what differs is the board state it reads.

**The rule that falls out, and the correction it forces:**

> The overlay makes the entering object visible to **effect applicability** —
> "does this filter match it" — and leaves it invisible to **enumeration** —
> "which permanents exist".

The first draft of this section said the overlay "sees a battlefield that
contains the entering object". Taken literally that is **wrong**: it would put
Thassa's own `{U}` into her devotion count and enter her tapped under Authority
of the Consuls, against the printed ruling. The two reads are different
questions that today happen to share one call, and separating them is the
overlay's real content:

| Read | Sees the entering object? | Sites |
|---|---|---|
| **Applicability** — is this object a permanent that a filter can match? | **yes** | `effect_applies_to`'s membership gate (`compute.rs:623`) |
| **Frame seed** — controller, counters, control-since-turn | **yes**, from the proposed values | `compute.rs:113`, `:180`, `:408` |
| **Registry slice** — which rows exist | **yes**, plus the object's own would-be rows | `compute.rs:242` |
| **Enumeration** — which permanents does a player control? | **no** | `battlefield_ordered` / `battlefield_ids_ordered`, and `evaluate_amount`'s `CountOf` / `CardTypesAmong` |

The enumeration row is the one the audit in §5's second note missed, because it
is not in `compute.rs` at all — it is `evaluate_amount` and the ordered sweeps.

**And when it was written it named a call site that did not exist** (corrected
2026-09-02, RC-4). `AmountExpr::CountOf` had no evaluator and no card, and
`CardTypesAmong`'s one arm reads graveyards; nothing in the layer walk
enumerated the battlefield, so the "invisible to counts" half of the boundary
had no behaviour to test. RC-4 gave `CountOf` its static-context evaluator —
one frame per permanent per query, asked at the current `layer_index` and
memoized for the walk, §12's quadratic by design — and registered Keldon
Warlord as its card. The entering object is invisible to it because the count
runs over `battlefield_ids_ordered`, which it is not on: the boundary is
structural, with no special case anywhere. Devotion still has no `AmountExpr`
and is not the cheap way to test this; the Warlord is.

**Grist is the same rule without the confound, and is the cleaner test to write
first.** Grist, the Hunger Tide reads "As long as Grist isn't on the
battlefield, it's a 1/1 Insect creature in addition to its other types"
(Scryfall, 2026-08-25) — a static ability functioning in every zone (CR 604.3),
whose condition is about *the object itself*. Its ruling: "Anywhere but on the
battlefield, Grist is a Legendary Planeswalker Creature — Grist Insect. Once it
enters the battlefield, it is no longer a creature and is just a planeswalker."
So an entering Grist is **not** a creature in the look-ahead frame, and
Authority of the Consuls does not apply to it.

Put beside Thassa, the pair states the rule with no room left:

| | Grist | Thassa |
|---|---|---|
| What the entering object's own static asks | "am *I* on the battlefield?" | "what is my controller's devotion?" |
| What the frame answers | **yes** — clause (2), the object's own characteristics in the would-be state | devotion is read off the **real** board, which does not contain her |
| Result | not a creature; ETB creature-replacements miss | not a creature *for the replacement check*; is one an instant later for triggers |

Both are clause (2) working. The difference is not which object is consulted —
it is the same object both times — but whether the question is *about this
object's characteristics* or *a count over the permanents a player controls*.
CR 614.12 licenses a hypothetical only for the former; the latter reads the
board as it actually is, which is exactly what the ruling's own justification
says ("because replacement effects are considered before the God is on the
battlefield").

**Neither is buildable, and not for the reason this section implied**
(corrected 2026-09-02, before RC-4's first test was written). Grist's "as long
as Grist isn't on the battlefield" and Thassa's "as long as your devotion to
blue is less than five" are both `Effect::Conditional` **static** abilities,
which `register_static_effects` could not lower and `debug_assert!`d on —
Deferred Migrations item 7f. **7f closed with LI-3 (2026-09-06) and neither
card is unlocked by it**, which is worth recording because the item read as
though it were the blocker: Thassa needs a `Condition` leaf for a numeric
comparison and an `AmountExpr` that counts mana symbols, and Grist needs a
condition about a zone the source is *not* in, which no leaf expresses. The
devotion arithmetic was the cheapest part of the God, not the expensive one.
RC-4 did not pull 7f in. Clause (2) is tested with a fixture
instead — a 2/2 whose own "creatures you control get +1/+1" makes it 3/3 to a
"creatures with power 2 or less enter tapped" — and the count boundary with
Keldon Warlord, above. When 7f lands, `test_grist_entering_is_not_a_creature`
is still the right first test of it.

**It also bounds the risk this section opened with.** The gods looked like
evidence the look-ahead is unboundedly hairy. What they actually produced is a
single sentence with a table behind it and a test — which shipped as
`test_an_entering_count_does_not_include_the_entering_object`, on Keldon
Warlord rather than a God. That is the shape to
insist on for the rest of RC-4 — an unintuitive ruling that reduces to a
mechanical rule is fine; one that does not is a signal to stop and re-read the
CR before writing code.

### 5b. Three corrections from a judge-corpus pass (2026-08-26)

Note the asymmetry this creates and do not smooth it over: clause (2) puts the entering object's own anthem into *its own* frame, but that anthem does not reach any other object's frame. One object is hypothetical; nothing else is.

→ The five interactions and the three corrections — Elvish Archdruid beside Master Biomancer, the batch-scoped frame two Biomancers need, Uphill Battle's one card, and a second copy effect overwriting the first: `plans/archive/replacement-architecture-landed.md`, "5b" (evicted 2026-09-15).

### 5c. Dress Down moves the RC seam

**The finding.** Dress Down ("Creatures lose all abilities") is on the
battlefield; a Clone is cast. Clone's "You may have this creature enter as a copy
of any creature on the battlefield" is an ETB replacement, and CR 614.12 clause
(3) says the look-ahead frame takes into account continuous effects that already
exist and would apply to the entering object. Dress Down is one. So in the frame
Clone **has no abilities**, its copy replacement does not exist, and it enters as
a 0/0 and dies. Xu-Ifit, Osteoharmonist is the same shape from a resolution
rather than a static, and its Gatherer ruling is explicit — a permanent it
returns "will lose that ability before it can trigger… before it can apply…
[including] Clone's ability that causes it to enter as a copy".

**Why this is not just another RC-4 card.** §5 split RC on the claim that its first half
handles "ETB replacements whose applicability does not depend on the frame —
`ObjectSet::SourceOnly`, unconditional… 'this land enters tapped'". That claim
is false, and Dress Down is the proof: whether the entering land *has* its
enters-tapped ability is itself a frame question. The design collapses two
questions into one:

1. **Does the entering object still have its own replacement ability?** Answered
   by the frame's Layer 6 output under clause (3). Prior to everything else.
2. **Which replacements apply to this event?** §5's clauses, the question the
   overlay was designed for.

Question 1 needs exactly the piece the overlay half was going to build: the membership gate
at `compute.rs:629` (✅ RC-3), which returned `false` for any filter-scoped
`ContinuousEffect` against an object not in `game.battlefield` — so an entering
Clone matched no filter, Dress Down included, and kept its ability. It now gates
on the battlefield *zone*, which the entering object is already in.

**The seam moves, the split survives.** The membership gate and the frame's
ability list — enough to ask "what abilities would this object have on the
battlefield", which is the whole of question 1 — land in **RC-3**, ahead of the
overlay. **RC-4** keeps clause (1)'s pending `EnterMods`, the filter-based
*applicability* of other permanents' replacements (Orb of Dreams), and the
614.13a/b exclusion sets. (Corrected 2026-09-02: RC-4 shipped without the
exclusion sets, which need the decision-bearing entry replacement CR 614.13
describes and are **RC-5**'s — sized in §9.) The earlier half is still the smaller, lower-risk one;
it is just not overlay-free, and shipping it overlay-free would enter a Dress
Downed Clone as a copy. Sizing RC after this finding is what turned two parts
into four — see §9.

**De-risking split.** RC-2 implements ETB replacements whose applicability does
not depend on the frame — `ObjectSet::SourceOnly`, unconditional. That is "this
land enters tapped" and "this enters with N +1/+1 counters", which is the
overwhelming bulk of the 773 + 580. RC-3 adds the membership gate and the frame's
ability list per §5c. RC-4 builds the rest of the overlay and turns on
filter-based ETB replacements (Orb of Dreams, Blood Moon interactions) and the
614.13a/b exclusion sets.

### 5d. The overlay as built (RC-4, 2026-09-02)

`engine/layers/lookahead.rs`'s module docs used to carry this; it lives here so
the source stays a summary and a pointer.

**A read-side overlay, not a clone** (§11 item 5). A `GameState` clone
duplicates `GameState.rng` against the determinism doctrine and produces a
second live copy of every v4 `ObjectId`, aliased rather than distinguishable.
So the hypothetical is expressed as the *reads* the layer walk makes of
concrete state. There are two, and each sits behind one accessor in
`compute.rs`:

| Accessor | Reads | Answers from the overlay when… |
|---|---|---|
| `FrameCache::entity` | the `PermanentState` the walk seeds from — controller, CR 302.6's clock, the counters layers 6 and 7c read | the object being computed is the entering one: `Lookahead::entity`, the entity `place_on_battlefield` would build |
| `rows_in_layer` | the registry's slice for a layer, in CR 613.7 order | the object being computed is the entering one: the registry's rows, then `Lookahead::rows`, the rows `register_static_effects` would write |

`base_controller` has the matching arm, so the seed and `effective_controller`'s
gated short-circuit agree by construction. The membership gate in
`effect_applies_to` (`in_battlefield_zone_or_entering`) admits the entering
object, which has not moved yet, and a token created in the zone with no
entity — which is what lets a CR 614.17d "can't enter" be asked of the entry
before anything moves.

**CR 614.12's clauses, against the read each perturbs:**

| Clause | Perturbed read |
|---|---|
| (1) replacements that already modified how it enters | the counters layers 6 and 7c read come from the pending `EnterMods`, on the would-be entity |
| (2) its own static abilities | `rows_in_layer` appends the would-be rows, timestamped as the entry would timestamp them |
| (3) effects that already exist | RC-3 admitted the battlefield *zone* to filter matching; the overlay only widens that to an object not yet in the zone |

Plus the seed: the frame's controller is the proposed one — CR 110.2b's
default, or a CR 616.1b rewrite of it — and CR 302.6's clock starts this turn.
`base_controller`'s battlefield probe never finds an entity for an entering
object, so without the seed a filter's "you control" would read the owner,
which is wrong for every permanent spell cast by a non-owner.

**Timestamps.** `Lookahead::new` reads `next_timestamp` without advancing it,
gives the entity that value and each counter kind the next ones, as
`place_on_battlefield` would. Only the order matters: later than every
registered row, which is where CR 613.7a puts an object's own static-ability
effects and CR 613.7c its counters. CR 613.7e's re-timestamp on attachment
does not disturb this: it fires when an Aura, Equipment or Fortification
*becomes attached*, which is at or after entry and so later still, and it
never re-timestamps the host (`gather.rs` makes the same point where it
splices the entering object's own replacements in last). The engine does not
model 613.7e's re-timestamp yet; attachment as a layers input is critical-path
item 6b.

**The gates.** The walk has two fast paths keyed off `RegistryScopeSummary`:
CR 613.6's "started applying" bookkeeping runs only when some effect occupies
more than one row, and `effective_controller` walks only when some row can
change control. The would-be rows are not in the registry, so a frame whose
own rows would flip either gate — an entering permanent with a two-row static,
or one that changes control of itself — would be answered wrongly by the
registry's summary alone. `Lookahead::summary` is the same struct computed by
the same function (`RegistryScopeSummary::of`) over the would-be rows, and each
gate reads both.

**One object is hypothetical; nothing else is (§5b).** The would-be rows are
appended only when computing the entering id, so the entering permanent's
anthem is in *its* frame and reaches no other object's; and an
`AmountExpr::CountOf` enumerates `battlefield_ids_ordered`, which the entering
object is not on. That is §5a's boundary — visible to filters, invisible to
counts — falling out of the structure rather than being special-cased.

**Where it lives — the decision-site invariant.** On the stack, threaded
through `FrameCache`. `codebase-state.md` item 40 asks of any state a decision
is taken against whether it is *outcome-bearing*: drop it and re-derive, and
does the game reach the same outcome? The frame is a pure function of
`GameState` and the proposal being decided, so it does — it is bookkeeping,
and bookkeeping may live off `GameState`. The proposal itself,
`apply_replacements`' `event`, is the outcome-bearing thing, and it is already
item 40's first violator.

**What review found after it shipped** is in `handoffs/rc-4-review.md`; the one
defect is the entry hop (`codebase-state.md`, "Before Triggered abilities"
item 4), which is not the overlay's but the two-event entry it sits on.

---

## 6. Engine interaction points

| Site | Change | Phase |
|---|---|---|
| `engine/actions.rs::execute_action` | the pipeline call; gains `ActionContext`; batch sibling `execute_actions` | RA ✅ / RB |
| `engine/actions.rs::change_zone` | gains `cause` and `ActionContext` | RA |
| `engine/zones.rs::move_object` | performer only; emission moves **out** of it, into `perform_action`'s `ZoneChange` arm — the only place that knows the `cause` and can capture the LKI frame before the object stops being a permanent | RA ✅ |
| `engine/zones.rs::play_land` | gains `ActionContext`; routes through `change_zone` with `PlayedAsLand`. **Not in the original table** — a fourth chokepoint bypass §11's derivation missed, and the most frequent zone change in the game | RA ✅ |
| `engine/zones.rs::draw_card` | routes through `execute_action(DrawCard)`; emits `CardDrawn` | RA |
| `engine/stack.rs` × 3 `// REPLACEMENT-BYPASS:` | closed by naming the state instead of dispatching around it: `GameState::resolving` records the popped object and CR 110.2b's controller, so `remove_from_zone_collection(Stack)` and `init_zone_state` can both consult it and the sites use plain `change_zone` | RA ✅ |
| `engine/stack.rs::resolve_top_of_stack` — the early pop | **stop popping.** The pattern has no surviving justification (below), and the CR keeps a resolving spell on the stack. Deletes the leniency branch `resolving` needed | RC, with `init_zone_state` |
| `engine/turns.rs::process_draw_step` | stop calling `draw_card` directly (CR 614.11, 614.10) | RA |
| `engine/turns.rs::process_untap_step` | route through `Untap`; emit `Untapped` (CR 122.1d); **one batch**, per CR 502.1's "simultaneously" | RA ✅ |
| `engine/costs.rs:150,165` | `Cost::Tap`, `{Q}` — emit `Tapped` | RA |
| `engine/combat/steps.rs:70` | attackers tap — emit `Tapped` | RA |
| `engine/combat/resolution.rs::apply_combat_damage` | batch, not a loop (CR 510.2, 615.7) | RA/RD |
| `engine/put_on_stack.rs::activate_ability` | emit `AbilityActivated`; resolution emits identity-bearing `AbilityResolved` | RA |
| `state/game_state.rs::StackEntry` | add `cast_from: Zone` — §8c, two customers | RA |
| `engine/sba.rs` | one CR 704.3 event: gather every condition against one game state, dedupe per object (704.7's same-result collapse), perform as one batch, `cause` on each move | RA ✅ |
| `engine/resolve.rs::Primitive::Destroy` | lowers to `GameAction::Destroy`, not to `ZoneChange` | RB |
| `state/game_state.rs::place_on_battlefield` | becomes the *performer* of an already-replaced `EnterBattlefield` | RC |
| `state/game_state.rs::register_static_effects` | skip `Effect::Replacement` bodies without tripping the loud-lowering assert | RB |
| `engine/layers/compute.rs` | ✅ RC-3 — the filter membership gate reads the battlefield *zone*, so it admits entering objects (§5c) | RC-3 |
| `engine/layers/compute.rs` | battlefield reads go through one accessor (overlay seam) | RC-4 |
| `engine/turns.rs::advance_turn` | gains `dp` (RA); proposes `BeginTurn` / `BeginPhase` / `BeginStep` ahead of each unit and proceeds past a dropped one (CR 614.10, 500.11) — "consults pending skips" was the first draft and is struck, §9 RE decision 6 | RA / RE-1 |
| `engine/keywords.rs::apply_lifelink` | the gain becomes an `execute_action(GainLife)` proposal — Tainted Remedy-class watchers must see lifelink. Found at audit 2026-08-25: it writes `life_total` directly and *emits* `LifeChanged`, which is exactly how a census of emissions missed it | RA |
| `engine/costs.rs:184` `Cost::PayLife` | routes through `execute_action(LoseLife)` — CR 119.4 makes paying life a life loss (Bloodletter doubles it). Same audit finding, same emit-without-propose shape | RA |
| `engine/actions.rs` `DealDamage` performer | CR 120.3 results decomposition — player damage contains a `LoseLife`, planeswalker damage removes loyalty counters (CR 120.3c, unimplemented; tracked in `codebase-state.md`) | RD |

### The `ActionContext` plumbing

`execute_action` has no `DecisionProvider`, and CR 616.1 requires one. Rather
than threading a bare `&dyn DecisionProvider`, thread the struct that also
carries the two payload requirements from Deferred Migrations item 3:

```rust
pub struct ActionContext<'a> {
    pub dp: &'a dyn DecisionProvider,
    /// The resolution this action belongs to, if any. Two customers:
    /// CR 614.15 self-replacement effects live here, not in the registry;
    /// and every emitted `GameEvent` gets stamped with it.
    pub resolution: Option<&'a ResolutionContext>,
}
```

**The plumbing is small.** Measured 2026-08-24: 7 production `execute_action`
call sites outside `actions.rs` (6 in `resolve.rs`, 1 in
`combat/resolution.rs`), 13 `change_zone`, 1 production `advance_turn`
(`state/game.rs:201`). `check_state_based_actions` and `resolve_effect` already
take a `&dyn DecisionProvider`. The 44 test `advance_turn` calls funnel through
`test_support::pass_turn`, which can use the existing `test_support::test_dp()`.

---

## 7. DecisionProvider surface

One new `ChoiceKind`, plus one reused shape:

```rust
// ui/choice_types.rs
/// CR 616.1 — two or more replacement/prevention effects want the same event.
/// Only asked when there is a genuine choice (2+ candidates in the forced
/// bucket); a single candidate applies without a prompt.
ChooseReplacementEffect { affected: ReplacementSubject, event: EventSummary },

/// CR 615.7 — one shield, two or more simultaneous damage sources. Reuses
/// `allocate`, the same call trample damage already uses.
AllocateNextDamage { source: ObjectId, remaining: u64 },  // named in RD's design check; per instance, not per subject
```

The 616.1 prompt is a `pick_n` with bounds `(1, 1)` over
`ChoiceOption::Object`-shaped candidates. **The option order is part of the
decision** (`CLAUDE.md`, determinism): candidates are gathered via
`battlefield_ids_ordered()` and registry insertion order, never a raw `HashMap`
sweep. A `fuzz_games --seed N` run must still reproduce line-for-line.

---

## 8. Performance

> **§8 through §8c are one argument** — will this scale, in cost and in code?
> §8 is the measurement discipline, §8a asks whether the event vocabulary is
> complete, §8b sizes the types, §8c answers where card breadth actually lands.
> Read them together; §8c carries the verdict.

`execute_action` is on every mutation, and the gather sweep reads effective
abilities for every permanent. That is a `compute_characteristics` call per
permanent per event — the same quadratic `layers-architecture.md` §12 measured
for priority sweeps, now on a much hotter path.

**Do not pre-optimize it.** Follow §12's own ordering:

1. Build it straight, measure with `fuzz_games --games 200 --seed 12345`, and
   record the number in this file. That is what the layers phases did, and it is
   why their optimizations are defensible.
2. If it bites, the first lever is **answer-preserving**: a per-`GameState`
   bitmask of which `EventPattern` kinds any live replacement source can match,
   recomputed by full walk on registry mutation and on battlefield/counter
   change — the `RegistryScopeSummary` pattern, including its explicit choice of
   a full walk over incremental counters.
3. **Never a semantics-assuming shortcut.** `can_change_abilities()` was worth
   5–8× and was deleted in the session it was written, because its failure mode
   is a silently wrong answer. A rules engine has nothing to trade for that.

One structural note that is free: the gather sweep can skip an object entirely
when its effective ability list is empty *and* it carries none of the three
CR 122 counter types. That is a check, not an assumption.

---

## 8a. Is the event vocabulary complete? No — and that is a bounded problem

Raised in review of Phase RE: *"are we sure these are all the replacement event
kinds?"* **No.** The list in §3.1 is not complete, **eight omissions are already
known and named below**, and more will surface with card breadth. What matters is that the
question is bounded and that the failure mode is loud rather than silent.

### The derivation

A replacement effect can only replace an event the engine actually **proposes**.
So the set of replaceable event kinds is not a fact about Magic's card pool — it
is exactly the set of `GameAction` variants. "Did we get all the replacement
event kinds" therefore reduces to **"did we get the mutation vocabulary right"**,
which is a question about `perform_action`, not about cards, and which can be
answered by reading one file instead of surveying 30,000 cards.

That reduction is the whole reason for the RA/RB split. RA's exit criterion —
every observable state mutation is emitted from exactly one place — is what makes
the `GameAction` enum an *enumeration of the engine's mutations* rather than a
list of the ones someone happened to need.

### The failure mode, and the guard

Today, a card needing a mutation with no `GameAction` gets that mutation written
inline somewhere, where it is **silently invisible to both CR 614 and CR 603**.
That is not hypothetical — it is the activation-invisibility gap
(`codebase-state.md`, Before Triggers item 2), where an entire ability
activation left no trace in the event log, and it is the same class as the 21
sites that read printed characteristics after Layer 4 landed.

The guard is the pattern `register_static_effects` already uses for lowering
(commit `67c5a72`, "make the card→registry step refuse to be quiet"): **make the
quiet path impossible rather than documenting it.** Concretely, as an RA exit
task:

- No `pub` field on `GameState` that a card-facing module can mutate directly —
  battlefield entries, life totals, tap state, counters and zone collections go
  behind `pub(crate)` with `perform_action` as the writer.
- A test that walks `perform_action`'s match arms and asserts one arm per
  `GameAction` variant, so a variant added without a performer fails to compile
  or fails the suite rather than becoming a no-op.
- A `debug_assert!` at each remaining bypass, of which RA leaves zero.

A new card then costs a `GameAction` variant plus a `perform_action` arm plus an
`EventPattern` arm — three edits in three known places, and per §3.2b usually no
`Rewrite` change at all. That is the cost this design is buying; it is not zero,
and pretending the list is closed would be the more expensive lie.

### Known-missing kinds, named now

Found while checking this review comment (Scryfall, 2026-08-24). None is large,
and none changes the architecture — they are listed so a later phase does not
rediscover them as surprises:

| Missing event | CR | Cards | Where it lands |
|---|---|---|---|
| **Losing the game** — Exquisite Archangel, Lich's Mirror, The Golden Throne, Stunning Reversal | 104.3, 704.5a | 4 | A `GameAction::PlayerLoses`. Interacts with RA's SBA batch, since the loss is SBA-driven |
| **Winning the game** — Laboratory Maniac's shape | 104.2 | 2 + | Same, `PlayerWins`. Lab Maniac itself is a *draw* replacement and is already in RE |
| **Discard as an event** — Library of Leng, Dodecapod, Loxodon Smiter | 701.9 | 17 | `ZoneChange { cause: Discarded }` exists in RA; the pattern arm does not. Note the printed wording is "causes you to discard", not "would discard" — the naive Scryfall probe returns 0 |
| **Turned face up** | 614.1e | 2 | Needs face-down permanents; out of scope |
| **Rolling dice** | 705/706 | 7 | Krark's Thumb, Barbarian Class. CR 705 is ❌ in `codebase-state.md` |
| **Scry / surveil as an event** | 701.24 | 2 | Eligeth, Crossroads Augur |
| **Search a library** | 701.19 | 1 | Aven Mindcensor |
| **Countering a spell** | 701.6 | 1 | Guile — and note the first probe here reported **zero**, because the printed wording is "would counter", not "would be countered" |

Damage-to-life-total replacement (Ali from Cairo, 8 cards) is a `DealDamage`
replacement and already covered; mill (2), paying life (1 — Ashiok, and it is
CR 614.13c's own example) and untap (2, plus 92 stun-counter cards) fold into
existing variants.

**Re-read at RE's sizing (2026-09-11), and the table is wrong in both
directions.** The discard row was written before RB and never re-read:
`EventPattern::ZoneChange { cause: Some(Discarded) }` has watched discards
since RB, so "the pattern arm does not [exist]" has been false for sixteen
days of RE being scheduled to add it. What the seventeen cards lack is a
*producer* — `Primitive::Discard` is `NotImplemented` and the cleanup discard
is the only site — plus a `caused_by` field (sixteen say "a spell or ability
an opponent controls causes") and a to-battlefield leg on the substitute — and
all three are RE-8's, on RD-1's precedent (`Primitive::Mill` landed inside a
replacement PR because a rider needed it); the first cut of §9's RE section
sent them to Phase 8 on this section's own sentence, and the review overturned
that on cost of delay. The scry row is the same shape and the same PR. And the Ali from Cairo sentence just above is
stale since RD decision 1: Ali watches the contained `LoseLife`, not the
damage, and is RE-3's. Two things the table could not have known are named in
§9 (RE, "The census"): mana production is a direct write with no event at all,
and a lost player keeps taking turns in any game of three or more.

**Re-read at the post-RE audit (2026-09-15), row by row, against the
22-variant `GameAction`.** Losing and winning: ✅ RE-6 (`PlayerLoses`,
`PlayerWins`). Discard: ✅ RE-8's producer beside RB's arm. Scry: ✅ RE-8
(`GameAction::Scry`). Four rows the tree still lacks, each dispositioned:

- **Turned face up** (CR 614.1e) — **scheduled, owner CV-6**
  (`copy-effects-architecture.md` §4.6 names it now). The table's 2 was an
  undercount: three printed cards say "As [this] is turned face up" (Scryfall
  2026-09-15: Bubble Smuggler, Gift of Doom, Hooded Hydra). The event is the
  turn-face-up special action CV-6 builds, and the replacement lands with the
  first morph creature that is turned up.
- **Rolling dice** (CR 706, and CR 705's coins with it) — **deferred, no
  surface**: no primitive, no event, no reader of `GameState.rng` for it.
  Seven printed "would roll … instead" cards outside Un-sets (Barbarian
  Class, Pixie Guide, Wyll, Blade of Frontiers, …) of ~84 that roll at all.
  `backlog.md` §2.31 is the entry, sized there.
- **Searching a library** (CR 701.23; Aven Mindcensor, 1 card) — **deferred
  with its producer**: `Primitive::Search` is one of `backlog.md` §2.5's
  unimplemented arms, and on RD-1's precedent (`Primitive::Mill` landed in
  the PR whose rider needed it) the event kind lands in the PR that lands the
  producer — a `GameAction::SearchLibrary` family, its `EventPattern` arm and
  its performer, ~300 lines by RE's per-kind measure.
- **Countering a spell** (CR 701.6; Guile, 1 card) — **closed as an event
  kind**, and the row was wrong the way the discard row was: countering is
  performed as a `ZoneChange` out of the stack with `ZoneChangeCause::Countered`
  (`Primitive::CounterSpell`, `resolve.rs`), which `EventPattern::ZoneChange
  { cause }` has watched since RB, so Guile's "instead exile that spell" is a
  `ZoneChangeTo(Exile)` the pipeline expresses today. What Guile still lacks
  is its rider — "you may play that card without paying its mana cost" —
  which is `backlog.md` §2.3's, casting from a non-hand zone.

### Two deliberate non-events, re-checked (2026-08-30, `rb-review.md` E4)

The section above asks what the vocabulary is *missing*. The mirror question is
what it deliberately leaves out: `Primitive::RemoveFromCombat` and
`Primitive::RemoveAllDamage` both write `PermanentState` directly, and both
justified it as "no card replaces this". An absence of cards is not a reason —
this design's premise is that any event the engine performs should be
replaceable — so the two were re-checked against Scryfall on 2026-08-30. **They
have different answers.**

**Removal from combat: sound, and for a better reason than the card count.**
CR 506.4 does not define an event. It defines a *consequence*, with seven
causes: leaving the battlefield, a controller change, phasing out, an effect
that specifically removes it, an attacked planeswalker or battle that stops
being one, and an attacking or blocking creature that regenerates, stops being
a creature, or becomes a battle. **Six of the seven follow from something else
the engine already models** — a proposed `ZoneChange`, a Layer 2 control
change, a type change out of the layer walk, CR 701.19's regeneration (phasing
is CR 702.26 and not built) — and the seventh *is*
`Primitive::RemoveFromCombat`.

**Nothing in the CR forbids "creatures you control can't be removed from
combat", and this section does not claim otherwise.** What CR 506.4's shape
decides is *where such a card would be enforced*: at six other places as well as
this one, which makes it a `RestrictionDef` consulted by each cause
(`cant-effects-architecture.md` §3, the sixth enforcement point) rather than a
`ReplacementDef` over an event this arm proposes. Giving this arm a `GameAction`
would buy that card nothing — it would still be unenforceable at the six. It is
also worth noticing *why* the card has never been printed: on the
leaves-the-battlefield cause it would have to mean something for a permanent
that is no longer there, which is the kind of rules problem WotC avoids by
construction. The card pool is unanimous today: 25 cards print "from combat"
(`o:"from combat"`) and **all 25 cause removal**; `o:"removed from combat"` as a
phrase returns zero, so nothing replaces it, forbids it, or triggers on it.

**Damage removal: the on-demand half is sound; the half the comment leaned on
was not.** The primitive itself is fine — the only printed card that removes all
damage is **Pyramids** ("The next time target land would be destroyed this turn,
remove all damage marked on it instead"), and it uses the removal as a
replacement's *substituted event*, never as something replaced. But the comment
justified the write as "the on-demand form of the CR 514.2 cleanup wipe, and a
direct write for the same reason it is", and **the cleanup wipe is restricted by
seven printed cards**: Ancient Adamantoise, Case of the Market Melee, Melt
Through, Patient Zero, Switchgrass Grazer, Uthgardt Fury and Victory of the
Pyrohammer all say damage isn't removed during cleanup steps
(`o:/damage isn.t removed/ -is:funny`).

Those cards are not replacement effects and they do not want a `GameAction`.
CR 514.2's removal is a turn-based action that "doesn't use the stack", and the
restrictions are per-permanent and filtered ("from creatures your opponents
control"), so what they need is an enforcement point *at the wipe* —
`cant-effects-architecture.md`'s sixth shape, not this pipeline's. Recorded as
`codebase-state.md`, Before Replacement item 20.

**What would reopen either: a trigger, not a replacement.** A direct write is
invisible to CR 603's detector for the same reason it is invisible to CR 614's
pipeline, and the event stream is what Phase 6 matches against. Today
`o:/whenever.*removed from combat/` returns zero, so the tripwire is a card that
watches one of these, not a card that replaces one.

**And what it costs to be wrong, which is the part that makes "no card does
this" an acceptable reason at all.** Take the shape that would stress it hardest
— a hypothetical *"if you would remove damage marked on enchanted creature,
create that many 1/1 Goblins instead"*, a replacement whose output depends on
the replaced event's magnitude. Priced against this tree:

| | Edit | Enforced by |
|---|---|---|
| 1 | `GameAction::RemoveDamage { object }` | — |
| 2 | a `perform_action` arm, taking the two-line write out of `resolve.rs` | §8a's one-arm-per-variant test |
| 3 | an arm in `subject_of` | the compiler — the match is exhaustive |
| 4 | `EventPattern::RemoveDamage` | §3.2a; `pattern_matches` is exhaustive |
| 5 | `Primitive::RemoveAllDamage` becomes an `execute_actions` batch over `battlefield_ids_ordered` — routing a sweep makes its order observable | review |
| 6 | the CR 514.2 cleanup wipe routes the same way, which is what 514.2's "simultaneously" wanted anyway | review |

Five of the six are mechanical and four are caught by a compiler or a test. The
**one genuinely new thing** is "that many": an amount read off the event being
replaced is `Rewrite`'s `Amount` arm, which §3.2b deliberately did not ship and
which **RD already owes** for damage multiplication — so even the expensive half
is on the schedule rather than a surprise.

That is the asymmetry worth stating plainly. "No card does this" is a sound
reason to leave a mutation outside the vocabulary **because the cost of guessing
wrong is a bounded, compiler-enforced diff**, not because the guess is certain.
Where a wrong guess would instead force a redesign — the applied set's identity
(H9), the copy row storing values rather than a reference — the same argument
would not be available, and those are decided ahead of the cards on purpose.

**One thing the check turned up that belongs to the census, not here.**
`cant-census.py` queries `o:"can't" -is:funny`, which is its stated scope — but
both families above phrase the restriction as "isn't"/"doesn't", so neither
appears in the 2,034 clauses, and one of them is not small: **248 cards print
"doesn't untap during ..."** (`o:/doesn.t untap during/ -is:funny`).
`cant-effects-architecture.md` §2.2, "What the census cannot see", is where that
belongs; it currently lists only the keyword-borne restrictions.

### What the residual actually is: CR 701 keyword actions

The §3.2c classification left 11 clauses unbucketed, and reading them gives the
generalization this section wanted. They are, without exception, **CR 701
keyword actions**: flip a coin (Krark's Thumb), connive (Leader, Super-Genius),
learn (Retriever Phoenix), explore (Topography Tracker, Twists and Turns),
proliferate (Tekuthal), assemble a Contraption (Steamflogger Boss), planeswalk
(Susan Foreman).

That is a much better-behaved answer than "cards keep inventing things".
**A keyword action is a replaceable event**, so the `GameAction` vocabulary must
eventually cover CR 701 — and CR 701 is an enumerated chapter of roughly sixty
entries, most of which are already `Primitive` variants. The growth axis is a
CR chapter, readable in an afternoon, not an open-ended card-driven set. When a
keyword action becomes an event kind, the card that motivated it usually needs
no `Rewrite` change at all.

**Phase RE's title is "the remaining event kinds" and should be read as "the
remaining event kinds we know of."** It gains `PlayerLoses` / `PlayerWins` and
the discard pattern arm from this audit; the seven above are budgeted against
the phases that need them, and the CR 701 sweep belongs with Phase 8 card
breadth rather than here.

---

## 8b. Sizing — how big do these types actually get?

→ The two bounds and what they are worth, what is watched and by which hook, the three findings, the projection and the per-card cost: `plans/archive/replacement-architecture-landed.md`, "8b" (evicted 2026-09-15). §0's "Three-and-a-half" keeps the numbers this section derived.

---

## 8c. Where card breadth actually lands — three axes, not one

**Two customers before a variant.** A predicate leaf with exactly one card behind it is the warning sign. One customer is a card-specific predicate; several is a grammar feature.

→ The three axes, the six cards worked through, why enum size is not the cost, the grammar risk with its other two guards and the 5% escape-hatch budget, "should the grammar work move earlier?", and the verdict with RD-3's first data point: `plans/archive/replacement-architecture-landed.md`, "8c" (evicted 2026-09-15).

---

## 9. Work-phase plan

One branch/PR per phase, matching the Layer phases' size (5–8 commits). Phases
RC and RE split into Parts A/B if they run long, as LD did. **RA ships as three
PRs** — sized against the tree 2026-08-25; see below.

### Phase RA — the event spine (no replacement behavior)

The Deferred Migrations item 3 ticket list, verbatim, plus the DP plumbing.
**Whole suite stays green with no behavior change** except newly-emitted events.

**The split.** The twelve tickets keep their numbers — `codebase-state.md` and §6
cite them — but they land in three groups. The grouping is dependency-clean:
nothing in a later group is a prerequisite for an earlier one.

| Sub-phase | Tickets | Shape | Measured size | Status |
|---|---|---|---|---|
| **RA-1 — plumbing** | 1, 2 | pure signature sweep, zero behavior change | 6 signatures, ~90 production + ~75 test call sites | ✅ PR #58 |
| **RA-2 — routing** | 3, 4, 5, 10, 11, 12 | six independent "make the silent site loud" tickets | 5 new `GameEvent` variants; ~10 sites each | ✅ PR #59 |
| **RA-3 — payloads** | 9, 6, 7, 8 *(that order)* | the deep half: batches, LKI, the bypass closure | 3 bypass sites, the SBA sweep, `apply_combat_damage` | ✅ 2026-08-25 |

**Why three, and not the two-way seam this doc proposed first.** Ticket 1 alone
is a session. Counted 2026-08-25: `execute_action` 7 external call sites,
`change_zone` 13 + 5 test, `advance_turn` 12 + 44 test, `pay_costs` 15,
`activate_mana_ability` 6 + 13 test, `apply_combat_damage` 3 + 6 test — and two
whole chains carry no `DecisionProvider` at all, so threading reaches every
function in them: `turns.rs` (`advance_turn` → `on_phase_begin` / `on_phase_end` /
`on_step_begin` / `on_step_end` / `on_turn_end` → `process_untap_step` /
`process_draw_step`) and `costs.rs` (`pay_costs` → `pay_single_cost`).
Bundling ~165 mechanical call-site edits with five behavior-adding routing
tickets is the session that overruns. Split off, RA-1 is the safest PR shape the
project writes: the diff is a signature sweep, and green-on-the-nose is the whole
test.

#### RA-1 — the plumbing (tickets 1–2) — ✅ landed 2026-08-25

1. `ActionContext` threaded through `execute_action` / `change_zone` /
   `advance_turn` / `apply_combat_damage` / the SBA sweep. `resolve_effect` and
   `resolve_primitive` already carry `(ctx, dp)` and just repackage them;
   `check_state_based_actions` and `resolve_top_of_stack` already carry a `dp`.
2. `ZoneChangeCause` on `ZoneChange`; every caller sets it.

   **Safe to land here precisely because nothing reads it yet** — there is no
   pipeline and no trigger matcher in RA, so labelling is additive and the
   no-catchall ban (§11) costs nothing to enforce. It rides with ticket 1 rather
   than waiting, because otherwise the `change_zone` sites churn twice.

   **Only 9 of the 13 production movers are labellable, and that is the finding.**
   Four (`cast.rs:99,116,146,214`) are CR 601.2 cast *rollbacks* — the game state
   is rewound, no object legally moved, and no replacement effect may ever see
   one. Under "a site with nothing honest to say is a site whose reason nobody
   worked out", the honest answer is that they are not zone changes: **take them
   back out of the chokepoint** as direct `move_object` calls tagged
   `// CAST-ROLLBACK:`, with a Deferred Migrations line. Decide this in RA-1, not
   by inventing a cause for it.

   Note also that 9 of the 10 object-moving `Primitive`s (`Exile`, `Sacrifice`,
   `ReturnToHand`, …) are still `NotImplemented` at `resolve.rs:533`, so seven of
   the §3.1 variants have no call site to label today. Define them anyway — the
   enum is documentation of the vocabulary and nothing matches on it in RA — but
   do not go looking for sites that do not exist.

   Test-side: `test_support` gains a ctx helper; `pass_turn` absorbs it.

**RA-1 exit:** `cargo test` green, `cargo build --all-targets` zero warnings,
three `fuzz_games` runs at one seed identical but for the timing lines. No new
events, no new behavior — if the fuzz numbers move, the sweep changed something
it should not have.

#### RA-2 — routing the silent sites (tickets 3–5, 10–12) — ✅ landed 2026-08-25

Six tickets, each independently testable, roughly one commit apiece.

3. Route the draw-step draw through the chokepoint; add `CardDrawn` (CR 121.5 —
   106 cards say "whenever you draw", 54 say "your second card").
   `state/game.rs:117`'s opening hands stay direct: pregame, nothing observes them.
4. Tap/untap through the chokepoint with `Tapped`/`Untapped` (CR 603.2e); the
   four silent sites in `costs.rs`, `turns.rs`, `combat/steps.rs`. While there,
   make the two `perform_action` arms loud — today they silently no-op for an
   object not on the battlefield, against the loud-lowering doctrine.

   **The untap sweep's ordering comment goes stale here, not in RA-3.**
   `process_untap_step` iterates `battlefield.keys()` under a comment saying the
   sweep "reaches no decision". True today; false the moment each untap is a
   replaceable `Untap` (stun counters, CR 122.1d), because CR 616.1 prompts when
   two effects want one untap and the proposal order is then observable. Move it
   to `battlefield_ids_ordered` in the same commit that makes it an action.
5. `AbilityActivated` + identity-bearing `AbilityResolved` (CR 603.7h).
10. `StackEntry.cast_from: Zone` — the origin a spell was cast from (§8c). Two
    customers: Don't Blink's "cast from exile", and CR 903.8's commander tax.
    Fully independent of everything else in RA; take it first if RA-2 wants a
    warm-up commit.
11. Route lifelink's life gain through `execute_action(GainLife)`
    (`engine/keywords.rs:58` — see §6; audit 2026-08-25).

    **This is the first re-entrant `execute_action`**: `apply_lifelink` runs
    *inside* `perform_action(DealDamage)`, so the proposal nests. Harmless in RA
    (the pipeline is a pass-through) and correct in RB under §3.2d's
    contained-event lineage — but it is the shape RD's CR 120.3 decomposition
    generalizes, so record it rather than rediscovering it there.
12. Route `Cost::PayLife` through `execute_action(LoseLife)`
    (`engine/costs.rs:184` — CR 119.4; same audit).

**RA-2 exit:** grep-provable — no production site outside `perform_action`'s own
arms writes `entry.tapped`, writes `life_total`, or moves a card library→hand.

#### RA-3 — payloads and structure (tickets 9, 6, 7, 8) — ✅ landed 2026-08-25

**Shipped.** `execute_actions`' batch form (combat damage and the SBA sweep use it), the payloads — the CR 603.10a LKI frame on battlefield-leaving moves, `cause`, batch id, resolution context — and the three `// REPLACEMENT-BYPASS:` sites closed by naming the in-between state as `GameState::resolving`. Tickets 9, 6, 7, 8, in that order.

→ The section as sized, what the building changed and the measurement: `plans/archive/replacement-architecture-landed.md`, "RA-3" (evicted 2026-09-11).

### Phase RB — the pipeline, with counters and regeneration as consumers — ✅ landed 2026-08-26

**Shipped.** `ReplacementDef`, `EventPattern`, `Rewrite`, `ReplacementClass`, `Uses`, `Effect::Replacement` with its `then`, `ReplacementEffectRegistry`, and `apply_replacements` — §4.1's CR 616.1 loop with 614.5, 616.1g and CR 101.4's APNAP — with CR 122.1c/d/h counters, CR 701.19 regeneration and Kalitas as consumers and 704.6d / 903.9b beside. Shipped at +5,475 across 33 files, 2.2× the band, because nobody counted first: `engineering-practices.md` §4's cautionary case.

→ The section as sized, what the building changed and the measurement: `plans/archive/replacement-architecture-landed.md`, "RB" (evicted 2026-09-11).

### Phase RC — ETB replacements (the big unlock)

**RC ships as four PRs.** The split is numbered `RC-1` … `RC-4`, matching RA's
convention; the `Part A` / `Part B` framing §5 and §5c argued in is superseded,
and where those sections say "Part A" they mean **RC-2 + RC-3**, where they say
"Part B" they mean **RC-4**.

#### Why four, and why sized before a line is written

**RB was one PR and should have been three.** Measured after the fact: +5,475 /
33 files, against RA-1's +362, RA-2's +919 and RA-3's +2,511 — 2.2× the largest
implementation PR this project had merged, and 1.4× all of RA combined. The
failure was not the decision to keep it whole; **it was that nobody sized it.**
§9 gave RA a table with a *Measured size* column and counted call sites before
splitting; it gave RB nine bullets and no measurement, so RB ran until it was
done. This section is that count, done first.

**One lesson from RB changes the seam, and it is not the obvious one.** The
tempting split is "engine first, consumers after". RB proves that wrong twice
over. Its item-3 commit was 1,306 lines with **zero integration tests**, because
the consumers are what make a pipeline testable at all — and the loop's one real
defect (a declined `exempt_from_614_5` optional re-offering forever) was
reachable only from item 9, the *last* consumer. A carefully reviewed
pipeline-only PR would have merged with a hang in it. **So every RC PR below
carries at least one consumer that exercises what it builds**, and the plan
accepts that a later PR may fix an earlier one.

| PR | Shape | Measured size | Risk |
|---|---|---|---|
| **RC-1 — delete the early stack pop** ✅ | pure deletion, zero new behavior | measured **12**, not 11: `stack.is_empty()` × **6** (the row said 5 — see below) + `GameState::resolving` × 6; deletes one leniency branch | low |
| **RC-2 — `EnterBattlefield` as an event** ✅ | the performer migration, plus enters-tapped as its first consumer | predicted **10** production `place_on_battlefield` sites; **two**, and the number that mattered was 92 direct callers with 88 in `#[cfg(test)]`. Shipped **+1,409 / −218 across 25 files** — 611 engine, 207 cards, 591 tests | medium |
| **RC-3 — the membership gate and the frame's ability list** ✅ | §5c's question 1 | predicted **1** site; **2**, because CR 614.12 is two membership rules and only clause (3) was counted — `compute.rs:629` and `gather`'s source 1a. Shipped **+717 / −37 across 8 files** | **high** |
| **RC-4 — the overlay** ✅ | §5's clauses (1)–(3), 614.17d, 616.1b, `CountOf`, §11 item 19. **614.13a/b moved to RC-5** | re-counted a third time at **9** reads and the count was the wrong instrument: **four kinds** of read needed perturbing, and only those moved. Shipped **+2,567 / −279 across 25 files** — 1,447 engine, 223 cards, 897 tests — over the band on tests alone, with RC-5 already split out in the doc before code | **highest** — and the risk that materialised was not the walk: it was item 19's theorem, which the frame falsified (finding 3) |
| **RC-4b — one proposal per entry, none per cast step** ✅ | The entry hop RC-4's review found (`EnterBattlefield` carries `from` and its performer moves), tokens, CR 608.3e, and the cast rewind's phantom zone change (`codebase-state.md` item 51) — one bundle under one rule, §11 item 20 | sized ~450–650; shipped **+378 / −250 engine across 12 files** and +568 tests — the engine inside the band, the tests over it, as RC-4's were | low, and the one reach into `gather` beyond the pattern arm was a finding, not a cost: source 1a read the entering permanent through a plain walk that was right only because the card had already moved (finding 1 below) |
| **RC-5 — auxiliary zone changes and a dynamic entry amount** ✅ | CR 614.13/13a/13b, a dynamic `EnterWith` amount. **Re-sized 2026-09-03 before code**: the batch-scoped frame was RC-4b's and CR 613.7m is RE's — see below | sized below: ~1,200 + ~250, after the re-size dropped the ~400 | medium — a new decision site, and the only piece the re-size did not shrink |

**Every RC PR that ships a card owes a *second* card of a different shape**
(`engineering-practices.md` §3.3). RB shipped exactly one — Kalitas — and the
consequence, measured 2026-08-31, is that CR 616.1's multi-candidate branch has
never been reachable in a fuzz game: Kalitas is Legendary, so no player controls
two, and two opposing copies each apply only to the *other* player's creatures.
The ordering choice, the applied set across instances, and the APNAP ordering
among simultaneous choosers are all dead code that tests green. RC-2's
enters-tapped consumer is the same trap in a friendlier costume — 773 cards
share one shape, and one of them exercises CR 616.1 exactly as poorly as one of
them exercises it now. `codebase-state.md` item 35 sizes the two cards that
close RB's half at zero engine cost; do not let RC open a second such gap.

#### RC-1 — delete the early stack pop — ✅ landed 2026-09-01

**Shipped.** The early stack pop deleted (`codebase-state.md` Before Replacement item 7): `resolve_top_of_stack` keeps the `StackEntry` and `move_object`'s `remove_from_zone_collection(Stack)` does the removal. Its own PR, first — the RA-1 of the phase.

→ The section as sized, what the building changed and the measurement: `plans/archive/replacement-architecture-landed.md`, "RC-1" (evicted 2026-09-11).

#### RC-2 — `EnterBattlefield` as an event — ✅ landed 2026-09-01

**Shipped.** `GameAction::EnterBattlefield { object, controller, mods }` with `place_on_battlefield` as its performer, `EventPattern::EnterBattlefield`, `Rewrite::EnterWith`, `EnterMods`; consumers enters-tapped (CR 110.5b, Idyllic Beachfront) and enters-with-counters (CR 122.6a, Chainbreaker), both pooled. It proposed the entry from *inside* the zone change's performer, which RC-4b undid.

→ The section as sized, what the building changed and the measurement: `plans/archive/replacement-architecture-landed.md`, "RC-2" (evicted 2026-09-11).

#### RC-3 — the membership gate and the frame's ability list — ✅ landed 2026-09-02

**Shipped.** CR 614.12's membership rule settled both ways: a filter-scoped layer effect reaches an entering permanent (the gate reads the battlefield *zone*, §5c's Dress Down finding), and an entering permanent's own filter-scoped replacement no longer reaches itself. `base_controller` grew its `resolving` leg here.

→ The section as sized, what the building changed and the measurement: `plans/archive/replacement-architecture-landed.md`, "RC-3" (evicted 2026-09-11).

#### RC-4 — the overlay — ✅ landed 2026-09-02

**Shipped.** `compute_as_entering` — the CR 614.12 frame as a read-side overlay through `FrameCache`'s accessor pair, never a `GameState` clone — CR 614.17d in both printed shapes, CR 616.1b (`EnterUnderControlOf`), the first `AmountExpr::CountOf` in the walk, and CR 616.1 no longer prompting for a choice with one outcome (§11 item 19). CR 614.13 and the batch-scoped frame sized out to RC-5 before code.

→ The section as sized, what the building changed and the measurement: `plans/archive/replacement-architecture-landed.md`, "RC-4" (evicted 2026-09-11).

#### RC-4b — entering is one event — ✅ landed 2026-09-02

**Shipped.** Entering is one event: `EnterBattlefield` carries `from`, its performer moves the card, no `ZoneChange` onto the battlefield is ever proposed; a substituted entry is one move with no LKI walk, a refused one leaves the card where it was (CR 608.3e), and `cast_spell`'s 601.2a move is `// CAST-ROLLBACK:` silent both ways. Trace page `rc-4b-entering-is-one-event.html`. The token residual — a substituted token logs `from: Battlefield` — was left as the cheap answer (`codebase-state.md` item 52) and is RE-4's.

→ The section as sized, what the building changed and the measurement: `plans/archive/replacement-architecture-landed.md`, "RC-4b" (evicted 2026-09-11).

#### RC-5 — auxiliary zone changes and a dynamic entry amount — ✅ landed 2026-09-03

**Shipped.** CR 614.13/13a/13b as `Rewrite::EnterAfterMoving(AuxiliaryMove)` with `// AUXILIARY-MOVE:` batches and `GameState::entry_selection`, `EnterMods.counters` given an amount the board decides; devour (Thunder-Thrash Elder) and Sutured Ghoul as consumers; +2,239 / −121. Re-sized against the tree before a line was written, which moved two of its four pieces. Trace page `rc-5-applying-an-entry-can-move-the-board.html`.

→ The section as sized, what the building changed and the measurement: `plans/archive/replacement-architecture-landed.md`, "RC-5" (evicted 2026-09-11).

### Phase RD — damage (CR 615, 609.7, 614.9, 120.3) — sized 2026-09-08, four PRs

**RD ships as four PRs, and the count comes from this phase's work, not from
RA's or RC's.** The seam test is `engineering-practices.md` §4's: a PR boundary
sits where a printed consumer can exercise everything before it and needs
nothing after it, with each piece inside the 1,500–2,500 band. Applied to the
engine changes below it cuts four times — a doubler needs only player scoping
and the `Amount` arm; a shield needs those plus the registry primitive, the
per-subject loop and consume-after-apply; a source predicate needs the pattern
fields and nothing of the loop; a redirect needs `Retarget` and
consume-after-apply and nothing of the sources. Three would put sources and
redirection together at ~2,500 with no consumer they share; five would split
RD-2's loop change from the shield that is its only test, which is RB's
"engine first, consumers after" mistake. A fifth PR exists as a *candidate*,
gated on a measurement rather than assumed (RD-5, Harm's Way, below). The
numbering follows RA and RC; the count does not. Until this section RD was one
paragraph naming eight rules and no consumer, which is exactly the state RB was
in before it shipped at 2.2× the band; everything below was written before a
line of code. Branches are `replacement/rd-<n>-…`.

The paragraph it replaces, kept for the record: *prevention shields (615.7
amount, 615.8 next-instance, 615.9/609.7b property recheck, 615.10 static
per-event, 615.11 per-creature at resolution, 615.12 unpreventable + 615.12a
single application), damage redirection (614.9), doubling (701.10g), the
simultaneous-damage shield allocation choice (615.7, needs RA's batch), and
CR 609.7a's source-choice validation; plus the CR 120.3 results-of-damage
decomposition — performing `DealDamage` against a player proposes a contained
`LoseLife` (fresh lineage, §3.2d), and against a planeswalker removes that many
loyalty counters (CR 120.3c, unimplemented; a planeswalker can never die to
damage today).* That is still the scope. What follows is its shape and its cost.

#### The design check — seven decisions, and the one nobody asked

The titles, in order, so `§9 RD decision N` still lands; each was read against CR 615 whole, 614.7a, 614.9, 609.7a–c, 120.3, 120.3c and 701.10g before a line of RD was written.

**0. The affected set must name a player, and that is RD's, not RE's.**

**1. Partial prevention is `Rewrite::Amount(AmountRewrite::PreventUpTo(n))`, and `Instead` gets no "N − k" template.**

**2. Resolution-created prevention lives in `ReplacementEffectRegistry`, through `Primitive::CreateReplacement(Box<ReplacementDef>, Duration)`, and the shield fits a duration registry because a row already has two ends.**

**3. CR 615.7's allocation ships with the first shield, in RD-2, and it is the same change as §11 item 15.**

**4. The contained `LoseLife` never meets a prevention effect, and what enforces that is the vocabulary, not the lineage rule.**

**5. CR 120.3c ships in RD-1, and its consumer is a fixture with its own name.**

**6. CR 615.11 needs no machinery; CR 615.12 is a property of the event, and its three printed shapes meet at one site.**

**7. A use is spent by what an application did, not by being chosen.**

→ Each decision's argument, its rule numbers and its consumers: `plans/archive/replacement-architecture-landed.md`, "The design check — seven decisions, and the one nobody asked" (evicted 2026-09-15).

#### Why four, and the count

| PR | Shape | Measured size | Risk |
|---|---|---|---|
| **RD-1 — the damage event's two subjects and its results** | `affected_players`; the CR 120.3 decomposition, `LoseLife.cause`, CR 120.3c; `Rewrite::Amount` with `Multiplier`, `Halve` and `PreventHalf`, and `Rounding`; `Rider` carries `EventSubject` and the event's amount, `AmountExpr::ReplacedAmount` and `Multiply`; `Primitive::Mill` (a stub today) for Angel of Suffering's rider | `set_affects` **1**, `chooser_for` **0** (already right), `Rider`/`resolve_rider` **2**; `perform_action`'s arm **1**, `GameAction::LoseLife` constructions **6**; `Rewrite` exhaustive matches **2** (`from_rewrite`, `apply_rewrite`); `ObjectSet` exhaustive matches **3**, all untouched by construction; `evaluate_amount` **2** leaves; `resolve.rs` **1** stub arm made real. Predicted **~560 engine, ~300 cards, ~750 tests ≈ 1,500–1,700** | medium — the decomposition moves a line of every game's log through a nested proposal, and the A/B's middle arm must show it and nothing else |
| **RD-2 — CR 615.7 prevention shields, and the loop's unit** | `Primitive::CreateReplacement`, `Uses::NextDamage`, `PreventUpTo`/`PreventRemaining`, consume-after-apply (decision 7), per-subject decisions and the per-instance allocation (decision 3), the rider's prevented amount and `AmountExpr::DamagePrevented` | `apply_replacements` **1** (the group form), `execute_batch_inner` **1**, `consume_use` **1**, `apply_rewrite` **1**; `DecisionProvider::allocate` impls **3** + dispatch; `ChoiceKind` exhaustive matches ≤ **3**; `evaluate_amount` **1**; `resolve.rs` **1** new arm beside `Regenerate`. Predicted **~700 engine, ~250 cards, ~800 tests ≈ 1,800–2,000** | **highest** — the only PR that changes the loop's unit, and the one whose defect shape is a silent wrong choice rather than an error |
| **RD-3 — sources** | `EventPattern::DealDamage { source, combat }`, CR 609.7a's chosen source (`SelectionFilter::DamageSource`, one `ChoiceKind`), 609.7b's recheck, 615.8 next-instance, 615.10 static partial, 609.7c; `AmountRewrite::Plus` (Torbran) and a resolution-created `PreventHalf` (Dark Sphere) | `pattern_watches` **1**, `EventPattern::DealDamage` constructions **4**; `enumerate_legal_selections` + `has_any_legal_choice` **2** (RS-2's rule that enumeration agrees with enforcement); `Cost::Tap`/`SacrificeSelf` already paid. Predicted **~370 engine, ~400 cards, ~700 tests ≈ 1,400–1,600** | medium — axis 2 of §8c takes real weight for the first time on a two-sided predicate (Torbran's; Daunting Defender is 615.10's own example and is target-side only), and the "two customers before a leaf" guard is applied live |
| **RD-4 — redirection and unpreventable damage** | `Rewrite::Retarget(RetargetSpec)` with CR 614.9's re-check at application, `DealDamage.unpreventable` (16 `Primitive::DealDamage` sites, 25 `GameAction::DealDamage` constructions, mechanical), the restriction consult at application, `PlayerSet` on `ApplyReplacement::to` | `apply_rewrite` **1**, `from_rewrite` **1**, the two site counts above; `is_prohibited` callers **+1**. Predicted **~300 engine, ~200 cards, ~500 tests ≈ 1,000–1,200** | low-medium — two independent features that share only the consume-after-apply rule RD-2 lands |

**≈ 5,700–6,500 across four, each at or inside the band, RD-2 at its top —
plus RD-5's ~300–400 if its gate is met.** RD-3 and RD-4 commute; RD-1 → RD-2
is a hard order (RD-2's shields need player scoping and the `Amount` arm),
and RD-3's static consumers need RD-2's `PreventUpTo` performer. Every PR carries at least one printed consumer and a
second of a different shape (§3.3, tier 2), registered where the engine can play
it, in the commit *after* the fix it needs (`register-a-card-only-once-the-
engine-can-play-it`). The counts are call sites read from the tree on
2026-09-08; the line predictions are calibrated against CM-1 (+2,219 / 42
files), CM-3 (+2,498 / 20) and CM-4 (+1,494 / 14).

#### RD-1 — the damage event's two subjects and its results — ✅ landed 2026-09-08

**Shipped.** Player scoping (`ReplacementDef.affected_players`), CR 120.3's results as a `DamageResults` list with the contained `LoseLife { cause }`, CR 120.3c on the Loyalty Probe fixture, `Rewrite::Amount` with `Multiplier`, `Halve`, `PreventHalf` and `Rounding`, the rider's `ReplacedAmount` and `Multiply`, `Primitive::Mill`. Furnace of Rath (pooled), Ghosts of the Innocent, Gisela, Angel of Suffering. +1,950 against a ~1,500–1,700 prediction — the rulings pass in the card file was the difference. The A/B's middle arm moved by exactly the predicted rows.

→ The section as sized, what the building changed and the measurement: `plans/archive/replacement-architecture-landed.md`, "RD-1" (evicted 2026-09-11).

#### RD-2 — CR 615.7 prevention shields, and the loop's unit — ✅ landed 2026-09-09

**Shipped.** `Primitive::CreateReplacement`, `Uses::NextDamage`, `PreventUpTo`/`PreventRemaining`, a use spent by what the application did (CR 609.7b), decisions per `(batch, subject)` with CR 615.7's allocation reaching across groups (`next_damage_shares`, `GameState::prevention_allocations`), CR 615.11's per-creature rows. Mending Hands (pooled), Samite Healer, Safe Passage, Samite Censer-Bearer; +2,393 / −237. Trace page `rd-2-a-decision-is-per-subject.html`.

→ The section as sized, what the building changed and the measurement: `plans/archive/replacement-architecture-landed.md`, "RD-2" (evicted 2026-09-11).

#### RD-3 — sources — ✅ landed 2026-09-09

**Shipped.** `EventPattern::DealDamage { source: Option<SourcePattern>, combat }`, CR 609.7a's chosen source (`SelectionFilter::DamageSource`, `PatternFill::ChosenDamageSource`), 615.8/9/10, `AmountRewrite::Plus`. Eight printed cards with Guardian Seraph pooled; +2,035 / −57, over on every axis. Two fixes shown failing first: `Primitive::DealDamage` never read `FilteredPermanents`, and a whole-event `Prevent` reported nothing prevented. Circle of Protection: Red's activation ran ~98 times a game, striking the prediction that it would be rare.

→ The section as sized, what the building changed and the measurement: `plans/archive/replacement-architecture-landed.md`, "RD-3" (evicted 2026-09-11).

#### RD-4 — redirection and unpreventable damage — ✅ landed 2026-09-09

**Shipped.** `Rewrite::Retarget(RetargetSpec)` with CR 614.9's re-check at application, `DealDamage.unpreventable` and `Restriction::ApplyReplacement { Prevention }` meeting at one predicate where a prevention arm applies; `ToFixed` cut on "an arm the pipeline cannot apply is worse than a missing one". Pariah (pooled), Palisade Giant, Pinpoint Avalanche, Reflect Damage; the `EachOther` fix (`codebase-state.md` item 103); RD-5's gate closed and Harm's Way sent to `backlog.md` §2.25.

→ The section as sized, what the building changed and the measurement: `plans/archive/replacement-architecture-landed.md`, "RD-4" (evicted 2026-09-11).

#### Out of RD, decided rather than absorbed

- **RE's kinds** — draw, skips, `CreateTokens` and the token residual
  (`codebase-state.md` items 46, 52), counter doublers, life-gain, mana,
  `PlayerLoses`/`PlayerWins`, the CR 701.9 discard arm. `CreateTokens` is
  adjacent to `Amount(Times)` and is not RD's.
- **Ali from Cairo's clamp** — a `LoseLife` replacement (its ruling), RE's,
  with RD-1's `cause` field waiting for it.
- **CR 120.3b/d/g/h** — poison, wither's counters, toxic, battles' defense
  counters — **scheduled, with an owner each, not deferred on precedent.**
  Infect, wither and toxic are `backlog.md` §2.6, which as of 2026-09-08 names
  them, their three CR 120.3 results, their size (~200–300 with the first
  infect card) and the seam RD-1 leaves them: `perform_action(DealDamage)`
  becomes one `match` per result on the source's keywords and the target's
  type, so each lands as one arm; the poison half proposes counters on a
  *player*, which is §2.16's map and decision 0's shape again. Battles are
  `backlog.md` §2.23, new today, because no doc owned CR 310 at all. RD-1
  opens a dated Deferred Migrations line per absent result with its
  reachability — none is stubbed, and none is "unreachable" for longer than
  the first registered card of its keyword.
- **Partial redirection** (Harm's Way) is **RD-5, a candidate, not out** —
  finding 23 has its shape and the measurement that decides it. Divine
  Deflection's pooled amount needs nothing RD lacks except
  `AmountExpr::Variable`.
- **Attacking a planeswalker** — combat's (`validation.rs:199`).
- **CR 615.13** — Phase 7, as §12 says. RD records the prevented amount where
  615.5 needs it and emits no new event: whether a prevention is something the
  performed stream announces is item 6's to decide, and the seam is
  `apply_rewrite`'s return value, which already carries it.
- **`codebase-state.md` item 20** (the CR 514.2 cleanup wipe has no enforcement
  point) and **"Before card breadth" item 6** (multi-attacker block damage is a
  silent stub) — damage-adjacent and combat's; neither is touched, and the
  second is worth re-reading after RD-2, since a blocker splitting damage over
  two attackers is the mirror of the board decision 3 groups.
- **Kitsune Palliator's "each player"**, **Deflecting Palm's "that source's
  controller"** as a rider recipient, **Combust's "can't be countered"** — each
  one leaf away, each with one customer, each waits for its second.

#### RD-5 — partial redirection (Harm's Way) — ❌ gate closed 2026-09-09, moved to `backlog.md` §2.25

**So `backlog.md` §2.25 owns Harm's Way**, with the shape, this table and the reason. Divine Deflection is *not* affected and never was: a `Filter` + `PlayerSet` row with `NextDamage(X)` is already a pooled amount, and it waits only on `AmountExpr::Variable` and `codebase-state.md` item 90.

→ The split the card needs, the gate and the ≈30 sites that fail it, and the three things worth keeping: `plans/archive/replacement-architecture-landed.md`, "RD-5" (evicted 2026-09-15).

#### Measured — what to expect, and why the direction is known

Three arms per PR through `plans/fuzz_ab.py` against a same-day `main` worktree, both pools, and the middle arm — the engine with `registry.rs` and `PERFORMANCE_POOL` unchanged — is the only one that attributes anything.

→ Each PR's prediction before running, and the fourth-binary rule: `plans/archive/replacement-architecture-landed.md`, "Measured", RD's (evicted 2026-09-15). The numbers are `fuzz-record.md`'s.

#### Trace page — decide at RD-2's close

`engineering-practices.md` §7's rule — "a phase that changes *how* a read is
answered rather than what the answer is" — is met by RD-2 and by nothing else in
RD: grouping by subject changes how every `DealDamage` proposal's decision is
reached. So the decision is taken at RD-2's close rather than RD-4's, and if
yes the page is `rd-2-a-decision-is-per-subject.html`, walking the boards the
design check argued about rather than the happy path: two shield counters under
two blockers (item 15), Furnace beside Mending Hands in both orders (the printed
ruling), a `NextDamage(3)` under sources of 2 and 4 with the allocation, and
Pinpoint Avalanche into a shield counter (the rider runs, nothing is spent).

**Decided at RD-2's close (2026-09-09): yes.**
`plans/traces/rd-2-a-decision-is-per-subject.html`, pinned at `fcc04da` (the
last commit of the PR). It walks three of the four boards above — two
shield counters under two blockers, Furnace beside Mending Hands in both
orders, a `NextDamage(3)` under sources of 2 and 4 with the allocation — and,
in place of Pinpoint Avalanche (RD-4's, since nothing can make damage
unpreventable yet), the two boards where "nothing was consumed" is the whole
answer: Safe Passage beside Mending Hands in both orders, and a `Once`
half-prevention chosen against 1 damage. The payload table lists each read that
differs between the per-member loop and the per-subject one.

#### Exit criteria

1. Four PRs merged in order, each with its consumers registered and its
   predicted `PERFORMANCE_POOL` move made or explicitly declined with the A/B
   that decided it.
2. Every atom listed above annotated `COVERS:` or `COVERS-PARTIAL:` with the
   partial's reason in the test; `python plans/specdb.py owed` still 9.
3. `cargo test` green and `cargo build --all-targets` warning-free at every
   commit but the red-test commits; `tests/determinism_test.rs` and three shell
   `fuzz_games` runs at one seed line-for-line outside `=== Timing ===`.
4. §11 findings 21–27 each closed, moved or re-dated; `codebase-state.md`
   items 25, 27 and the CR 120 row updated by the PR that touches them; a
   Deferred Migrations line for every arm left absent above.
5. The trace-page decision recorded at RD-2's close; RD-5's gate decided and
   recorded either way after RD-4; `check_state_of_play.py --write` after each
   merge; `plans/handoffs/rd.md` deleted by the last RD PR to land.

**All five met at RD-4 (2026-09-09).** Four PRs, each with its consumers
registered and its `PERFORMANCE_POOL` move made — Furnace of Rath, Mending
Hands, Guardian Seraph, Pariah. Every atom the four sections list is annotated
and `owed` is 9, unchanged, as the "On filing" note said it must be by
construction. Findings 21–27 are each closed, moved or re-dated; `codebase-state.md`
items 25 and 27 and the CR 120 row are updated. RD-5's gate is closed against
it and Harm's Way is `backlog.md` §2.25, which makes RD-4 the last RD PR — so
`plans/handoffs/rd.md` is deleted here, its one category-(c) block already
carried in full by `codebase-state.md` item 90.

### Phase RE — the remaining event kinds we know of (see §8a) — sized 2026-09-11, nine PRs

**RE ships as ten PRs: seven event kinds, one PR of CR 701 producers, one that
makes a lost player leave the game, and one that gives the turn a plan.** It
was sized at seven the morning of 2026-09-11, re-cut to nine the same afternoon
on review, and to ten at RE-1's own review that evening — §11 item 49, which
found decision 6's "written once" promise true of the turn level and false one
level down. Every addition is a cost-of-delay argument rather than scope creep: the producers had a
precedent the first cut missed (RD-1 landed `Primitive::Mill` because a rider
needed it), and the leave-the-game rules become *reachable and wrong* the day
the game's-end PR lands its four-player fuzz mode. Two more things moved on
the same review — skips go first, because they are item 6's prerequisite, and
`advance_turn` is written as a turn queue so `backlog.md` §2.17 does not
rewrite it a second time (**true of §2.17's turn half only** — the phase half
needs a cursor change, §11 item 49); and counters on players (§2.16) join the
counters PR while the type is on the table.

RD was one kind cut four ways along its mechanisms; RE is seven kinds that share
one mechanism — each adds a `GameAction` family, its `EventPattern` arm, a
performer that moves an existing direct write behind the chokepoint, and printed
cards that watch it. `engineering-practices.md` §4's seam test cuts between
every pair of kinds and inside none of them, and no two kinds share a consumer
except where one card carries two statics (Doubling Season, Alhammarret's
Archive), which is an ordering constraint and not a merge. Every number below
was read from the tree and from Scryfall on 2026-09-11, before a line of code,
and **the census disagreed with the paragraph it replaces in both directions**.
Branches are `replacement/re-<n>-…`.

The paragraph it replaces, kept for the record: *draw replacement (614.11,
614.11a/b, 121.2a's outer event, 121.6a empty library), skips (614.10/a/b —
per-player consumable `pending_skips` consulted at step/phase/turn begin), token
and counter doublers (614.16 — `CreateTokens` as a proposal, which is also where
RC-4b's token residual lands: a creation whose destination the entry's decision
sets, `codebase-state.md` item 52 and "Before card breadth" item 8 — a Phase 8
back-stop, not a nicety; and where `Primitive::CreateToken`'s loop over
`propose_entry` stops, which is the other half of `codebase-state.md` item 46
and the first plural entry the engine will produce), life-gain replacement
(119.10), mana replacement (106.6a), and the three kinds §8a's audit added:
`PlayerLoses` / `PlayerWins` (CR 104, 6 cards) and the discard pattern arm
(CR 701.9, 17 cards).* That is still the scope, minus one mechanism that should
not be built, plus two debts it never named and two neighbours the review pulled
in.

#### The census — the tree against the paragraph

`GameAction` has twelve variants and `EventPattern` seven arms (`CounterChange`
covers two variants). **Three variants have no arm and are RE's**: `DrawCard`,
`GainLife`, `LoseLife`. The fourth, `Attach`, has none on purpose. Three
`match`es are exhaustive over `GameAction` — `subject_of`, `event_amount`,
`perform_action` — so a new variant costs three compiler-forced arms plus one
`pattern_watches` arm, which is not forced: it falls through to `false`, and a
variant with no pattern arm is silently unwatchable, which is `Attach`'s
intended state and the trap for everything else. Counted per kind:

| Kind | The paragraph said | The tree on 2026-09-11 | Printed customers (Scryfall, same day) |
|---|---|---|---|
| **Skips** | `pending_skips` counters | nothing — and `pending_skips` is the wrong shape (decision 6); `GameEvent::{TurnBegin, PhaseBegin, StepBegin}` exist and are emitted **nowhere**, which item 6's 2,656 "at the beginning of" triggers (§8b) will read | "skip" **58**: draw step 18, turn 22 ("your next turn" 9), untap step 8, combat 6, upkeep 2; static "Players skip their …" 2; "each player skips" 1; 614.10b's "skip … then" **0**. "doesn't untap during" 249 is `backlog.md` §2.14's, not a skip |
| **Draw** | a `DrawCard` arm | `DrawCard { player }` exists, no arm, no count field; two producers (`Primitive::DrawCards` loops it, the draw step proposes it), one performer (`draw_card`); `apply_replacements`' `inherited` set handed `{}` at its one call site (item 29) | `o:/would draw a card/` **45**; `o:/if you would draw/` 26; "draw two cards instead" 12; "except the first one you draw" **10**; "next time you would draw a card this turn" 8 (the five *Words of* and three more); Lab Maniac's shape 5; "two or more cards" 2 |
| **Life** | `GainLife`/`LoseLife` arms | both exist, no arm; `LoseLife.cause` since RD-1; `never_happens` already drops a 0 gain (119.10) | "would gain life" **21** (six of them "twice"); "would lose life" 2; Ali from Cairo's clamp **7**; "can't gain life" **25** and "can't lose life" 2 — RS's, and `Restriction::Event` has **no `PlayerSet`** to hold them |
| **Tokens** | `CreateTokens` | no variant; `Primitive::CreateToken` (one producer, Kalitas's rider) loops `propose_entry` one token per batch; a substituted token entry logs `from: Battlefield` (item 52) | "would create one or more tokens" 9; "twice that many … tokens" **10**; "would create … instead" 15; "that many … tokens are created instead" 9; "can't create tokens" 0 |
| **Counters** | doublers on `CounterChange` | arm exists since RB; `AddCounters` carries no *putter* and no *player* subject (`PlayerState.poison_counters: u32` hard-codes one kind); entry counters bypass it — `place_on_battlefield` calls `add_counters` directly (`game_state.rs:741`), which CR 122.6 says a doubler must see | "counters would be put on … instead" **21**; "that many plus one" 18; "would put one or more … counters" 7; "would get counters" 1 (a player); "can't have counters" 4 |
| **Losing / winning** | `PlayerLoses`/`PlayerWins`, 6 cards | nothing: four SBA loops write `player_lost` directly (main item 6), `GameResult` lives on `Game`, `has_drawn_from_empty_library` is set and **never cleared**, and `advance_turn` and the priority loop rotate over lost players; nothing removes a lost player's objects (item 108) | "would lose the game" **4** (three "if", one "next time"); "would win the game" **0**; "can't lose" 11 and "can't win" 9 — RS's; effects that say "you win" 38 and "loses the game" 54, nearly all triggers |
| **Mana** | `ProduceMana` | nothing — **two silent writers** (`mana.rs:91`, `resolve.rs:337`) and `GameEvent::ManaAdded` emitted at zero sites: RA's census missed it because nothing emitted | "if you tap a permanent for mana" **2**; "produces twice/three times" 3; "would add … mana … instead" 1; "for mana, add an additional" 8 — triggered mana abilities, CR 605.1b, item 6's |
| **Discard** | "the pattern arm does not [exist]" | **it does**, since RB: `EventPattern::ZoneChange { cause: Some(Discarded) }`. What is missing is a producer — `Primitive::Discard` is `NotImplemented`; the cleanup discard is the only effect site and `Cost::Discard` the only other — and two pieces decision 8 names | "causes you to discard" **17**, sixteen of them "a spell or ability *an opponent controls* causes"; "would discard" 0 |
| **Scry** (§10's Eligeth test) | "RE at the earliest" | `Primitive::Scry` is `NotImplemented` | "would scry" 2 |

Two corrections the census forces, both argued below rather than absorbed:
**skips go through the pipeline** with the proposal built by the turn machinery,
exactly as §11 item 6 already said, and the paragraph's `pending_skips` is
struck; and **discard is not an RE kind, but it is an RE PR** — its arm exists,
and what the seventeen cards need is a producer, which RE-8 builds on RD-1's
precedent (decision 8). The first cut of this section sent that producer to
Phase 8 on §8a's sentence; the review asked what the delay would cost, and the
answer was item 6's 1,045 discard-watchers testing against fixtures and a pool
that never discards outside cleanup. Two debts the paragraph did not name and
RE inherits: mana production is a chokepoint violation today (a direct write
with no event), and a lost player keeps taking turns in any game with three or
more — the two-player assumption `CLAUDE.md` said would hide in `PlayerLoses`,
found where it said.

#### The design check — nine decisions

The titles, in order, so `§9 RE decision N` still lands; each was read against CR 104, 106.6a, 119.5, 119.10, 121.2–121.6, 122.1, 122.6, 500.7, 500.11, 614.1b, 614.10–614.11, 614.16, 616.1g, 616.2, 701.9, 701.22, 704.5a–c, 704.7 and 800.4 before a line of RE was written.

**0. Seven kinds, one algebra, no new `Rewrite` arm.**

**1. Draw is two events, and the instruction is the outer one.**

**2. Life gains its arms, `LoseLife`'s `cause` gets its first reader, and the one new `AmountRewrite` reads player state.**

**3. `CreateTokens` is the outer event, each entry is contained, and a substituted entry is a creation somewhere else.**

**4. The entry is a door for the counter pattern too, who puts the counters on is a fact on the event, and a player can be the subject.**

**5. Losing and winning are proposals; the SBA sweep builds them as batch members; CR 704.7's collapse gains a per-player leg.**

**6. Skips are the pipeline's; the proposal is built by a turn queue that `advance_turn` drains; `pending_skips` is struck.**

**7. Mana is one proposal from two silent writers, and "tapped for mana" is a fact about the activation.**

**8. A CR 701 producer that a replacement customer is waiting on lands in RE, on RD-1's precedent.**

→ Each decision's argument, its rule numbers and its consumers: `plans/archive/replacement-architecture-landed.md`, "The design check — nine decisions" (evicted 2026-09-15).

#### Why ten, and the count

| PR | Shape | Measured size | Risk |
|---|---|---|---|
| **RE-1 — skips, and the turn queue** — ✅ landed | `BeginTurn`/`BeginPhase`/`BeginStep`, three arms, three small performers emitting the three begin events; `advance_turn` as a queue drainer with proceed-past; `Primitive::ExtraTurn` and CR 500.7's order | Predicted **~1,700–1,900**; shipped **+1,770 / −155** before the docs. The three exhaustive matches and `pattern_watches` were exactly as counted; `Game::setup`'s "first turn" was a *new* site rather than an existing one, and `turn_rotation` was a field nobody predicted (§11 items 47, 48) | medium — and the risk landed where it was named: the fixtures, six of which counted turn positions by hand |
| **RE-2 — draw** | `DrawCards` outer + `DrawCard.cause`; two pattern arms; `GameActionTemplate::DrawCards { n, player }`; the outer performer's decomposition with the inherited applied set (item 29's producer) | exhaustive matches **3** + `pattern_watches` **2**; `DrawCard` producers **2** rewritten to the outer, performer **1**, test constructions **1**; `execute_batch_inner`'s `inherited` **1** call site; `Primitive::DrawCards` **1**. Predicted **~550 engine, ~350 cards, ~800 tests ≈ 1,700–1,900** | **highest** — the first decomposition, whose defect is a hang, and the `cause` stamping rule across nested outers |
| **RE-3 — life** | `EventPattern::GainLife`, `LoseLife { cause }`; `AmountRewrite::LifeFloor`; `GameActionTemplate::{GainLife, LoseLife} { amount: TemplateAmount }`; `Restriction::Event.affected_players` | `pattern_watches` **2**; `apply_rewrite`'s `Amount` arm **1** and `Instead` arm **2**; `Restriction::Event` constructions **~6** + `is_prohibited`'s union **1**; `GainLife` producers **2**, `LoseLife` **3**, untouched. Predicted **~350 engine, ~450 cards, ~600 tests ≈ 1,400–1,600** | low-medium — patterns over events that already flow; the clamp is the one new arithmetic |
| **RE-4 — tokens** — ✅ landed | `CreateTokens` + its pattern arm; the plural entry batch (item 46); `CreateTokenIn` and `TokenCreated` (item 52); `Amount` over a `Vec` | exhaustive matches **3** ×2 variants; `Primitive::CreateToken` **1** producer + **1** performer restructured; `apply_rewrite`'s `Instead(ZoneChangeTo)` entry arm **1**. Predicted **~600 engine, ~350 cards, ~700 tests ≈ 1,650–1,850**; shipped **+1,867 / −171** — engine 594, cards 322, tests 951 — with the two loop findings' 90 lines in it, and the three exhaustive matches exactly as counted plus `pattern_watches`, `reads_the_amount` and `display.rs` (compiler-forced); `game_state.rs` needed nothing for the variants | medium — the first performer that proposes a batch from inside a performer, and the log line item 52 is about is the test |
| **RE-5 — counters, on permanents and players** | `CounterChange`'s entry door and `by`; `AddCounters.by` and its `CounterSubject`; item 43's `EnterMods` player half; `Amount` on an entry's mods; `PlayerState`'s counter map (§2.16) | `pattern_watches` **1** more arm; `AddCounters` constructions **2** + performer **1**; `EnterMods`/`EnterModsTemplate` merge **2**; `apply_rewrite`'s `Amount` arm **1**; `poison_counters` readers **4** (one production, `sba.rs:142`) → the map. Predicted **~650 engine, ~550 cards, ~800 tests ≈ 1,900–2,100** | medium-high — top of the band; an `Amount` arm that edits `EnterMods` is new, and the subject enum touches every counter site |
| **RE-6 — the game's end** | `PlayerLoses`, `PlayerWins`, their arms; four SBA loops → batch members; 704.7's player leg; the flag reset; `GameResult` onto `GameState`; `Primitive::{LoseGame, WinGame, SetLifeTotal}` (CR 119.5); 800.4j/k at two rotation sites; `--players 4` | exhaustive matches **3** ×2; `sba.rs` loops **4**; the dedupe **1**; `check_game_over` **1** + `Game.result` readers **~4**; `advance_turn` **1**, priority loop **1**; `fuzz_games` **~50 lines**. Predicted **~550 engine, ~400 cards, ~80 harness, ~800 tests ≈ 1,800–2,100** | **high** — top of the band; the sweep's shape changes, and the N-player half is measured for the first time |
| **RE-7 — leaving the game (CR 800.4a–e, 800.4m)** | inside `PlayerLoses`' performer, as 800.4a says ("as soon as the player leaves"): owned objects leave the game with one `LeftTheGame` event each, control-changing rows in the departed player's favor end, their stack objects not represented by cards cease, objects they still control are exiled through `change_zone` with a new cause; 800.4b/d refusals at `propose_entry` and the token performer; 800.4e at combat assignment; 800.4m on the three duration registries | the five zone collections + the stack **6** sweeps; `ContinuousEffect` rows keyed by controller **1**; `propose_entry` **1**, `CreateTokens` **1**, `assign_combat_damage` **1**; `remove_expired_at_turn_start` **3**. Predicted **~400 engine, ~450 tests ≈ 800–950**, no cards: Act of Treason is in the pool and is the consumer both ways round | medium — the first sweep that removes objects from every zone at once, and the four-player fuzz is the only board that runs it unforced |
| **RE-8 — the producers (CR 701.9, 701.22)** — ✅ landed | `Primitive::Discard` with 701.9b's chooser; **`ReplacementDef::by`** rather than the `caused_by` this row sized onto the zone-change pattern; `GameAction::Scry`, its arm, `Primitive::Scry` and two scry choice kinds. **The to-battlefield leg did not ship** — every printed customer is on a card in hand, which is CR 113.6 and critical-path item 6a (§11 item 87) | `resolve.rs` stubs **2** made real; `pattern_watches` **1** arm and no field, the cause predicate having moved off the pattern; exhaustive matches **3** for `Scry` plus `display.rs`, compiler-forced as in RE-4; `DecisionProvider` impls **0** — the CLI's prompt text only, the others being generic. Predicted at the design check **~200 types, ~430 engine, ~330 cards, ~600 tests, ~15 harness ≈ 1,575**; shipped **+1,737 / −115** — types 195, engine 445, cards 371, tests 710, harness 16 | low-medium, and it landed there; the risk that showed up was neither producer but CR 514.1's cleanup discard, which had been N events where the rule says one |
| **RE-10 — extra phases, and the turn plan** — ✅ landed | `TurnPlan` + `PlannedPhase`; `drain`'s cursor becomes an index; `next_phase`'s chain deleted; `Primitive::ExtraPhases` splicing at the cursor; `Primitive::Untap` gains the `FilteredPermanents` arm | `next_turn_unit` **1** and `drain` **1** (the cursor), `next_phase` **1** deleted + **~8** readers; `GameState` **1** field, seeded **1** and rebuilt **1**; `Primitive` exhaustive matches **1**; `Primitive::Untap`'s recipient **1**. Predicted **~700 engine and cards, ~400 tests ≈ 1,100–1,300**; shipped **+1,066 / −171** — engine 149, state 136, types 24, cards 147, tests 600, seam 10. **Three of this row's counts were wrong** (see the section's findings): `next_phase` had one production caller and not ~8 readers, its wrap arm was already dead, and "re-counts nobody's fixtures" missed 46 hand-written positions. The split inverted — the engine half came in at 466 against ~700 because the seam's 46 sites are one line each, and the test half at 600 against ~400 | low-medium — the second and last rewrite of `advance_turn`, and the first turn structure that is data rather than a `match`; it changes no turn's *shape*, so nothing else's fixtures move |
| **RE-9 — mana** — ✅ landed | `ProduceMana`, its arm, one performer replacing two writers, `ManaAdded` emitted, `tapped_for_mana` from the activation cost; **and**, added at the design check on §4's eighty-line rule, `GameActionTemplate::ProduceMana` for CR 106.12b's six type-changers, with Deep Water | exhaustive matches **3** + `reads_the_amount` + `display.rs`; writers **2** → **1**; `resolve_mana_effect` **1** (plus a signature the row did not count), `Primitive::ProduceMana` **1**; `pattern_watches` **1**; `substitute` **1** leg and three predicate arms. Predicted **~300 engine, ~200 cards, ~450 tests ≈ 950–1,150**, re-counted at the design check to **1,300–1,450**; shipped **+1,423 / −29** — engine 334, cards 285, tests 734, harness 14 | low on shape, **the one whose A/B could say no** — a proposal on every land tap. **It said yes: +1.2% at two seats, under the 2.5-point gate, and lever 2 stays unbuilt** |

**≈ 14,300–16,300 across ten, each inside the band, RE-5 and RE-6 at its top
and RE-7 and RE-10 below its floor.** Hard orders: RE-2 → RE-3 (Words of Worship needs the draw pattern and
the life template; Alhammarret's Archive needs both halves); RE-3 → RE-8
(Eligeth's `ReplacedAmount`); RE-4 → RE-5 (Doubling Season is registered whole,
in RE-5); RE-2 → RE-6 (Laboratory Maniac's draw; Stunning Reversal's rider
draws seven); RE-6 → RE-7 (the performer and the fuzz mode). RE-1 first, on
decision 6's argument, and it depends on nothing; RE-9 last, on decision 7's.
**RE-10 depends on RE-1 and nothing else, nothing depends on it, and its cost
of delay is near zero** — it changes `advance_turn`'s *cursor* and not a turn's
*shape*, so unlike RE-1's CR 508.8 refusal it re-counts nobody's fixtures. It
is the one RE PR that can be slotted wherever it is convenient.
The rest commutes. The counts are call sites read from the tree on 2026-09-11;
the line predictions are calibrated against RD-1 (+1,950 against a
~1,500–1,700 prediction — the card files' rulings pass was the difference),
RD-2 (+2,393) and RD-3 (+2,035 against ~1,400–1,600), so the card columns
above are written with the rulings pass in.

#### RE-1 — skips, and the turn queue (CR 614.1b, 614.10, 614.10a, 500.7, 500.11) — ✅ landed 2026-09-11

**Shipped.** `GameAction::{BeginTurn, BeginPhase, BeginStep}` with their three
`EventPattern` arms and three performers, emitting the three `GameEvent`s the
log has carried since it was written and nothing emitted — which is what item
6's 2,656 "at the beginning of" triggers read. `advance_turn` rewritten as a
drainer over CR 500.1's sequence with CR 500.11's proceed-past at all three
levels; `GameState::turn_queue` (CR 500.7's stack) fed by
`Primitive::ExtraTurn`, and `turn_rotation` beside it so an extra turn does not
move the natural rotation. `Rewrite::Prevent` is the skip; no new `Rewrite`
arm. CR 508.8 and CR 800.4k joined the proposal site as rules ahead of the
pipeline. Eon Hub (pooled), Yawgmoth's Bargain, Meditate, Time Walk, Moment of
Silence; `backlog.md` §2.17 graduated.

**Two things the sizing did not have.** `turn_rotation` is a field nobody
predicted and it is what makes the queue N-player-shaped (§11 item 47), and
`Game::setup` never began the first turn at all, so turn 1's untap step had run
no turn-based action in any game the engine has played (§11 item 48).

→ The section as sized, what the building changed and the measurement:
`plans/archive/replacement-architecture-landed.md`, "RE-1" (evicted
2026-09-11).

#### RE-2 — draw (CR 614.11, 614.11a, 121.2, 121.2a, 121.6a/b, 616.1g) — ✅ landed 2026-09-11

**Shipped.** `GameAction::DrawCards { player, n, cause }` as CR 121.2a's
instruction and `DrawCard { player, cause }` as the draw, with
`EventPattern::DrawCards { at_least }` and `DrawCard { cause }`,
`GameActionTemplate::DrawCards { n, player }`, and the outer performer's
one-at-a-time decomposition handing each inner the applied set its own CR 616.1
loop accumulated — the first producer of `apply_replacements`' `inherited`,
empty at its one call site since RB (§11 item 18, `codebase-state.md` item 29).
`DrawCause { TurnBased, Effect }` is stamped by the outer's performer off the
loop index and inherited through a substitution, which is the whole of "except
the first one you draw in each of your draw steps" and needs no count of cards
drawn this step. Both producers rewritten; `Game::setup` still calls the
performer (CR 103.4) and its comment now says so. Thought Reflection (pooled),
Teferi's Ageless Insight, Alms Collector, Notion Thief.

**Three things the sizing did not have.** Alms Collector is not the `then` case
§3.2d filed it as, and its own ruling says so — as `Prevent` plus riders the
card is an infinite loop against an opponent's Thought Reflection, and CR
614.5's "any modified events that may replace that event" is why half of it has
to be the rewrite (§11 item 53, and §3.2d is corrected in place for the second
time). `GameState::draw_cards` had had no caller since before RA and is an
N-draw path with no proposal at all (item 50). And the first three-round timing
said +6.0% on a 27% outlier; seven rounds say **+0.5%** (item 54).

**Trace page:** `plans/traces/re-2-a-draw-carries-its-lineage.html` — two
Thought Reflections, Teferi beside one in the draw step, Alms Collector in both
encodings with the loop the sized one produces, and the three-Thief board.

→ The section as sized, what the building changed and the measurement:
`plans/archive/replacement-architecture-landed.md`, "RE-2" (evicted
2026-09-11).

#### RE-3 — life (CR 119.10, 119.7's "can't gain", the CR 120.3a loss as a replaceable event) — ✅ landed 2026-09-12

**Shipped.** `EventPattern::{GainLife, LoseLife { cause }}`,
`AmountRewrite::LifeFloor(i64)`, `GameActionTemplate::{GainLife, LoseLife}` over
a new `TemplateAmount { Fixed, ReplacedAmount }`, and
`Restriction::Event.affected_players: PlayerSet` unioned by the same
`set_affects` the other two pairs use — so `is_prohibited` refuses a `GainLife`
ahead of the pipeline and §11 items 26 and 45 both close. Rhox Faithmender
(pooled), Tainted Remedy, Words of Worship, Ali from Cairo, Alhammarret's
Archive, Skullcrack. `is_prevention` untouched, as the section required.

**Four things the sizing did not have.** `EventPattern::LoseLife` carries a
`LifeLossCausePattern` and not a `LifeLossCause`, because the cause's `Damage`
arm holds the damage's *source* and Ali is about damage from anything
(`DestructionSourcePattern` is the precedent). `Primitive::Restrict` could only
build an object-scoped row, so Skullcrack — its first printed customer — moved
the target-filling from a demand to a marker (§11 item 57). `LifeFloor` is the
first `AmountRewrite` arm `apply` cannot answer for, which is why the damage and
gain legs refuse the pairing rather than letting it return a plausible number.
And the CR 616.1 suppression premise was written about damage for the third
time, so it is now a property of `EventPattern` rather than a list of kinds
(item 58).

**Decided here, and written down**: Ali from Cairo does not clamp a
`LifeLossCause::Cost`, structurally and twice over; `never_happens` gains no
`LoseLife` arm, on RE-2's reasoning; `Restriction::Event.affected_players`
unions rather than replaces, and `PlayerSet::Nobody` on an object-only
restriction keeps meaning what it always meant.

**Trace page: no**, decided at the close — `engineering-practices.md` §7 named
RE-2 and RE-4 and not this one, and the premise generalization does not change
it: moving a read's answer from a list of shapes to a property of the type
answers the *same* question in the same place. The rule that would catch it is
"how a read is answered", and nothing here reads differently.

→ The section as sized, what the building changed and the measurement:
`plans/archive/replacement-architecture-landed.md`, "RE-3" (evicted
2026-09-12).

#### RE-4 — tokens (CR 614.16's token half, 111.5, 616.1g; items 46 and 52) — ✅ landed 2026-09-13

**Shipped.** `Primitive::CreateToken` resolves as one `GameAction::CreateTokens
{ defs, controller }`, whose performer creates the objects and proposes every
entry as one contained batch (item 46's producer; CR 616.1g as the order of
two loops); `EventPattern::CreateTokens` and `Rewrite::Amount(Multiplier)`
over the `Vec`, repeating each def in place; a token's substituted entry is
`GameAction::CreateTokenIn { object, zone }` — an appearance, announced as
`GameEvent::TokenCreated`, with no `ZoneChange` at all (item 52); `TokenDef`
can say an ability, a supertype, rules text and an enchant filter, its name
is CR 111.4's default when the effect gives none, and its power and toughness
are `Option`s. Parallel Lives and Raise the Alarm pooled; Hordeling Outburst
and Hallowed Moonlight registered. Sized 1,650–1,850, shipped **+1,867 /
−171** before the docs, two of them older defects the four-player A/B reached
(§11 items 77–78: a rider carries the replaced event's applied set, and the
lineage assertion is per lineage). `PERFORMANCE_POOL` 81 → 83. **Trace page: no**,
decided at the close. The design record, the shape as built, the measurement
and the findings are in `plans/archive/replacement-architecture-landed.md`,
"RE-4"; `codebase-state.md`'s RE-4 lines (items 126–128) are what was left
absent, each with its customer named. The review added a rider carrying its
lineage (§11 item 77), the one-exit suppression shape (81), and the creation
pattern's kind, the creation template and two more cards (82).

#### RE-5 — counters, on permanents and players (CR 614.16's counter half, 122.1, 122.6, 122.6a; item 43, `backlog.md` §2.16) — ✅ landed 2026-09-13

**Shipped.** A counter's subject is an object or a player —
`CounterSubject` on `GameAction::AddCounters` and `RemoveCounters` and on
`GameEvent::CountersChanged` — and the putter rides on the event as
`AddCounters::by`. CR 122.6's entry counters are watched through a second
door on `EventPattern::AddCounters`: an `EnterBattlefield` whose mods carry
a matching kind with one or more, each row carrying its putter — the player
the effect named, else CR 122.6a's default, the entry's controller;
`Rewrite::Amount` rewrites a proposal's count and each matched kind in an
entry's mods. `PlayerState.counters` is
the kind → count map (`Poison`, `Energy`), CR 704.5c reads it, and
`Primitive::GetCounters` is Oracle's "you get". Doubling Season whole,
Hardened Scales (pooled), Vorinclex, Monstrous Raider, Winding Constrictor,
Live Fast, Primal Vigor. Sized ~1,850, shipped **+1,911 / −101** before the
docs. `PERFORMANCE_POOL` 83 → 84. Item 43 closed and built — at the review,
after a first close on an empty Scryfall query that the owner's rule rejects
(§11 item 83); the review also split `CounterChange { adding }` into
`AddCounters` and `RemoveCounters`, one arm per variant. Item 47's condition (c) fired from the multiplier side and the
predicate's entry clause is the re-derivation (item 84); the additive pairs
RE-5 makes reachable are `backlog.md` §2.29's next rows, not a shape
(item 85); a cost's counters are `codebase-state.md`'s "Found by RE-5" line.
**Trace page: no**, decided at the close. The design record, the shape as
built, the measurement and the findings are in
`plans/archive/replacement-architecture-landed.md`, "RE-5".

#### RE-6 — the game's end (CR 104.2b, 104.3e, 104.4a, 704.5a–c, 704.7, 119.5, 800.4j–k) — ✅ landed 2026-09-12

**Shipped.** `GameAction::{PlayerLoses, PlayerWins}` with two fieldless
`EventPattern` arms and `GameActionTemplate::PlayerWins`; the four state-based
loss loops as members of the CR 704.3 batch, deduped per player by
`subject_of` (704.7's per-player leg); CR 704.5b's window closed at the check;
`GameResult` on `GameState`, written by the `PlayerWins` performer and by a
batch's settlement of CR 104.2a/104.4a; CR 104.1 at the chokepoint, the check
and the priority loop; CR 800.4j at the rotation and the three turn-based
actions a departed active player has nobody to perform; `Primitive::{LoseGame,
WinGame, SetLifeTotal}`, `Primitive::Exile` made real, `AmountExpr::StartingLifeTotal`,
`Condition::LibraryEmpty` with the gather's "as long as" leg; `fuzz_games
--players N`. Laboratory Maniac (pooled), Exquisite Archangel, Stunning
Reversal, Platinum Angel. Items 6 (the loss half), 112, 113 (the priority
half) and 123 close; 108 is re-dated and measured; 122 is re-owned.

**Four things the sizing did not have.** CR 104.1 is a line at the top of
`execute_batch_inner`, so a decomposition's later inners and the riders of the
batch that ended the game perform nothing (§11 item 62). CR 704.3's "performed"
is read off the event log, because a loss Exquisite Archangel replaced performs
no member but its rider does, and Stunning Reversal's "immediately after" needs
that to count (item 61). `Primitive::Exile` and CR 608.2m at the stack were
nowhere in the row: the Archangel's rider and Stunning Reversal's second
instruction both exile the effect's own source. And the four-player run's first
finding was the harness's own board — a departed seat still offered as an
attack target (CR 506.2), fixed here (item 66).

**Decided here, and written down**: `Condition::LibraryEmpty` is asked at
gather through `settled_holds` — CR 604.2 and 614.4, with CR 121.6a putting the
proposal in front of that gather (item 64); `GameResult` is the chokepoint's,
settled per *batch* and never per member, which is the four-loss Stunning
Reversal board (item 62); `never_happens` gains no arm, since the game's end
has no amount for a zero to mean anything, and a departed player not losing
again is CR 800.4k's gate at the proposal rather than a non-event.

**Trace page: no**, decided at the close — the same CR 616.1 loop over a new kind, and the two boards worth walking are two tests.

→ As sized, as built and as measured: `plans/archive/replacement-architecture-landed.md`,
"RE-6" (evicted 2026-09-12).

#### RE-7 — leaving the game (CR 800.4a–e, 800.4c, 800.4m) — ✅ landed 2026-09-13

**Shipped.** CR 800.4a's four clauses inside the `GameAction::PlayerLoses`
performer, as the rule's own "this is not a state-based action. It happens as
soon as the player leaves the game": `engine/leaving.rs`'s
`owned_objects_leave`, `end_control_given_to`, `uncarded_stack_objects_cease`
and `exile_objects_no_player_in_game_controls`, with
`GameState::remove_from_game` as the performer for a destination CR 400.11 says
is not a zone and `GameEvent::LeftTheGame` as its emitter. Clause 4's exile and
CR 800.4c are one predicate — the effective controller is not in the game —
asked at the departure and again at the two moments a control-changing effect
can end. CR 800.4b and 800.4d's token half are refusals at `propose_entry`,
`Primitive::CreateToken` and Layer 2's `SetController`; CR 800.4e is one
`retain` over the finished combat-damage assignments; CR 800.4m expires a
departed seat's "until your next turn" rows in `next_turn_taker`, at the
turn boundary that skips them. No cards — Act of Treason is the consumer both
ways round.

**CR 800.1 is the gate on all of it**, and it is what makes "byte-identical at
two seats" a fact about the rules rather than about the code:
`GameState::is_multiplayer` reads the seat count the game *began* with, and
CR 800.4's own first sentence is "unlike two-player games, multiplayer games can
continue after one or more players have left the game".

**The CR 704.3 batch performs its losses last**, because a loss is the only
member whose performer removes other members' subjects and every performer is
loud about the board it finds — a member that removes objects performs after the
members decided against them.

**Measured:** "Departed-owned permanents" **32.2 → 0.0** and 34.2 → 0.0 at four
seats, closing `codebase-state.md` item 108; CPU/game −15.8% on a board 32
permanents smaller, against `Layer walks` +4.5% for CR 603.6c's frame; both
two-player pools `IDENTICAL` outside `=== Timing ===`. **Trace page: no**,
decided at the close.

→ As sized, as built and as measured: `plans/archive/replacement-architecture-landed.md`,
"RE-7" (evicted 2026-09-13).

#### RE-8 — the producers (CR 701.9, 701.9b, 701.22) — ✅ landed 2026-09-14

*Body evicted to `plans/archive/replacement-architecture-landed.md` under the
same heading: the section as sized, the design check's nine decisions, what was
built, what it measured, and the trace-page argument.*

**Shipped:** `Primitive::Discard(n, DiscardChooser)` with CR 701.9b's default
and "at random" choosers, N cards as one batch of N members; `GameAction::Scry`
with `EventPattern::Scry`, `GameEvent::Scried` and a performer that reorders
the library in the arm and proposes nothing; CR 701.22b as `never_happens`'
fourth arm; `ChoiceKind::{Discard, Scry, ScryOrder}`; `ReplacementDef::by:
Option<SourceFilter>` — CR 101.2's "by" asked of a replacement effect, and
**not** the `caused_by` on `EventPattern::ZoneChange` decision 8 wrote, because
`Restriction::Event` reuses that pattern and already carries a `by`;
`GameActionTemplate::DrawCards.n` as a `TemplateAmount`. CR 514.1's cleanup
discard reshaped to the same one-batch unit the rule describes. Mind Rot, Hymn
to Tourach, Nephalia Academy, Opt and Eligeth, Crossroads Augur; **`PERFORMANCE_
POOL` +2**, Mind Rot and Opt.

**Not shipped, with the facility named:** the to-battlefield entry
substitution and the five cards that print it — Dodecapod, Wilt-Leaf Liege,
Loxodon Smiter, Nullhide Ferox, Obstinate Baloth. All five put the clause on a
card in **hand**, and `gather` has no source that asks one; that is CR 113.6,
critical-path item 6a, sized at §11 item 9 and found here by §8's rules pass
(§11 item 87). Nephalia Academy is a *Land* and is what gave `by` a printed
customer without it.

**Measured:** a **fifth** arm with CR 514.1's old loop restored is `IDENTICAL`
to `main` on both pools at both seat counts, which attributes the engine arm's
whole movement to that reshape and leaves the two producers costing a board
with nothing watching them exactly nothing. Pooled is a re-record. → the
archive, and `plans/fuzz-record.md`.

#### RE-9 — mana (CR 106.6a, 106.12; RA's unnamed debt) — ✅ landed 2026-09-15

*Body evicted to `plans/archive/replacement-architecture-landed.md` under the
same heading: the section as sized, the design check's thirteen decisions and
their review, what was built, and what it measured.*

**Shipped:** `GameAction::ProduceMana { player, source, mana, special,
tapped_for_mana }` — CR 106.12b's "mana production event" — proposed by the two
writers that had written the pool directly since Phase 5-Pre and performed by
one arm; `GameEvent::ManaAdded` emitted for the first time, its `HashMap` a
`Vec` (§11 item 96); `tapped_for_mana` read off the activation cost, which is
CR 106.12's definition and not the ruling decision 7 leaned on (item 93);
`EventPattern::ProduceMana { tapped_for_mana, source }`; `Amount(Multiplier)`
over every unit with the restricted atoms repeated, CR 106.6a's "all mana
produced" (item 95); `GameActionTemplate::ProduceMana { mana_type, amount }`,
CR 106.12b's "specific type", added at the design check on §4's eighty-line
rule (item 94); the `Mana productions` diagnostic row. Mana Reflection
(**`PERFORMANCE_POOL` +1**), Nyxbloom Ancient, Deep Water; Contamination's and
Infernal Darkness's lines as fixtures until item 6 owns their upkeeps.
`ATOM-106.6a-001` and `ATOM-106.6-001` covered, `ATOM-106.12a-001` partial.

**Not shipped, with the facility named** (`codebase-state.md`, "Found by
RE-9", items 132–135): the pattern's `mana_type` field (False Dawn); Hall of
Gemstone's chosen color, Naked Singularity's per-subtype map, Harvest Mage's
choice inside the substitution; a mixed production under `Fixed`, refused
rather than guessed; and the `EngineCounters` rename (§11 item 97), landed as A4m.

**Measured:** the engine arm **identical to `main`** on every gameplay and
layer row at two seats and four; gathers +80 / +151 a game against 81 / 151
productions; **CPU/game +1.2% and +0.7%**, rounds straddling — under §11 item
54's 2.5-point gate, so **lever 2 stays unbuilt**, as decision 11 predicted.
Pooled is a re-record and a bigger board. At review, two more arms: a mana
replacement *applying* on every tap, game held fixed, is **+2.7% / +2.2%**,
and the gate returns **nothing** for this kind — the bare +1.2% is the
chokepoint's fixed work (§11 item 99, item 136). → the archive, `fuzz-record.md`.

Shipped **+1,423 / −29** before the docs — engine 334, cards 285, tests 734,
harness 14 — against 1,300–1,450. **RE closes with it: ten PRs, no handoff
left open, `owed` still 9.**

#### RE-10 — extra phases, and the turn plan (CR 500.8, 500.11, 505.1) — ✅ landed 2026-09-14

*Body evicted to `plans/archive/replacement-architecture-landed.md` under the
same heading: the section's four design decisions, the six the tree posed and
this PR answered, the three of the row's counts that were wrong, and what it
measured.*

**Shipped:** `GameState.turn_plan` — CR 500.1's phase sequence as a `Vec` the
drainer indexes, rebuilt in place per turn, so a skipped turn builds no plan
(CR 614.10a for free); `PlannedPhase`, holding a phase and **not** its steps,
which is where CR 500.10's `Option<Vec<StepType>>` goes when item 6 makes it
buildable; `TurnUnit::Phase(usize)`; `state::game_state::next_phase` **deleted**,
its wrap arm having been dead already. `Primitive::ExtraPhases(Vec<PhaseType>)`
splices at `cursor + 1`, and CR 500.8's *"most recently created phase will occur
first"* is that splice's own behaviour with no comparator anywhere.
`Primitive::Untap` gains the `EffectRecipient::FilteredPermanents` arm and both
its recipient paths join one batch (CR 603.2c). Aggravated Assault, registered
**unpooled**; `PERFORMANCE_POOL` +0 as predicted, with a `--require` row instead.

**The cursor is the drainer's, not the performer's** — RE-1's own sentence about
`turn_queue` one level down, since a cursor is *schedule* rather than board.
That is what kept this a tenth PR instead of a rewrite of RE-1: `BeginPhase`,
its pattern arm and its event are untouched, and the PR adds no event kind.

**What the row did not price:** the position is two facts now, and **46 sites
wrote one of them by hand**, 19 of which then drain. `GameState::set_turn_position`
is the seam; the `debug_assert` in `advance_turn` found all 40 affected tests in
one run. → §11 item 49, closed with that.

**Measured:** `performance` counters **IDENTICAL** to `main` — every row — and
CPU **+0.1%** at `--rounds 7`. No `fuzz-record.md` block: no table moved.
Shipped **+1,066 / −171** against a predicted 1,100–1,300.

#### Out of RE, decided rather than absorbed

- **`pending_skips`** — struck (decision 6). Not a deferral: a mechanism that
  should not be built.
- **CR 802's defending player**, and 800.4f–h's choices by a departed player
  — B3's ("Before Commander" item 4), with RE-7 having built the seam.
- **Extra phases** (CR 500.8) — cut at RE's sizing, re-opened at RE-1's review
  and **taken in**, as RE-10 (**✅ landed 2026-09-14**): the "written once" argument that pulled CR 500.7
  into RE-1 applies one level down and was unkept there (§11 item 49), 46
  printed cards make an additional combat phase, and Moment of Silence is
  already *registered* with a ruling only a fixture covers. **Extra steps**
  (CR 500.9, 500.10) stay out and cannot come in: their only producer, Obeka,
  is a triggered ability, so they are item 6's whatever RE does — RE-10
  decision 4 names the one field they want. **Step- and phase-scoped
  durations** (`backlog.md` §2.12) — hang off RE-1's emitters; the PR after
  RE-1 when an "until end of combat" consumer appears.
- **Concession** (104.3a), **CR 104.3f**, **CR 104.4c** — each with no atom
  a test could claim or no producer; named in RE-6.
- **CR 121.2c** — each player's draws in APNAP order — needs an each-player
  recipient `EffectRecipient` lacks (RD-2's note); Phase 8's atom, no
  producer. Laboratory Maniac's second ruling is the same gap.
- **Costs paid in energy** (`{E}` in a cost) — with their first card, after
  RE-5 made the counters exist.
- **`Condition::IsYourTurn`** (Bloodletter of Aclazotz) — one customer for a
  leaf; recorded as a shape in RE-3.
- **Triggered mana abilities** (CR 605.1b, 8 cards) — item 6's. **Any-color
  mana** (`backlog.md` §2.19) — the PR after RE-9, since both rewrite
  `resolve_mana_effect`.
- **"Another player chooses" discards** (701.9b's third shape), **Chronatog's
  "activate only once each turn"**, **Lich's Mirror's shuffle**, **Library of
  Leng's hand size**, **Loxodon Smiter's "can't be countered"**, **Academy
  Manufactor's predefined tokens** (each needs a Treasure, Food or Clue the
  token vocabulary cannot express; §2.19 for Treasure) — each one facility
  away, each with one customer here.

#### Cut, and argued

RD cut `RetargetSpec::ToFixed` on "an arm the pipeline cannot apply is worse
than a missing one". RE's cuts, on the same rule and its §8c corollary ("two
customers before a leaf"):

- **No `reason` on `EventPattern::PlayerLoses`.** Every printed replacement and
  every printed "can't" applies to every reason; a field would be matched by
  nothing.
- **No outer `Discard` event.** Nothing prints "would discard"; the seventeen
  cards watch the zone change, and Library of Leng's ruling ("decide on each
  of the cards") says the instruction is not what they see.
- **No `GameActionTemplate::CreateTokens`.** Words of Wilding ("create a 2/2
  Bear instead" of a draw) wants a fixed def; Divine Visitation ("that many
  4/4 Angels … instead") wants the event's count with a substituted def. Two
  shapes, one customer each. Both recorded; the arm arrives with the second
  customer of either.
- **No prompt for CR 613.7m on a token batch** — decision 3: one outcome.
- **No `DrawCause::Cost`.** Nothing prints "draw a card" as a cost; the enum
  has the two causes the ten cards distinguish.
- **`ProduceMana` carries no "could produce" (CR 106.7)** — that is a query
  about abilities that have not resolved, not an event, and it is
  `backlog.md`'s the day a card asks.

#### Measured — what to expect, and why the direction is known

Three arms per PR, and unlike RD **five of the nine middle arms add proposals**, so the counters move on those and each PR predicts the number before running. **Corrected at RE-4's review (§11 item 80): that middle arm reads the engine on `performance` only** — on `stress` the registry *is* the pool, so a phase that wants the engine's `stress` cost builds a fourth binary with its cards unregistered.

→ Every PR's prediction and its measured answer, RE-1's +447 gathers and +5.0% CPU, and the gate that was measured twice and not built: `plans/archive/replacement-architecture-landed.md`, "Measured", RE's (evicted 2026-09-15). The tables are `fuzz-record.md`'s.

#### Trace page — ✅ written at RE-2's close; **no** at RE-1's, RE-3's, RE-4's, RE-5's, RE-6's, RE-7's, RE-8's, RE-9's and RE-10's

**RE-1: no** and **RE-10: no**, recorded at the post-RE audit (2026-09-15) —
the two RE phases this heading had not named, because neither produced a
candidate under `engineering-practices.md` §7's rule. RE-1 adds three
proposal kinds and a consumable queue: what is *proposed*, RE-4's argument.
RE-10 moves where the turn's position is stored (a `Vec` the drainer indexes)
and touches no read on any event's path. Each phase's boards are one test with
one assertion — `two_meditates_make_a_player_skip_their_next_two_turns`,
`two_extra_turns_are_taken_most_recently_created_first`, and RE-10's splice at
`cursor + 1`. Recorded here rather than in the archive's "Trace-page
decisions", which is for phases that produced a candidate and declined it.

`engineering-practices.md` §7's rule is met twice: RE-2 changes *how* the
applied set is answered for a decomposed event (a draw carries its lineage), and
RE-4 changes how an entry's frame is answered inside a plural batch (a token is
decided against a board its siblings have not entered). Neither is the happy
path, and both are the boards §3.2d and §5b argued about. Decide each at that
PR's close. **RE-2: yes** — `plans/traces/re-2-a-draw-carries-its-lineage.html`,
written 2026-09-11. It walks the three boards named here plus Teferi beside a
Thought Reflection in the draw step, and Alms Collector is traced in **both**
encodings, because the difference between them is a loop and a test can only
show that the loop does not happen.

**RE-3: no**, **RE-6: no**, **RE-7: no**, **RE-4: no**, **RE-5: no**, **RE-8:
no** — each decided at that
PR's close, each recorded because the phase produced a candidate, and each
argued where the phase's own record is:
`plans/archive/replacement-architecture-landed.md`, "Trace-page decisions". The
five share one shape, which is the summary this heading keeps: a phase that
changes what is *proposed*, or where an answer is written down, or how wide the
board is, changes no read's path. RE-4 was the second phase this section named
in advance, and the read it was named for — an entry decided against a board
its siblings have not entered — turned out to be RC-5's page's last trace.

#### Exit criteria

1. Ten PRs merged in the orders above, each with its consumers registered and
   its predicted `PERFORMANCE_POOL` move made or explicitly declined with the
   A/B that decided it; RE-9's and RE-10's decisions recorded either way.
2. Every atom listed above annotated `COVERS:` or `COVERS-PARTIAL:` with the
   partial's reason in the test; `ATOM-614.11b-001` and `ATOM-614.10b-001`
   uncovered with their reasons in the test files; `python plans/specdb.py
   owed` still 9 — none of RE's atoms is in a shipped phase, so the gate cannot
   move by construction, and the Phase 6 CR 614.10/11/16 slice is what each
   PR's list above closes.
3. `cargo test` green and `cargo build --all-targets` warning-free at every
   commit but the red-test commits; `tests/determinism_test.rs` and three shell
   `fuzz_games` runs at one seed line-for-line outside `=== Timing ===`, and
   the same for `--players 4` from RE-6 on.
4. §11 findings 42–46 each closed, moved or re-dated; `codebase-state.md`
   items 6 (the loss half), 9, 29, 43, 46, 52, 108, 111–114, "Before card
   breadth" item 8 and "Before Commander" item 4 updated by the PR that
   touches each; `backlog.md` §2.16 and §2.17 struck as graduated by RE-5 and
   RE-1; a Deferred Migrations line for every arm left absent above.
5. The trace-page decisions recorded at RE-2's and RE-4's close; §3.2d's
   Notion Thief correction and §10's Eligeth row landed with RE-2 and RE-8;
   `backlog.md` §2.17's phase half struck as graduated by RE-10, CR 500.8 given
   an atom in `session-4.md`, and `plan` added to `check_glossary.py`'s
   `POLYSEMOUS` with both senses;
   `check_state_of_play.py --write` after each merge; `plans/handoffs/re.md`
   opened by the first RE PR that spans a session and deleted by the last RE
   PR to land.

### Phase RF — the gather's zone leg (CR 113.6, §3.3 source 2) — landed 2026-09-16

#### RF — the gather's zone leg — ✅ landed 2026-09-16

*Body evicted to `plans/archive/replacement-architecture-landed.md` under the
same heading: the finding that sets the scope, the seven decisions and what
the PR refused, the pieces measured, and the A/B. Trace page:
`plans/traces/rf-a-source-off-the-battlefield.html`.*

**Shipped:** the read leg for §3.3's source 2 —
`GameState::zone_replacement_ability_sources`, a map the registration doors
keep (`arrive_in_zone`, `place_on_battlefield`, and the new `create_in_zone`
for a card created in a zone) from each object off the battlefield to its
printed replacement defs, swept after the battlefield in CR 613.7d timestamp
order with the entering object skipped and a frame computed only when a
printed def could apply; the registry summary's unattributed legs — a
`Filter` row names zones, a named row names objects, read by name; `zone_
function::functions_in` asked of every ability the sweeps read; the affected
side's zone check in `set_affects` in place of the `debug_assert`, with Rest
in Peace, Leyline of the Void and Nephalia Academy saying what their cards
say; `GameAction::ShuffleLibrary`, `GameEvent::LibraryShuffled` and
`Primitive::ShuffleLibrary` for the card's rider; Darksteel Colossus (pooled)
and Nexus of Fate (registered) with three fixtures. `Board::seed` and
`membership` untouched. The engine arm is `IDENTICAL` to `main` at two seats
and four; the numbers are the archive's "Measure" and `fuzz-record.md`'s
block. Closes §11 item 4 and critical-path 6a.

→ The section as written at the close, what the building changed and the
measurement: `plans/archive/replacement-architecture-landed.md`, "RF"
(evicted 2026-09-16).

### Interleaved — Commander

Per `CLAUDE.md`, the Commander/multiplayer track interleaves after item 5 rather
than sequencing against it. `GameConfig::commander()`, commander designation,
commander tax (CR 903.8, needs cost modification), and CR 800 priority rotation
are not gated on this doc beyond RB items 7–8.

**~~Cost modification needs a phase marker of its own~~** — ✅ it has a
document, `plans/cost-architecture.md` (2026-09-07), and CM-1 is in. The
paragraph below is kept as written. (Audit 2026-08-25):
"interleaved" has left it with no home, and it is not small — the
`apply_cost_modifications` stub, the `SourcePower`-class `AmountExpr` gap, and
CR 613.11/601.2f sequencing all live there (`codebase-state.md`, Before
Replacement effects → item 3 of the Layers section). Commander tax runs through
it, so it blocks the Commander skeleton being *playable*, not just complete.
Suggested slot: its own small phase between RB and RD, once the pipeline shape
is stable — it does not depend on RC/RD/RE and nothing in them depends on it.

---

## 10. Testing

Same discipline as the layer phases:

- **Annotate at write time.** `// COVERS:` only when the test builds the atom's
  whole scenario. 58 atoms are written directly against CR 614/615/616; 88 of
  Phase 6's 124 are replacement-family once CR 122/400/609/701 are counted in.
  Run `specdb orphans` and `specdb suspicious` before each PR.
- **Every bugfix fails first.** `git stash push mtgsim/src`, watch it fail, then
  commit. Non-negotiable per `CLAUDE.md`.
- **Mutation-check the assertions.** The Layer 2 phase found four vacuous
  assertions this way (`fc68600`). A replacement test that passes because the
  effect never fired is the exact failure this phase is prone to — assert on the
  *modified* outcome and on the pipeline having been entered, not just on the
  final board.
- **Determinism.** After RA and after RB, run `fuzz_games` three times at one
  seed from the shell; every line must match except the three timing lines
  (`Total time`, `Time/game`, `CPU/game`).
- **N-player.** `test_support::setup_game(4)` exists. Any test touching CR 616.1
  ordering gets a 4-player form, because APNAP with one nonactive player is the
  same answer as no APNAP at all.

**Four named acid tests**, each pinning a rule the design would otherwise get
quietly wrong. Write them as the phase's first tests, not its last:

| Test | Pins | Fails how |
|---|---|---|
| `test_two_teferis_draw_four_not_infinity` | §3.2d lineage inheritance on decomposition | **hangs**, not fails — give it a bounded iteration guard |
| `test_two_doubling_seasons_quadruple` | `Amount` composing inside one CR 616.1f loop (2ⁿ by the other route) | wrong number |
| `test_god_entering_does_not_count_itself_for_devotion` | §5a's filters-vs-counts boundary | silently wrong on 15 gods |
| `test_declined_optional_is_not_reoffered` | §4.1's decline path marking applied without consuming a use | hangs |
| `test_kalitas_simultaneous_deaths_each_exile_and_make_a_zombie` | §4.2 per-event applied sets — Kalitas's printed ruling | one Zombie instead of N; N−1 cards reach the graveyard |
| `test_then_rider_resolves_after_the_performed_event` | §4.1a rider timing (CR 615.5) | passes vacuously until events/LKI order is asserted — assert on the event log, not the end state |
| `test_goggles_with_eligeth_draws_two_and_never_scrys` | §4.1a's instruction split + kind-changing `Instead` (needs the `Scry` event kind — **RE-8, 2026-09-11**: the `Scry` producer lands there with Opt and Eligeth, and the test is written first as `test_opt_with_eligeth_draws_two_and_never_scrys`; the Goggles of Night form scries from a *trigger* and waits for item 6) | wrong draw count, or a scry event exists in the log |

The first and fourth hang rather than fail, which is the argument for writing
them before the code they check.

Test cards go in `src/cards/phase_r*_cards.rs`; integration tests in
`tests/phase_r*_integration_test.rs`.

---

## 11. Findings and open questions

**Which of these block work, and which resolve as the phases run** (asked in
review, 2026-08-24). Only one needs an answer before code starts:

| # | Item | Verdict |
|---|---|---|
| 1 | CR 903.9 is half an SBA | **Answered.** A finding, not a question — `codebase-state.md` corrected |
| 2 | `ObjectSet` reuse | **Answered.** A constraint to preserve, not a question |
| 3 | Self-replacement (CR 614.15) plumbing | **Deferred past RE (post-RE audit, 2026-09-15).** RB gave `SelfReplacement` its CR 616.1a bucket and no producer, and nothing through RE-9 needed one. The item below carries the reachability line and the size; item 12 answers *who sets the class* |
| 4 | Replacement effects outside the battlefield | **✅ Closed 2026-09-16 by RF** (§9). CR 113.6 landed as LK (2026-09-14) as a registration leg; RF is the gather's read leg — item 9's (c), `roadmap-v2.md` A5b — with Darksteel Colossus and Nexus of Fate. The item below says what shipped and what it refused |
| 5 | Overlay shape | **Answered** — read-side accessor, closed on measurement |
| 6 | Skips are not `execute_action` events | **Answered.** A design note; the work is in RE |
| 7 | `ScriptedDecisionProvider` blast radius | **Answered, and the watch held (RB, 2026-08-26).** Every test now traverses the pipeline and zero new prompts appeared. The rule was never relaxed |
| 8 | `then` timing | **Answered 2026-08-25 (audit).** Riders queue at application and resolve after the performed event (CR 615.5, 615.12); card-text "A, then B" never enters the pipeline as a unit (CR 608.2c). §4.1a |
| 9 | Batch `applied`-set scope | **Answered 2026-08-25 (audit).** Per event, never per batch — a first draft shared one set and Kalitas's own ruling refutes it; CR 704.7 is a same-result dedupe, not a share. §4.2 |
| — | **`ZoneChangeCause`** | **Not the list — the *catchall ban*.** Needed before RA's first commit; the list itself is derived, not researched. See below |

**No catchall variant, ever.** Every call site names its reason. A site with
nothing honest to say is a site whose reason nobody has worked out, which is
the bug.

→ Why the list was derived rather than researched, the 25 raw inputs and the four judgments that merge them to 18, and what cards actually ask for: `plans/archive/replacement-architecture-landed.md`, "11. Findings and open questions" (evicted 2026-09-15). Items 3, 4 and 14 are live below; every other item is one line here and its body is there.


1. **CR 903.9 is half an SBA, and `codebase-state.md` said otherwise.** → archive

2. **`ObjectSet` is reused rather than re-invented, and the reuse is load-bearing.** → archive

3. **Deferred — how self-replacement effects (CR 614.15) reach the pipeline**
   (Open until the post-RE audit, 2026-09-15). They belong to the resolving
   spell/ability, not to any registry, so they arrive through
   `ActionContext::resolution`. `ResolutionContext` is
   `{source, ability_source, controller, targets, replaced_amount,
   damage_prevented}` today and needs one more field. Low breadth
   (Aang's Journey and kin), so the hook landed in RB and the field lands with
   the first card that needs it. Do not build a general mechanism first.

   **Reachability (2026-09-15):** unreachable — `ReplacementClass::SelfReplacement`
   has its CR 616.1a step and `gather` discards everything else when an event
   is blocked (CR 614.17c), and nothing produces one: no registered card, no
   fixture. The printed self-replacements whose condition is fixed at cast
   time (Aang's Journey's kicked clause, "if this spell was kicked … instead")
   are `Effect::Conditional(SpellWasKicked, …)` and never enter the pipeline,
   which is CR 616.1a by construction — the effect resolves its own clause
   before the event is proposed, so what the other replacements see is already
   the self-replaced event. The bucket's first *real* customer is a
   self-replacement whose condition reads the event **as modified by others**
   (`ATOM-616.1a-001`'s "if this damage would be prevented, deal double
   instead"), and no printed card of that shape was found; the CR names the
   facility, so a fixture is the customer (`codebase-state.md`'s section
   header, 2026-09-14).

   **Sized:** ~80 lines — the field on `ResolutionContext`, a third gather leg
   reading it, `ReplacementClass::from_rewrite`'s override for it — plus the
   fixture, as a small PR of its own at Phase 8's opening. Four Phase 6 atoms
   are re-filed here (`backlog.md` §3.3): `ATOM-614.15-001`, `-002`,
   `ATOM-616.1a-001`, `ATOM-614.17c-001`.

4. **Scheduled — replacement effects functioning outside the battlefield**
   (Open until the post-RE audit, 2026-09-15). Source 2 in §3.3, sized at
   item 9 (~390 cards, six keyword families). What it needed was CR 113.6,
   and **LK landed that on 2026-09-14** — as a *registration* leg only:
   `register_static_effects` takes a zone, and `replacement::gather`'s sweep
   still visits `battlefield_ids_ordered` alone (item 9's (c)). The
   `debug_assert` on `ObjectSet::Filter { zones }` in `gather.rs` is the
   window, and the failing assert is how the builder finds the site.
   **Owner: `roadmap-v2.md` A5's third PR**, with its own card — one of the
   five "would be put into a graveyard from anywhere, shuffle it into its
   owner's library instead" (Blightsteel Colossus and kin), which functions in
   every zone and is the leg's cleanest first consumer. Not a zone parameter
   on a separate registry: LJ found the sweep is the easy half and the
   working set (`Board::seed`, `membership`) the real one.

   **✅ Closed 2026-09-16 by RF** (§9, "Phase RF"). The read leg, its
   gate and the affected side's zone check; Darksteel Colossus rather than
   Blightsteel (infect is unbuilt, and a card may not wear a real name while
   behaving differently), Nexus of Fate beside it as the stack-shaped second
   card. "Not a zone parameter on a separate registry" cashed out as: no
   registry at all on the source side — a second candidate *set* the
   registration doors keep, read off the effective list like source 1 — and
   **the working set untouched**: a source off the battlefield is read as a
   non-member, which LJ's `Board::seed` and `membership` already make
   correct, so neither was opened. What the sizing above missed is that the
   card's clause needs a library shuffle the engine had no in-game writer
   for (§3.1's `ShuffleLibrary` row), and that three registered rows were
   hiding behind the `debug_assert`.

5. **The overlay's shape — closed by performance, not by taste.** → archive

6. **CR 614.10's skips are not `execute_action` events.** → archive

7. **Watch the `ScriptedDecisionProvider` blast radius.** → archive

### Answered on the review's second pass (2026-08-30)

8. **`apply_rewrite` does not grow per `(template, event)` pair, and the shape that would fix it is worse** → archive

9. **§3.3 source 2 — static abilities functioning in other zones — is sized, and it should not be inside this phase** → archive

10. **Skullbriar is the wrong reason to change the counter model, and there is a right one** → archive

11. **"Unconditional once queued" is right, and RB cited the wrong rule for half of it** → archive

12. **CR 614.15's `SelfReplacement` class is *derived*, never authored** → archive

13. **`ZoneChangeCause::Returned` keeps both returns, and the reason is structural** → archive

14. **Open — does declining CR 903.9b exhaust its CR 614.5 exemption?** (H8.)
    `declined` is keyed on the instance id, so a decline is final for the whole
    event however the destination changes; an owner who declines the command
    zone for a hand-bound event is never re-asked if a later replacement
    redirects it to the library. The rule says 903.9b "may apply more than once
    to the same event" and does not say whether *being offered and refusing* is
    one of those times.

    Unreachable (K7, and no second hand/library redirector exists), so this is
    a research note, not a bug. **The one thing an answer must not do** is
    exempt `declined` the way `applied` is exempted: that is the termination
    argument §4.1 documents, and removing it is a hang. If the rules answer
    turns out to be "re-ask on a changed destination", the key becomes
    `(instance, destination)` — still finite, because destinations are.

    **Deferred at the post-RE audit (2026-09-15), with the reading it adopts.**
    CR 614.5 gives an effect "one opportunity to affect an event or any
    modified events that may replace that event", and the engine spends that
    opportunity on a refusal, which is right for every ordinary optional.
    CR 903.9b's sentence — "may apply more than once to the same event. This
    is an exception to rule 614.5" — exempts the *effect* from the
    one-opportunity rule, not the player from having chosen; so when a later
    replacement changes the destination, the modified event is one 903.9b
    may apply to again, and the owner is asked again. That is the
    `(instance, destination)` key, and it is the only reading under which the
    exemption does any work, since without a second destination there is
    never a second application to exempt. Unpinned until a board exists.

    **Reachability (2026-09-15):** unreachable — nothing registered redirects
    an event bound for a hand or a library to the other, and RE-8's discard
    and RE-2's draw both leave the destination alone. **Sized:** ~20 lines,
    the key and one fixture with a synthetic second redirector; the fixture is
    the customer, per the section header's rule.

15. **RD's design must open with this: CR 614.5's identity may be per `(event, affected object)`, not per batch member** → archive

### Found by a rider read-through (2026-08-30)

16. **`Rider.subject` is `Option<ObjectId>`, and the `Option` is doing two jobs.** → archive

17. **A rider's object reach is its subject, and that ceiling belongs to the `Effect` tree rather than to this document.** → archive

18. **§3.2d's lineage rule ships with no producer** → archive

### Found by asking what RC-3's own test proves (2026-09-02)

19. **CR 616.1 prompts for a choice with one outcome on every entry, and for `Rewrite::EnterWith` that is a theorem rather than a coincidence.** → archive

20. **The replaceable event must be the outermost proposal** → archive

### Found by RD's design check (2026-09-08)

21. **Player scoping is RD's, and `types/replacement.rs` says RE.** → archive

22. **`consume_use` runs before `apply_rewrite`, and three rules need it after.** → archive

23. **One printed shape splits a damage event in two, and §3.2d's `Option` cannot hold it — so it is a candidate PR with a gate, not an exclusion.** → archive

24. **§11 item 15 is answered: decisions are per `(batch, subject)`, rewrites per member, and CR 615.7's allocation is the one non-uniform rewrite.** → archive

25. **"Unpreventable" is a property of the event, `is_prevention` is derived, and `cant-effects-architecture.md` §4.7's `ReplacementKind` is superseded.** → archive

26. **Every "damage can't be prevented" card but two carries a half the engine lacks.** → archive

27. **CR 120.3 has eight results, RD ships two of them, and the other four have an owner each as of today.** → archive

28. **The inverse of a doubler is printed, it rounds, and the rounding is the card's.** → archive

### Found by the RD-1 review (2026-09-08)

29. **Two Furnaces prompt, and multiplication commutes — so §11 item 19's suppression theorem has a second candidate, and it is narrower than "N identical effects".** → archive

30. **A test that asserts "nothing happened" cannot say *why* nothing happened, and the trace sink is the instrument.** → archive

### Found by RD-3 — sources, and its review (2026-09-09)

31. **`Rewrite::Prevent` reports how much damage it prevented, and RD-4 will read it.** → archive

32. **A new `EventPattern` field costs nothing until a def writes it, and the A/B is the proof.** → archive

33. **The `--require` reachability instrument counts casts, and an activated ability's *activations* are invisible to it.** → archive

34. **Asked at the RD-3 review: should CR 616.1's prompt be skipped by *simulating* both orders — `Lookahead`, compare the states, suppress if they agree — rather than by a static predicate? No, and the case that prompted the question is the reason why.** → archive

### Found by RD-4 — redirection and unpreventable damage (2026-09-09)

35. **A group's subject stops being its members' the moment a redirect applies, and three questions were reading the wrong one.** → archive

36. **`Rewrite::Retarget` makes `codebase-state.md` item 25 reachable for the first time, and it is still not worth building.** → archive

37. **The gate that decides an arm ships fired against `RetargetSpec`, and caught the arm §9 named.** → archive

38. **`ObjectFilter::EachOther` was refused in an affected set, and the instrument that measured it as harmless measured the wrong thing.** → archive

### Found by the RD-4 review (2026-09-09)

39. **Two of RD-4's three "missing destination" tests were the same branch, and the `COVERS-PARTIAL` on one of them claimed a leg no test reached.** → archive

40. **Nothing tested that `unpreventable` survives a redirect, and "the field is copied in one line" is not a reason it did not need to.** → archive

41. **`Restriction::ApplyReplacement`'s `to` was half a pair wearing the name of the whole thing.** → archive

### Found by RE's sizing (2026-09-11)

42. **§3.2d's Notion Thief encoding contradicts the card's ruling, and the lineage rule is what catches it.** → archive

43. **Mana production is a chokepoint violation, and RA's census could not have seen it.** → archive

44. **A lost player keeps taking turns and receiving priority in any game of three or more, and `has_drawn_from_empty_library` never clears.** → archive

45. **`Restriction::Event` carries no `PlayerSet`, so four printed "can't" families have no row shape.** → archive

46. **This document held two answers on skips for twelve days.** → archive

### Found by building RE-1 (2026-09-11)

47. **A turn queue needs a second field, and two players hide it.** → archive

48. **The game's first turn had never begun, and no test could see it.** → archive

49. **"`advance_turn` is written as a turn queue so `backlog.md` §2.17 does not rewrite it a second time" is an overclaim, and it covers the turn level only.** → archive

### Found by building RE-2 (2026-09-11)

50. **`GameState::draw_cards` has had no caller since before RA and is a draw path with no proposal.** → archive

51. **`Game::setup`'s opening-hand comment claimed a route the code does not take, and had since RA-2.** → archive

52. **CR 121.2c orders two players' draws and nothing can express it.** → archive

53. **Alms Collector as `Prevent` plus riders is an infinite loop, and its own ruling says so.** → archive

54. **A three-round median said +6.0% and the number was noise.** → archive

### Found by the RE-2 review (2026-09-12)

55. **Two draw doublers commute, and RE-2 shipped a prompt between them.** → archive

56. **The acid test's bound was the prompt, and suppressing the prompt took it away — so the bound moved into the engine, derived.** → archive

### Found by building RE-3 (2026-09-12)

57. **`Primitive::Restrict` could only build an object-scoped row, and its first printed customer is the one that needed a player-scoped one.** → archive

58. **The CR 616.1 suppression premise was written about damage for the third time in three PRs, and it is now a property of the type.** → archive

59. **A phase can close `specdb`'s claim about its own corpus and should check it.** → archive

60. **The fourth suppression shape has the shortest premise of the four, and it was nearly not built because the reason for skipping it was a cost argument.** → archive

### Found by building RE-6 (2026-09-12)

61. **CR 704.3's "performed" is not the performed set, and it took a ruling to say so.** → archive

62. **CR 104.2a is a fact about a batch, and CR 104.1 is a line at the chokepoint.** → archive

63. **A rider runs after the batch, and one printed ruling wants it inside.** → archive

64. **The first `Condition` a replacement effect reads is asked at gather, and the argument is two rules that were already there.** → archive

65. **CR 104.1 removed the post-mortem tail, and that is the one two-player stream change — read the way RE-3's review said to.** → archive

66. **The four-player run's first table was a measurement of the harness.** → archive

67. **The Deferred Migrations list had become a queue of small fixes, and the review said so.** → archive

### Found by building RE-7 (2026-09-13)

68. **CR 800.1 is the gate, and without it "byte-identical on both two-player pools, by construction" was going to be false.** → archive

69. **A member that removes objects performs after the members decided against them, and CR 704.3 is why that is not a contradiction.** → archive

70. **`ATOM-800.4c-001`'s board could not reach its own rule, and the CR says so in one sentence.** → archive

71. **CR 608.2n's tail had to learn that the object might be gone.** → archive

72. **The A/B could not see the row this PR exists to zero.** → archive

73. **CR 603.6c answers the question RE-7 recorded as open, and it answers it in the sentence that names this event.** → archive

74. **CR 111.4 names the token, and every def in the tree had been naming it wrong.** → archive

75. **`TokenCreated` has two emitters, and CR 111.13 is the line between a token the event announces and one it does not.** → archive

76. **The first performer to propose a batch from inside a performer set the precedent: the outer reports the event as decided, the log counts what happened.** → archive

77. **A rider's proposals carry the replaced event's applied set, and the loop RE-4's A/B found was the engine's, not the rules'.** → archive

78. **`decomposition_depth`'s assertion counted across a rider, and fired on a legal board.** → archive

79. **Four arms left absent, each with its customer named** → archive

80. **The middle arm was the wrong instrument for `stress`, not the byte-identical check.** → archive

81. **An exit beside `EnterWith`s is one outcome, and the predicate now says so** → archive

82. **Four ledger lines were arms the PR had the type open for, and the review made three of them code** → archive

### Found by building RE-5 (2026-09-13)

83. **CR 122.6a's named putter was closed on an empty Scryfall query, and the review reopened and built it.** → archive

84. **Item 47's condition (c) fired, from the multiplier side, and the re-derivation is recorded.** → archive

85. **Additive beside mods-adding commutes on one kind and not across kinds, so §2.29's table needs the kind axis.** → archive

86. **A cost that puts counters must not be an effect that puts counters, and the event has no field for it yet.** → archive

87. **`gather` has no source that asks a card off the battlefield, and five of RE-8's six consumers needed one.** → archive

88. **CR 514.1's cleanup discard was N events and the rule says one.** → archive

89. **A discard redirected into a hidden zone has undefined characteristics, and RE-8 is the first PR that can produce one.** → archive

90. **A discard whose graveyard move is replaced loses the fact that it was a discard.** → archive

91. **`fuzz_games` counted a resolution by its cause, and CR 608.2m's move is replaceable like any other.** → archive

92. **`substitute` grows with templates, not with templates × actions, and the count says when to split it.** → archive

### Found by RE-9's design check (2026-09-15)

93. **"Tapped for mana" is a CR definition, not a ruling, and it names the activation cost.** → archive

94. **The census undercounted the family by a factor of two, and the miss is a substitution leg.** → archive

95. **Decision 7's "`special` riding through unchanged" is wrong for a multiplier.** → archive

96. **`GameEvent::ManaAdded` has carried a `HashMap` since the log was written, and the first emitter would have been the determinism regression.** → archive

97. **`GameState.counters` is the diagnostics and its two sibling fields are CR 122's, and the review is where the collision was seen.** → archive

98. **The census regex read the card's phrase and not the rule's, and it hid two of CR 106.12b's three axes.** → archive

99. **The event-kind gate returns nothing for mana, so §8's one pre-approved lever does not touch the cost the phase was worried about.** → archive

---

## 12. Explicitly out of scope

Six bullets, written 2026-08-24; each re-read against the tree at the
post-RE audit (2026-09-15) and amended in place where the tree moved.

- **Layer 1 / the copy system (CR 707).** 23 Phase-6 atoms, a separate system.
  CR 616.1c got its ordering *bucket* in RC-4 so the classification is complete.
  **Layer 1 landed as CV-1 (2026-09-02, `EffectModification::CopyFrom`)**; what
  is still out is copy-on-*enter*, which is CV-2's, and the two Phase 6 atoms
  that need it (`ATOM-616.1c-001`, `ATOM-613.1a-001`) are re-filed there.
- **CR 614.14 / 607 linked abilities.** Needs the CR 607 work (`backlog.md`
  §2.2). Unchanged; `ATOM-607.2b-001` and `-2g-001` re-filed there, and
  Sutured Ghoul (`codebase-state.md` item 59) is the wrong answer waiting on it.
- **CR 614.12b** — "combined costs of those effects to not be payable" across
  simultaneous entries. **Was parked on cost modification, and CM-1–CM-4
  landed (2026-09-07/08) without touching it, because that was the wrong
  prerequisite.** It needs an entry replacement whose choice has a *cost*
  (shocklands' "you may pay 2 life" — none registered; `EnterModsTemplate`
  carries tapped and counters only) and a plural non-token entry (RE-4's
  plural batch is tokens'), and the rule bites only with both.
  `codebase-state.md` main item 134 carries the reachability and the size.
- **CR 614.12c anchor words.** Linked abilities again — §2.2.
- **CR 615.13** — triggers on prevention. **Critical-path item 6** ("Phase 7"
  is the archived plan's name). RD-4 left the seam: `apply_rewrite`'s return
  value carries the prevented amount, and whether a prevention is announced on
  the performed stream is item 6's to decide.
- **CR 731 loop detection.** Survives from `state-tracking-architecture.md`
  Tiers 1–3, re-based on performed-action transcripts; not this phase. **Now
  captured** as `backlog.md` §2.28 (2026-09-14), a capture and not a design.

---

## 13. Documents this phase owes

Update as part of the work that changes them, not in a later pass. Brought
current at the post-RE audit (2026-09-15); "through RB" had stood since
2026-08-26.

- `codebase-state.md` — ✅ through RE-9 and the audit. The CR 614–616 row is
  ✅ with its own "not yet" list; the TL;DR was rewritten in place rather than
  appended to; Deferred Migrations carries a dated block per phase (RD-1
  through RE-9, and "Found by the post-RE audit") and each closed item is a
  stub over `plans/archive/codebase-state-closed.md`. Keep adding a line per
  stub.
- `CLAUDE.md` — ✅. The authority-table row, the chokepoint invariant with
  RA-3's sub-rules, and "The replacement pipeline (CR 614–616)" as its own
  section — six rules and the two growth contracts, each with a pointer here.
- `layers-architecture.md` §9 / §15.2 item 3 — ✅ the overlay decision,
  recorded with RC-4 (2026-09-02). RA-3's LKI frame had needed neither the
  accessor pair nor a clone — a `compute_characteristics` call taken *before* a
  mutation is not a hypothetical about a perturbed board — and RC-4's frame is
  the first thing that did.
- `cards-unlocked-ledger.md` — 🟡 RA, RB, RC (five rows) and RE (six rows)
  are in; **RD has no section** — its four PRs' unlocks (Furnace of Rath's
  family, the shield cycle, the CR 609.7 sources, redirection) are recorded
  only in the phase stubs above. Owed with the next ledger pass (A4b's
  rulings ledger is the natural sitting).
- `engineering-practices.md` — ✅ §5.1's `owed` gap closed 2026-09-15 (Phase 6
  in `SHIPPED_PHASES`); §7's trace-page list names RE-2 ✓ and RE-4 ✗.
- `fuzz-record.md` — ✅ a block per phase that moved a pool, RE-9's the newest.

---

## 14. The phase in hindsight — written 2026-09-15, at the post-RE audit

Pointers, not prose. §11 holds every finding item by item, the archive holds
every phase body, `fuzz-record.md` holds every number; this is the index a
reader wants before any of those, written once at the close and not
maintained (the post-RE audit's decision 1; its record is `codebase-state.md`,
"Was critical-path item 5 done, and what sits before item 6? — audited 2026-09-15").

### What it was

- **Twenty-four PRs, 2026-08-25 → 2026-09-15**: RA-1–3 (#58–#60), RB (#62),
  RC-1–5 with RC-4b, RD-1–4 (#119–#122), RE-1–10 (#126–#140), and the sizing
  and eviction docs PRs between them (#118, #124, #125). Interleaved on the
  same `main` with CM-0–4, LH, LI, LJ, LK, CV-1 and RS-1.
- **The shape that held from RA's first commit**: propose, decide, perform,
  announce — `CLAUDE.md`'s chokepoint invariant, §2a as built, §4.1's loop.
  Every later kind (entering, damage, draw, tokens, counters, life, the
  game's end, a turn's units, mana) was one more `GameAction` family through
  the same arms: §8a's derivation holding, and RE's per-kind measure coming
  out as predicted (§9, "Why ten, and the count").
- **Cards**: `cards-unlocked-ledger.md` Part 3 (RD's rows owed — §13).
  159 registered, 89 pooled at RE-9's close (`state-of-play.md`).
- **Cost**: RE-9's block in `fuzz-record.md` is the last reading — +1.2%
  CPU per game at two seats and +0.7% at four for making every mana
  production a proposal. The per-event fixed cost is `codebase-state.md`
  item 136; §8's event-kind gate was measured twice (RE-5, RE-9) and not
  built, because it returned nothing.

### What the building changed, against the design

Each is a §11 item; the number is the pointer.

- The rider queues at application and resolves after the performed event —
  8; card-text "A, then B" never enters the pipeline — §4.1a.
- The applied set is per event, never per batch — 9 — and its identity is
  per `(event, affected object)`, which RD-2 made the loop's unit — 15.
- Entering is one event carrying `from`, not a zone change plus an entry —
  RC-4b, the largest re-cut (`codebase-state.md`, the RC-4 nesting audit's
  item 4, archived).
- A draw carries its lineage: the applied set travels with a decomposed
  event — RE-2's trace page, items 50–56, and 53 is the one to read.
- `pending_skips` was not built; a skip is `Rewrite::Prevent` on a proposed
  unit — RE decision 6; then 47 (`turn_rotation`), 48 (`Game::setup` never
  began turn 1), 49 (the cursor one level down, which became RE-10).
- Mana production was a direct write with no event, RA's unnamed debt —
  93–98, and RE-9.
- A substitution overwrites the replaced event's `cause` — 90, 91;
  `codebase-state.md` item 131 carries it to critical-path item 6.
- CR 614.5's identity, CR 615.7's allocation across groups, CR 609.7b's
  recheck at the event — RD's design check, seven decisions and the one
  nobody asked (the archive, "RD").

### What was learned about the process

- **Size against the tree, not the paragraph.** RE was sized at seven,
  re-cut to nine on review and to ten at RE-1's review, and its census
  disagreed with the paragraph it replaced in both directions (§9, RE).
  RD's "Why four" and RE's "Why ten" are the arguments to reread.
- **The ≤40-line stub rule** — each PR evicts its own body and keeps the
  heading `grep` finds — is what kept this document usable through
  twenty-four PRs. The pre-build reasoning was the half it did not cover,
  which is the eviction the audit planned and ran as pass 1b (PR #151).
- **Trace pages: four across the track** (RC-4b, RC-5, RD-2, RE-2), each
  decided at the close against `engineering-practices.md` §7's one rule,
  each refusal argued in the archive's "Trace-page decisions".
- **The `owed` gate was never armed for Phase 6** until this audit
  (`engineering-practices.md` §5.1; `backlog.md` §3.3's second block). What
  held the track was the `// COVERS:` discipline per PR — enough, and not a
  gate: four atoms were found proven and unclaimed at the close-out.
- **"Not wrong today" read as wrong today** on the board for a week — the
  instrument, not the items (`check_state_of_play.py`, 2026-09-15).
- **A CR-named facility is not closed by an empty Scryfall search** — item
  43, closed that way and reopened at RE-5's review; the rule is in
  `codebase-state.md`'s section header now, and item 3's fixture-first
  disposition above is its first application.

### What stays open, and where each waits

- §11: item 3 (the self-replacement producer — deferred, sized, fixture
  first), item 4 (effects off the battlefield — scheduled, A5's third PR;
  **closed 2026-09-16 by RF**, §9), item 14 (CR 903.9b's exemption and a
  refusal — deferred, unreachable).
- §8a's four kinds: turned face up (CV-6), dice (`backlog.md` §2.31),
  search (with `backlog.md` §2.5's producer), countering (closed — a zone
  change with a cause).
- §12's six, re-read 2026-09-15 and amended in place.
- `codebase-state.md`: 59 (CR 607, `backlog.md` §2.2), 60 (`backlog.md`
  §2.30), 122 and 131 (critical-path item 6), 134 (CR 614.12b, new);
  118 fixed at the audit.
- Phase 6's corpus: `owed` clean, 41 atoms uncovered with a recorded
  deferral each (`backlog.md` §3.3).

### What critical-path item 6 inherits

The performed stream is post-replacement truth — the reason item 5 came
before item 6. What the trigger phase reads, and the one thing it must not do:

- reads `GameEvent::{TurnBegin, PhaseBegin, StepBegin}` (RE-1), every
  `ZoneChange` with its cause and its CR 603.10a LKI frame (RA),
  `DamageDealt` with CR 120.3's results (RD-1), `ManaAdded` with
  `tapped_for_mana` (RE-9);
- carries `codebase-state.md` items 121 (Eon Hub's two tests), 122
  (CR 121.2c's order), 131 (a substitution's `cause`), CR 615.13 (§12), and
  the second cleanup step it can now see (118);
- must not add outcome-bearing state anywhere but `GameState` —
  `codebase-state.md` item 40, the one design constraint with a deadline
  (the post-RE audit's fork model, `codebase-state.md` main item 41).
