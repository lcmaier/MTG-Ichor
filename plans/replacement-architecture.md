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

**Yes, and the prerequisites are met.** Recorded so a later session does not
re-litigate it:

- `CLAUDE.md` → "Critical path to v1" item 5. Items 1–4 (layers core, CDAs,
  Layer 6, Layer 2) are ✅ as of 2026-08-24. 634 tests green, zero warnings.
- `codebase-state.md` → Deferred Migrations → "Before Replacement effects":
  item 1 ✅, item 2 ✅ (with three tagged bypasses this phase closes), item 4
  **resolved 2026-08-24** (performed-action event stream; the delta log is
  rejected). Item 3 is not a prerequisite — it is *this phase's opening ticket
  block*, and Phase RA below is that block.
- Nothing else in the tree blocks it. The chokepoint (`engine/actions.rs:86-89`)
  exists and every mutating action already routes through it or through a site
  tagged for this phase.

**Why not triggers (CR 603) first.** Triggers consume the *performed* event
stream, which only exists downstream of the CR 614 pipeline —
`GameAction` (proposed) → replacement → perform → `GameEvent` (performed).
Building CR 603 against the pre-replacement stream and refitting is the
expensive path, and it is the path the 2026-08-24 fork decision explicitly
closed. CR 614 also sits upstream in the code: `execute_action` runs before
`perform_sba_and_triggers` (`engine/priority.rs:234`, whose placement stub is at
`:240`) can ever see anything.

**Why not the CR 613.8 dependency cluster first.** It is back-stopped to land
before Phase 8 card breadth, not before this. It also gets cheaper for waiting:
§5 shows the CR 614.12 look-ahead forces `compute.rs`'s five concrete state
reads behind one accessor pair, which is plumbing 613.8's step-4 hypothetical
check would otherwise have to do itself. (Only the *seam* is shared — 613.8's
own perturbation is frame-level and much cheaper; §5 and §11 item 5 price both.)

**The honest cost of going first.** Three corpus atoms in the CR 614/615 family
are tagged Phase 7 because they assert on triggers, not on replacement:
`ATOM-614.6-001` (a modified event triggers abilities), `ATOM-614.8-002`
(regeneration: damage triggers still fire), `ATOM-615.6-001`. They stay
unclaimable until CR 603 lands. That is 3 of the 58 atoms written directly
against CR 614/615/616 — the corpus already tags them, and no design decision
here changes it.

### Scope, measured

`specdb` (2026-08-24): **88 of Phase 6's 124 atoms are replacement-family.** The
other 36 are the copy system (23 × CR 707, 4 × CR 613.1–2 Layer 1), linked
abilities (2 × CR 607), and stragglers. **"Phase 6" is not one system** — it is
CR 614–616 plus Layer 1/copy, and only the first is this work.

Card breadth this unlocks (Scryfall, 2026-08-24, `unique=cards`):

| Shape | CR | Cards |
|---|---|---|
| "enters tapped" | 614.1c/d, 110.5b | **773** |
| "enters with [N] counters" | 614.1c, 122.6a | **580** |
| "prevent…" | 615 | 525 |
| regenerate | 614.8, 701.19 | 419 |
| "as [this] enters…" | 614.1c | 289 |
| "would die…instead" | 614.6, 616.1 | 98 |
| stun counters | 122.1d | 92 |
| damage replacement ("would deal…instead") | 701.10g, 609.7 | 57 |
| skip effects | 614.10 | 54 |
| "would be put into a graveyard…exile instead" | 400.6, 122.1h | 53 |
| draw replacement | 614.11, 121.2a | 47 |
| finality counters | 122.1h | 41 |
| shield counters | 122.1c | 31 |
| counter doublers | 614.16 | 26 |
| life-gain replacement | 119.10 | 21 |
| token doublers | 614.16 | 9 |

The top two rows are the point. **CR 614.1c/d is the single largest card-unlock
in the engine's remaining work**, it needs no triggers, and it is Phase RC.

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
| `ReplacementDef` | `types/replacement.rs` | One replacement effect as data, nine fields in declaration order: `pattern` + `affected` + `rewrite` + `then` + `class` + `uses` + `is_regeneration` + `exempt_from_614_5` + `optional` | grows by *field*, rarely; per-mechanic variety goes in `then` |
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
| `EventSubject` | `engine/replacement/gather.rs` | What a proposed event is *about* — an object or a player. Named for the event because `AffectedSet` already answers the other question, which objects an *effect* applies to | closed by what an event can be about |
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
  │         ├─ forced_bucket() → CR 616.1a–e's highest non-empty class
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
| `BeginStep` / `BeginPhase` / `BeginTurn` | 614.10 | RE | skips replace these |
| `ProduceMana` | 106.6a | RE | |

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
    /// `AffectedSet`; `SourceOnly` vs `Filter` is exactly CR 614.12's "affects
    /// only that permanent (as opposed to a general subset of permanents that
    /// includes it)".
    pub affected: AffectedSet,
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
use would have written `BattlefieldEntity.counters` from inside `consume_use`,
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
than a closure for the same reason `PermanentFilter` is: closures cannot be
compared, cloned cheaply, or inspected by the loop detector.

**Growth contract: exactly one arm per `GameAction` variant, and it grows on no
other axis.** It is a projection of §3.1's table. If a card needs a pattern
`EventPattern` cannot express, the missing thing is a `GameAction` variant or a
field on one, and that is where the fix goes — because a replacement effect can
only watch for an event the engine actually proposes (§8a). A change that adds
an `EventPattern` arm without a corresponding `GameAction` change is the smell
this contract exists to catch.

Within an arm, constraints on the event's fields reuse existing vocabulary —
`PermanentFilter`, `PlayerRef`, `ZoneChangeCause`, `CardType` — rather than
inventing per-mechanic predicates. "If a *red* source would deal damage to a
*Cleric you control*" (Daunting Defender, CR 615.10's own example) is `ByColor`
on the source and `And(BySubtype(Cleric), ByController(You))` on the target.
Both already exist.

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
  `Amount(Times(2))`. Two famously-confusable cards, one arm, two
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
three … eight times" — is `Amount(Times(2))` composing *within a single event's*
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
2. **Static abilities functioning in other zones.** Same sweep, other zones.
   Deferred past RE; recorded in §11 with its breadth.
3. **Continuous effects with a duration, from resolutions.** CR 614.3 — "Prevent
   all damage that would be dealt this turn". These go in the replacement
   registry with a `Duration`, expiring through the same cleanup/turn-start
   hooks `ContinuousEffectRegistry` already uses.
4. **Shields from resolutions.** CR 615.7/615.8, CR 701.19a regeneration. Also
   the registry, with `Uses::Once` or `Uses::Shield(n)`.
5. **Counters.** CR 122.1c (shield), 122.1d (stun), 122.1h (finality). These
   come from the *counter*, not from any ability — nothing on the card says so.
   Synthesized during the sweep from `BattlefieldEntity.counters`.

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
- **No per-layer existence re-check.** CR 613.7a's re-check exists because a
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

        bucket  = forced_bucket(cands)                   # CR 616.1a -> b -> c -> d -> e
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
- **CR 616.1a–e** — `forced_bucket` returns the highest-priority non-empty class
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
- **Fresh lineage.** A rider's actions are new events the replacement caused,
  not modified forms of the original, so they re-enter the pipeline with a
  fresh applied-set (§3.2d containment). Not theoretical: Kalitas plus Doubling
  Season makes two Zombies — the rider's `CreateTokens` is itself replaceable.

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

What a shared set was reaching for is **CR 704.7**, and that rule is a
*same-result collapse*, not a cross-member share: multiple state-based actions
with the same result at the same time (a player who would lose the game for
both life and poison) merge into **one** event before the pipeline runs, and
that one event has one applied-set. Implement 704.7 as a dedupe step on the
batch, upstream of `apply_replacements`.

`Uses` needs no batch special-casing either way: `consume_use` writes game
state, so a regeneration shield spent on batch member one is correctly gone
when member two asks (CR 701.19a — one shield, one destruction replaced).

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
| (3) existing effects | `effect_applies_to` hard-requires `game.battlefield.contains_key(&id)` for `AffectedSet::Filter` — an entering object matches no filter-based effect at all |

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
  that permanent", i.e. `AffectedSet::SourceOnly` — a filter-based one
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
  | What is re-evaluated | the whole layer walk for one object | `permanent_matches_filter(A.filter, chars, …)` |
  | Cost of the perturbation | game-state-shaped | `EffectiveCharacteristics` clone — measured 0.37 → 0.27 µs over N=10–80, i.e. flat (`layers-architecture.md` §12) |
  | Frequency | once per entering permanent | up to O(effects²) per layer, inside a per-permanent walk |

  613.8's check is **frame-level**. "Recompute A's `affected` with B applied" is
  a clone of `chars`, one `EffectModification` applied to it, and one
  `permanent_matches_filter` call — it never asks whether an object is on the
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
which `register_static_effects` cannot lower and `debug_assert!`s on — Deferred
Migrations item 7f — and Thassa additionally needs a `Condition` arm for a
numeric comparison and an `AmountExpr` that counts mana symbols, none of which
exist. The devotion arithmetic was the cheapest part of the God, not the
expensive one. RC-4 did not pull 7f in. Clause (2) is tested with a fixture
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

Five card interactions were put to this design by the owner. Two confirm it, one
moves the RC seam (see §9's RC-1…RC-4 split), and two add rules it did not state.

**The overlay is about one object, and the rest of the board is read as it
actually is — second worked example.** Elvish Archdruid ("Other Elf creatures you
control get +1/+1") enters while Master Biomancer ("Each other creature you
control enters with a number of additional +1/+1 counters on it equal to this
creature's power", a 2/4 **Elf** Wizard) is on the battlefield. Archdruid enters
with **2** counters, not 3: Biomancer's power is read off the real board, where
Archdruid's anthem is not yet applying, because Archdruid is not yet a permanent
and CR 604.3 makes its static ability function on the battlefield. This is §5's
second note working, stated from the other side — Thassa shows a *count* over the
board is not perturbed; Archdruid shows another *object's characteristics* are
not either. §5a's table gains a row:

| Read | Sees the entering object? |
|---|---|
| **Any other object's characteristics** (an `AmountExpr` reading a permanent's power, a filter's `you`) | **no** — computed against the real board |

Note the asymmetry this creates and do not smooth it over: clause (2) puts the
entering object's own anthem into *its own* frame, but that anthem does not reach
any other object's frame. One object is hypothetical; nothing else is.

**Simultaneous entries are computed against the board before any of them
entered.** Two Master Biomancers entering as one event give each other nothing —
neither is on the battlefield when the other's replacements are applied. RA-3's
`execute_actions` is the mechanism (one batch, one `BatchId`), and the rule RC
owes is that the *frame* is batch-scoped: every member's look-ahead reads the
pre-batch board. This is a different axis from §4.2's per-member `applied` set,
which is about CR 614.5 and stays per-event. **Delivered by RC-4b rather than by
the RC-5 piece that was sized for it** — once the entry is a phase-1 proposal
every member is decided before any is performed, so the frame is batch-scoped by
construction; RC-5 supplies the two-Biomancer test and §9's RC-5 entry has the
re-size.

**Not every judge example is a requirement.** Uphill Battle ("Creatures played
by your opponents enter tapped") looked like a demand for a "played by" filter
leaf until it was counted: `o:"played by"` matches **1 card in all of Magic**
(Scryfall, 2026-08-26). It stays a worked example of why CR 110.2b's default
controller matters (`codebase-state.md` item 9) and buys no `PermanentFilter`
vocabulary. Apply §8c's two-customers-before-a-variant guard to interaction
findings as well as to cards — an illuminating example is not automatically a
breadth argument.

**Two copy effects: the later one overwrites the earlier, riders included.** A
copy `Rewrite` is a *set* of the copiable values, not a modification of them
(CR 707.2), so a second copy-on-enter replacement discards the first's result
*and* its "except ..." clauses. §3.2b's algebra must not model copy as a
composable modify, and §3.2c's 0/574 completeness claim was measured without
copy-on-enter in scope — re-check it when Layer 1 lands and RC-4 fills the
CR 616.1c bucket.

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
`AffectedSet::SourceOnly`, unconditional… 'this land enters tapped'". That claim
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
not depend on the frame — `AffectedSet::SourceOnly`, unconditional. That is "this
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
| `FrameCache::entity` | the `BattlefieldEntity` the walk seeds from — controller, CR 302.6's clock, the counters layers 6 and 7c read | the object being computed is the entering one: `Lookahead::entity`, the entity `place_on_battlefield` would build |
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
| `engine/cast.rs::activate_ability` | emit `AbilityActivated`; resolution emits identity-bearing `AbilityResolved` | RA |
| `state/game_state.rs::StackEntry` | add `cast_from: Zone` — §8c, two customers | RA |
| `engine/sba.rs` | one CR 704.3 event: gather every condition against one game state, dedupe per object (704.7's same-result collapse), perform as one batch, `cause` on each move | RA ✅ |
| `engine/resolve.rs::Primitive::Destroy` | lowers to `GameAction::Destroy`, not to `ZoneChange` | RB |
| `state/game_state.rs::place_on_battlefield` | becomes the *performer* of an already-replaced `EnterBattlefield` | RC |
| `state/game_state.rs::register_static_effects` | skip `Effect::Replacement` bodies without tripping the loud-lowering assert | RB |
| `engine/layers/compute.rs` | ✅ RC-3 — the filter membership gate reads the battlefield *zone*, so it admits entering objects (§5c) | RC-3 |
| `engine/layers/compute.rs` | battlefield reads go through one accessor (overlay seam) | RC-4 |
| `engine/turns.rs::advance_turn` | gains `dp`; consults pending skips (CR 614.10) | RA / RE |
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
AllocatePreventionShield { shield_remaining: u64 },
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

### Two deliberate non-events, re-checked (2026-08-30, `rb-review.md` E4)

The section above asks what the vocabulary is *missing*. The mirror question is
what it deliberately leaves out: `Primitive::RemoveFromCombat` and
`Primitive::RemoveAllDamage` both write `BattlefieldEntity` directly, and both
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

§8a establishes that the growth axis is `GameAction`. That is only reassuring if
`GameAction` has a knowable size, so this section measures it. Asked in review:
*a bird's-eye view of roughly how big these structs get would ground the
performance and sprawl claims.* Agreed — here it is.

### Method, and what it is worth

Two independent bounds, top-down and bottom-up.

**Top-down:** CR 701 enumerates the keyword actions. `tmnt.txt` has **67** of
them (701.2 Activate through 701.68 Blight). That is the whole universe of named
game actions; it grows by a few per set and never by more.

**Bottom-up:** a keyword action needs its own `GameAction` variant only if some
card watches it **as a unit** — a replacement ("if you would X … instead") or a
trigger ("whenever … X"). Otherwise it decomposes and needs nothing. Both hooks
are counted because §2's spine gives them one vocabulary. 46 keyword actions plus
10 core mutations were queried against Scryfall (2026-08-24), one text search
per phrasing.

**These numbers are the record; there is no script behind them.** A `sizing`
subcommand existed briefly and was **deleted rather than fixed** (2026-08-24),
because two of its readings were known-wrong — Regenerate came back 0, since it
replaces *destruction* and nothing says "would be regenerated", and
`EnterBattlefield`'s 7,211 is every ETB creature ever printed. A tool that
prints answers known to be wrong is worse than no tool. This was a one-time
bird's-eye question; if it needs re-asking, write a better instrument then.

**Precision caveat, stated up front:** these are per-phrasing text searches, so
they are order-of-magnitude signals, not exact counts. Two known distortions:
Regenerate reports 0 because nothing says "would be regenerated" — it replaces
*destruction*, which is the point, not an omission; and `EnterBattlefield`'s
7,211 triggers is essentially "every ETB creature ever printed", which is true
but not informative. Read the column ordering, not the digits.

### What is watched, and by which hook

| Event kind | replacements | triggers | Needs its own `GameAction`? |
|---|---|---|---|
| Enters the battlefield | 15 | 7211 | **yes** — RC |
| Step/phase/turn begins | 9 | 2656 | **yes** — RE |
| Cast a spell | 0 | 2263 | **no** — event only, see below |
| Zone change | 110 | 1436 | **yes** — RA |
| Discard | 7 | 1045 | no — `ZoneChange { cause: Discarded }` |
| Deal damage | 145 | 817 | **yes** — exists |
| Draw | 47 | 391 | **yes** — RA/RE |
| Gain life | 21 | 351 | **yes** — exists |
| Sacrifice | 0 | 278 | no — `ZoneChange { cause: Sacrificed }` |
| Create tokens | 32 | 249 | **yes** — RE |
| Counter a spell | 1 | 172 | **yes** — RE |
| Tap / untap | 2 | 150 | **yes** — RA |
| Fight | 0 | 121 | no — two `DealDamage` |
| Lose life | 10 | 107 | **yes** — exists |
| Counters placed | 21 | 72 | **yes** — RB |
| Exile | 0 | 74 | no — `ZoneChange { to: Exile }` |
| Connive / Explore / Mill / Scry | 6 | 176 | mixed — see below |
| Lose the game | 4 | 26 | **yes** — §8a |
| Roll dice / flip coin | 8 | 27 | **yes** — CR 705/706, Phase 8 |
| Search, shuffle, reveal, vote, goad, transform, proliferate, exert, attach, activate, surveil, monstrosity, investigate, discover, manifest, amass, learn, forage, incubate, adapt, clash, play | 5 | ~200 total | mostly Phase 8, mostly decomposing |
| Exchange, Regenerate, Populate, Venture, Support, Bolster, Meld, Detain, Fateseal | 0 | 0 | **never** — watched by nothing |

**Nine of the 46 sampled keyword actions are watched by nothing at all.** They
can never need a variant, because a variant exists only to be matched against.

### Three findings that do the sizing work

**1. `ZoneChange { cause }` is where the collapse happens.** Discard (1,045
triggers), sacrifice (278), exile (74), mill (50), destroy-as-a-result, shuffle-
into, return, and put-onto-battlefield are all one variant plus a field on
`ZoneChangeCause`. That is **eight keyword actions and ~1,500 trigger-watching
cards absorbed by one arm.** It is the single most important structural decision
in §3.1 and the reason the enum does not track the card pool.

**2. The trigger vocabulary is strictly larger than the replaceable one, and
this phase only pays for the smaller.** Cast is the clearest case: 2,263 cards
trigger on it, **zero** replace it, so casting is a `GameEvent` with no
`GameAction`. Same for attack/block declarations and ability activation. So
`EventPattern` — one arm per `GameAction`, per §3.2a — stays smaller than the CR
603 matcher will eventually need, and CR 603's extra breadth lands in `GameEvent`
(already 28 variants) rather than here.

**3. Most events are of a kind nothing watches, which is what makes a gate
work.** Seven event kinds carry ~390 of the 574 replacement clauses. On a real
board the overwhelming majority of proposed actions are of kinds with zero
registered watchers, so §8's answer-preserving event-kind bitmask should skip the
gather sweep outright for most calls. That is now a measured expectation rather
than a hope — and it is still gated on measuring first.

### The projection

| Type | At end of RE | Commander-viable pool | Ceiling |
|---|---|---|---|
| `GameAction` | **~16** | ~30 | low 40s |
| `EventPattern` | = `GameAction`, by contract (§3.2a) | | |
| `Rewrite` | **5** | 5 | 5 + `AmountRewrite`'s clamp |
| `ZoneChangeCause` | **18, derived** (§11) | ~20 | ~20 |
| `ReplacementClass` | 5 | 5 | 5 (CR 616.1a–e is closed) |
| `Uses` | 4 | 4 | 4 |

The ceiling is derived, not guessed: 67 CR 701 actions, minus the ~30 that
provably decompose into other mutations, minus the 9 nothing watches, plus the
~10 non-701 core mutations (damage, life, draw, counters, zone change, tap,
untap, step begin, mana, game loss).

**For calibration, against types this codebase already runs on:**

| Existing enum | Variants |
|---|---|
| `Primitive` | 36 |
| `GameEvent` | 28 |
| `EffectModification` | 21 |
| `CounterType` | 16 |
| `ChoiceKind` | 13 |
| `Layer` | 10 |

`GameAction` at 16→30 lands squarely inside the size class of types that already
exist here and have not caused trouble — and unlike `Primitive`, its growth is
bounded by an enumerated CR chapter rather than by what cards decide to do next.

### The per-card cost, stated concretely

Adding a replacement-effect card costs, in the common case, **one `ReplacementDef`
value in `src/cards/*.rs` and nothing else** — that is §0 commitment one, and the
§3.2c pass says 549 of 574 clauses land there. When a card needs an event kind
the engine does not propose, it costs **three edits in three known files**: a
`GameAction` variant, a `perform_action` arm, an `EventPattern` arm. It never
costs a `Rewrite` arm — 0 of 574 did.

---

## 8c. Where card breadth actually lands — three axes, not one

§8b counted keyword actions, and review pushed back correctly: **plenty of
mechanically distinct cards are not keyword-shaped at all.** Alms Collector,
Don't Blink, Krark's Thumb, Tekuthal, Aeon Engine, Aboleth Spawn. The worry
behind the list — could `GameEvent` reach "thousands of entries", and would that
sink performance and maintainability — deserves a direct answer rather than
another count.

**The answer is that three different things are being sized as one.** Card
variety is real and unbounded. It lands almost entirely on the axis that is
*designed* to absorb it, and almost not at all on the enums.

| Axis | Shape | Grows with | Size |
|---|---|---|---|
| **1. Event kinds** — `GameAction`, `GameEvent` | closed enum | the engine's mutations, bounded by CR 701 | 16→50 (§8b) |
| **2. Predicates over events** — `EventPattern`'s field constraints, trigger conditions | **composed grammar**, not enumerated | card variety — **this is where breadth lands** | unbounded expressions, small grammar |
| **3. Results** — `Effect` / `then` | existing tree | card variety | already exists, already absorbs it |

Nothing in the design has a variant per card, which is the only way to reach
thousands. A card contributes a *value*, not an arm.

### The six cards, worked through

| Card | Event kind | Predicate (axis 2) | Result | New engine surface |
|---|---|---|---|---|
| **Alms Collector** — "if an opponent would draw two or more cards, instead you and that player each draw a card" | `DrawCards { player, n }`, the CR 121.2a *outer* event — Phase RE | `player is opponent && n >= 2` | `Instead(DrawCards{opp,1})` + `then:` you draw 1 | **none** |
| **Don't Blink** — "if one or more creatures would enter from exile or after being cast from exile, their owners shuffle them into their libraries instead" | `EnterBattlefield` — RC | `is creature && (from == Exile \|\| cast_from == Exile)` | `Instead(ZoneChange → Library, shuffled)` | **one struct field** — `StackEntry.cast_from: Zone` |
| **Krark's Thumb** — "if you would flip a coin, instead flip two coins and ignore one" | `FlipCoin` — CR 705, already on §8a's list | `player is you` | `Instead(FlipCoins{2})` + `then:` DP picks one to ignore | one variant, already budgeted |
| **Tekuthal** — "if you would proliferate, proliferate twice instead" | `Proliferate { times }` — Phase 8 | `player is you` | `Amount(Times(2))` on `times` | one variant, already budgeted |
| **Aeon Engine** — "reverse the game's turn order" | **not an event at all** | — | — | a `GameState` field |
| **Aboleth Spawn** — "whenever a creature entering under an opponent's control causes a triggered ability of that creature to trigger, you may copy that ability" | `AbilityTriggered { source, ability, cause }` — CR 603 | `cause is EnterBattlefield && cause.controller is opponent && ability.source == cause.object` | (a trigger, not a replacement) | **one `GameEvent` variant, carrying its cause** |

Total new engine surface across six deliberately awkward cards: **three enum
variants, all already budgeted in §8a/§8b; one struct field; one `GameState`
field. Zero new `Rewrite` arms. Zero per-card code.** Four of the six are pure
data.

Three of them teach something worth keeping:

- **Alms Collector proves the outer `DrawCards { n }` event is load-bearing**, not
  a convenience. CR 121.2a exists precisely so "would draw two or more" has an
  event to match; without the outer action the card is inexpressible. It also
  shows where "you *and* that player each draw" goes: the replaced event is the
  opponent's draw, and your draw is the `then` half (CR 615.5's shape). That
  boundary is a modelling judgment the CR does not always draw sharply — when it
  is ambiguous, ask which half can itself be replaced, because that half is the
  event.
- **Don't Blink needs one field the engine does not have** — where a spell was
  cast from. `StackEntry` carries controller, targets, modes, X and costs, but
  not the origin zone. Second customer: CR 903.8's commander tax counts casts
  *from the command zone*. (Flashback belongs to the same *class* — a stack
  object remembering how it got there — but not to this field: CR 702.34a keys
  on whether the flashback **cost was paid**, which `StackEntry.
  chosen_alternative_cost` already records.) One field, two customers, lands
  with RA.
- **Aeon Engine is the useful negative.** Reversing turn order is a
  game-rule-modifying continuous effect (CR 611.2c), not an event and not a
  replacement. The corpus already has `ATOM-611.2c-002` for exactly this class.
  **Not everything mechanically strange is an event** — some of it is a field on
  `GameState`, and mistaking the two is how an event vocabulary starts growing
  without bound.

**Aboleth Spawn is the one that looks worst and is not.** A trigger that watches
other triggers sounds like it needs the trigger system to know about itself. It
needs one `GameEvent` variant — "an ability triggered" — carrying *why* it
triggered. The card's specificity ("a creature entering under an opponent's
control causes that creature's own ability to trigger") is entirely axis 2. What
it genuinely needs beyond that is CR 706 ability copying and an inspectable
pending-trigger queue, which is CR 603's problem and already on its list.

### Performance: enum size is not the cost, and a bounded enum is the fix

Worth separating firmly, because the review put them together.

**Enum size costs nothing.** A `match` over 50 variants compiles to a jump
table. Going from 28 `GameEvent` variants to 50 is not measurable.

**What costs is watchers consulted per event** — and that scales with *board
size*, not vocabulary size. Concretely, at the v1 target: 4-player Commander,
~15–25 permanents each, so 60–100 permanents carrying maybe 100–300 abilities
between them. That is the population, and it is hundreds, not thousands.

**And this is where the closed enum pays for itself.** Because event kinds are a
small closed set, watchers can be indexed by kind in a dense array — a `u8`
discriminant into `Vec<Vec<WatcherId>>` — so an event consults only the handful
registered for its kind and skips the sweep entirely when that bucket is empty.
§8b's distribution says most buckets *are* empty: seven kinds carry ~390 of the
574 replacement clauses. An open-ended vocabulary (strings, per-card ids) would
force a hash lookup and make the empty-bucket fast path impossible. **The
bounded enum is not a constraint the performance story survives; it is the
mechanism the performance story runs on.**

### The genuine risk, named: axis 2 becoming a mini-language

The design's real exposure is not enum count. It is that `EventPattern`'s
constraint vocabulary and the filter types it borrows (`PermanentFilter`, 9
variants; `AmountExpr`, 9) grow one variant per awkward card until they are an
untyped DSL nobody can review. That is the failure mode to watch, and it is the
one this document cannot close by measurement today — it needs Phase 8 data.

Three guards, adopted now because they are cheap now:

1. **Compose, do not enumerate.** `And`/`Not`/`ByController`/`ByType` already
   compose; a new leaf must be a genuinely primitive question, never a card's
   whole condition spelled as one variant.
2. **Two customers before a variant.** A predicate leaf with exactly one card
   behind it is the warning sign. One customer is a card-specific predicate;
   several is a grammar feature.
3. **Budget an escape hatch, and measure it.** Every mature engine has per-card
   code for a long tail, and `design_doc.md` reserved `Custom(CardId)` for it.
   Keep that reservation. The boundary is principled: a card may get custom code
   for a unique *predicate* or *result*; it may never get a custom *event kind*,
   because that is what breaks the index above and hides the mutation from CR 614
   and CR 603 both. **If more than ~5% of the Phase 8 pool needs the hatch, the
   grammar is wrong and should be revisited rather than patched** — record the
   fraction in `codebase-state.md` as the pool grows.

### Should the grammar work move earlier?

Asked at merge (2026-08-24), given that the predicate grammar is the last open
item: is it worth pulling that work forward? **No — but change what RB ships,
not when it ships.**

**Reordering is not available.** The grammar's first real pressure is Phase RD's
two-sided damage predicates (CR 615.10's own Daunting Defender: "if a source
would deal damage to a Cleric creature you control"), and its strangest is RE's
*stateful* ones (Teferi's and Notion Thief both say "except the first one you
draw in each of your draw steps", which needs a per-turn draw counter, not a
filter). Neither can move: RD needs RB's pipeline and RA's batch form, RE needs
both. Moving them earlier means building the pipeline twice.

**But there is a real hazard in leaving it, and it is cheap to fix.** Everything
RB currently ships has a *trivial* predicate — shield/stun/finality counters are
"this permanent", regeneration is `SourceOnly`, and RC-2 is `SourceOnly`
unconditional by design. So `EventPattern` would be **defined in RB under no
pressure at all** and first stressed two phases later. That is the
designed-against-the-easy-case failure, and this project has paid for it before:
`AffectedSet::Filter` carried a controller snapshot until CR 109.5 proved it
wrong, because nothing at design time had a moving controller.

**The fix is one card, not a reordering.** Add a filter-based, two-sided
replacement to RB's card list so the grammar takes real weight the moment its
type is written. Kalitas, Traitor of Ghet is the natural pick — "If a nontoken
creature an opponent controls would die, instead exile that card and create a
2/2 black Zombie creature token" exercises `EventPattern` over a zone change, `AffectedSet::Filter` with
two clauses and an opponent-relative controller, and the `then` half, all at
once. It also immediately demands one grammar leaf `PermanentFilter` lacks —
nontoken — which is the *point*: it is a live test of the "two customers before
a variant" guard at the moment the guard is cheapest to apply.

**And one bounded research pass, not a census.** Predicates are free-form text,
so a regex bucketing would be the same instrument that just got the `sizing`
numbers wrong. Instead: read ~50 clauses by hand during RB and tabulate which
predicate leaves they actually need. An hour, honest, and it lands before RD
commits to a shape.

### Verdict

Manageable, on the evidence available, with one honestly open question.

Closed by measurement: axis 1 is bounded (§8b), `Rewrite` is closed and tested
at 0/574 (§3.2c), and six adversarially-chosen cards cost three budgeted
variants and two fields between them.

Open until Phase 8: whether the axis-2 grammar stays a grammar. That is the
right thing to be nervous about, and thinking about it now is not premature —
the guards above cost nothing today and are expensive to retrofit once a hundred
cards depend on the shape.

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
   `then` half, and it forces the first `PermanentFilter` leaf decision
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
  a spent use — and a use would have written `BattlefieldEntity.counters` from
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
  `PermanentFilter::Token` for its nontoken clause. `CreateTokens` as a
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
battlefield branch, and the branch was the `BattlefieldEntity` creation — which
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
`BattlefieldEntity`. Both facts are commented at the site; neither is
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
| replacement pipeline | `gather::set_affects` → `GameState::permanent_matches_filter` | `ReplacementDef.affected`, `RestrictionDef.affected` | **none** |

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
  `BattlefieldEntity::add_counters`, which is where the number ends up. The
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

§5c's finding, and the reason this is a PR rather than a paragraph inside RC-2.
`effect_applies_to`'s `game.battlefield.contains_key(&id)` gate at
`compute.rs:629` returned `false` for any filter-scoped `ContinuousEffect`
against an object not on the battlefield — so an entering Clone matched no
filter, **Dress Down included**, and kept the ability Dress Down should have
taken away.

**One line of code, and it is in the hottest path in the engine.** That is the
whole reason it is separated: `compute_characteristics` is what every layer
query runs, `layers-architecture.md` §12 measured the ungated CR 613.7a
existence check at 5.2×–8.0×, and a gate that starts admitting non-battlefield
objects changes what that walk does on every board. RC-3's deliverable is as much
the measurement as the behavior.

**Five findings.**

**1. The predicate is the battlefield *zone*, and that is what makes it free.**
`move_object` writes `obj.zone` before the `EnterBattlefield` performer builds
the `BattlefieldEntity` — RC-2's one-`emit`-wide window, documented in the
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
reads `chars.controller` — finding 5. `PermanentFilter::Or` is new, for
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
`PermanentFilter` has no zone leaf, so the scenario is inexpressible rather than
unimplemented. Determinism: three 200-game runs per pool byte-identical outside
`=== Timing ===`, and `--threads 1` vs `--threads 8` identical on the
engine-work block. Event streams at 40 games: **36/40 `performance` and 34/40
`stress` games byte-identical**, and every one of the 10 divergences has a
Humility or Blood Moon on the battlefield ahead of it and a permanent entering
modified under it. A behaviour change cascades where RC-1's deletion did not, so
the claim that can be checked is the *first* divergence per game — not the
whole-stream diff, which is noise past that point.

#### RC-4 — the overlay — ✅ landed 2026-09-02

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
   `Effect::Conditional` statics (item 7f) and Master Biomancer needs a dynamic
   counter amount (RC-5). What was buildable: an anthem creature for clause
   (2), Keldon Warlord for the count boundary, Containment Priest as the first
   replacement whose *filter* reads the frame — a Sol Ring returned under March
   of the Machines is a creature to it — and Dryad Arbor as the only road a
   fuzz game has to a creature that "wasn't cast". Dryad Arbor also reached a
   CR 205.1a bug in `apply_set_subtypes` (Blood Moon made it a Mountain with no
   creature type), fixed in its own commit.

6. **Two engine-cost changes rode along, both attributable.**
   `targeting::permanent_matches_filter` now takes one layer walk per filter
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
exactly one production place — `place_on_battlefield`, one `BattlefieldEntity`
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
same commit.** `order_invariant_entry_bucket`'s theorem has two halves — every
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

### Phase RD — damage (CR 615, 609.7, 614.9)

Prevention shields (615.7 amount, 615.8 next-instance, 615.9/609.7b property
recheck, 615.10 static per-event, 615.11 per-creature at resolution, 615.12
unpreventable + 615.12a single application), damage redirection (614.9),
doubling (701.10g), the simultaneous-damage shield allocation choice (615.7,
needs RA's batch), and CR 609.7a's source-choice validation.

Plus the **CR 120.3 results-of-damage decomposition**: performing `DealDamage`
against a player proposes a contained `LoseLife` (fresh lineage, §3.2d —
without it the §3.2c Bloodletter example never sees combat damage, its
headline use), and against a planeswalker removes that many loyalty counters
(CR 120.3c — unimplemented today; `perform_action` marks damage on any
battlefield object and nothing reads it off a planeswalker, so one can never
die to damage. Unreachable until a planeswalker is registered, but Lightning
Bolt's "any target" already validates them as targets). Lifelink's contained
`GainLife` is the same shape and is why RA routes it (RA item 11).

### Phase RE — the remaining event kinds we know of (see §8a)

Draw replacement (614.11, 614.11a/b, 121.2a's outer event, 121.6a empty
library), skips (614.10/a/b — per-player consumable `pending_skips` consulted at
step/phase/turn begin), token and counter doublers (614.16 — `CreateTokens`
as a proposal, which is also where RC-4b's token residual lands: a creation
whose destination the entry's decision sets, `codebase-state.md` item 52 and
"Before card breadth" item 8 — a Phase 8 back-stop, not a nicety; **and where
`Primitive::CreateToken`'s loop over `propose_entry` stops**, which is the
other half of `codebase-state.md` item 46 and the first *plural* entry the
engine will produce),
life-gain replacement (119.10), mana replacement (106.6a), and the three kinds
§8a's audit added: `PlayerLoses` / `PlayerWins` (CR 104, 6 cards) and the discard
pattern arm (CR 701.9, 17 cards).

### Interleaved — Commander

Per `CLAUDE.md`, the Commander/multiplayer track interleaves after item 5 rather
than sequencing against it. `GameConfig::commander()`, commander designation,
commander tax (CR 903.8, needs cost modification), and CR 800 priority rotation
are not gated on this doc beyond RB items 7–8.

**Cost modification needs a phase marker of its own** (audit 2026-08-25):
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
| `test_goggles_with_eligeth_draws_two_and_never_scrys` | §4.1a's instruction split + kind-changing `Instead` (needs the `Scry` event kind — RE at the earliest, §8a) | wrong draw count, or a scry event exists in the log |

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
| 2 | `AffectedSet` reuse | **Answered.** A constraint to preserve, not a question |
| 3 | Self-replacement (CR 614.15) plumbing | **Deferred past RB, as planned.** RB gave `SelfReplacement` its CR 616.1a bucket and no producer; `ResolutionContext` still has three fields. The field lands with the first card that needs it — and item 12 answers *who sets the class* |
| 4 | Replacement effects outside the battlefield | **Answered 2026-08-30 — item 9.** Sized at ~390 cards, and the blocker is not the sweep: it is CR 113.6, which the engine has nowhere and the layer system needs for the same cards |
| 5 | Overlay shape | **Answered** — read-side accessor, closed on measurement |
| 6 | Skips are not `execute_action` events | **Answered.** A design note; the work is in RE |
| 7 | `ScriptedDecisionProvider` blast radius | **Answered, and the watch held (RB, 2026-08-26).** Every test now traverses the pipeline and zero new prompts appeared. The rule was never relaxed |
| 8 | `then` timing | **Answered 2026-08-25 (audit).** Riders queue at application and resolve after the performed event (CR 615.5, 615.12); card-text "A, then B" never enters the pipeline as a unit (CR 608.2c). §4.1a |
| 9 | Batch `applied`-set scope | **Answered 2026-08-25 (audit).** Per event, never per batch — a first draft shared one set and Kalitas's own ruling refutes it; CR 704.7 is a same-result dedupe, not a share. §4.2 |
| — | **`ZoneChangeCause`** | **Not the list — the *catchall ban*.** Needed before RA's first commit; the list itself is derived, not researched. See below |

**The one that blocks — and it is not what the first draft said.** Review pushed
back that pinning `ZoneChangeCause` sounds like it needs a carefully crafted
Scryfall query, and could produce a long tail of variants with one card each.
That pushback exposed a framing error, and correcting it makes RA *less* blocked,
not more.

**`ZoneChangeCause` is not a card question.** It records *what the engine was
doing when it moved the object*, so it is derived from call sites, not
researched from the pool. The derivation is finite and already readable today:

| Source | Count | Where |
|---|---|---|
| `Primitive`s that move an object | **10** — Destroy, Exile, Sacrifice, ReturnToHand, ReturnToBattlefield, PutOnTopOfLibrary, PutOnBottomOfLibrary, ShuffleIntoLibrary, Mill, Discard | `types/effects.rs`, "Zone movement (rule 701)" |
| SBA sweep reasons | **8** — CR 704.5d tokens, 704.5f zero toughness, 704.5g lethal damage, 704.5h deathtouch, 704.5i loyalty, 704.5j legend rule, 704.5m/n attachment | `engine/sba.rs`, 5 live sites today |
| Stack exits | **4** — resolved→battlefield, resolved→graveyard, countered, fizzled | `engine/stack.rs`, the three `REPLACEMENT-BYPASS:` sites |
| Turn structure & casting | **3** — cleanup discard, draw, cast (hand→stack) | `state/game.rs`, `engine/zones.rs`, `engine/cast.rs` |

That is the whole input set, and the production tree currently has **13 zone-move
call sites**, of which **9 are labellable** — the other four are cast rollbacks
that leave the chokepoint instead (§9, RA-1). An afternoon of reading, no query
required.

Merging the 25 raw inputs down to the **18 variants** in §3.1 takes four
judgments, all checkable:

- **704.5g + 704.5h → one variant.** CR 701.8b calls both "destroyed"; no card
  distinguishes lethal damage from deathtouch as a *cause*.
- **704.5n is not a mover.** It unattaches an Equipment and leaves it on the
  battlefield, so it produces no zone change at all.
- **704.5d is not a zone change.** A token ceasing to exist is removal from the
  game, and `TokenCeasedToExist` already exists as its own event.
- **Library position is a field, not a cause.** Top, bottom and shuffled-in
  share `PutIntoLibrary`; *where* in the library is a parameter of the action.

Everything else survives one-to-one, which is what "derived" is supposed to
mean.

**Cards decide only the granularity, and they ask for less than the call sites
offer.** The demand side is short and flat (Scryfall, 2026-08-24): dies 1,287
triggers, sacrifices 278, discards 266, exiled 74, milled 50, destroyed 5. And
the feared long tail is measurably absent — **"destroyed by" appears on 1 card in
all of Magic, "was sacrificed" on 3, and "if it was destroyed" on 0.** No printed
card asks for a cause *finer* than the engine can name from its own call site, so
there is no research problem to solve.

**What actually blocks is the catchall, not the list.** Widening the enum later
is only expensive in one scenario: an existing site was labeled with a coarse
variant that should have been finer, and re-triaging it is guesswork that fails
silently. That scenario requires an `Other` / `Unknown` variant to lump into. Ban
it, and the failure mode disappears:

- **No catchall variant, ever.** Every call site names its reason. A site with
  nothing honest to say is a site whose reason nobody has worked out, which is
  the bug.
- **A genuinely new mutation arrives with its own new call site**, so it adds a
  variant and touches nothing existing — a normal diff, not an audit.
- The enum is `#[non_exhaustive]`-free and matched exhaustively wherever it is
  read (§3.1 limits that to the replacement pipeline and the trigger matcher), so
  a new variant fails to compile at every reader rather than defaulting.

So the pre-RA task is one line of policy plus an hour of labelling, and the
"get the list right first" framing was overcautious. §8b's projection of ~17 is
close to the derivation's 25 raw inputs before merging the ones that collapse
(the four stack exits share `from: Stack`; several SBA reasons differ only by
rule number) — confirm the merge at labelling time.


1. **CR 903.9 is half an SBA, and `codebase-state.md` said otherwise.**
   Current Oracle splits it: **903.9a** (commander in graveyard or exile → its
   owner *may* put it in the command zone) is a **state-based action**, listed
   at **CR 704.6d**, not a replacement effect. Only **903.9b** (hand or library)
   is a replacement — and it carries the rules' only explicit exception to CR
   614.5 ("may apply more than once to the same event"). Consequence: **the
   graveyard/exile half of Commander's zone redirection is not blocked on this
   phase at all** — `check_state_based_actions` already takes a
   `&dyn DecisionProvider`, which is everything 704.6d needs. Corrected in
   `codebase-state.md` alongside this document.

2. **`AffectedSet` is reused rather than re-invented, and the reuse is
   load-bearing.** `SourceOnly` vs `Filter` is precisely CR 614.12's "affects
   only that permanent (as opposed to a general subset of permanents that
   includes it)". If a future refactor collapses those variants, 614.12 breaks
   silently.

3. **Open — how self-replacement effects (CR 614.15) reach the pipeline.**
   They belong to the resolving spell/ability, not to any registry, so they
   arrive through `ActionContext::resolution`. `ResolutionContext` is
   `{source, controller, targets}` today and needs a fourth field. Low breadth
   (Aang's Journey and kin), so the hook lands in RB and the field lands with
   the first card that needs it. Do not build a general mechanism first.

4. **Open — replacement effects functioning outside the battlefield.** Source 2
   in §3.3. Deferred past RE. Size it before building it (`dont-over-defer`):
   count the cards, then decide whether it is a zone parameter on the sweep or a
   separate registry.

5. **The overlay's shape — closed by performance, not by taste.**
   `layers-architecture.md` §15.2 item 3 left "clone vs. CoW overlay" open for
   the dependency algorithm. Asked again in review — *are there performance
   considerations that favour one?* — and the answer is yes, decisively, but the
   two customers have to be priced separately because §5 establishes they are
   not the same operation.

   **CR 613.8's check should clone the frame, and that is already cheap.** Its
   perturbation is `EffectiveCharacteristics`-shaped: clone `chars`, apply one
   `EffectModification`, re-run `permanent_matches_filter`. `layers-architecture.md`
   §12 measured frame construction at **0.37 µs at N=10 falling to 0.27 µs at
   N=80** — flat in board size, "3% and flat" in its own words, explicitly not
   the bottleneck. So the expensive thing about a
   snapshot was never the frame; it was the idea of copying the *game*. 613.8
   does not need to.

   That matters because of where the check sits: up to O(effects²) candidate
   pairs per layer, inside a walk that runs per permanent, inside a sweep that
   runs per priority check. §12's table already shows this shape going
   superlinear when a per-effect cost is added — the ungated CR 613.7a existence
   check ran 5.2× at N=10 and 8.0× at N=80 for exactly this reason. A
   game-state-shaped snapshot in that position is not a slow path, it is a
   different complexity class.

   **CR 614.12's look-ahead genuinely needs game-state-shaped perturbation** —
   battlefield membership, controller, the entering object's own registry rows —
   but it runs **once per entering permanent**, which is a few times per turn.
   Its budget would tolerate a `GameState` clone.

   **It still should not use one**, and the reasons are correctness rather than
   speed: a clone duplicates `GameState.rng`, which the determinism doctrine
   forbids reaching for a second time; and it produces a second live copy of
   every `ObjectId`, which is a v4 UUID and therefore *aliased* rather than
   distinguishable, so any code that reads an id back out of the snapshot cannot
   tell which game it belongs to. A read-side view has neither problem and needs
   the accessor pair that §5's five audited sites want anyway.

   **Decision: read-side accessor pair, no clone at either call site.** Record it
   in `layers-architecture.md` §9/§15.2 when RC-4 lands, and measure the RC-4
   overlay with `fuzz_games --games 200 --seed 12345` against the pre-RC-4
   baseline — a look-ahead that runs a few times per turn should not move the
   number, and if it does, the accessor indirection has leaked into the hot walk
   and that is the bug to find.

   **Built and recorded (RC-4, 2026-09-02).** `engine/layers/lookahead.rs` is
   the overlay, `FrameCache` carries it, and `layers-architecture.md` §9 /
   §15.2 item 3 now say so. The measurement is in §9's RC-4 subsection.

6. **CR 614.10's skips are not `execute_action` events.** They replace the
   *beginning of a step/phase/turn*, which happens in `advance_turn`, not in a
   mutation. They still go through the pipeline (they are replacement effects
   per 614.1b/614.10), but their proposal is built by the turn machinery. Note
   also that `GameState.skip_first_draw` (CR 103.8a) is a **game rule**, not an
   effect, and stays a bool.

7. **Watch the `ScriptedDecisionProvider` blast radius.** Every existing test
   that reaches `execute_action` will now traverse the pipeline. The §4.1 rule
   — no DP call with fewer than two candidates — is what keeps that at zero
   prompts on today's card pool. If a phase finds itself relaxing that rule to
   make something work, it has found a design error, not a test problem.

### Answered on the review's second pass (2026-08-30)

`rb-review.md` themes D, F and H asked eight modelling questions rather than
reporting defects. Their answers live here because they are decisions about
shape, and a decision recorded in a findings ledger dies with the ledger.

8. **`apply_rewrite` does not grow per `(template, event)` pair, and the shape
   that would fix it is worse** (`rb-review.md` D1). The match is
   `template` × *{events that template can be built from}*, and every template
   so far reads at most one event kind: `ZoneChangeTo` reads `ZoneChange`'s
   `object` and `from`; `RemoveCountersFromAffected` reads nothing from the
   event at all — it takes the *subject* — so it matches `_`. Growth is
   therefore **one arm per template plus one error arm per template that
   constrains its input**, which is linear in `GameActionTemplate`.

   The symmetric alternative — make `GameActionTemplate` a projection of
   `GameAction` with `Option` fields meaning "inherit from the event", the way
   `EventPattern` is — is the wrong trade. A template's whole job is to say
   what it takes from the event, and templates cross event kinds: CR 122.1c
   turns a `Destroy` into a `RemoveCounters`. A uniform inheritance rule cannot
   express that, so the per-field rules come back as data instead of as code,
   and the error the current shape can give — "its `EventPattern` and its
   `Rewrite` describe different events" — becomes unreachable.

   **The tripwire is a template that must be built from more than one event
   kind, differently.** At that point move construction onto the template
   (`fn apply(&self, event, subject) -> Result<GameAction>`); that is
   mechanical and carries no rules content. Nothing before then. §3.2c's
   census is the evidence for the bound: 561 cards and 574 "would … instead"
   clauses put the pressure entirely on the `GameAction` vocabulary.

9. **§3.3 source 2 — static abilities functioning in other zones — is sized,
   and it should not be inside this phase** (D5, closing K3).

   **The count, Scryfall 2026-08-30: ~390 cards.** Six keyword families carry
   almost all of it — flashback 210 and buyback 39, rebound 35, aftermath 27,
   jump-start 13 (all CR-defined replacements functioning **on the stack**),
   madness 61 (functioning **in hand**) — plus five cards whose "would be put
   into a graveyard from anywhere, … shuffle it into its owner's library
   instead" functions in *every* zone: Blightsteel Colossus, Darksteel
   Colossus, Legacy Weapon, Nexus of Fate, Progenitus. Deliberately **not**
   counted: the 34 "…exile it instead" hits on the same query are disturb
   double-faced backs, and CR 712.8a gives a DFC only its front face's
   characteristics outside the battlefield and the stack, so they are ordinary
   battlefield sources.

   **The phase should close without it, and the count is not why.** The sweep
   is the easy half — one loop over a zone list instead of
   `battlefield_ids_ordered`. What source 2 actually needs is **CR 113.6**, the
   fourteen-subrule answer to "which of an object's abilities function in which
   zone", and the engine has that nowhere. A graveyard sweep that does not ask
   113.6 gathers Wonder's flying grant and Bridge from Below's trigger
   alongside the one replacement it wanted.

   **And it is not this phase's to build**, because the layer system needs the
   identical facility for the identical cards: `codebase-state.md`'s layers
   section already records that timestamps must move off `BattlefieldEntity`
   onto `GameObject` precisely because Wonder — a *continuous* effect
   functioning from a graveyard, CR 113.6b — has no timestamp to read. Two
   systems, one missing facility, and building half of it inside RA–RE would
   put CR 113.6 in `engine/replacement/`.

   **Sized, then:** (a) CR 113.6's zone-function predicate, shared with layers
   and eventually triggers (CR 113.6k); (b) an object timestamp off the
   battlefield, which the layer system owes anyway; (c) a **gate leg per zone**
   — `replacement_ability_sources` is populated at ETB, so a hand, graveyard or
   stack source is invisible to `gather`'s fast path, which is `CLAUDE.md`'s own
   "a new gather source needs a gate leg" rule; (d) the sweep. Schedule it with
   (b), after RE, not inside it.

10. **Skullbriar is the wrong reason to change the counter model, and there is
    a right one** (F1). Two cards want CR 122.2's exception — Skullbriar, the
    Walking Grave and Me, the Immortal (`o:/counters remain on/ -is:funny`,
    2026-08-30) — which is a thin case for touching counters. But the
    *capability* they need is counters on an object that is not on the
    battlefield, and that has a real constituency: **71 suspend cards** put
    time counters on a card in exile, and CR 122.1a and 122.1b are written for
    "a card in a zone other than the battlefield" in their own words. The
    engine can express none of it — `counters` lives on `BattlefieldEntity`,
    and `perform_action`'s `AddCounters` arm errors for anything else.

    **So: do not honour Skullbriar on its own.** The change is one move, the
    `counters` map from `BattlefieldEntity` to `GameObject`, and it is contained
    — 12 direct `.counters` sites outside `src/cards`, plus the
    `add_counters` / `remove_counters` / `counter_count` accessors. The one part
    needing thought is `CounterStack.timestamp`: CR 613.7c timestamps a counter
    as it is put on, and that only means something where layers read it.

    Once the map is on the object, Skullbriar costs a predicate at
    `move_object`'s clear point — CR 122.2's "counters cease to exist" becomes
    "unless the object's effective abilities say otherwise". **Trigger:** the
    first suspend card or the first CR 122.1b keyword counter off the
    battlefield. `codebase-state.md` Deferred Migrations.

11. **"Unconditional once queued" is right, and RB cited the wrong rule for
    half of it** (F3). Three separate things were tangled:

    **The motivating card is out of scope.** Academy Rector's "you may exile
    it. If you do, search…" is a *triggered ability*; card-text "A, then B"
    resolves inside the effect (CR 608.2c) and never enters the pipeline as a
    rider at all — §4.1a's "the other 'then'".

    **The objection survives its example.** CR 615.12 is prevention-only, and
    the code cited it for every rider. The correct citation for an `Instead`
    rider is CR 614.1a + 614.6: the "and also" is part of what happens instead,
    so it happens when the substitution does. Same conclusion, different
    argument — and the case that separates them is an `Instead` queueing a
    rider and a *later* replacement in the same 616.1f loop then preventing the
    surviving event. Under 615.12 the rider still runs; under 614.6 the rules
    say nothing, and "the modified event never happened" is at least arguable.

    **Unreachable today, and checkably so.** RB's only `Prevent` producers are
    CR 122.1c's shield damage half and CR 701.19a's regeneration, which watch
    `DealDamage` and `Destroy`; RB's only `Instead`-with-a-rider is Kalitas,
    whose output is a `ZoneChange` to exile that nothing watches. No RB batch
    can build the case.

    **And "if you do" is not a rider.** It is a conditional inside an effect,
    and `then` is an `Effect`, so it lands as `Effect::Conditional` (Phase 6)
    with no replacement vocabulary at all — which is exactly what §3.2's
    "per-mechanic variety goes in `then`" was supposed to buy. Authoring an "if
    you do" as a bare unconditional `then` is a card-authoring error and should
    be called one. **RD owns the decision**, with item 15, because RD is where
    `Prevent` gets producers.

12. **CR 614.15's `SelfReplacement` class is *derived*, never authored** (F5),
    and this codebase has already settled the general form of the question.

    614.15 says a self-replacement effect "is an effect of a resolving spell or
    ability that replace[s] part or all of that spell or ability's own
    effect(s)". That is **pure provenance**: the identical sentence printed as a
    static ability on a permanent is not a self-replacement, and no text a card
    can print makes an effect self-replacing on its own.

    The precedent is CDAs. `AbilityDef.is_characteristic_defining` asserts only
    what the ability's *text* satisfies, while CR 604.3a(2)'s provenance is
    owned by whoever writes the ability onto an object — `CLAUDE.md` states it,
    and a Layer 6 `GrantAbility` must clear the flag. CR 614.15 is the same
    split with **no text half at all**, so the flag has nothing to assert.

    **What that means when item 3's producer lands:** `gather` stamps
    `class = ReplacementClass::SelfReplacement` on every instance it builds
    from `ActionContext::resolution`, and a card file never writes it. Assert
    the other direction too — no authored `ReplacementDef` carries the class —
    which is the analogue of the Layer 6 grant clearing the CDA flag. Note the
    nuance 614.15 adds and the derivation survives: "the text can be a separate
    ability, particularly when preceded by an ability word", so the class
    cannot be derived from *which ability the text sits in*; it is derived from
    the effect arriving through the resolution.

13. **`ZoneChangeCause::Returned` keeps both returns, and the reason is
    structural** (F7). §11's merge criterion asks whether a printed card
    distinguishes two reasons *as a cause*. It does not apply here: the two
    "returns" differ by **destination**, and destination is a field on the
    event. The enum exists for what `(from, to)` cannot recover — a sacrifice
    and a destruction share `(Battlefield, Graveyard)` — and return-to-hand and
    return-to-battlefield share nothing.

    Checked against the pool anyway: every card printing "is returned to" names
    its destination (5 cards, Scryfall 2026-08-30 — Azorius Aethermage,
    Justice Vance Astrovik, Puppet Master, Stormfront Riders, Warped Devotion;
    all of them "hand"). Nothing watches a return without a zone, so the
    trigger matcher always has both halves.

    **The tripwire runs the other way.** If a card ever needs "returned to the
    battlefield" as distinct from "put onto the battlefield" at the *same*
    `(from, to)`, that is a missing variant on the *put* side — not a split of
    `Returned`.

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

15. **RD's design must open with this: CR 614.5's identity may be per
    `(event, affected object)`, not per batch member** (H9).

    §4.2 keys the applied set per batch member. Two printed rulings pull in
    opposite directions against that, and both are real. Kalitas's says a
    board wipe killing N of an opponent's creatures makes N Zombies — different
    objects, N applications, which the per-member shape gets right. CR 122.1c's
    says that when several sources damage one shielded creature at once, **one**
    shield counter is removed — but CR 510.4 makes simultaneous combat damage
    one event, and the engine models each source as its own batch member with a
    fresh applied set, so a creature blocking two attackers with two shield
    counters loses two. (With one counter the answer is accidentally right: both
    damages are prevented and the second rider removes nothing.)

    **Keying per `(event, affected object)` within a batch reconciles them
    exactly** — Kalitas's deaths are different objects, the two damages are one
    object.

    **Why it cannot wait for a card.** CR 615.7 is "one prevention shield,
    several simultaneous sources, the shield's controller chooses how much to
    apply to each" — the same modelling question asked officially — and building
    615.7's allocation on the per-member shape gives each member its own shield
    with nothing to allocate across. What the change costs is small and is about
    *ownership*: `apply_replacements` already takes the applied set as a
    parameter, so `execute_batch_inner` would key one map by affected object
    instead of handing each member a fresh clone. §3.2d's `inherited` lineage
    rule is untouched — it is about decomposition, not simultaneity.

### Found by a rider read-through (2026-08-30)

Three items from reading `ReplacementDef.then` end to end against
`engine::resolve`. §4.1a settled *when* a rider runs; none of these was asked
there, because all three are about **what a rider can reach**.

16. **`Rider.subject` is `Option<ObjectId>`, and the `Option` is doing two jobs.**
    One is honest: an event has exactly one subject — `subject_of` is total over
    `GameAction` and returns a scalar — so a rider carrying one subject records
    the *event's* arity rather than any limit on the rider. The other is a
    silent loss. `subject_object` maps `EventSubject::Player(_)` to `None`, so
    the player case does not survive into the `ResolutionContext` at all.

    **The fix is to stop flattening, not to widen.** Carry `EventSubject` and
    let `resolve_rider` emit `ResolvedTarget::Player(pid)`; both types exist and
    `resolve_player_for_self` already reads that variant. `codebase-state.md`
    item 27 has the sizing and the trigger.

    **Why it earns a line rather than a fix now:** the failure is not an error,
    it is a rider quietly acting for the effect's controller where the card said
    "that player". Notion Thief hides it, because for Notion Thief those are the
    same player — which is exactly why the one card in the tree does not catch it.

17. **A rider's object reach is its subject, and that ceiling belongs to the
    `Effect` tree rather than to this document.** `resolve_primitive` consults
    the `EffectRecipient` only for player-directed primitives; the 24 arms that
    affect objects read `ctx.targets` and ignore the recipient's
    `SelectionFilter` and `TargetCount` entirely. `EffectRecipient::Choose`
    therefore buys a rider nothing today, and `regeneration_rider`'s filter and
    count are inert data.

    **This pins the same boundary item 11 pins, and it is worth stating once for
    both.** `then` is an `Effect`, so anything a rider cannot say is something
    the `Effect` tree cannot say. The test for whether such a gap is *this*
    document's is whether closing it changes `types/replacement.rs`. For "if you
    do" (item 11) and for set-valued recipients, it does not — which is the
    §3.2 growth contract working as intended, not a hole in it.

    `codebase-state.md` item 28 has the arm count and why it is deliberately
    left unsized.

18. **§3.2d's lineage rule ships with no producer**, and the parameter that
    carries it is handed an empty set at its only call site. Recorded because
    the failure mode §3.2d names is a **hang** rather than a wrong answer, and
    because "a correct mechanism with no customer" is precisely the shape
    `codebase-state.md`'s Deferred Migrations section exists to catch. Item 29
    there has the sizing; the regression is the one §3.2d already names,
    `test_two_teferis_draw_four_not_infinity`, and **RE** is the phase that
    needs it.

### Found by asking what RC-3's own test proves (2026-09-02)

19. **CR 616.1 prompts for a choice with one outcome on every entry, and for
    `Rewrite::EnterWith` that is a theorem rather than a coincidence.** Raised
    against `test_two_registered_cards_make_cr_616_1_ask`: Root Maze and
    Idyllic Beachfront both rewrite one `EnterBattlefield` into
    `EnterWith(tapped)`, the pipeline asks which applies first, and **no
    assertion can tell the two orders apart**. Three facts make that general:

    - `EnterMods::merge` is `tapped |= other` and per-kind counter `+` — both
      commutative and associative.
    - `EventPattern::EnterBattlefield` matches `GameAction::EnterBattlefield { .. }`
      on the variant alone, and `set_affects` keys on the event's *subject*.
      Neither reads `mods`, so applying an `EnterWith` cannot change which
      effects are applicable on the next CR 616.1f iteration.
    - So for a bucket whose members are all `EnterWith`, the loop is
      order-invariant. The prompt is real, CR-mandated and pure noise.

    **The v1 stake is the CLI harness**: "highly parallel AI games over the
    CLI" pays a decision round-trip per entry under two entry replacements, for
    an answer that cannot matter.

    **The sound rule is narrow and provable, and it is not "collapse identical
    rewrites".** That would be a semantics-assuming shortcut of exactly the kind
    `layers-architecture.md` §12 item 3 says never to reach for. The provable
    form is: *skip the prompt when every member of the bucket is a
    non-optional, non-counter-derived `EnterWith` with no `then`*. The `then`
    exclusion is load-bearing — riders queue in choice order and run in queue
    order (CR 615.5), so two candidates that both carry one make the order
    observable through the event log even though the board is identical.

    **Do not ship it without the card that keeps the branch alive**, and this is
    the whole reason it is a question rather than a patch. Suppressing the
    prompt returns CR 616.1's multi-candidate branch to dead code in a fuzz
    game — the ordering choice, the applied set across instances and CR 101.4's
    APNAP ordering among simultaneous choosers — which is the Kalitas gap for a
    third time, now caused by a fix. What keeps it reachable is a registered
    entry replacement that **does not** commute: a `Rewrite` that drops the
    event (CR 614.6) beside an `EnterWith`, where the order decides whether the
    `EnterWith`'s CR 614.5 slot is spent and its rider queues at all.
    Containment Priest is the printed shape and needs a "wasn't cast" predicate
    `EventPattern` does not have. **Order: card first, suppression second.**

    **Landed in RC-4 (2026-09-02), card first** — Containment Priest, the
    `Instead` beside an `EnterWith` — **and the predicate is narrower than the
    theorem above, because the frame falsified one premise.** "Neither
    `EventPattern::EnterBattlefield` nor `set_affects` reads `mods`" stopped
    being true the moment `set_affects` read the look-ahead: a `PowerLE`
    filter matches a 0/0 before its +1/+1 counters land and not after, so
    Adaptive Shimmerer under "creatures with power 1 or less enter tapped" is
    a real choice — tapped or untapped by the order — and
    `test_a_power_filter_beside_counters_is_a_real_choice` says so. The rule
    that shipped (`pipeline::order_invariant_entry_bucket`) admits only members
    whose applicability no `EnterMods` field can move: `EnterWith`, mandatory,
    static, under CR 614.5, not counter-derived, no rider, and an `affected`
    over leaves the counters cannot reach (`filter_is_mods_invariant`). It is
    a semantics-assuming shortcut in `layers-architecture.md` §12's sense, so
    it carries its three expiry conditions in code and in `codebase-state.md`
    and a debug-build check that re-gathers after the suppressed choice and
    asserts the rest still apply.

---

20. **The replaceable event must be the outermost proposal** (found by the
    RC-4 review's nesting audit, 2026-09-02). A performer may propose a
    contained event only when the outer event is real whether or not the
    contained one survives replacement — otherwise the log records the outer
    half of something the CR says never happened, and no keying rule for a
    trigger matcher can repair a wrong `from`. Every nesting in
    `perform_action` and its callers, audited against that test:

    | Nesting | Is the outer event real on its own? | Verdict |
    |---|---|---|
    | `ZoneChange { to: Battlefield }` → `EnterBattlefield` | **No** — entering *is* the zone change (CR 614.1c, 603.6a) | the entry hop; ✅ RC-4b |
    | `CreateToken` → `EnterBattlefield` | **No** — the token is created in the zone before its entry is decided | ✅ RC-4b, the cheap token answer (§9) |
    | `cast_spell`'s 601.2a move → the rest of casting | **No** — CR 601.2 rewinds it, and four rewind sites do, silently | ✅ RC-4b, design item 7 |
    | `Destroy` → `ZoneChange { Destroyed }` | Yes — CR 701.8b; regeneration replaces the outer, a finality counter the inner | fine |
    | `DrawCard` → `ZoneChange { Drawn }` | Yes — CR 121.1; a draw replacement (RE) replaces the outer and the move never proposes | fine |
    | `DealDamage` → `LoseLife` | Yes — CR 120.3's results of damage | fine |
    | lifelink → `GainLife` | Yes — CR 702.15 | fine |
    | cost payment → `Tap` / `Sacrificed` | Yes — paid after 601.2h's can-pay check, and no rewind site exists after payment begins | fine |

    The test to apply to any new performer: if the contained proposal were
    replaced with nothing, would the CR still say the outer event happened?
    Destroying a regenerated creature, drawing from an empty library and
    dealing prevented damage all answer yes. Entering and casting answer no,
    and both have to be one proposal or none.

## 12. Explicitly out of scope

- **Layer 1 / the copy system (CR 707).** 23 Phase-6 atoms, a separate system.
  CR 616.1c gets its ordering *bucket* in RC-4 so the classification is complete,
  but nothing produces a copy-on-enter replacement until Layer 1 lands.
- **CR 614.14 / 607 linked abilities.** Needs the CR 607 work (`backlog.md` §2.2).
- **CR 614.12b** — "combined costs of those effects to not be payable" across
  simultaneous entries. Needs cost modification; revisit with commander tax.
- **CR 614.12c anchor words.** Linked abilities again.
- **CR 615.13** — triggers on prevention. Phase 7.
- **CR 731 loop detection.** Survives from `state-tracking-architecture.md`
  Tiers 1–3, re-based on performed-action transcripts; not this phase.

---

## 13. Documents this phase owes

Update as part of the work that changes them, not in a later pass:

- `codebase-state.md` — ✅ through RB. The CR 614–616 row, the CR 9 table
  (item 11.1 above), and Deferred Migrations: **item 3 closed with RA-3, item 2's
  three bypasses closed with RA-3**, and two new items were opened at commit time
  (the CR 601.2a announcement, and the SBA mutations with no `GameAction`
  variant). Keep adding a line per stub.
- `CLAUDE.md` — ✅ through RB. The authority-table row exists; the chokepoint
  invariant is stated, and RA-3 added its one-performer/one-emitter and
  simultaneity sub-rules and struck the `// REPLACEMENT-BYPASS:` exemption.
- `layers-architecture.md` §9 / §15.2 item 3 — ✅ the overlay decision,
  recorded with RC-4 (2026-09-02). RA-3's LKI frame had needed neither the
  accessor pair nor a clone — a `compute_characteristics` call taken *before* a
  mutation is not a hypothetical about a perturbed board — and RC-4's frame is
  the first thing that did.
- `cards-unlocked-ledger.md` — ✅ RB's entry is in. The ETB unlock is the largest
  single entry the ledger will take; add it with RC.
