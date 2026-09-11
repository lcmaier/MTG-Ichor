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
own perturbation is frame-level and much cheaper; §5 and §11 item 5 price both.) **Since 2026-09-04 it does precede
triggers** (`roadmap-v2.md` §3a): the accessor pair landed with RC-4, so the
waiting bought what it could, and the performance pool now builds a 613.8
wrong answer. Nothing here changes — RA–RE stay ahead of CR 603.

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
target it is **both** of CR 614.1's halves: `AffectedSet::Filter {
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
and the two effects trade cards until CR 104.4b calls the game a draw. Measured
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
2. **Static abilities functioning in other zones.** Same sweep, other zones.
   Deferred past RE; recorded in §11 with its breadth.
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
- **Fresh lineage — and that is a constraint on what may be a rider, not just a
  fact about one.** A rider's actions are new events the replacement caused, not
  modified forms of the original, so they re-enter the pipeline with a fresh
  applied-set (§3.2d containment). Not theoretical in either direction: Kalitas
  plus Doubling Season makes two Zombies, because the rider's `CreateTokens` is
  itself replaceable; and Alms Collector's draws had to *stop* being riders,
  because CR 614.5 covers "any modified events that may replace that event" and
  its own ruling says it does not apply to the draw it produced. Reading this
  bullet as permission to put an effect's whole output in `then` is what
  produced that loop, so §3.2d states the converse as a card-authoring rule:
  whatever must carry the lineage goes in the rewrite.

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
  | What is re-evaluated | the whole layer walk for one object | `object_matches_filter(A.filter, chars, …)` |
  | Cost of the perturbation | game-state-shaped | `EffectiveCharacteristics` clone — measured 0.37 → 0.27 µs over N=10–80, i.e. flat (`layers-architecture.md` §12) |
  | Frequency | once per entering permanent | up to O(effects²) per layer, inside a per-permanent walk |

  613.8's check is **frame-level**. "Recompute A's `affected` with B applied" is
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
controller matters (`codebase-state.md` item 9) and buys no `ObjectFilter`
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
| `engine/cast.rs::activate_ability` | emit `AbilityActivated`; resolution emits identity-bearing `AbilityResolved` | RA |
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
constraint vocabulary and the filter types it borrows (`ObjectFilter`, 9
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
once. It also immediately demands one grammar leaf `ObjectFilter` lacks —
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

**First data point, RD-3 (2026-09-09).** The section predicted RD's two-sided
damage predicates as the grammar's first real pressure, and they landed:
`EventPattern::DealDamage` grew a `SourcePattern` and eight printed cards were
written against it. **`ObjectFilter` grew by nothing.** The leaves the source
side reached — `ByColor`, `ByController`, `And` — all existed with customers
of their own, and guard 2 refused the one leaf a card wanted: "the effect's own
host as the source", one customer (Sokrates, Athenian Teacher), recorded rather
than written. Guard 1 is what made that possible — composition meant Torbran's
whole two-sided condition was two existing leaves and an `And`, not a variant.
One data point is not the answer to the open question, but it is the answer
going the right way, and it is the *hardest* single case the phase plan named.

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

Read against CR 615 whole, 614.7a, 614.9, 609.7a–c, 120.3, 120.3c and 701.10g
(`MTG-Rules/versions/tmnt.txt`), §4.1's six rules, §3.2c/d, §8a, §8c and §11
items 11, 15, 16, 18, 20. Every consumer named below had its oracle text and
rulings fetched from Scryfall on 2026-09-08 (`engineering-practices.md` §3.4);
the counts are from the same sitting.

**0. The affected set must name a player, and that is RD's, not RE's.**
`types/replacement.rs` records the decision that `EventPattern` gets no
`DrawCard`/`GainLife`/`LoseLife` arm until "the player-scoping mechanism" lands
in RE with the draw cards, and `gather::set_affects` returns `false` for every
`EventSubject::Player`. That was the right call for RB and it does not survive
contact with the damage family: of the consumers below, only Daunting Defender,
Samite Censer-Bearer and Pyroclasm's targets are object-only. Circle of
Protection, Reverse Damage, Guardian Seraph, Safe Passage, Pariah, Palisade
Giant, Fog, Mending Hands' "any target", Kitsune Palliator's "each player" and
Furnace of Rath's "permanent or player" all shield or modify damage *to a
player* — Scryfall: `o:"prevent all damage that would be dealt to you"` **23**,
`o:/would deal damage to (you|a player), prevent/` **12**, and nearly all of the
**46** "source of your choice" cards say "to you". A damage phase that cannot
scope an effect to a player has no consumers. **So RD-1 builds it**, and RE
inherits it.

The shape is **a second field, not a variant**. `AffectedSet` is read by three
systems — the layer walk's `row_affected`, the restriction sweep and this
pipeline — and a `Player` arm would be a variant two of the three must reject at
every match, which is the "one type with a flag" smell §11 item 2 warns about
from the other side. `ReplacementDef` gains `affected_players: PlayerSet`
(`{ Nobody, You, Opponents, Everyone, Fixed(Vec<PlayerId>) }`, resolved against
the instance's controller exactly as `Filter`'s `PlayerRef` is, CR 109.5), with
union semantics: `set_affects` consults `affected` for an object subject and
`affected_players` for a player subject. Fog is `Filter { All }` + `Everyone`;
Safe Passage is `Filter { creatures you control }` + `You`; a Circle is
`Fixed(vec![])` + `You`; a targeted "any target" is filled in at resolution as
`Fixed([object])` or `Fixed([player])` the way `Primitive::Regenerate` fills its
set today. Nothing about `AffectedSet` moves, so §11 item 2 holds byte for byte
and the CR 614.12 `SourceOnly` check in `gather` is untouched.
`Restriction::ApplyReplacement { to }` needs the same second field for
"damage can't be prevented" over damage to a player; RD-4 adds it there.

Two things ride on this field. `chooser_for` already answers a player subject
with that player, so CR 616.1's chooser is free. `Rider.subject` flattens a
player subject to `None` (§11 item 16, `codebase-state.md` item 27) and its
trigger was named as RD; RD-1 carries `EventSubject` on the rider and emits
`ResolvedTarget::Player`, because Reverse Damage's "you gain life" is the
first rider that rides routinely on a player-subject event.

**`Uses::NextDamage(remaining)` is named for the rule's own phrase**, "the next
3 damage", because it counts damage and never uses: 615.7's last sentence is
"such effects count only the amount of damage; the number of events or sources
dealing it doesn't matter", and a first name, `DamagePoints(remaining)`, could
be read as either. It deliberately does not take the word "shield" in code —
**three different things are called a shield here**, and which is which is
`plans/glossary.md`, under *shield*. That disambiguation was written out in this
section, because RD-2 is where the three meet; the glossary pass (2026-09-11)
moved it, since a reader who needs it is usually not reading RD. Decision 3
below is still the one place senses 2 and 3 meet.

**1. Partial prevention is `Rewrite::Amount(AmountRewrite::PreventUpTo(n))`,
and `Instead` gets no "N − k" template.** §3.2b already lists `Amount` as the
sixth arm with its rule numbers — CR 614.5's own doubling example and 615.7 —
and it is absent from the enum only because an arm the pipeline cannot apply is
worse than a missing one. So this is the planned arm arriving, not a new claim
on the closed algebra. CR 615.10 is the sentence that permits *partial*
prevention as an operation on the event's amount ("prevents only the indicated
amount of damage in any applicable damage event") and 615.7 the same for
shields ("each 1 damage … is prevented"). Four reasons it is not an `Instead`
carrying `DealDamage { amount: N − k }`, and the first is the CDA lesson:

- **Two channels to one answer.** An `Instead` template that reads the event's
  amount and subtracts is the same arithmetic as `Amount`, spelled in the
  unbounded arm. The tree would then have two ways to say "prevent 1 of that
  damage", and a reviewer could not tell an authoring error from a choice.
- **Composition is the whole test.** Furnace of Rath's printed ruling —
  prevent 4 then double the remaining 1, or double to 10 then prevent 4 — is
  CR 616.1's choice made *observable* only because each application reads the
  amount the previous one left. `Amount` arms do that by construction; an
  `Instead` that overwrites the event does it only if its template happens to
  read the right field.
- **The pipeline has to know how much was prevented.** CR 615.5's rider may
  "refer to the amount of damage that was prevented" (Reverse Damage, Divine
  Deflection), a 615.7 shield is reduced by exactly that amount, and CR 615.13
  triggers on "some or all" of it. An `Instead` erases the number; `Amount`
  reports it.
- **`Prevent` stays a separate arm.** "Prevent that damage" (CR 615.6, the
  whole event never happens) and "prevent 3 of that damage" (a smaller event
  survives for later effects to see) are different claims, and regeneration and
  CR 122.1c already use the first. A `PreventUpTo` that empties the event
  leaves a 0-damage proposal that `never_happens` drops on the next iteration
  (CR 614.7a), so the two routes agree on the board and differ only in what
  they assert.

`AmountRewrite` ships six variants, each with a customer in the PR that lands
it: `Multiplier(u64)` (Furnace of Rath; CR 701.10g — not `Times`, which reads
as "the number of times something happens" and is the wrong noun for a factor;
§3.2c's and §3.2d's `Amount(Times(2))` are renamed with it), **`Halve(Rounding)`**
(Ghosts of the Innocent, "deals half that damage, rounded down … instead" —
the inverse, and it is printed), `Plus(u64)` (Torbran, RD-3), `PreventUpTo(u64)`
(Daunting Defender, Guardian Seraph; CR 615.10), **`PreventHalf(Rounding)`**
(Gisela, Blade of Goldnight's "prevent half that damage, rounded up" in RD-1;
Dark Sphere's "rounded down", from a resolution with a chosen source, in RD-3),
and `PreventRemaining` (CR 615.7), which reads its cap off the instance's
`Uses::NextDamage(remaining)` — the remaining amount lives in one place, the use
count, and the arm names it rather than repeating it.

**Rounding is authored, never inferred.** CR 107.1a: "If a spell or ability
could generate a fractional number, the spell or ability will tell you whether
to round up or down." So `Rounding { Up, Down }` has no `Default`, and every
halving names its direction — the same doctrine as `Duration` on
`Primitive::Restrict`. `Halve` and `PreventHalf` are two arms and not one with
a flag because CR 615.12 treats them differently and the rulings say so in
words: Ghosts of the Innocent "isn't a damage prevention effect" and still
halves Excruciator's unpreventable 7 to 3, while Gisela prevents nothing of
it; and only the prevention arm reports a prevented amount. Ghosts' other
rulings are RD-1's tests: *half of 1 rounded down is 0, so a 1-damage source
deals no damage at all* (a rewrite to 0 meets `never_happens` on the next
iteration, CR 614.7a); *three Ghosts turn 14 into 7, 3, 1* (three instances,
each once); *with Furnace of Rath the order is the affected player's, and it
matters when the amount is odd — 3 halves to 1 then doubles to 2* (the first
non-commuting CR 616.1 choice reachable from two printed statics, and
`COMP-614-DAMAGE-ORDERING-001`'s board); *redirected damage is halved once*
(CR 614.5's applied set survives a `Retarget`, RD-4). A rider can halve too —
Sokrates, Athenian Teacher's granted "each draw half that many cards, rounded
down" — but that rounding sits on the rider's `AmountExpr` (a `Half(Box<
AmountExpr>, Rounding)` leaf beside `Multiply`), not on any rewrite; Sokrates
himself waits on RS-2's hexproof and is recorded under RD-3 as the shape.
§3.2c's
Ali from Cairo clamp is **not** here: Ali's own ruling says "this effect does
not prevent damage, it prevents the damage from turning into loss of life", so
it watches the contained `LoseLife` and is RE's, once RE gives `LoseLife` a
pattern arm (decision 4 gives it the `cause` field Ali needs).

**2. Resolution-created prevention lives in `ReplacementEffectRegistry`,
through `Primitive::CreateReplacement(Box<ReplacementDef>, Duration)`, and the
shield fits a duration registry because a row already has two ends.** Settled
against its two twins rather than alone: RS-1's `Primitive::Restrict(def,
Duration)` into `RestrictionRegistry`, and `cost-architecture.md` §3.10's
`Primitive::ModifyCost(def, Duration)` into a `DurationRegistry` of its own.
This is the third instance of one pattern — a def the card authors, a
`Duration` the card authors (CR 608.2c hands scope to a human reader, so no
engine may infer it; `cant-effects-architecture.md` §9 finding 1), a row
carrying controller and `created_on_turn`, expiry through the CR 514.2 hooks
the registry already runs. The registry is not new: `Primitive::Regenerate` has
been putting rows in it since RB. The commented-out
`Primitive::ApplyPrevention(PreventionEffectDef)` in `types/effects.rs` is
deleted rather than filled in — CR 615.1 opens "like replacement effects", a
prevention effect *is* a `ReplacementDef` whose rewrite prevents, and a second
def type would be one mechanism reached through two channels.

The consumable-amount question has an in-tree answer. A regeneration row is
`Uses::Once` **and** `Duration::UntilEndOfTurn`: it ends on use through
`consume_use`'s `remove(row)` and on time through
`remove_expired_at_cleanup`, and neither end knows about the other.
`Uses::NextDamage(remaining)` is the same row with a number where `Once` has a
bit — decremented in place through `DurationRegistry::update_rows`, removed at
zero, expired at cleanup with the rest. "Expires on use, not on time" was the
wrong dichotomy: it is both, it always was, and the registry was built for it.
(`types/replacement.rs` promised the variant as `Shield(u64)`; the name changes
for the reason the glossary above gives, and RD-2 corrects the comment.)

What the primitive does at resolution follows `Regenerate` and `Restrict`: one
row per resolved target, the resolution filling an authored empty `Fixed` with
that target (CR 615.11 is exactly this — "creates a prevention shield for each
applicable creature when the spell or ability … resolves" — so Samite
Censer-Bearer is N rows of `NextDamage(1)`, one per creature it found, and Kitsune
Palliator's ruling "doesn't affect creatures that enter later" falls out of
`Fixed`); or one row as authored when the def carries a `Filter` or a
`PlayerSet` and no target (Safe Passage's ruling, the opposite one: "will
prevent damage dealt to creatures that weren't on the battlefield at the time
it resolved" — `Filter`, evaluated at the event). A pooled amount across
several subjects needs no third shape: a `Filter` + `PlayerSet` row carrying
`NextDamage(n)` *is* one pool by construction, which is Divine Deflection's
"the next X damage that would be dealt to you and/or permanents you control"
— waiting only on `AmountExpr::Variable` — and Harm's Way's 2, which waits on
nothing here and is RD-5's (finding 23).

**3. CR 615.7's allocation ships with the first shield, in RD-2, and it is
the same change as §11 item 15.** "If damage would be dealt to the shielded
permanent or player by two or more applicable sources at the same time, the
player or the controller of the permanent chooses which damage the shield
prevents." Two blockers on one attacker, or two attackers on one player, is
that board, and the registered pool builds it in every combat step — so the day
Mending Hands is registered the first-come answer the per-member loop gives is
a **silent wrong choice**, the shape `engineering-practices.md` §3.3 says a
phase must not open. It cannot be a later PR's.

Item 15 asked RD to open with CR 614.5's identity, because a creature blocking
two attackers with two shield counters loses two counters where CR 122.1c's
ruling says one. The allocation is the same question from the other side, and
one rule answers both: **decisions are per `(batch, subject)`; rewrites apply
per member.** In phase 1, members that share an `EventSubject` form a group;
the CR 616.1 loop runs once per group, with one applied set and one chooser;
a chosen instance's rewrite is applied to every member of the group, its rider
queued once, its use spent once. Checked against every ruling the batch has
had to satisfy:

| Board | Per member (today) | Per subject (RD-2) | The rule |
|---|---|---|---|
| Kalitas, N opposing creatures die | N Zombies | N Zombies — N subjects | CR 614.5 per event; Kalitas's ruling |
| two shield **counters**, two blockers | 2 counters, both prevented | **1** counter, both prevented | CR 122.1c; the SNC ruling ("that damage is prevented and only one shield counter is removed") |
| two shield **counters**, one blocker with first strike and one without | 2 counters | **2** counters — CR 510.4 makes two combat damage steps, so two batches and two groups | CR 510.4: the key is the *batch*, and this is the contrast that proves it |
| two Furnaces, one source | ×4 | ×4 | CR 614.5's own example |
| two Furnaces, two attackers on one player | ×4 each | ×4 each — chosen once, applied to each member | CR 614.5 per instance |
| Daunting Defender, Pyroclasm on two Clerics | 1 each | 1 each — two subjects | CR 615.10's own example |
| Daunting Defender, two sources on one Cleric | 1 each | 1 each — 615.10's "separately to … events that would happen at the same time" | CR 615.10 |
| a 3-shield, sources of 2 and 4 | 2 then 1, no choice | one `allocate` over the group | **CR 615.7** |

The last row is the one place a rewrite is not member-uniform, and it is the
whole of the new prompt: `ChoiceKind::AllocateNextDamage { source, remaining
}`, asked through `DecisionProvider::allocate` (the trample call), options the
members in batch order — `assign_combat_damage` walks `battlefield_ordered`,
so the order is process-independent — total `min(remaining, Σ amounts)`,
per-bucket max the member's amount. **The allocation is per instance, not per
group**, and the two words mean different things. A *subject group* is the
set of batch members about one object or player — the two attackers' damage
to *you* — and it is the unit the CR 616.1 loop decides for. An *instance* is
one replacement effect — one registry row, one static ability on one object,
one counter-derived effect — and it is the unit that owns a `NextDamage`
count. One row asks once across every member of the batch it applies to,
whatever their subjects. For Mending Hands that is one subject's group; for Divine
Deflection's and Harm's Way's "you and/or permanents you control" it spans
subjects, which their rulings spell out ("you choose which of that damage to
prevent"; "1 damage … to each of two different recipients"). The chooser is
the affected side's, and it is well-defined because every printed
multi-subject amount shield is scoped to one player and that player's
permanents; the engine asserts one chooser rather than guessing between two.
**When it is asked**: the first time the instance is chosen in any group's
loop, over the members it still applies to at their *then-current* amounts —
Divine Deflection's ruling is "you don't decide until the point at which the
damage would be dealt" — and the answer is kept for the groups decided after.
A doubling chosen ahead of the shield in a later group's loop then moves that
member, not the allocation; that is CR 616.1's own per-subject ordering
showing through, and it is recorded as a corner for RD-2's review rather than
smoothed over.
Never with one member: CR 615.7's choice exists only among "two or more", and
with one source every point is prevented unasked. The decisions live in a
batch-scoped struct on `GameState` beside `EntrySelectionScope`, on
`codebase-state.md` item 40's rule, and are empty outside phase 1. §3.2d's
lineage rule is untouched — it is about decomposition, and this is about
simultaneity — and RC's entries are unaffected, since an entry is its own
subject. Phase 1's other standing claim, `codebase-state.md` item 25 (members
are decided one whole CR 616.1f loop at a time, in APNAP order), was checked
for collateral and is untouched: grouping changes the loop's *unit*, not the
order choosers are asked in.

**4. The contained `LoseLife` never meets a prevention effect, and what
enforces that is the vocabulary, not the lineage rule.** Performing
`DealDamage` against a player proposes `LoseLife { cause: Damage { source } }`
from inside the performer, joining the damage's batch as lifelink's gain does
(CR 120.3a/f, 120.4c/d; `GameEvent::LifeChanged` keeps its `source`, so the
log line is unchanged). It re-enters `apply_replacements` with a **fresh**
applied set — containment, §3.2d — which is what lets an RE-era Bloodletter
double it while a shield that already applied to the damage does not apply
again. Fresh lineage says nothing about *which* effects may apply, and does
not need to: a prevention effect is one whose rewrite is `Prevent`,
`PreventUpTo` or `PreventRemaining` on `EventPattern::DealDamage` (CR 615.1a,
derived from the def — see decision 6), `LoseLife` has no pattern arm today,
and when RE adds one `apply_rewrite` refuses `PreventUpTo`/`PreventRemaining` on a
non-damage event with the same "its `EventPattern` and its `Rewrite` describe
different events" arm the entry rewrites use. Life loss from damage is not
separately preventable because nothing can be written that prevents it.

**A rider can refer to two different numbers, and CR 615.12 is what tells
them apart.** Reverse Damage gains "life equal to the damage prevented this
way"; Angel of Suffering mills "twice that many cards", where "that many" is
the damage that *would have been dealt*. Under damage that can't be prevented
both riders run (615.12), and Angel's own ruling says it "still mill[s] twice
that many" while Reverse Damage's amount is 0. So `Rider` carries both the
event's amount and the prevented amount, read by two `AmountExpr` leaves —
`ReplacedAmount` (a rider's "that much"/"that many") and `DamagePrevented` —
and the dovetail test in RD-4 is the one that separates them: a fixture
"damage can't be prevented" row, Lightning Bolt to the face under both cards,
3 damage dealt, 6 cards milled, 0 life gained. Angel of Suffering needs
`AmountExpr::Multiply(Box<AmountExpr>, u64)` for "twice" and a working
`Primitive::Mill`, both RD-1's (below).

`LoseLife` gains `cause: LifeLossCause { Damage { source }, Effect, Cost }` —
six construction sites, and a **fact** on §8a's triage: whether a life loss was
a result of damage is unrecoverable a moment later, Ali from Cairo's family and
CR 120.3b's poison both read it, and the `LifeChanged` line needs the source
today. The performer marks damage on a creature and removes loyalty from a
planeswalker (decision 5) in the same arm, each result independent, since
CR 120.3 says "one or more of the following results" and a creature
planeswalker gets both.

**5. CR 120.3c ships in RD-1, and its consumer is a fixture with its own
name.** The three options the brief offered, priced: registering a printed
planeswalker fails `register-a-card-only-once-the-engine-can-play-it` — loyalty
abilities have no `AbilityType` and no activation path (`phase_sba_cards.rs`
row 704.5i), so a real Jace would be a card with three dead abilities wearing a
real name, which `engineering-practices.md` §3 forbids; leaving the arm out
leaves `perform_action` marking damage on a permanent the targeting code
already validates as "any target", which is the wrong answer waiting for a
card. The fixture is the middle: **Loyalty Probe**, a `Planeswalker` with
printed loyalty 3 and no abilities, on the `graveyard_probe` convention §3.3
already admits, registered in the stress pool and not in `PERFORMANCE_POOL`.
It is a genuine consumer, not a test prop: Lightning Bolt's `SelectionFilter::
Any` validates planeswalkers, so the random agent bolts it, CR 120.3c removes
counters through a contained `RemoveCounters { Loyalty }` proposal, and CR
704.5i — **0** across the 2026-09-01 fuzz re-audit, "a planeswalker can never
die" — becomes reachable from a game. Attacking one stays out (`validation.rs`
refuses planeswalker attack targets; combat's, not this phase's).

**6. CR 615.11 needs no machinery; CR 615.12 is a property of the event, and
its three printed shapes meet at one site.** 615.11 is decision 2's per-target
rows and Samite Censer-Bearer is its consumer. 615.12 prints three shapes and
they are three different things: *"Damage can't be prevented this turn"*
(Skullcrack, Unstable Footing, 11 instants) is a **resolution's** CR 101.2
restriction with a duration — `Primitive::Restrict(ApplyReplacement { kind:
Prevention, .. }, UntilEndOfTurn)`, a registry row; *"Damage can't be
prevented"* on a permanent (Leyline of Punishment, Everlasting Torment) is a
**static ability's** restriction — `Effect::Restriction` on the static,
discovered by RS-1's sweep off the source's *effective* ability list, so
Humility strips it for free and it lasts exactly while the source is on the
battlefield with no `Duration` row at all (`cant-effects-architecture.md`
§3.4 source 1), and it may carry a filter where a printed card does; *"The
damage can't be prevented"* (Combust, Pinpoint Avalanche, 9 cards) is a fact
about **one event**, set by the effect that proposes it. The first two are
two *sources* of one `Restriction` arm, which `is_prohibited` already unions;
the third is not a restriction at all. So `GameAction::DealDamage` gains
`unpreventable: bool`, `Primitive::DealDamage` becomes a struct carrying it
(16 mechanical sites), and it is a property of the event rather than of the
source because a source's other damage is preventable and because the flag has
to survive a redirect — Retarget copies it. The two routes meet where the
prevention arms are *applied*: `prevented = 0` when the event is unpreventable
or `is_prohibited(ApplyReplacement { Prevention, to: subject })`, the rider is
queued regardless (615.12's "any additional effects they have will take
place"), and nothing is consumed (615.12's "existing shields won't be
reduced"). Protection's damage half (CR 702.16e) is a prevention effect a
keyword synthesizes — `backlog.md` §2.6's — and Leyline's ruling that "static
abilities that prevent damage (including protection abilities) don't do so"
falls out the day it exists, because it will be a prevention def like any
other. That consult is at application and **not** at `gather`'s door where
regeneration is withheld: CR 701.19c says a regeneration shield is "not
applied", CR 615.12 says a prevention effect "is still applied", and the two
`ReplacementKindFilter` arms therefore act at two different sites. 615.12a's
"just once" is the applied set — the instance was chosen, it is in the set,
and nothing re-offers it.

`cant-effects-architecture.md` §4.7 expected RD to widen `is_regeneration:
bool` into a `ReplacementKind`. **It does not**: CR 615.1a makes "prevention"
a fact about the effect's text ("effects that use the word 'prevent'"), which
the def carries in its rewrite and pattern, so `ReplacementDef::is_prevention()`
is *derived* and no card can forget to set it — the CDA and CR 614.15 lessons
(§11 item 12). Regeneration keeps its authored bit because nothing about its
def distinguishes it from any other `Prevent`-with-a-rider.

**7. A use is spent by what an application did, not by being chosen.** Not one
of the brief's questions, and it changes one line of §4.1's loop. Today
`consume_use` runs *before* `apply_rewrite`. Three rules need it after:
CR 609.7b ("if for any reason the shield prevents no damage or replaces no
damage, the shield isn't used up"), CR 614.9 (a redirect whose destination is
gone "does nothing", and ATOM-614.9-001's "the shield is NOT used up"), and
CR 615.12 (an unpreventable event reduces no shield). `apply_rewrite` returns
what it did — the event, and for a damage arm the amount it prevented or moved
— and `consume_use` spends that: `Once` iff the rewrite took effect, `Shield`
by the amount. Regeneration is unchanged, because a `Prevent` on a `Destroy`
always takes effect. 609.7b's *property* recheck needs nothing here: a source
that is no longer red fails `pattern_watches`, the instance is never gathered,
and nothing was ever there to spend.

#### Why four, and the count

| PR | Shape | Measured size | Risk |
|---|---|---|---|
| **RD-1 — the damage event's two subjects and its results** | `affected_players`; the CR 120.3 decomposition, `LoseLife.cause`, CR 120.3c; `Rewrite::Amount` with `Multiplier`, `Halve` and `PreventHalf`, and `Rounding`; `Rider` carries `EventSubject` and the event's amount, `AmountExpr::ReplacedAmount` and `Multiply`; `Primitive::Mill` (a stub today) for Angel of Suffering's rider | `set_affects` **1**, `chooser_for` **0** (already right), `Rider`/`resolve_rider` **2**; `perform_action`'s arm **1**, `GameAction::LoseLife` constructions **6**; `Rewrite` exhaustive matches **2** (`from_rewrite`, `apply_rewrite`); `AffectedSet` exhaustive matches **3**, all untouched by construction; `evaluate_amount` **2** leaves; `resolve.rs` **1** stub arm made real. Predicted **~560 engine, ~300 cards, ~750 tests ≈ 1,500–1,700** | medium — the decomposition moves a line of every game's log through a nested proposal, and the A/B's middle arm must show it and nothing else |
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

Harm's Way — "The next 2 damage that a source of your choice would deal to
you and/or permanents you control this turn is dealt to any target instead" —
is one card, never played competitively and rare in Commander, and it is
*not* out of scope the way Panglacial Wurm is: nothing in it is a rules
problem, and its rulings are a complete specification. *"If the chosen source
would deal just 1 damage … Harm's Way's effect will redirect that damage and
still have a 'shield' left for another 1 damage from that source later in the
turn"* is decision 7's rule — a `Retarget` row with `NextDamage(2)`, spent by
the amount moved. *"You choose which 2 damage is redirected … 1 damage … to
each of two different recipients"* is decision 3's per-instance allocation
across members. What it needs that RD-1 through RD-4 do not build is the
**split**: 3 damage to you becomes 1 to you and 2 to the target — one
`DealDamage` becoming two, which §3.2d's `Option` cannot return. The shape
(finding 23) is a phase-1 member insertion: the rewrite returns the residue
for this member and a split-off member that **continues the lineage** (the
moved damage is "the same damage", so an effect that already applied to the
whole does not apply again to the part), decided in the same pass and
performed in phase 2 beside its sibling.

**The gate.** The split path is entered only when a `Retarget` instance with
`NextDamage` is chosen, so it cannot cost the hot path anything by
construction; what it can cost is the per-member path's *shape*, since phase 1
would have to accept insertions while iterating. RD-5 ships if that insertion
lands without touching the code every combat step runs — the middle-arm A/B
flat within spread, and no change to `apply_replacements`' single-member
signature — and is sized ~300–400 with Harm's Way as its consumer. If the
insertion cannot be kept off that path, Harm's Way goes to `backlog.md` with
this shape attached and the measured cost as the reason, which is the only
reason one card should ever be excluded.

##### Decided at RD-4's close (2026-09-09): **the gate fails, and Harm's Way is `backlog.md` §2.25**

The criterion's second half is the one that decides it, and it decides it by
reading the tree rather than by opinion. **A split-off member has no batch
index**, and every signature between the rewrite and the performer is keyed by
one:

| Site | Count | What a split moves |
|---|---|---|
| `apply_rewrite` return sites | **20** | `(Option<GameAction>, Applied)` grows a third thing, in every arm |
| `Member.index` reads/writes | **4** | `usize` → `Option<usize>`, or a parallel tail |
| `finish(members)` | **2** | the closure that maps members back to batch positions |
| `apply_replacements` signature + its caller | **2** | `Vec<(usize, Option<GameAction>)>` cannot name an event that was not proposed |
| `execute_batch_inner`'s `decided[i] = action` | **1** | phase 2's write, which every combat damage step runs |

≈ 30 mechanical sites, and the last two rows are the code every combat step
runs. So the gate's own words — "without touching the code every combat step
runs … no change to `apply_replacements`' signature" — are not met, and the
answer §9 wrote for that case is the one that applies.

**Three things worth keeping, because they change what a later PR starts
from.**

- **The cost is a type change, not work.** Nothing in the table is reached
  unless a `Retarget` instance carries a `NextDamage` count, and no def on
  either pool does. A middle-arm A/B of the type change alone would be flat by
  construction. The gate excluded it on the *shape* half of its criterion, not
  on the speed half, and saying so is the difference between a measurement and
  a verdict.
- **RD-4 already paid the part that was hardest to see coming.** The split's
  awkward case was a member whose subject is not the group's; RD-4 made the
  chooser, the prompt and `RemoveCountersFromAffected` read the event rather
  than the group key, because CR 614.9's whole-event redirect needed it first
  (§11 item 35). A later RD-5 starts with that done.
- **What is genuinely new work, and was not in the ~300–400 sizing:**
  `next_damage_shares` would have to allocate a CR 615.7 count across members
  *and* decide how much of each member's amount moves — Harm's Way's own ruling
  ("1 damage … to each of two different recipients") makes those the same
  choice. That is new logic in the one function §11 item 24 calls the
  non-uniform rewrite, not a mechanical edit.

**So `backlog.md` §2.25 owns Harm's Way**, with the shape, this table and the
reason. Divine Deflection is *not* affected and never was: a `Filter` +
`PlayerSet` row with `NextDamage(X)` is already a pooled amount, and it waits
only on `AmountExpr::Variable` and `codebase-state.md` item 90.

#### Measured — what to expect, and why the direction is known

Three arms per PR through `plans/fuzz_ab.py` against a same-day `main`
worktree (`../mtgsim_v2_main`, rebuilt first), both pools, and the middle arm —
the engine with `registry.rs` and `PERFORMANCE_POOL` unchanged — is the only
one that attributes anything. Damage is on every combat step, so unlike CM-4
the counters **will** move, and each PR predicts the direction before running:

- **RD-1's middle arm:** `replacement gathers` rises by exactly the number of
  `DamageDealt` events whose target is a player (one contained `LoseLife`
  proposal each) and by nothing else; `Layer walks` flat (the sweep's fast
  path returns before any walk when no source, row or counter exists);
  40-game dumps byte-identical under the two id masks, because `LifeChanged`
  keeps its source. The shipped arm adds walks only while Furnace is on the
  battlefield — one per damage event, since the per-permanent gate walks only
  `replacement_ability_sources`.
- **RD-2:** gathers flat on the middle arm (grouping changes decisions, not
  proposals); the shipped arm's `allocate` count is the reachability row.
- **RD-3 and RD-4:** flat on the middle arm; a shipped-arm move is the card.

A middle-arm delta beyond the ±4–6% spread that the prediction does not name
wants a **fourth binary** with the suspect reverted, as CM-4 needed. §3's table
is re-recorded once per PR that moves the pool, at 50 games, after the A/B.

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

Read against CR 104, 106.6a, 119.5, 119.10, 121.2–121.6, 122.1, 122.6, 500.7,
500.11, 614.1b, 614.10–614.11, 614.16, 616.1g, 616.2, 701.9, 701.22, 704.5a–c,
704.7 and 800.4 (`MTG-Rules/versions/tmnt.txt`), §3.2d, §4.1's six rules, §8a,
§8b and §11 items 6, 18, 21, 26. Every consumer named below had its oracle text
and rulings fetched from Scryfall on 2026-09-11 (`engineering-practices.md`
§3.4); one ruling contradicted this document, and decision 1 says which.

**0. Seven kinds, one algebra, no new `Rewrite` arm.** Every kind lands on an
arm that already has a customer: draw on `Instead` and `Prevent`; life, tokens,
counters and mana on `Amount`; losing on `Prevent` with a rider; winning on
`Instead`; skips on `Prevent` (CR 614.1b's "replaced with nothing" *is*
614.6). What grows is the two open payloads §3.2b already names as open —
`AmountRewrite` gains one variant (decision 2) and `GameActionTemplate` gains
three arms (`DrawCards`, `GainLife`/`LoseLife`, `PlayerWins`), each with a
printed customer in the PR that lands it. §3.2c's census said 0 of 574
clauses needed a sixth arm; RE is the half of that census that was not yet
built, and it holds.

**1. Draw is two events, and the instruction is the outer one.** CR 121.2a:
"an instruction to draw multiple cards can be modified by replacement effects
that refer to the number of cards drawn. This modification occurs before
considering any of the individual card draws." So `GameAction::DrawCards {
player, n, cause }` is the instruction and `DrawCard { player, cause }` the
draw; the outer's performer proposes `n` inners one at a time (CR 121.2), and
**those inners inherit the outer's applied set** — §3.2d's decomposition rule,
whose parameter has waited since RB for this producer (§11 item 18,
`codebase-state.md` item 29). `test_two_teferis_draw_four_not_infinity` is the
regression and hangs rather than fails, so it is written first with a bounded
guard. **Every draw instruction proposes the outer, including "draw a card"**
— one path, and Alms Collector's ruling is the reason it is honest: "count how
many times the word 'draw' is used". Its "two or more" is
`EventPattern::DrawCards { at_least: Option<u64> }`; `EventPattern::DrawCard {
cause: Option<DrawCause> }` is everything else. CR 616.1g's "the second effect
can't be chosen until after the first" is the nesting: the outer is decided and
performed before an inner exists.

`DrawCause { TurnBased, Effect }` is a fact on the event, on RD-1's
`LifeLossCause` reasoning: ten printed cards say "except the first one you draw
in each of your draw steps" and the answer is unrecoverable a moment later. The
draw step's instruction is `TurnBased`; **its first inner is `TurnBased` and
every later one is `Effect`**, at every level of decomposition — so Thought
Reflection doubling the draw-step draw yields a first card Teferi's Ageless
Insight excepts and a second it doubles, which is what "the first one" means.

**The ruling that corrects this document.** §3.2d encodes Notion Thief as
`Prevent` with a rider ("that player skips that draw *and* you draw a card").
Its 2018-03-16 ruling describes two Thieves in a two-player game: the drawing
player applies one, then "the player whose Notion Thief's effect was chosen
repeats this process among the remaining", each "applied to the card draw only
once", and "it really will be that player who draws a card". That only holds if
the Thief's draw is **the same event with a new subject** — CR 614.5's "modified
events that may replace that event" — because a rider's draw is a fresh
proposal with a fresh applied set, and two Thieves would trade the draw forever.
So Notion Thief is `Instead(DrawCards { n: 1, player: Some(You) })`, a subject
change that keeps the lineage, and the `DrawCards` template carries `player:
Option<PlayerRef>` with `None` meaning the affected player — one arm, two
customers (Thought Reflection's `{ n: 2, player: None }`). Alms Collector stays
`Prevent` with a rider: "you and that player each draw a card" is heterogeneous
(§3.2d's own category), and its ruling that Thought Reflection may then double
the resulting draws while Alms "does not apply again" holds either way, since
neither resulting draw is "two or more". §3.2d is corrected in place, dated.

CR 614.11 / 121.6a — a draw replacement applies with an empty library — is
already true by construction: `draw_card` flags the empty library *after* the
pipeline, which is why its comment says so. CR 614.11a / 121.6b — the
replacement completes before the sequence resumes — is the outer performer
proposing one inner at a time and each inner's riders running before the next
(§4.1a). CR 614.11b / 121.6c (an additional action on a drawn card is not
performed when the draw is replaced) has no producer: nothing in `Primitive`
does something to *the card it drew*, so `ATOM-614.11b-001` stays uncovered
with that reason, on the corpus's Phase 6.

**2. Life gains its arms, `LoseLife`'s `cause` gets its first reader, and the
one new `AmountRewrite` reads player state.** `EventPattern::GainLife` and
`EventPattern::LoseLife { cause: Option<LifeLossCause> }`. Ali from Cairo —
"damage that would reduce your life total to less than 1 reduces it to 1
instead" — watches the contained `LoseLife { cause: Damage }` (its own ruling:
"this effect does not prevent damage, it prevents the damage from turning into
loss of life"; RD decision 4 built the `cause` for it) and needs the clamp
§3.2c budgeted: `AmountRewrite::LifeFloor(i64)`, the loss capped so the total
does not drop below the floor, applied against the affected player's life *now*
— the second arm of `apply_rewrite` that reads `GameState` (item 53), and a
read, not a write. The name is open; the CR has no noun for it and §3.2c said
"clamp". `is_prevention` is untouched: its first test is "the pattern is
damage", and this one is not.

Two kind-changing substitutions land here, and they are the mechanism §10's
Eligeth test wanted: `GameActionTemplate::GainLife { amount }` and `LoseLife {
amount }` with `amount: TemplateAmount::{Fixed(u64), ReplacedAmount}`. Words of
Worship turns a draw into "gain 5 life" (`Fixed`); Tainted Remedy turns an
opponent's gain into "loses that much life" (`ReplacedAmount`). Each template
arm has its two customers before it is written.

"Players can't gain life" is a **"can't"**, and Skullcrack and Leyline of
Punishment have waited on it since RD-4 (§11 item 26). `Restriction::Event {
pattern, affected, by }` has no player set, so RE-3 gives it
`affected_players: PlayerSet` — RD-1's field, RD-4's `to_players` precedent,
unioned the way `set_affects` unions — and `is_prohibited` refuses the
`GainLife` proposal ahead of the pipeline (CR 101.2). Leyline of Punishment's
own ruling then falls out of the order of the two checks: "effects that replace
an event with gaining life (like Words of Worship's) will end up replacing the
event with nothing" — the substituted `GainLife` is proposed, refused, and
nothing happens.

**3. `CreateTokens` is the outer event, each entry is contained, and a
substituted entry is a creation somewhere else.** `GameAction::CreateTokens {
defs: Vec<TokenDef>, controller }` — `Vec`, as §3.1 decided for Academy
Manufactor's "one of each" and Anointed Procession's ruling ("twice as many of
each kind"). Its performer creates the objects, then proposes **every entry as
one batch** — `codebase-state.md` item 46's first plural entry, so two tokens
entering together are decided against the pre-batch board (RC-4b's frame) and
"can't apply to … any other permanent entering the battlefield at the same
time as it" (Winding Constrictor's and Pir's rulings) is true of them the day
they exist. CR 616.1g makes the token half of Doubling Season a fresh choice per
token (§3.2d's contrast case) — the entries are *contained*, not decomposed,
and get fresh applied sets.

**The residual, item 52.** A substituted token entry — Hallowed Moonlight's
"exile it instead", whose ruling is "it's put into exile instead and then
ceases to exist" — is performed today as `ZoneChange { from: Battlefield }` for
an object that was never there, which is the line Dour Port-Mage and Aang read.
The honest event is an *appearance*: the token is created in exile, from
nowhere. `GameAction::ZoneChange.from` stays `Zone` — twenty constructions in
`src/`, five in tests, three readers of `from` in the move path, and an
`Option` there is a catchall-shaped `None` on every card move for one token's
sake. Instead the `Instead(ZoneChangeTo)` arm on an entry with `from: None`
returns a `from`-less variant — working name `GameAction::CreateTokenIn {
object, zone }` — whose performer puts the object into that zone's collection
and emits a new `GameEvent::TokenCreated { object_id, zone }`; CR 704.5d takes
it from there. The name is open. It gets no `EventPattern` arm (nothing prints
"if a token would be created in exile") and joins `Attach` as the second
deliberate exemption from one-arm-per-variant, said in its doc. A *dropped*
entry still un-creates the object (CR 111.5), unchanged.

**CR 613.7m's decision point is not asked, and the reason is RC-4's.** Tokens
created by one effect are identical and enter under one controller, so "in the
order of that player's choice" is a choice with one outcome, and RC-4's rule
("never prompt for a choice with one outcome") applies. The first creation with
distinguishable members — Bestial Menace, Academy Manufactor — is the prompt's
customer, and neither is in reach; "Before card breadth" item 4 records the
prompt as owed at that card, and the batch order is the creation's order until
then.

**4. The entry is a door for the counter pattern too, who puts the counters
on is a fact on the event, and a player can be the subject.** CR 122.6:
"putting counters on that object … refers … also to an object that's given
counters as it enters the battlefield" — Doubling Season's ruling is
"planeswalkers will enter with double the normal number of loyalty counters",
and Hardened Scales', Primal Vigor's and Corpsejack Menace's each say the same
of +1/+1. Entry counters are `EnterMods.counters` and never an `AddCounters`
proposal (RC-2's decision, kept: they are part of the entry event, CR 614.1c),
so `EventPattern::CounterChange` gains a second door exactly as
`EventPattern::ZoneChange` did in RC-4b: it watches an `EnterBattlefield` whose
`mods.counters` carries a matching kind, and `Amount` applied to an entry
rewrites that kind's count in the mods. The Master Biomancer + Doubling Season
board then needs nothing new: Biomancer's `EnterWith` makes the doubler
applicable (CR 616.2), the affected player orders them (616.1e), and 614.5
stops the doubler re-applying to counters added after it — which is the printed
interaction.

`AddCounters` gains `by: PlayerId` and `EnterMods.counters` its
`Option<PlayerId>` — `codebase-state.md` item 43, sized there at ~60–80 lines
and told to land "before RE's first doubler". Vorinclex, Monstrous Raider is
the reader: "if *you* would put one or more counters … twice that many; if an
opponent would … half that many, rounded down", and its ruling says it "cares
deeply about who is putting the counters on". `EventPattern::CounterChange {
counter, adding, by: Option<PlayerRef> }` is RD-3's `SourcePattern` on the
player axis. CR 122.6a's default — the object's controller puts entry counters
on — is what `default_enter_mods` and every `EnterWith` write unless the effect
names a player, and today none does.

**Counters on players join here, because the type is open on the table.**
CR 122.1 lets "a player" have counters, and `backlog.md` §2.16 already has the
design: `PlayerState.poison_counters: u32` becomes a kind → count map sharing
`CounterType`, since CR 701.34a's proliferate sweeps permanents and players in
one pass. `AddCounters`' subject becomes `CounterSubject { Object(ObjectId),
Player(PlayerId) }` — `DamageTarget`'s shape, `subject_of` answers the player,
CR 616.1's chooser is that player — and widening it later would be a site
sweep of exactly the kind RD-4 paid for `DealDamage` (44 sites) when RE-5 can
do it at three. The producer is a printed card with nothing else in it, Live
Fast ("you draw two cards, lose 2 life, and get {E}{E}"), whose rulings say
energy counters are "a kind of counter that a player may have" and that
"effects that interact with counters a player gets … can interact with" them;
the watchers are Vorinclex's "or player" and Winding Constrictor's "if you
would get one or more counters". CR 704.5c reads the map for poison, and RD-1's
seam for infect's poison half (`backlog.md` §2.6, 120.3b) and Commander's
experience counters are the next producers. Costs that pay energy wait for
their first card.

Two rulings fall out of RC-3 and RC-4b rather than being built: "can't apply to
itself as it's entering" (RC-3's membership rule — an entering permanent's own
filter-scoped replacement does not reach itself) and "or to any other permanent
entering the battlefield at the same time" (decision 3's plural batch).

**5. Losing and winning are proposals; the SBA sweep builds them as batch
members; CR 704.7's collapse gains a per-player leg.** `GameAction::PlayerLoses
{ player, reason: LossReason }` and `PlayerWins { player }`. The four loops in
`check_state_based_actions` that write `player_lost` (704.5a/b/c and 903.10a)
become members of the CR 704.3 batch, evaluated against the one board the rest
of the sweep already reads; a player who would lose for two reasons in one check
is **one member** (CR 704.7 — Lich's Mirror's ruling: "a single Lich's Mirror
will replace all of them"), carrying the first reason in CR order, which is
what RA-3's per-object dedupe does for zone changes. `EventPattern::PlayerLoses`
carries no `reason` field: every printed replacement applies to every reason
(Exquisite Archangel's ruling, "any time you would lose the game"), so a field
would be one nothing reads. The performer marks `player_lost`, emits
`PlayerLost`, and clears `has_drawn_from_empty_library` — today it is never
cleared, which CR 704.5b's "since the last time state-based actions were
checked" forbids and Exquisite Archangel's ruling ("you won't lose again until
you try to draw again") makes observable. `PlayerWins`' performer records the
result on `GameState` — CR 104.1, "immediately" — so `GameResult` moves there
from `Game` and `check_game_over` reads it.

**N-player, where the paragraph would have hidden a two-player assumption.**
In a game of three or more, a lost player does not leave the rotation today:
`advance_turn` takes `(active_player + 1) % num_players` and the priority loop
does the same. CR 800.4k ("if a player who has left the game would begin a
turn, that turn doesn't begin") and 800.4j (priority passes over them) are two
sites and ~40 lines, and RE-6 carries them with the `--players 4` fuzz mode
"Before Commander" item 4 sized at ~50 lines, because a `PlayerLoses` performer
that leaves the player in the turn order is the two-player shape wearing an
N-player event. **CR 800.4a–e — their objects leaving the game — is RE-7, the
PR after, and not B3's any more.** The first cut left it with `codebase-state.md`
item 108 and B3; the review's objection stands: the day RE-6 lands, item 108
flips from "unreachable" to "reachable and wrong" in the four-player run, and
a dead player's permanents on the battlefield distort every four-player number
measured after it. The ledger's own rule ("the four wrong answers are the
phase-independent output") says a reachable wrong answer is fixed first, so
RE-7 follows RE-6 immediately; CR 802's defending player stays B3's.

"Can't lose" and "can't win" are `Restriction::Event` rows over the two new
patterns through decision 2's `affected_players`, refused ahead of the pipeline;
Platinum Angel's ruling — "no game effect can cause you to lose … you keep
playing" — is that refusal, and Exquisite Archangel's "if an effect says that
you can't lose the game, Exquisite Archangel's effect doesn't apply" is the
CR 101.2 order the two checks already have. Concession (104.3a) is a *leave*
that then loses, not a replaceable loss ("does nothing if you concede", every
ruling above), and it arrives with the harness that offers it; CR 104.3f
(win and lose at once → lose) has no atom and no consumer and is recorded, not
built.

**6. Skips are the pipeline's; the proposal is built by a turn queue that
`advance_turn` drains; `pending_skips` is struck.** §11 item 6 said the first
half on 2026-08-30 and §9's RE paragraph, older, said the other thing. A
per-player counter consulted at step begin is a second mechanism for a CR 614
effect — it cannot express a static ("Players skip their upkeep steps", Eon
Hub) without a third shape, it cannot be stripped by Layer 6 or CR 305.7, and
it cannot be ordered against another effect on the same event by CR 616.1.
Three variants, because the three units name three different things and have
three different performers: `BeginTurn { player, turn }`, `BeginPhase { phase,
player }`, `BeginStep { step, player }`, each with its pattern arm (`BeginStep
{ step: Option<StepType> }`, `BeginPhase { phase: Option<PhaseType> }`,
`BeginTurn`). The subject is the player whose turn it is, so CR 616.1's chooser
is that player and Eon Hub on a four-player table asks nobody. The performer is
small: it writes `phase`/`step`/`active_player` and emits the
`TurnBegin`/`PhaseBegin`/`StepBegin` that exist today and nothing emits, which
is what item 6's "at the beginning of" triggers will read — **and that is why
this PR goes first**: §8b counts 2,656 of them, more than any other kind, and
item 6 cannot start until the three events exist through the chokepoint. Turn-
based actions stay in `on_step_begin` and `process_turn_based_actions`, run only
for a unit that began.

**The queue is the shape, and it is why `backlog.md` §2.17 graduates here.**
`advance_turn` is rewritten once, as a drainer: the next unit is the head of a
queue — an extra turn, phase or step if one is pending, else the natural next —
and it is *proposed* before it starts. A dropped proposal is proceeded past as
though the unit did not exist (CR 500.11, 614.10): a skipped phase proposes none
of its steps, a skipped turn advances no turn number and expires nothing —
"until your next turn" waits for the first turn that is not skipped, 614.10a's
own sentence — and the first turn's untap step is proposed like any other.
CR 500.7's extra turns are pushed "most recently created … first", so the
queue's turn half is a stack fed by a new `Primitive::ExtraTurn`, with Time
Walk as its consumer; the Meditate + Time Walk board — a skip consuming an
extra turn, 614.10a's "the first occurrence that isn't skipped" — is the test
that shows the two belong in one PR rather than in two rewrites of one
function. Extra phases and steps (500.8, 500.10, Relentless Assault's class)
are the same queue one level down and wait for their first card. `backlog.md`
§2.12's step- and phase-scoped durations hang off the same emitters and wait for
an "until end of combat" consumer.

Three CR sentences then cost nothing: "once a step … has started, it can no
longer be skipped" is the proposal site (Moment of Silence's ruling, "must be
used before the combat phase starts or it has no effect", is a `Uses::Once` row
that meets no proposal until the next combat); 614.10a's "two effects … skip the
next two" is two `Uses::Once` rows, the first spent on the first proposal and
the second surviving to the next; and 800.4k is a rule at the same site, ahead
of the pipeline, once RE-6 gives it `player_lost`. 614.10b ("skip …, then take
another action") has **zero printed cards** by the census and no consumer; its
atom stays uncovered with that reason.

**7. Mana is one proposal from two silent writers, and "tapped for mana" is a
fact about the activation.** `GameAction::ProduceMana { player, source, mana:
Vec<(ManaType, u64)>, special: Vec<ManaAtom>, tapped: bool }`, performed by
the one arm that writes the pool and emits `ManaAdded` — an event that has
existed since the log was written and has never once been emitted, which is how
the RA census missed the two writers (`resolve_mana_effect` for a mana ability,
`Primitive::ProduceMana` for a spell). `EventPattern::ProduceMana { tapped:
Option<bool> }`; `Amount(Multiplier)` scales every type in `mana`; CR 106.6a's
"any restrictions or additional effects … apply to all mana produced" is
`special` riding through unchanged. `tapped` is CR 106's "tapping a permanent
for mana" — Mana Reflection's ruling: "only if you're activating a mana ability
of that permanent that includes the {T} symbol in its cost" — and CM-3's
lock-in already knows what was paid. Mana abilities activated inside CM-4's
payment window propose through the same site with the same `ctx`, and
`resolve_mana_effect`'s `Fixed`-only limitation is untouched. Triggered mana
abilities ("whenever you tap a land for mana, add an additional …", 8 cards)
are CR 605.1b's and item 6's, and Mana Reflection's ruling says so in as many
words. This is the hottest path RE touches — every land tap — so RE-9 goes
last, where its A/B decides nothing else; `backlog.md` §2.19's any-color mana
rewrites the same function and is ordered after it.

**8. A CR 701 producer that a replacement customer is waiting on lands in RE,
on RD-1's precedent.** `Primitive::Mill` was a stub until Angel of Suffering's
rider needed it, and RD-1 built it rather than wait for Phase 8's CR 701 sweep.
Discard and scry are the same case with the same argument, and the first cut
of this section got it wrong by reading §8a's sentence without the precedent.
`Primitive::Discard(n)` moves cards through `change_zone(.., Discarded)` with
CR 701.9b's chooser — the discarding player by default, "at random" as the one
other shape a registered card prints (Hymn to Tourach; "another player
chooses" waits for its card) — asked through the same `ChoiceKind` the cleanup
discard uses. It proposes no outer event: nothing prints "would discard", and
the seventeen cards watch the zone change. What they need beside the producer
is `caused_by: Option<PlayerRef>` on `EventPattern::ZoneChange`, read off the
batch's resolution stamp (the cleanup discard has none, which is right — it is
not "a spell or ability"), and a to-battlefield leg on `Instead(ZoneChangeTo)`:
a substitute whose destination is the battlefield returns an
`EnterBattlefield` proposal, since RC-4b's performer refuses a `ZoneChange`
onto it, and the loop's next iteration builds the entry's frame from the
rewritten event, which it already does. `GameAction::Scry { player, n }` is an
event because Eligeth, Crossroads Augur replaces it ("if you would scry a
number of cards, draw that many cards instead" — `Instead(DrawCards { n:
ReplacedAmount })`, decision 2's template amount on decision 1's template);
its performer asks one `ChoiceKind::Scry` and reorders the library in the arm,
proposing no zone change (CR 701.22 moves nothing between zones).

#### Why ten, and the count

| PR | Shape | Measured size | Risk |
|---|---|---|---|
| **RE-1 — skips, and the turn queue** — ✅ landed | `BeginTurn`/`BeginPhase`/`BeginStep`, three arms, three small performers emitting the three begin events; `advance_turn` as a queue drainer with proceed-past; `Primitive::ExtraTurn` and CR 500.7's order | Predicted **~1,700–1,900**; shipped **+1,770 / −155** before the docs. The three exhaustive matches and `pattern_watches` were exactly as counted; `Game::setup`'s "first turn" was a *new* site rather than an existing one, and `turn_rotation` was a field nobody predicted (§11 items 47, 48) | medium — and the risk landed where it was named: the fixtures, six of which counted turn positions by hand |
| **RE-2 — draw** | `DrawCards` outer + `DrawCard.cause`; two pattern arms; `GameActionTemplate::DrawCards { n, player }`; the outer performer's decomposition with the inherited applied set (item 29's producer) | exhaustive matches **3** + `pattern_watches` **2**; `DrawCard` producers **2** rewritten to the outer, performer **1**, test constructions **1**; `execute_batch_inner`'s `inherited` **1** call site; `Primitive::DrawCards` **1**. Predicted **~550 engine, ~350 cards, ~800 tests ≈ 1,700–1,900** | **highest** — the first decomposition, whose defect is a hang, and the `cause` stamping rule across nested outers |
| **RE-3 — life** | `EventPattern::GainLife`, `LoseLife { cause }`; `AmountRewrite::LifeFloor`; `GameActionTemplate::{GainLife, LoseLife} { amount: TemplateAmount }`; `Restriction::Event.affected_players` | `pattern_watches` **2**; `apply_rewrite`'s `Amount` arm **1** and `Instead` arm **2**; `Restriction::Event` constructions **~6** + `is_prohibited`'s union **1**; `GainLife` producers **2**, `LoseLife` **3**, untouched. Predicted **~350 engine, ~450 cards, ~600 tests ≈ 1,400–1,600** | low-medium — patterns over events that already flow; the clamp is the one new arithmetic |
| **RE-4 — tokens** | `CreateTokens` + its pattern arm; the plural entry batch (item 46); `CreateTokenIn` and `TokenCreated` (item 52); `Amount` over a `Vec` | exhaustive matches **3** ×2 variants; `Primitive::CreateToken` **1** producer + **1** performer restructured; `apply_rewrite`'s `Instead(ZoneChangeTo)` entry arm **1**. Predicted **~600 engine, ~350 cards, ~700 tests ≈ 1,650–1,850** | medium — the first performer that proposes a batch from inside a performer, and the log line item 52 is about is the test |
| **RE-5 — counters, on permanents and players** | `CounterChange`'s entry door and `by`; `AddCounters.by` and its `CounterSubject`; item 43's `EnterMods` player half; `Amount` on an entry's mods; `PlayerState`'s counter map (§2.16) | `pattern_watches` **1** more arm; `AddCounters` constructions **2** + performer **1**; `EnterMods`/`EnterModsTemplate` merge **2**; `apply_rewrite`'s `Amount` arm **1**; `poison_counters` readers **4** (one production, `sba.rs:142`) → the map. Predicted **~650 engine, ~550 cards, ~800 tests ≈ 1,900–2,100** | medium-high — top of the band; an `Amount` arm that edits `EnterMods` is new, and the subject enum touches every counter site |
| **RE-6 — the game's end** | `PlayerLoses`, `PlayerWins`, their arms; four SBA loops → batch members; 704.7's player leg; the flag reset; `GameResult` onto `GameState`; `Primitive::{LoseGame, WinGame, SetLifeTotal}` (CR 119.5); 800.4j/k at two rotation sites; `--players 4` | exhaustive matches **3** ×2; `sba.rs` loops **4**; the dedupe **1**; `check_game_over` **1** + `Game.result` readers **~4**; `advance_turn` **1**, priority loop **1**; `fuzz_games` **~50 lines**. Predicted **~550 engine, ~400 cards, ~80 harness, ~800 tests ≈ 1,800–2,100** | **high** — top of the band; the sweep's shape changes, and the N-player half is measured for the first time |
| **RE-7 — leaving the game (CR 800.4a–e, 800.4m)** | inside `PlayerLoses`' performer, as 800.4a says ("as soon as the player leaves"): owned objects leave the game with one `LeftTheGame` event each, control-changing rows in the departed player's favour end, their stack objects not represented by cards cease, objects they still control are exiled through `change_zone` with a new cause; 800.4b/d refusals at `propose_entry` and the token performer; 800.4e at combat assignment; 800.4m on the three duration registries | the five zone collections + the stack **6** sweeps; `ContinuousEffect` rows keyed by controller **1**; `propose_entry` **1**, `CreateTokens` **1**, `assign_combat_damage` **1**; `remove_expired_at_turn_start` **3**. Predicted **~400 engine, ~450 tests ≈ 800–950**, no cards: Act of Treason is in the pool and is the consumer both ways round | medium — the first sweep that removes objects from every zone at once, and the four-player fuzz is the only board that runs it unforced |
| **RE-8 — the producers (CR 701.9, 701.22)** | `Primitive::Discard` with 701.9b's chooser; `caused_by` on the zone-change pattern; the to-battlefield leg; `GameAction::Scry`, its arm, `Primitive::Scry` and `ChoiceKind::Scry` | `resolve.rs` stubs **2** made real; `pattern_watches` **1** field + **1** arm; `apply_rewrite`'s `Instead(ZoneChangeTo)` **1** leg; exhaustive matches **3** for `Scry`; `DecisionProvider` impls **3** for the scry choice. Predicted **~450 engine, ~400 cards, ~500 tests ≈ 1,300–1,500** | low-medium — two producers of the plainest kind; the leg's frame rebuild is already how the loop runs |
| **RE-10 — extra phases, and the turn plan** | `TurnPlan` + `PlannedPhase`; `drain`'s cursor becomes an index; `next_phase`'s chain deleted; `Primitive::ExtraPhases` splicing at the cursor; `Primitive::Untap` gains the `FilteredPermanents` arm | `next_turn_unit` **1** and `drain` **1** (the cursor), `next_phase` **1** deleted + **~8** readers; `GameState` **1** field, seeded **1** and rebuilt **1**; `Primitive` exhaustive matches **1**; `Primitive::Untap`'s recipient **1**. Predicted **~700 engine and cards, ~400 tests ≈ 1,100–1,300** | low-medium — the second and last rewrite of `advance_turn`, and the first turn structure that is data rather than a `match`; it changes no turn's *shape*, so nothing else's fixtures move |
| **RE-9 — mana** | `ProduceMana`, its arm, one performer replacing two writers, `ManaAdded` emitted, `tapped` from the activation | exhaustive matches **3**; writers **2** → **1**; `resolve_mana_effect` **1**, `Primitive::ProduceMana` **1**; `pattern_watches` **1**. Predicted **~300 engine, ~200 cards, ~450 tests ≈ 950–1,150** | low on shape, **the one whose A/B could say no** — a proposal on every land tap |

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

→ The section as sized, what the building changed and the measurement:
`plans/archive/replacement-architecture-landed.md`, "RE-2" (evicted
2026-09-11).

#### RE-3 — life (CR 119.10, 119.7's "can't gain", the CR 120.3a loss as a replaceable event)

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

#### RE-4 — tokens (CR 614.16's token half, 111.5, 616.1g; items 46 and 52)

**Builds:** decision 3 — `CreateTokens { defs, controller }`,
`EventPattern::CreateTokens`, the performer that creates the objects and
proposes their entries as one batch, `CreateTokenIn { object, zone }` with
`TokenCreated`, and `Amount` over a `Vec` (a multiplier repeats each def).
`Primitive::CreateToken(def, amount)` becomes one proposal. **Consumers:**

- **Parallel Lives** — "If an effect would create one or more tokens under
  your control, it creates twice that many of those tokens instead."
  `CreateTokens`, `Fixed(vec![])` + `You` (the subject is the controller the
  tokens are created under), `Amount(Multiplier(2))`, `Uses::Static`.
  Rulings, two, both tests: *two Parallel Lives create four times* →
  `Amount` composing (the Furnace pair, third kind); *everything specified
  by the creating effect is true of the extra tokens* → structurally true of a
  repeated def, asserted on a token's characteristics. `ATOM-614.16-001` —
  "token replacement applies to tokens from other replacements" — is Kalitas's
  rider making a Zombie under Parallel Lives: two Zombies, and the atom's
  board is in the pool already.
- **Raise the Alarm** and **Hordeling Outburst** — "Create two 1/1 white
  Soldier creature tokens." / "Create three 1/1 red Goblin creature tokens."
  The first plural creations in the crate, and so **the first plural entry
  batch** (item 46): the test is RC-5's
  `test_two_biomancers_entering_together_give_each_other_nothing` reached from
  a printed card — two Soldiers under Master Biomancer each get Biomancer's
  counters and neither gets the other's — plus Root Maze asking once per token
  (CR 616.1g's per-entry fresh choice, §3.2d's contrast case).
- **Hallowed Moonlight** — "Until end of turn, if a creature would enter and
  it wasn't cast, exile it instead. Draw a card." A `Primitive::CreateReplacement`
  row, `EnterBattlefield { cast: Some(false) }`, `Filter { creatures }` +
  `Everyone`, `Instead(ZoneChangeTo { Exile })`, `UntilEndOfTurn` — every
  piece exists since RC-4b, and it is the card that reaches item 52. Rulings,
  two, both tests: *a creature token is put into exile instead and then
  ceases to exist* → the log holds `CreateTokens`, `TokenCreated { Exile }`
  and CR 704.5d's `TokenCeasedToExist`, and **no `ZoneChange { from:
  Battlefield }`** — the line Dour Port-Mage would have read, asserted absent;
  *a cast creature is unaffected, from any zone* → Grizzly Bears resolves
  normally under it.

**`PERFORMANCE_POOL` +2, Parallel Lives and Raise the Alarm**, predicted: the
first `CreateTokens` static source, and the producer that makes a plural entry
batch happen in a measured game — the engine path item 46 wanted measured, and
the first board on which CR 616.1 is asked once per token. Kalitas's rider
already makes single tokens in the stress pool, so the middle arm moves by one
gather per Zombie and nothing else.

**Atoms:** `ATOM-614.16-001`; `ATOM-111.5-002` (Phase 8 — a token not created
when a permanent with its characteristics can't enter: Worms of the Earth's
`ZoneChange` prohibition over a land token, covered where it is, not
re-filed); `ATOM-613.7m-001` stays on its `L03` ticket with decision 3's
"not asked" reason written beside it, since a homogeneous batch has no
observable order.

#### RE-5 — counters, on permanents and players (CR 614.16's counter half, 122.1, 122.6, 122.6a; item 43, `backlog.md` §2.16)

**Builds:** decision 4 — the second door on `CounterChange`, `Amount` on an
entry's mods, `AddCounters.by` and its `CounterSubject`, item 43's `EnterMods`
player half with the merge keyed on `(kind, player)`, `CounterChange.by`, and
`PlayerState`'s kind → count map with CR 704.5c reading it. **Consumers:**

- **Doubling Season** — both abilities, registered whole now that RE-4 built
  the first. Tokens: Parallel Lives' def. Counters: `CounterChange { counter:
  None, adding: true, by: None }`, `Filter { ByController(You) }`,
  `Amount(Multiplier(2))`. Rulings, five, four are tests: *planeswalkers
  enter with double loyalty* → Loyalty Probe enters with 6, through the entry
  door; *permanents that enter with counters* → Chainbreaker's rust counters,
  and Master Biomancer's grant on a creature entering (the CR 616.2 ordering
  board, decision 4); *loyalty paid as a cost is not doubled* → `Cost::
  AddCounters` does not propose, and no `CounterChange` sees it — asserted
  against the RD-1 fixture; *two Seasons quadruple* → §10's
  `test_two_doubling_seasons_quadruple`, on both halves.
- **Hardened Scales** — "If one or more +1/+1 counters would be put on a
  creature you control, that many plus one +1/+1 counters are put on it
  instead." `CounterChange { counter: Some(PlusOnePlusOne), .. }`, `Filter {
  Creature ∧ ByController(You) }`, `Amount(Plus(1))` — RD-3's `Plus`, second
  kind. Rulings, three, all tests: *enters with that many plus one* → the
  entry door on a +1/+1 entry (Master Biomancer's grant); *you choose the
  order no matter who controls the sources* → Scales beside an opponent's
  Doubling Season, the affected permanent's controller asked; *each extra
  Scales adds one* → two rows, each once.
- **Vorinclex, Monstrous Raider** — "Trample, haste. If you would put one or
  more counters on a permanent or player, put twice that many … instead. If
  an opponent would put one or more counters …, they put half that many …
  rounded down." Two rows: `CounterChange { by: Some(You) }` +
  `Multiplier(2)` and `{ by: Some(Opponent) }` + `Halve(Down)`, both over
  `Filter { All }` + `Everyone` — decision 4's putter predicate on both
  subjects, and RD-1's rounding on a new kind. Rulings, three: *cares who is
  putting* → an opponent's Battlegrowth on your creature halves to 0 (a 0
  count meets the `RemoveCounters`-style no-op guard, not `never_happens` —
  CR 614.7a is written about damage and life gain, and the test says which
  guard fired); *122.6a's default* → the entry door with `by` unset reads the
  controller; *ordering* → as Scales. Its "or player" half is **built**: your
  Live Fast under your Vorinclex gets four energy.
- **Winding Constrictor** — "If one or more counters would be put on an
  artifact or creature you control, that many plus one of each of those
  kinds …. If you would get one or more counters, you get that many plus one
  of each of those kinds of counters instead." The second player-subject
  watcher, on `Plus(1)`, and the object half's "each of those kinds" is
  `Amount` applied per matching kind, which the entry door already does.
  Rulings, six, four are tests: *enters with that many plus one* → entry
  door; *multiple instructions in one effect each get plus one* → two
  `Primitive::AddCounters` in one resolution, two proposals; *two
  Constrictors: plus two* → two rows; *can't apply to itself or to anything
  entering with it* → RC-3's membership rule and RE-4's batch, asserted.
- **Live Fast** — "You draw two cards, lose 2 life, and get {E}{E}." The
  producer: `AddCounters { subject: Player(you), counter: Energy, n: 2, by:
  you }`, and every other instruction it carries exists (RE-2's draw, RA's
  loss). No rulings beyond energy's definition, which is the test: a player
  has a map, the map has a kind, and nothing else changes.
- **Primal Vigor** — "If one or more tokens would be created, twice that many
  … If one or more +1/+1 counters would be put on a creature, twice that many
  …" — `Everyone` on both halves, and the ruling "it doesn't matter who
  controls the tokens or the creature" is the four-player test: an opponent's
  Raise the Alarm makes four.

**`PERFORMANCE_POOL` +1, Hardened Scales**, predicted: a one-mana static that
opens the sweep on every `AddCounters` (Battlegrowth is in the pool) and every
counter-bearing entry (Chainbreaker, Master Biomancer's grants, Loyalty Probe
in the stress pool). Doubling Season is registered and not pooled — a
five-drop, and its token half would double the pool's Zombies, which is a
gameplay change the A/B should not carry with the engine change.

**Atoms:** `ATOM-122.6a-001` moves from the default half to a named-player test
(item 43's field, read by Vorinclex); `ATOM-122.1f-001` and `ATOM-704.5c-001`
(Phase 5-Pre, poison — read off the map now, and covered where they are); the
rest of §2.16 is thin under its phrasings, and the doubling tests say so.

#### RE-6 — the game's end (CR 104.2b, 104.3e, 104.4a, 704.5a–c, 704.7, 119.5, 800.4j–k)

**Builds:** decision 5 — the two variants and arms, the SBA batch members,
704.7's per-player leg, the flag reset, `GameResult` on `GameState`,
`Primitive::LoseGame`, `Primitive::WinGame`, `Primitive::SetLifeTotal` (CR
119.5 — "the player gains or loses the necessary amount", proposed through
`GainLife`/`LoseLife` so Rhox Faithmender's "3 becomes 10 becomes 17" ruling
is a test here), `GameActionTemplate::PlayerWins`, the two rotation sites, and
`fuzz_games --players N`. **Consumers:**

- **Laboratory Maniac** — "If you would draw a card while your library has no
  cards in it, you win the game instead." `DrawCard { cause: None }`, `You`,
  `Instead(PlayerWins)`, gated on a new `Condition::LibraryEmpty` evaluated
  at gather — the leaf's second customer is Jace, Wielder of Mysteries, a
  planeswalker, so the leaf is written for one card and says so; CR 121.6a
  is what makes the proposal reach the pipeline at all. Rulings, two, both
  tests: *if you can't win (Angel's Grace), you don't lose for the attempted
  draw either — the draw was still replaced* → Platinum Angel's row refuses
  the `PlayerWins`, the draw never performs, no flag is set; *two Maniacs and
  an each-player draw: APNAP, and the game ends at the first win* → needs an
  each-player draw producer, which `EffectRecipient` lacks (RD-2's Kitsune
  Palliator note); recorded on the card, not tested.
- **Exquisite Archangel** — "Flying. If you would lose the game, instead exile
  this creature and your life total becomes equal to your starting life
  total." `PlayerLoses`, `You`, `Prevent` with a rider of `Exile(self)` and
  `SetLifeTotal(starting)`. Rulings, nine, five are tests: *lethal damage to
  it and to you at once: its effect applies, and you choose exile or
  graveyard* → the SBA batch decides against one board, and the rider's exile
  meets the death's `ZoneChange` as two proposals on one object — the CR 704.7
  same-object collapse from RA-3, now with a player loss beside it; *drew
  from an empty library: you won't lose again until you try again* → the
  flag reset; *two Archangels: you choose which* → CR 616.1 among two
  printed statics; *an effect saying you can't lose: doesn't apply* → Platinum
  Angel's row ahead of the pipeline; *life -4 becomes 20 is a 24-life gain,
  and cards that interact with gain see it* → Rhox Faithmender makes it 44,
  which is the `SetLifeTotal` decomposition observed. **`ATOM-704.7-001`** —
  Lich's Mirror's board, 0 life and an empty library in one check, one
  replacement replacing both — is built with the Archangel; `COVERS` if the
  atom's board is generic, `COVERS-PARTIAL` naming Lich's Mirror if it is not,
  read at write time. Lich's Mirror itself waits on `ShuffleIntoLibrary`.
- **Stunning Reversal** — "The next time you would lose the game this turn,
  instead draw seven cards and your life total becomes 1. Exile Stunning
  Reversal." A `CreateReplacement` row, `PlayerLoses`, `Uses::Once`,
  `UntilEndOfTurn`, `Prevent` with a rider of `DrawCards(7)` and
  `SetLifeTotal(1)`, then `Exile(self)` as the resolution's second instruction
  (CR 608.2c, not part of the row). Rulings, ten, four are tests: *fewer than
  seven cards: you lose immediately after* → the rider's draws flag the empty
  library and the next check proposes a loss the spent row cannot see;
  *everyone would lose at once but this applies to you: you win as soon as
  everyone else has lost* → the four-player board, CR 104.2a from a batch
  with four `PlayerLoses` members and one replaced; *can't lose: can't
  apply*; *does nothing if you concede* → recorded, no harness.
- **Platinum Angel** — "Flying. You can't lose the game and your opponents
  can't win the game." Two `Effect::Restriction` statics, `Event {
  PlayerLoses, affected_players: You }` and `Event { PlayerWins, Opponents }`.
  Rulings, three: *no game effect can cause you to lose — 0 life, empty
  library, ten poison, Phage — you keep playing* → the SBA proposal refused
  every check, and the game continues through it (the first fuzz-reachable
  game that runs past a lethal board); *concession still loses* → recorded;
  *effects saying the game is a draw are unaffected* → CR 104.4c has no
  producer, recorded.

**`PERFORMANCE_POOL` +1, Laboratory Maniac**, predicted: a three-drop static
draw watcher, and the first card that makes decking a *win* in a measured game
— fuzz games deck out rarely, so the `--require` count and "games ended by a
win" are the rows to read. Platinum Angel is registered and not pooled: a
seven-drop that turns lethal boards into stalls would move avg turns by design
and not by engine.

**The four-player run is this PR's second measurement.** `fuzz_games
--players 4` at 200 games on `performance`: zero panics is the bar, and every
row it moves is written down as the *starting point* for RE-7 — item 108's
permanents that stay — and for B3's 802.

**Atoms:** `ATOM-704.7-001`, `ATOM-614.11-002` (Laboratory Maniac, the
corpus's own example), `ATOM-104.2b-001` and `ATOM-104.3e-001` (Phase 8, the
two primitives — covered where they are, not re-filed), `ATOM-119.5-001` and
`-002` (Phase 8, `SetLifeTotal`, same), `ATOM-104.4a-001` (ALREADY-IMPL,
gains its `COVERS:` from the four-loss batch), `ATOM-800.4j-001` as
`COVERS-PARTIAL` (the priority half; "the turn continues without an active
player" is RE-7's).

#### RE-7 — leaving the game (CR 800.4a–e, 800.4m)

**Builds:** the rest of decision 5's N-player half, inside `PlayerLoses`'
performer as CR 800.4a says — "this is not a state-based action. It happens as
soon as the player leaves the game." In the rule's own order: every object the
player owns leaves the game (removed from hand, library, graveyard, exile,
command zone, battlefield and stack, one `GameEvent::LeftTheGame { object }`
each — not a zone change, CR 400.11: outside the game is not a zone); every
control-changing row in that player's favour ends (a Layer 2 row keyed by the
departed controller, `ContinuousEffect.controller`); their stack objects not
represented by cards cease to exist; and anything they still control is exiled
through `change_zone` with a new `ZoneChangeCause::ControllerLeftTheGame`,
which no catchall may absorb. 800.4b and 800.4d are refusals at
`propose_entry`, `CreateTokens`' performer and Layer 2's control change for a
departed player; 800.4e is a refusal at combat damage assignment; 800.4m sets
each "until that player's next turn" duration to expire when that turn would
have begun, on all three duration registries. 800.4c — a control effect ending
with no default controller left — is main item 9's revert and lands here as
its own arm. **Consumer:** Act of Treason, which is in the pool, both ways
round — the thief loses and the stolen creature goes home (800.4a's second
clause); the owner loses and the creature the thief still controls leaves the
game (first clause) — plus `setup_game(4)` tests for each clause and the
four-player fuzz run RE-6 opened, whose "permanents that stay" row this PR
zeroes. No cards.

**`PERFORMANCE_POOL`: no change.** The measurement is the four-player table:
RE-6's starting point against this PR's, and CPU on the two-player pools
byte-identical, since nothing here runs before a third player exists.

**Atoms:** `ATOM-800.4a-001`, `ATOM-800.4a-002`, `ATOM-800.4b-001`,
`ATOM-800.4c-001`, `ATOM-800.4d-001`, `ATOM-800.4e-001` (all Phase 9, covered
where they are); `COMP-800-PLAYER-LEAVES-COMMANDER-001` as `COVERS-PARTIAL`
until commander damage exists; `ATOM-800.4j-001` completes.

#### RE-8 — the producers (CR 701.9, 701.9b, 701.22)

**Builds:** decision 8 — `Primitive::Discard(n, DiscardChooser)` with
`ChoiceKind::Discard` (the cleanup discard's, reused) and "at random" from
`GameState.rng`; `caused_by: Option<PlayerRef>` on `EventPattern::ZoneChange`;
the to-battlefield leg on `Instead(ZoneChangeTo)`; `GameAction::Scry { player,
n }`, its arm, `Primitive::Scry` and `ChoiceKind::Scry`. **Consumers:**

- **Mind Rot** — "Target player discards two cards." The default chooser;
  the target's choice through `EffectRecipient::Target` on a player. Its test
  is Notion Thief's ruling from RE-2, now against a printed card: a
  draw-then-discard the Thief modified still discards.
- **Hymn to Tourach** — "Target player discards two cards at random." The
  second chooser, from the owned `rng` (`CLAUDE.md`: randomness is never
  ambient), so `tests/determinism_test.rs` covers it by construction.
- **Dodecapod** — "If a spell or ability an opponent controls causes you to
  discard this card, put it onto the battlefield with two +1/+1 counters on
  it instead of putting it into your graveyard." `ZoneChange { from:
  Some(Hand), cause: Some(Discarded), caused_by: Some(Opponent) }`,
  `SourceOnly`, `Instead(ZoneChangeTo { to: Battlefield })` — the leg, and
  the entry it returns carries `EnterMods` with two counters, which RE-5's
  doubler then sees. Rulings, three, all tests: *the opponent's spell had you
  choose: still applies* → Mind Rot; *you still discarded — discard triggers
  will trigger* → the performed `ZoneChange { Discarded }` is in the log
  beside the entry, item 6's; *"the discarded card" refers to the Dodecapod
  on the battlefield* → recorded, no reader.
- **Wilt-Leaf Liege** — the same clause on a lord ("other green creatures you
  control get +1/+1", twice), so the leg's second customer is a permanent the
  layer walk already understands; and its ruling — *Leyline of the Void and
  the Liege both want the discarded card: you choose* — is a CR 616.1 prompt
  between an RB card and an RE-8 one.
- **Opt** — "Scry 1. Draw a card." The scry producer, and CR 608.2c's "A,
  then B" in one resolution.
- **Eligeth, Crossroads Augur** — "Flying. If you would scry a number of
  cards, draw that many cards instead. Partner." `EventPattern::Scry`, `You`,
  `Instead(DrawCards { n: ReplacedAmount, player: None })`. Partner is
  CR 702.124's deck-construction ability with no in-game text, so nothing is
  dead under the name. Its test is §10's acid test with a different producer:
  `test_opt_with_eligeth_draws_two_and_never_scrys` — Opt under Eligeth draws
  two and the log holds no scry — proves §4.1a's instruction split and the
  kind-changing `Instead` reading the event's amount; the Goggles of Night
  form waits for item 6, since Goggles scries from a *trigger*.
- **Library of Leng** and **Loxodon Smiter** stay out: a hand size
  (`backlog.md` §2.15) and "can't be countered" (§8a's missing counter event),
  each one facility away.

**`PERFORMANCE_POOL` +2, Mind Rot and Opt**, predicted: the pool's first
discard outside cleanup and its first scry, so `ZoneChange { Discarded }` and
`Scry` become rows item 6 can read from a measured game, and the random agent
finally reaches CR 701.9b's choice. Dodecapod is registered and not pooled:
four mana for a 3/3 in a pool that rarely discards is a slot with nothing to
measure until Mind Rot is common.

**Atoms:** `ATOM-701.9b-001`, `ATOM-701.9b-002` as `COVERS-PARTIAL` ("another
player chooses" has no printed producer here), `ATOM-701.22a-001`,
`ATOM-701.22b-001` (Phase 8, covered where they are, not re-filed; scry 0 is a
`never_happens` arm, CR 701.22b's own words).

#### RE-9 — mana (CR 106.6a; RA's unnamed debt)

**Builds:** decision 7 — `ProduceMana`, its arm, the one performer replacing
`resolve_mana_effect`'s and `Primitive::ProduceMana`'s direct writes,
`ManaAdded` emitted for the first time, and `tapped` read off the activation's
paid cost. **Consumers:**

- **Mana Reflection** — "If you tap a permanent for mana, it produces twice as
  much of that mana instead." `ProduceMana { tapped: Some(true) }`,
  `Fixed(vec![])` + `You`, `Amount(Multiplier(2))`. Rulings, four, three are
  tests: *only a mana ability with {T} in its cost* → Dark Ritual (in the
  pool) adds three, not six; *a triggered mana ability is unaffected* → item
  6's, recorded; *restrictions and riders apply to all the mana* →
  `ATOM-106.6a-001`, on a fixture ability with a `special` atom, since no
  registered land restricts its mana; *two Reflections quadruple* → the
  `Amount` pair, fourth kind.
- **Nyxbloom Ancient** — "Trample. If you tap a permanent for mana, it
  produces three times as much of that mana instead." `Multiplier(3)`, the
  second factor the arm has seen. Ruling: *two Ancients: nine times* → test.

**`PERFORMANCE_POOL` +1, Mana Reflection**, predicted: a six-drop static on
the hottest path in the engine. Its row is the one this PR exists to read.

**Atoms:** `ATOM-106.6a-001`; `ATOM-106.6-001` (Phase 5-Pre, the restriction
survives the type — covered by the same fixture, and it is not in `owed`'s nine
because its ticket is not `NEW`).

#### RE-10 — extra phases, and the turn plan (CR 500.8, 500.11, 505.1)

**Added at RE-1's review (2026-09-11), on §11 item 49.** Decision 6 pulled
CR 500.7 into a skips PR so that `backlog.md` §2.17 "does not rewrite
`advance_turn` a second time"; that holds for extra *turns* and not for extra
*phases*, because RE-1's cursor names a phase **type** and a turn can hold two
combat phases. This PR is that cursor, its producer, and the card. It is a
tenth RE PR rather than a rewrite of RE-1 because **it adds no event kind** —
the proposal is still `BeginPhase { phase, player }`, unchanged.

**The design, in four decisions.** Sized after the other nine, so it carries
its own rather than pointing at "The design check".

**1. The cursor becomes an index into a per-turn plan, and `next_phase`'s chain
goes.** `GameState.turn_plan: TurnPlan` is CR 500.1's five phases, seeded by
`GameState::new` and rebuilt by `on_turn_begin` — so a **skipped turn builds
no plan**, which is CR 614.10a for free — and `drain`'s cursor becomes
`Option<usize>` into it. `TurnUnit::Phase` carries the index. This finally
spends `state::game_state::next_phase`'s pre-RE-1 TODO, which described this
type by name and which RE-1 rewrote into a pointer.

**2. "After this phase" is an insertion at the cursor, and CR 500.8's ordering
falls out of it.** `Primitive::ExtraPhases(Vec<PhaseType>)` splices at
`cursor + 1`. *"If multiple extra phases are created after the same phase, the
most recently created phase will occur first"* is then the splice's own
behaviour — a second insertion at the same index pushes the first later — with
no comparator anywhere, exactly as CR 500.7's "most recently created turn"
turned out to be `Vec::pop`.

**3. An inserted main phase is `PhaseType::Postcombat`, and the choice is
unobservable.** Checked rather than assumed: **every production reader of the
two main types matches them as one arm** — `cast.rs:659`, `zones.rs:190` and
`oracle/legality.rs:50`, each asking "is this a main phase". The CR does not
name an additional main phase either way; CR 500.8 says only "directly after
the specified phase". The axis a later card would grow is a field on the plan
entry, and its customer is the first card that prints a distinction between the
two main phases the engine must keep.

**4. CR 500.9 and 500.10 stay out, and the field they want is named.** An extra
*step* needs a plan entry that overrides its phase's natural step list —
500.10's "any other steps that phase would normally have are skipped" **is**
that override, and it is one `Option<Vec<StepType>>` on `PlannedPhase`. Its
only producer is Obeka, Splitter of Seconds, whose ability is **triggered**: it
cannot land before item 6 whatever RE does. So the field waits, on "an arm the
pipeline cannot apply is worse than a missing one", and this is the paragraph
that says where it goes.

**Builds:** `TurnPlan` and `PlannedPhase`; `GameState.turn_plan`; `drain`'s
index cursor and `next_turn_unit` reading the plan; `next_phase`'s chain
deleted; `Primitive::ExtraPhases` with its splice; `Primitive::Untap` gaining
the `EffectRecipient::FilteredPermanents` arm that `DealDamage` and
`CreateReplacement` already have. **Consumers:**

- **Aggravated Assault** — {2}{R} Enchantment. *"{3}{R}{R}: Untap all
  creatures you control. After this main phase, there is an additional combat
  phase followed by an additional main phase. Activate only as a sorcery."*
  The producer, and **the only one of the 46 in reach whole**:
  `ActivationRestriction::OnlyAsSorcery` exists and is enforced
  (`cast.rs:345`, where Bonesplitter's Equip is its current customer), and its
  untap is the new recipient arm. Seize the Day needs flashback, World at War
  needs rebound *and* "creatures that attacked this turn", Relentless Assault
  needs the second of those, and the rest are triggers. Rulings pass at
  registration, in the card file's doc comment.
- **Moment of Silence** — registered by RE-1, and this is what gives its first
  ruling a board the engine produces: *"if they manage to have two combat
  phases, then only their next one combat phase is skipped."* RE-1 tests it
  against a second `BeginPhase { Combat }` proposal the fixture makes by moving
  the cursor and says so; RE-10 deletes the cursor move.

**Tests:** two phases inserted after the main phase the ability resolved in, in
that order; two activations, the second's phases taken first (CR 500.8); combat
state reset between two combat phases in one turn — `on_phase_end(Combat)`
already clears all five fields, and the test is what keeps that true; mana
emptying at each inserted phase's end (CR 500.5); a turn's position count with
and without; Moment of Silence's ruling without the fixture; four players.

**Atoms: none, and that is a gap this section names rather than absorbs.**
`session-4.md` files 500.8, 500.9 and 500.10 DEFERRED with no atom ids and
assigns them to *Phase 9*, whose stated content is formats and multiplayer —
which reads like the era's `TurnPlan` TODO rather than a judgement. `specdb
owed` therefore cannot move either way. **CR 500.8 should have an atom**, and
authoring one is the corpus's own work (`session-4.md`), not this PR's.

**`PERFORMANCE_POOL` +0 predicted, and the A/B decides.** Aggravated Assault is
{2}{R} plus a {3}{R}{R} activation — eight mana to use once — and every
activation makes the game *bigger*, which is the shape §3.1a rejected Altar's
Reap for at +20.2%; the reachability answer is a `--require` row, as it was for
Circle of Protection: Red. The engine's change needs no pooled card at all: the
plan replaces the chain on **every turn of every game**, so the middle arm
walks it unforced. Predicted `Replacement gathers`, `Layer walks` and every
gameplay row **flat**, and CPU flat or slightly down — an index beats a chained
`match` per unit. **If the middle arm is not flat, the cursor change did
something it should not have**, and that is the whole reading of this PR's A/B.

**Measured size:** ~700 engine and cards, ~400 tests ≈ **1,100–1,300** — under
the band's floor, like RE-7's 800–950, and for the same reason: one type, one
cursor, one producer, one card.

**One naming decision to make before the type is written**, found by RE-1's
glossary pass: **the crate already has a plan.** `plan_payment`, `pay_with_plan`
and `plan_and_pay` are the cost system's, and `planned_sacrifices` sits one
letter from `PlannedPhase`. That is the `ToSource` / `ToSourceController` shape
the glossary's polysemy gate exists for, caught this time *before* the build
rather than mid-PR. Keep `TurnPlan` — it is the name
`state::game_state::next_phase`'s own TODO used and the payment one is a local
idiom rather than a type — and **land the collision as `plan`'s two numbered
senses in `plans/glossary.md`** with this PR. It cannot be added earlier: the
gate asserts every defined term resolves in `mtgsim/src`, and `TurnPlan` does
not exist yet.

#### Out of RE, decided rather than absorbed

- **`pending_skips`** — struck (decision 6). Not a deferral: a mechanism that
  should not be built.
- **CR 802's defending player**, and 800.4f–h's choices by a departed player
  — B3's ("Before Commander" item 4), with RE-7 having built the seam.
- **Extra phases** (CR 500.8) — cut at RE's sizing, re-opened at RE-1's review
  and **taken in**, as RE-10: the "written once" argument that pulled CR 500.7
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

Three arms per PR through `plans/fuzz_ab.py` against a same-day `main`
worktree, both pools, the middle arm being the engine with `registry.rs` and
`PERFORMANCE_POOL` unchanged — and unlike RD, **five of the nine middle arms
add proposals**, so the counters will move on those and each PR predicts the
number before running:

- **RE-1 — measured 2026-09-11, and it is the one number the rest of RE reads.**
  `Replacement gathers` **+447 per game** (507 → 954 on `performance`, 200
  games / seed 12345) against a predicted ~+450, and `Restriction queries`
  moved with it — one `is_prohibited` per proposal, which the prediction
  missed. `Layer walks`, `Board walks`, `Layer frames` and `Dependency checks`
  **flat**, and every gameplay row identical to `main`. `Memo hits` +1.6%,
  because the fast path stops the walk and not the query. **CPU/game +5.0%**
  (12.92 → 13.57 ms), inside the stated 2–6% spread but cleanly separated
  round to round. A fourth arm with a hard-coded event-kind gate measured
  **+2.5%**, so the sweep is half the cost and the chokepoint is the other
  half — **the gate was not built**, with the reasoning and the trigger in
  `plans/archive/replacement-architecture-landed.md`, "RE-1". **RE-9 is the PR
  that reads this**: 2.5 points is what a gate would return it, and its own
  bullet already calls it the one whose A/B could say no.
- **RE-2:** `Replacement gathers` +1 per draw instruction (the outer), so
  roughly +1 per turn plus one per cantrip; `Layer walks` flat; 40-game event
  dumps identical but for one `CardDrawn` line's neighbour. The shipped arm
  adds walks only while Thought Reflection is on the battlefield.
- **RE-3:** flat on the middle arm — patterns over events that already flow.
  A move is the card.
- **RE-4:** gathers +1 per token creation (Kalitas's rider on `stress`; zero on
  `performance` until Raise the Alarm is pooled); `Layer walks` +N per plural
  creation for the frame each entry is decided against, which RC-5 measured
  per entry already.
- **RE-5:** flat on the middle arm; the entry door is a `pattern_watches`
  branch that no def reaches until Hardened Scales is registered, and the
  subject enum changes no proposal's count.
- **RE-6:** +1 gather per game (the loss), flat everywhere else. The
  four-player run is its own table, not an A/B.
- **RE-7:** byte-identical on both two-player pools, by construction; the
  four-player table against RE-6's is the measurement.
- **RE-8:** +1 gather per discard and per scry that resolves, on the shipped
  arm only (no pooled card discards or scries before it); middle arm flat.
- **RE-9:** +1 gather per mana ability that resolves — every land tap, several
  per turn — and the same fast-path argument as RE-1. The prediction is flat
  CPU and a moved `Replacement gathers` row; a fourth binary with the proposal
  reverted is the recipe if it is not, and the §8 gate is the fix. **If the
  gate is needed and does not close the gap, RE-9 is the PR the owner decides
  against, and the two writers keep their direct write with a Deferred
  Migrations line that names the number.**

- **RE-10:** the one arm in RE predicted **flat in every row**, and that is the
  whole reading. The plan replaces `next_phase`'s chain on every turn of every
  game, so the middle arm walks the new cursor unforced with no pooled card;
  gathers, walks and every gameplay counter should be byte-identical to `main`,
  and CPU flat or slightly down, since an index beats a chained `match` per
  unit. **A move in any counter means the cursor changed a turn's shape**, and
  the only two that legitimately could — a turn's position count, and the
  order phases are proposed in — are exactly what the arm is checking.
  `PERFORMANCE_POOL` +0 predicted, with a `--require` row instead.

§3's table is re-recorded once per PR that moves the pool, at 50 games, after
the A/B; from RE-6 on, the four-player table beside it.

#### Trace page — decide at RE-2's close, and again at RE-4's

`engineering-practices.md` §7's rule is met twice: RE-2 changes *how* the
applied set is answered for a decomposed event (a draw carries its lineage), and
RE-4 changes how an entry's frame is answered inside a plural batch (a token is
decided against a board its siblings have not entered). Neither is the happy
path, and both are the boards §3.2d and §5b argued about. Decide each at that
PR's close; if yes, `re-2-a-draw-carries-its-lineage.html` walks two Thought
Reflections, Alms Collector beside one, and the three-Thief board.

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
   `EffectModification`, re-run `object_matches_filter`. `layers-architecture.md`
   §12 measured frame construction at **0.37 µs at N=10 falling to 0.27 µs at
   N=80** — flat in board size, "3% and flat" in its own words, explicitly not
   the bottleneck. So the expensive thing about a
   snapshot was never the frame; it was the idea of copying the *game*. 613.8
   does not need to.

   That matters because of where the check sits: up to O(effects²) candidate
   pairs per layer, inside a walk that runs per permanent, inside a sweep that
   runs per priority check. §12's table already shows this shape going
   superlinear when a per-effect cost is added — the ungated CR 604.2 existence
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
   effect, and stays a bool. **Confirmed at RE's sizing (2026-09-11): RE-1, §9
   RE decision 6 — and §9's older paragraph, which said `pending_skips`, is
   struck; this item was right and it was not. Built and closed 2026-09-11
   (RE-1): the proposal is `GameState::advance_turn`'s, the three units are
   `GameAction::{BeginTurn, BeginPhase, BeginStep}`, and `skip_first_draw` is
   still a bool in `process_draw_step`.**

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
   section already records that timestamps must move off `PermanentState`
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
    engine can express none of it — `counters` lives on `PermanentState`,
    and `perform_action`'s `AddCounters` arm errors for anything else.

    **So: do not honour Skullbriar on its own.** The change is one move, the
    `counters` map from `PermanentState` to `GameObject`, and it is contained
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
    there has the sizing; the regression is the one §3.2d already names, and
    **RE-2** is the PR that needs it (§9, sized 2026-09-11: the outer draw's
    performer is the producer).

    **Closed 2026-09-11 by RE-2.** The producer is `GameAction::DrawCards`'s
    performer and the regression shipped as
    `test_two_thought_reflections_draw_four_not_infinity` — on Thought
    Reflection rather than the Teferi the name predicted, because Teferi's
    Ageless Insight is legendary and the two-copy board it takes is unbuildable.
    The bound the item asked for is the `ScriptedDecisionProvider`: a correct run
    makes exactly one CR 616.1 prompt, so a provider primed with one turns the
    second into a red test at depth two rather than a binary that never returns.
    Mutation-checked both ways.

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
    that shipped (`pipeline::ordering_cannot_change_outcome`) admits only members
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

### Found by RD's design check (2026-09-08)

Seven items from sizing Phase RD against CR 615, 614.9, 609.7 and 120.3 and
against the tree. §9's RD section carries the decisions; these are the facts
that fell out of making them, recorded here so they outlive the section that
found them.

21. **Player scoping is RD's, and `types/replacement.rs` says RE.** *Closed by
    RD-1 (2026-09-08): `ReplacementDef.affected_players: PlayerSet` shipped,
    `set_affects` unions the two sets, and both comments are corrected.* The
    module doc and `gather::set_affects` both record that an effect applying to
    a *player* waits for RE's draw cards. The damage family is where the
    pressure actually is — 23 "prevent all damage that would be dealt to you",
    12 "would deal damage to you, prevent", 46 "source of your choice … to
    you", and Furnace of Rath's "permanent or player" — so RD-1 lands
    `ReplacementDef.affected_players: PlayerSet` and RE inherits it. A second
    field rather than an `AffectedSet` variant, because that type has three
    readers and two of them would have to reject the arm. The module doc and
    `set_affects`'s comment are corrected by RD-1; §3.2a's "lands in Phase RE"
    reads as history from here.

22. **`consume_use` runs before `apply_rewrite`, and three rules need it
    after.** *Closed by RD-2 (2026-09-09): `consume_use` runs after
    `apply_rewrite` and spends `Applied { took_effect, prevented }` — `Once`
    only when the rewrite took effect, `NextDamage` by exactly the damage
    prevented. The regression is a `Once` "prevent half, rounded down" chosen
    against 1 damage, which prevents 0 and stays
    (`a_once_prevention_that_prevents_nothing_is_not_used_up_dark_sphere_is_rd_3s`).
    RD-1 had left it, correctly: it shipped no arm whose application could do
    nothing.* CR 609.7b ("if for any reason the shield prevents no damage or
    replaces no damage, the shield isn't used up"), CR 614.9 (a redirect whose
    destination is gone "does nothing"; `ATOM-614.9-001` says the shield is
    not spent) and CR 615.12 (an unpreventable event reduces no shield). Every
    RB and RC rewrite takes effect whenever it is chosen, which is why the
    order was never wrong before — regeneration's `Prevent` on a `Destroy`
    cannot do nothing. RD-2 makes `apply_rewrite` report what it did and spends
    the use from that. The applied set is *not* moved: a chosen effect that did
    nothing has still had CR 614.5's one opportunity, and re-offering it is the
    hang §4.1 warns about.

23. **One printed shape splits a damage event in two, and §3.2d's `Option`
    cannot hold it — so it is a candidate PR with a gate, not an exclusion.**
    *Gate closed by RD-4 (2026-09-09): it fails, and Harm's Way is now
    `backlog.md` §2.25. A split-off member has no batch index, and ≈ 30 sites
    between `apply_rewrite`'s return and `execute_batch_inner`'s `decided`
    write are keyed by one — including the two the gate named as
    disqualifying. §9's RD-5 section carries the table, the three things worth
    keeping, and the one piece of genuinely new work the ~300–400 sizing did
    not contain (`next_damage_shares` allocating a count and a split with one
    choice). The original entry follows, unchanged, because the shape is still
    the shape.* Harm's Way — "The next 2 damage that a source of your choice would deal to
    you and/or permanents you control this turn is dealt to any target
    instead" — redirects *part* of one event: 2 of a 3-damage Lightning Bolt
    goes to the chosen target and 1 stays on you. That is one `DealDamage`
    becoming two with different targets, which is exactly the fan-out §3.2d
    removed `Split` for, and its rulings pin the rest: the 2 may be split "1
    damage … to each of two different recipients" across *members* of a
    batch (decision 3's per-instance allocation), and redirecting 1 leaves
    "a 'shield' … for another 1" (decision 7's spend-by-amount). Whole-event
    redirection (Pariah, Palisade Giant, Kor Chant, Reflect Damage — 15 cards
    on `o:/would be dealt to you.*dealt to .* instead/`) needs none of this
    and is RD-4's. The shape for the split is **not** a `Rewrite` that
    returns two and **not** a rider (the moved damage is dealt simultaneously,
    by the original source, as part of the same event — CR 614.9): it is a
    phase-1 member insertion, the residue staying the event and the moved
    part a split-off member that continues the lineage. §9's RD-5 carries the
    gate — off the per-member path or into the backlog with the measured
    cost — because one card that was never played competitively should be
    excluded on a number, not on a feeling. Divine Deflection is *not* this
    shape: a `Filter` + `PlayerSet` row with `NextDamage(X)` is already a
    pooled amount, and it waits only on `AmountExpr::Variable`.

24. **§11 item 15 is answered: decisions are per `(batch, subject)`, rewrites
    per member, and CR 615.7's allocation is the one non-uniform rewrite.**
    *Built by RD-2 (2026-09-09): `execute_batch_inner` groups by `subject_of`
    and `apply_replacements` takes the group; the shield-counter board is
    `two_shield_counters_under_two_blockers_lose_one_counter_and_take_no_damage`,
    and its first-strike twin is what shows the key is the batch. The
    allocation is `next_damage_shares`, per instance across every member it
    applies to — later groups' members included — kept on
    `GameState::prevention_allocations`. Closed.* The
    table in §9's RD section checks the rule against every ruling the batch
    has had to satisfy — Kalitas's N Zombies, CR 122.1c's one counter under two
    blockers, 614.5's ×4, 615.10's "separately to … events that would happen
    at the same time", and 615.7 itself. The per-member shape got the counter
    board wrong and the per-batch shape §4.2 rejected got Kalitas wrong; the
    subject is the key both rulings agree on. RD-2 builds it; the shield
    counter under two blockers is the regression.

25. **"Unpreventable" is a property of the event, `is_prevention` is derived,
    and `cant-effects-architecture.md` §4.7's `ReplacementKind` is
    superseded.** ***Closed by RD-4 (2026-09-09), and all three of its claims
    shipped as written.*** `GameAction::DealDamage.unpreventable` is the
    per-event shape; `Restriction::ApplyReplacement { kind: Prevention }` is
    the other two, with the `to_players: PlayerSet` this item's own decision 0
    predicted; `ReplacementDef::is_prevention()` stayed derived and
    `is_regeneration` kept its authored bit. The three meet in one predicate,
    `pipeline::is_unpreventable`, called at the two sites a prevention arm
    applies — two rather than the one this item assumed, because RD-3 gave
    `Rewrite::Prevent` a prevented amount of its own (item 31). Not at
    `gather`'s door, exactly as argued. Pinpoint Avalanche is the registered
    per-event consumer; both restriction routes are fixtures, for item 26's
    reason. *The RD-1 note this supersedes:*  *Unchanged by RD-1, which ships
    neither the flag nor the consult: `AmountRewrite::PreventHalf` applies unconditionally there, and
    Ghosts of the Innocent's Excruciator ruling is RD-4's for exactly that
    reason. `AmountRewrite::prevented()` is the reporting half, and it shipped.* CR 615.12 has three printed shapes: a resolution's
    restriction with a duration ("damage can't be prevented this turn", 11
    instants — a registry row), a static ability's restriction ("damage can't
    be prevented" on Leyline of Punishment and Everlasting Torment — swept off
    the effective ability list, no row, filterable), and a fact about one
    event ("the damage can't be prevented", 9). The first two are two sources
    of RS-1's `Restriction::ApplyReplacement { kind: Prevention }`, which
    `is_prohibited` already unions; the third is
    `GameAction::DealDamage.unpreventable`, set by the proposer and carried
    through a redirect. All three meet at one site — where a prevention arm is
    *applied* — and not at `gather`'s door, because CR 701.19c withholds a
    regeneration shield ("not applied") while CR 615.12 applies a prevention
    effect and lets it prevent nothing. Recognising a prevention effect needs
    no authored bit: CR 615.1a defines it by the word "prevent", which the def
    carries as `Prevent` or `Amount(PreventUpTo | PreventRemaining)` on
    `EventPattern::DealDamage`, so `ReplacementDef::is_prevention()` is a
    method and a card cannot forget to set it — the CDA and CR 614.15 argument
    (item 12) for the third time. `is_regeneration` keeps its bit, since
    nothing in a regeneration def distinguishes it from any other
    `Prevent`-with-a-rider.

26. **Every "damage can't be prevented" card but two carries a half the engine
    lacks.** *Discharged by RD-4 as far as it can be (2026-09-09): Pinpoint
    Avalanche is registered and CR 615.12's per-event route is exercised by a
    printed card; both **restriction** routes are built and tested as fixtures
    in `tests/phase_rd4_integration_test.rs` — a `RegisteredRestriction` row for
    the turn-scoped form and a static `Effect::Restriction` on a fixture
    permanent for Leyline's — and neither has a registrable consumer, which the
    PR said in as many words. The item stays open as the note that says why,
    and it closes when RE's `GainLife` pattern arm lets Skullcrack land the row
    in a game.* Skullcrack and Leyline of Punishment print "players can't gain
    life" (RE's `GainLife` pattern arm), Unstable Footing has kicker, Stomp is
    an adventure, Flaring Pain has flashback, Wild Slash is a `Conditional`,
    Everlasting Torment has wither, Combust "can't be countered" (§8a's missing
    counter event). Pinpoint Avalanche is clean and is RD-4's per-event
    consumer; the restriction consult is built against a fixture row and
    Skullcrack lands it in a game in RE-3 (§9, sized 2026-09-11), which also
    gives `Restriction::Event` the `affected_players` the row needs — see
    item 45. Recorded so the arm's producer
    status is not misread as an omission.

27. **CR 120.3 has eight results, RD ships two of them, and the other four
    have an owner each as of today.** *Closed by RD-1 (2026-09-08), with one
    addition the sizing did not name: 120.3e is now gated on the target being a
    creature, because writing the arm as a list of results made the ungated
    marking visible as bookkeeping the rule does not have.* 120.3a (life loss) and 120.3c (loyalty)
    are RD-1's. 120.3b and 120.3g (poison — infect and toxic) and 120.3d
    (wither's and infect's counters) have no keyword flag behind them and are
    `backlog.md` §2.6's, which now names the three keywords, their results,
    their size and RD-1's seam; 120.3h (a battle's defense counters) has no
    card type behind it and is `backlog.md` §2.23's, new today because no doc
    owned CR 310. The ledger's `T21c` was the only thing pointing at them,
    and a `T##` names work without owning it. The performer's arm is written
    as one `match` per result on the source's keywords and the target's type
    so each lands as one arm; none is stubbed, and each gets a dated Deferred
    Migrations line at RD-1's commit. 120.3e (marked damage) and 120.3f
    (lifelink) were already there.

28. **The inverse of a doubler is printed, it rounds, and the rounding is
    the card's.** Asked on review: "half that damage, rounded down" (Ghosts
    of the Innocent, an `Instead`-shaped halving), "prevent half that damage,
    rounded up" (Gisela) and "rounded down" (Dark Sphere), and a rider's
    "half that many, rounded down" (Sokrates) are four printings of one
    arithmetic in three places. CR 107.1a puts the direction on the card, so
    `Rounding { Up, Down }` carries no default, and it appears in exactly the
    places the text puts it — `AmountRewrite::Halve` and `PreventHalf` on the
    rewrite, `AmountExpr::Half` on an amount — never as an engine-wide
    convention. `Halve` and `PreventHalf` stay two arms because CR 615.12
    separates them (Ghosts halves unpreventable damage; Gisela cannot
    prevent any of it) and because only the prevention arm reports what it
    prevented. `Multiplier`'s inverse is therefore not a `Divide(n)`: nothing
    printed divides by anything but two, and a general divisor would be an
    arm with no second customer.

### Found by the RD-1 review (2026-09-08)

29. **Two Furnaces prompt, and multiplication commutes — so §11 item 19's
    suppression theorem has a second candidate, and it is narrower than "N
    identical effects".** Asked on review: is a bucket of N identical effects
    that ask the player nothing always order-invariant, so CR 616.1's prompt is
    noise? For *triggers* the answer is trivially yes — they go on the stack in
    one APNAP pass and their relative order is fixed at that moment. For
    **replacement effects it is not**, and the reason is the one the reviewer
    reached unprompted: CR 616.1f re-gathers after every application, so the
    bucket is re-formed between members and "the other members" is not a fixed
    set. That is exactly the premise `ordering_cannot_change_outcome` spends its
    longest clause on.

    **What is provable is narrower and still worth having.** A bucket every
    member of which is `Amount(Multiplier(n))` is order-invariant: multiplication
    over `u64` is commutative and associative (saturating included, since
    saturation is monotone), and no multiplier can remove another's
    applicability — an amount above 0 stays above 0 under any `n ≥ 1`, so
    `never_happens` cannot fire between members, and `EventPattern::DealDamage`
    carries no amount predicate for a member to fall out of. Two Furnaces are
    that bucket, and the prompt in `two_furnaces_multiply_by_four_and_ask_once`
    is noise a human player would resent.

    **What breaks the moment the bucket is mixed** is the phase's own headline
    board: `Halve` beside `Multiplier` does not commute (3 → 1 → 2 or 3 → 6 → 3),
    which is `COMP-614-DAMAGE-ORDERING-001`. So the clause cannot be "all
    members are `Amount`"; it has to be "all members are `Amount(Multiplier)`",
    and `PreventHalf` and `Plus` each need their own argument if they ever want
    one. `Plus` beside `Plus` commutes; `Plus` beside `Multiplier` does not.

    **Is it worth it?** The cost is one more clause on a
    semantics-assuming shortcut that already carries expiry conditions
    (`layers-architecture.md` §12 item 3), plus a `check_order_invariance`
    arm — and the clause goes false the day an `EventPattern` field reads the
    amount, or a `Multiplier(0)` is printed, or `Uses::NextDamage` makes a
    member spendable mid-bucket. **The verdict is: not RD-1's, and RD-2 decides
    it**, because RD-2 changes the loop's unit and the suppression predicate is
    read at exactly the site it changes. Deciding it earlier would mean writing
    the clause twice.

    **Decided at RD-2's close (2026-09-09): yes, and built.**
    `ordering_cannot_change_outcome` — the predicate, renamed for item 65's
    reason now that "entry" had become wrong as well as implementation-shaped —
    admits a set of choosable effects that is entirely `Amount(Multiplier(n ≥ 1))` on
    `EventPattern::DealDamage` beside the all-`EnterWith` one, under the same
    shared clauses (mandatory, static, no rider, under CR 614.5, not
    counter-derived). Two Furnaces ask nothing; the composite atom
    `COMP-614-616-DOUBLE-REPLACEMENT-001` drops to `COVERS-PARTIAL`, because
    "Player A chooses order" is the half deliberately not built. The debug
    re-gather checks per member, since a candidate may apply to a subset of a
    group from RD-3 on. `codebase-state.md` item 47 carries the two new expiry
    conditions: an `EventPattern::DealDamage` field that reads the *amount*,
    and a printed `Multiplier(0)`, which the `n ≥ 1` clause refuses. Measured
    as its own arm of RD-2's A/B (§9, RD-2 as landed).

30. **A test that asserts "nothing happened" cannot say *why* nothing
    happened, and the trace sink is the instrument.** Raised against Ghosts of
    the Innocent's "half of 1 rounded down is 0, so a source that would deal 1
    damage won't deal damage at all": the test asserts 0 damage marked and no
    `DamageDealt` event, and neither distinguishes "Ghosts halved 1 to 0 and
    CR 614.7a dropped the emptied proposal" from "the damage never reached the
    pipeline". **The differential closes most of it** and was added — the same
    1 damage without Ghosts marks 1, and 2 damage with Ghosts marks 1, so the
    instance demonstrably applies on that board — but the *rule* that dropped
    the event is still not observable, and no assertion available today makes
    it so.

    That is a general property of `Rewrite::Prevent` and of every arm that can
    empty an event, so it is the trace sink's case rather than this test's, and
    the sink is already scheduled as its own PR (`roadmap-v2.md` A4c, moved out
    of A6 on 2026-09-08 and now after CM-4). **Nothing about RD moves it
    earlier**: the differential is available on every board RD can build, and
    RD-2's boards — a shield that prevents 0 without being spent — are where
    the argument for the sink actually gets stronger, since "nothing was
    consumed" has no event at all. Re-ask at RD-2's close, alongside the trace
    *page* decision §9 already schedules there.

    **Re-asked at RD-2's close (2026-09-09): A4c stays where it is.** RD-2's
    "nothing was consumed" boards turned out *not* to strengthen the case,
    because a CR 615.7 count is materialized state: `Uses::NextDamage(remaining)`
    sits on the row before and after, so "the shield was not spent" is a direct
    assertion (`counts(&game)` in `phase_rd2_integration_test`), and a `Once`
    row that prevented nothing is still in the registry to be counted. RD-1's
    Ghosts board had only an absence to assert; RD-2's have a number. The
    trace page walks the same boards by hand, which is tier 1's job; the
    sink's argument and its slot are unchanged.

### Found by RD-3 — sources, and its review (2026-09-09)

31. **`Rewrite::Prevent` reports how much damage it prevented, and RD-4 will
    read it.** CR 615.6's "prevent that damage" is the whole amount, and
    CR 615.5's rider may refer to it — Reverse Damage's "you gain life equal to
    the damage prevented this way" is the printed reader, and it was the first
    def in the crate to ask a whole-event `Prevent` what it prevented (RD-1's
    Angel of Suffering rides on `ReplacedAmount`; RD-2's counts reach the number
    through an `Amount` arm). The arm reports it rather than the caller
    deriving it from a dropped event, and the reason is the same line
    `ReplacementDef::is_prevention` draws: **only the arm knows the event was
    damage**, and a `Prevent` on a `Destroy` is regeneration, which prevents no
    damage at all (CR 615.1 is about damage, CR 701.19c treats the two
    differently). A caller-side derivation would have had to re-ask the pattern
    and would have got regeneration wrong the day a regeneration shield gained
    a rider that read a number.

    **The RD-4 consequence is direct.** CR 615.12's "those effects won't prevent
    any damage, but any additional effects they have will take place" needs
    `Applied.prevented` to be **0 on an unpreventable event while the rider
    still runs** — so RD-4's consult belongs at the same site this number is
    computed, not at `gather`'s door, and the `Prevent` arm is now one of the
    two places (with `Rewrite::Amount`'s prevention arms) that has to ask.

32. **A new `EventPattern` field costs nothing until a def writes it, and the
    A/B is the proof.** RD-3's middle arm — the whole engine with `registry.rs`
    and `PERFORMANCE_POOL` unchanged — is **byte-identical to `main` outside
    the timing block on both pools at 200 games**. `pattern_watches` reads
    `source` and `combat` through `Option`, and every def written before RD-3
    carries `None`, so the added match arms are not reached. That is worth
    keeping as a general fact about §3.2a's growth contract: widening an arm is
    free at the measurement, and the cost of a phase like this is its *cards*.
    It also means a middle-arm timing delta with identical counters is spread
    and must be read as such — RD-3's was +4.8%, from one slow round out of
    three, with round 1 at +0.1%.

33. **The `--require` reachability instrument counts casts, and an activated
    ability's *activations* are invisible to it.** RD-3 predicted Circle of
    Protection: Red would be rare because "its activation competes for mana the
    random agent rarely has"; the `--require` row says it resolves in 130 of
    200 stress games, which answers a different question. The activation count
    had to come from `--dump-events` and a grep: **12,660 activations across
    200 games, in 129 of them**, about 98 per game the Circle is on the
    battlefield — a `{1}` with nothing else to spend on is something a random
    agent does until it runs out. Two things follow. The prediction is struck.
    And **a phase whose consumer is an activated ability should measure the
    activation, not the cast** — the report has no counter for it, and until it
    does the recipe is one `--dump-events` run and
    `grep "AbilityActivated: <name>"`.

34. **Asked at the RD-3 review: should CR 616.1's prompt be skipped by
    *simulating* both orders — `Lookahead`, compare the states, suppress if
    they agree — rather than by a static predicate? No, and the case that
    prompted the question is the reason why.**

    The case is two Guardian Seraphs, both `Amount(PreventUpTo(1))`,
    `Uses::Static`, no rider — the shape of item 29's multiplier bucket, and
    the arithmetic does commute: `max(a − p − q, 0)` whichever applies first.
    `ordering_cannot_change_outcome` does not admit it, and that is correct
    rather than an omission.

    **Three reasons, in increasing order of how hard they are to work around.**

    - **A rewrite is not a pure function of the event, so "simulate it" is not
      free of consequences.** `apply_rewrite` takes `&mut GameState` because
      CR 614.13 is the rules' own statement that applying an entry replacement
      *moves other objects* — `EnterAfterMoving` performs auxiliary zone
      changes through `execute_actions_new_batch` and prompts a player for the
      set. An ordering containing one cannot be speculatively executed and
      rolled back without the `DecisionProvider` having been asked a question
      that did not happen. `Lookahead`/`EntryFrame` is a *characteristics*
      look-ahead — it answers "what would this permanent be" — and it is not a
      board fork; the fork-and-search question (`codebase-state.md` item 44)
      is a different and much larger piece of work.
    - **Board equality is the wrong equivalence.** CR 616.1 gives the choice to
      a player, so suppressing it is sound only when *no rules-legal question*
      distinguishes the orders — and the event log is such a question.
      CR 615.13 triggers "each time a prevention effect is applied to one or
      more simultaneous damage events and prevents some or all of that damage",
      which counts *applications*, not life totals. Two orders that reach the
      same board by applying a prevention once versus twice are different games
      the moment item 6 lands. A `GameState` comparison cannot see that; a
      comparison that included the log would be comparing the thing the
      shortcut exists to avoid producing.
    - **And `PreventUpTo` is exactly that case.** Take `a = 2` against
      `PreventUpTo(5)` and `PreventUpTo(1)`. Apply the 5 first: the event is
      emptied, CR 614.7a drops it on the next iteration, and the 1 is never
      gathered — **one** application. Apply the 1 first: 2 → 1, then the 5 →
      0 — **two**. Same board, different number of CR 615.13 events. Deciding
      which case a given board is in requires the **amount**, and
      `ordering_cannot_change_outcome`'s signature is `(&[Candidate],
      Option<ObjectId>)` — `Candidate` carries the instance and the member
      *indices*, never their events. Reading the amount is precisely what
      `codebase-state.md` item 47's condition (d) forbids, so the predicate
      **structurally cannot** admit this arm. Two Guardian Seraphs keep their
      prompt, and the prompt is real.

    **The frequency says this was never a hot-path question.** Two Guardian
    Seraphs share a battlefield in **19 of 200 games with the card forced into
    every deck** (10%; four at once at the extreme, measured 2026-09-09). A
    simulator would run on every multi-candidate prompt to remove a prompt that
    rare.

    **What is available, if a prompt ever does need removing:** another clause
    on the static predicate, arriving the way item 29's did — with the rule
    number that makes the orders indistinguishable, and with the event log
    counted among the things that must agree. The bar is a proof, not a
    comparison.

### Found by RD-4 — redirection and unpreventable damage (2026-09-09)

35. **A group's subject stops being its members' the moment a redirect
    applies, and three questions were reading the wrong one.**
    `apply_replacements` captured one `EventSubject` at entry — the key
    `execute_batch_inner` grouped by — and passed it to `apply_rewrite`, to the
    CR 616.1 prompt and into every queued `Rider`. That was exactly right while
    no rewrite could move a member's subject, which is every rewrite RB, RC and
    RD-1 through RD-3 shipped. CR 614.9 moves it.

    **The failure is loud rather than subtle, which is the only good news in
    it.** Redirect damage from a player onto a creature carrying a shield
    counter: the next iteration gathers CR 122.1c's replacement half against the
    *rewritten* event, correctly, and its `Instead(RemoveCountersFromAffected)`
    then asks the group key which object to take a counter from — and the key is
    a player. `subject_object` answers `None` and the arm returns `Err`, failing
    the whole batch mid-performance.

    **The fix is to read the event, which is what CR 616.1f already says.**
    `subject` is re-derived per iteration from the first live member and
    per *member* at application; the rider takes the subject of the first member
    the application touched, read before its own rewrite, because CR 615.5's
    "that much" is about the event the effect replaced. The group key keeps its
    one job — one applied set, one chooser, one loop — and stops being consulted
    about objects. `a_redirect_hands_the_event_to_the_destinations_own_shield_counter`
    is the regression.

36. **`Rewrite::Retarget` makes `codebase-state.md` item 25 reachable for the
    first time, and it is still not worth building.** Item 25 — CR 101.4d's
    APNAP restart, a nonactive player's choice forcing an earlier player to
    choose again — has been unreachable since RB because nothing in phase 1
    could change *who* chooses about a member. A redirect can: an opponent's
    Pariah enchanting **your** creature moves damage aimed at them onto an
    object you control, so the CR 616.1 chooser for that member becomes you,
    and your own group may already have run to completion.

    **Reachable is not the same as reached.** It needs a two-sided board — one
    batch, two subjects with different choosers, and a redirect that crosses
    between them — and no card in either pool builds one, because both printed
    redirects here are `PlayerSet::You` scoped and Pariah on an opponent's
    creature is a play a random agent makes only by accident. The sizing is
    unchanged and still the one item 25 records: turning phase 1 inside out,
    a per-member state machine under an outer APNAP round-robin. **Revisit at
    Phase 6**, where triggers make the second half of CR 101.4 live anyway —
    not before, and no longer "at RD".

37. **The gate that decides an arm ships fired against `RetargetSpec`, and
    caught the arm §9 named.** `AmountRewrite`'s rule — every arm ships with a
    printed customer in the PR that lands it — admitted `ToEffectSource`,
    `ToHost` and `ToDamageSourceController` and refused `ToFixed(DamageTarget)`.
    The refusal is not "no card wants it": Harm's Way and Divine Deflection both
    do. It is that **no card can author it**, because a damage target is chosen
    at cast and a card file cannot name one — so the arm's only possible filler
    is `RegisteredReplacementEffect.targets`, threaded onto the instance, which
    is `codebase-state.md` item 90's work and arrives with its own card. An arm
    whose customer cannot reach it is worse than a missing one for exactly the
    reason §3.2's growth contract gives, and this is the first time that rule
    has excluded something a *design doc* wrote down rather than something a
    build wanted.

38. **`ObjectFilter::EachOther` was refused in an affected set, and the
    instrument that measured it as harmless measured the wrong thing.**
    `codebase-state.md` item 103 ran the three causes of
    `object_matches_filter`'s swallowed `Err` over 600 fuzz games, found zero,
    and left them on the argument that a mid-game panic is worse than a card
    doing nothing. Palisade Giant made one of the three live on its first
    board.

    **The measurement was sound and the inference was not.** Zero reachability
    over a pool that contains no card using a leaf says nothing about the leaf;
    it says the pool does not use it. The two remaining causes — an id with no
    object behind it, and `PowerLE` against something with no power — are
    genuinely card-authoring errors, and item 103's argument still holds for
    them. `EachOther` never belonged in that list: the layer walk has answered
    it off `FilterPlayers::source` since the layer system, `set_affects` has
    carried the same `source` since RB, and the two simply were not connected.
    **The general lesson, worth more than the fix:** a reachability zero is
    evidence about the *pool*, and it can only retire a concern that the pool
    could have exercised.

### Found by the RD-4 review (2026-09-09)

39. **Two of RD-4's three "missing destination" tests were the same branch, and
    the `COVERS-PARTIAL` on one of them claimed a leg no test reached.** Asked
    on review whether an unattached Aura is a real board. It is — `change_zone`
    detaches every attachment when a permanent leaves and **leaves the Aura on
    the battlefield** for CR 704.5m to find, so a single resolution that
    destroys a creature and then damages its controller reaches it. But that
    means `pariah_whose_host_has_left_the_battlefield_does_nothing` exercises
    `retarget_destination` answering `None`, not `redirection_is_legal`'s
    "no longer on the battlefield" — the same branch as the
    attached-to-nothing test beside it. Verified by probing `attached_to` after
    the host's zone change: `None`.

    **CR 614.9's first clause needs a destination that still exists and is
    still named**, and an Aura structurally cannot supply one. A **registry
    row** can: it keeps the `source` it was created with and CR 608.2c's
    duration outlives the permanent, so a resolution-created `ToEffectSource`
    redirect whose source has died still names it.
    `a_registry_row_whose_source_has_left_the_battlefield_redirects_nothing`
    is that board, and the `COVERS-PARTIAL` moved onto it. Mutation-checked:
    replacing the `contains_key` guard with `true` fails it.

40. **Nothing tested that `unpreventable` survives a redirect, and "the field
    is copied in one line" is not a reason it did not need to.** CR 614.9 moves
    "the same damage", so the flag travels with `is_combat` — and `is_combat`
    had a test (Pariah's first ruling) while the flag did not. A `Retarget` arm
    that forgot `unpreventable` left **every other test in the file green**;
    measured, not assumed, by setting it to `false` in the arm and running the
    suite. `a_redirect_carries_the_unpreventable_flag_onto_the_destination` is
    the twin, and it is the only test the mutation fails.

41. **`Restriction::ApplyReplacement`'s `to` was half a pair wearing the name of
    the whole thing.** `{ kind, to, to_players }` reads as an affected set with
    a player modifier hung off it; the two are unioned and neither is primary.
    Renamed `to_objects` — 12 sites, no behaviour. `ReplacementDef`'s
    `affected`/`affected_players` keeps its own names, where `affected` reads
    as the generic noun rather than as a preposition, and the asymmetry there is
    the older one.

### Found by RE's sizing (2026-09-11)

42. **§3.2d's Notion Thief encoding contradicts the card's ruling, and the
    lineage rule is what catches it.** "That player skips that draw and you
    draw a card" was filed as `Prevent` + `then`, the heterogeneous case.
    Its ruling walks two Thieves and says each is "applied to the card draw
    only once" and that in a two-player game "it really will be that player
    who draws" — which a rider's fresh proposal cannot deliver, since two
    Thieves would trade the draw forever. The Thief's draw is the same event
    with a new subject, `Instead(DrawCards { n: 1, player: Some(You) })`,
    and §3.2d is corrected in place. Alms Collector stays a rider. Worth
    keeping because it is the second time (after Furnace of Rath's) that a
    printed ruling decided a rewrite's *arm* rather than its numbers — the
    rulings pass is doing design work, not just test work. → RE-2.

    **Landed 2026-09-11, and it has a twin.** The encoding shipped as written
    and `two_notion_thieves_hand_the_draw_across_the_table_and_back` is the
    two-player board the ruling's last sentence names. What this item did not
    see is that Alms Collector is the *same* mis-filing — item 53 — and that
    the rule underneath both is CR 614.5's "any modified events that may replace
    that event" rather than anything about the word "instead". §3.2d now states
    it as a rule about where to split a printed "instead X and Y".

43. **Mana production is a chokepoint violation, and RA's census could not
    have seen it.** `resolve_mana_effect` (`mana.rs:91`) and
    `Primitive::ProduceMana` (`resolve.rs:337`) both write the pool directly,
    and `GameEvent::ManaAdded` — in the enum since the log was written — is
    emitted at zero sites. RA's audit walked *emissions* (§6's table was built
    from `events.emit` sites), so a mutation that emitted nothing was invisible
    to it, the mirror of the lifelink finding, which emitted without proposing.
    Two printed replacements (Mana Reflection, Nyxbloom Ancient) and item 6's
    "whenever you tap a land for mana" family read the event. → RE-9, and
    `codebase-state.md` item 111.

44. **A lost player keeps taking turns and receiving priority in any game of
    three or more, and `has_drawn_from_empty_library` never clears.**
    `advance_turn` takes `(active_player + 1) % num_players` and the priority
    loop rotates the same way; nothing reads `player_lost` on either path.
    `fuzz_games` plays two, where a loss ends the game in the same sweep, so it
    has never shown. This is exactly the two-player assumption `CLAUDE.md`
    said `PlayerLoses` would hide, found by looking where it said. The flag is
    the same shape one rule over: CR 704.5b's window is "since the last time
    state-based actions were checked", and Exquisite Archangel's ruling ("you
    won't lose again until you try to draw again") is what a never-cleared
    flag gets wrong. → RE-6; `codebase-state.md` items 112 and 113 — and
    the objects a departed player leaves behind (item 108) are RE-7's, the PR
    after, since the day RE-6's four-player mode lands they are reachable and
    wrong.

45. **`Restriction::Event` carries no `PlayerSet`, so four printed "can't"
    families have no row shape.** "Players can't gain life" (25 cards),
    "can't lose life" (2), "you can't lose the game" (11), "your opponents
    can't win the game" (9) all scope a prohibition to players, and
    `Restriction::Event { pattern, affected, by }` has only the object set.
    RD-1 added `affected_players` to `ReplacementDef` and RD-4 added
    `to_players` to `ApplyReplacement`; this is the third instance of the same
    field and the last type without it. → RE-3 (Skullcrack), read by RE-6
    (Platinum Angel); `codebase-state.md` item 114; closes item 26.

46. **This document held two answers on skips for twelve days.** §11 item 6
    (2026-08-30) said skips go through the pipeline with the proposal built by
    the turn machinery; §9's RE paragraph (older) said a per-player
    `pending_skips` counter consulted at step begin. Nobody read them against
    each other until RE was sized, and the paragraph was the one a builder
    would have started from. Same for §8a's discard row, which said the
    pattern arm did not exist for sixteen days after RB built it. The rule
    that follows is `state-of-play.md`'s reason generalised: **a scope
    paragraph for an unsized phase goes stale in the merge that builds part of
    it, and re-reading it is the first step of sizing, not a courtesy** — the
    prompt for this sizing said so, and it was right twice. The review the
    same afternoon found a third: the first cut sent discard's producer to
    Phase 8 on §8a's sentence while RD-1 had built `Primitive::Mill` inside
    a replacement PR for the same reason — a precedent seventeen days old
    that a re-read of §9's own RD-1 section would have surfaced.

### Found by building RE-1 (2026-09-11)

47. **A turn queue needs a second field, and two players hide it.** The
    sizing asked whether the queue holds the player or the `(player, turn)`
    pair and got the right answer (the player). It did not ask how the
    *natural* rotation survives an extra turn, and `(active_player + 1) % n`
    does not: CR 500.7 inserts an extra turn **after** a turn, so the rotation
    resumes from the player whose natural turn it was. Every printed extra turn
    in RE-1's reach says "you take an extra turn", and on two players with a
    sorcery that is the same answer by accident — which is why nothing in the
    sizing caught it. Final Fortune is an *instant*, so a four-player table has
    P1 taking an extra turn during P0's and then their own natural one, and the
    arithmetic skips the second. `GameState.turn_rotation` is the field, and it
    advances when a natural turn is **proposed** rather than when one begins,
    because CR 614.10a proceeds past a skipped turn rather than re-offering it.

    **And a printed ruling says the field out loud** (Timesifter, fetched at
    the review on 2026-09-11): *"Remember which player would have taken the
    next turn if Timesifter's ability hadn't triggered the first time. After
    Timesifter leaves the battlefield and all extra turns have been taken, that
    player takes the next turn."* That is `turn_rotation`'s whole
    specification, written by the people who adjudicate it, on the card
    notorious for the deepest queue in the format — *"with two Timesifters on
    the battlefield, two extra turns are created for each turn taken"*, in a
    four-player game. **Third time a rulings pass has decided a design rather
    than a test** (after Furnace of Rath's numbers and Notion Thief's arm, item
    42), and the first where the ruling arrived *after* the code and confirmed
    it. Timesifter itself waits on item 6's triggers; the shape is tested
    without it — `phase_re1_integration_test::
    a_deep_queue_drains_most_recent_first_and_leaves_the_rotation_where_it_was`.

    Worth keeping because it is `CLAUDE.md`'s "write new systems N-player-shaped
    from the start" biting a *queue* rather than a player set — the shape was
    right and the cursor it moved was wrong. → RE-1, `codebase-state.md`
    item 115.

48. **The game's first turn had never begun, and no test could see it.**
    `GameState::new` wrote turn 1, player 0 and the beginning phase's untap
    step, and nothing ever called `on_step_begin` for that step — so turn 1's
    untap sweep and land-drop reset had not run in any game the engine has
    played. It is invisible because the untap step of an empty battlefield does
    nothing and `lands_played_this_turn` is already zero, and it was about to
    become visible for a much worse reason: item 6's 2,656 "at the beginning
    of" triggers would have read every turn's events except the first's.
    `Game::setup` now calls `start_first_turn`. **The class is the finding**:
    a constructor that writes a *state* the engine elsewhere reaches through a
    *transition* is a missing event, and RA's census could not have seen this
    one either, because it walked emissions and this site emitted nothing —
    the same blind spot as finding 43's mana pool. → RE-1,
    `codebase-state.md` item 116.

49. **"`advance_turn` is written as a turn queue so `backlog.md` §2.17 does not
    rewrite it a second time" is an overclaim, and it covers the turn level
    only.** Decision 6's own sentence, and the argument that pulled CR 500.7
    into a skips PR. It holds for extra *turns*: the queue is a list beside a
    cursor and the cursor never had to change shape for it. It does not hold
    for CR 500.8's extra **phases**, and the reason is the cursor RE-1 wrote.
    `next_turn_unit` answers "what follows" from `(Option<PhaseType>,
    Option<StepType>, phase_began)` — a phase **type** — so a turn holding two
    combat phases cannot say which one the cursor is at. The shape that can is
    the per-turn `TurnPlan` that `state::game_state::next_phase`'s pre-RE-1
    TODO already described, indexed rather than chained; RE-1 rewrote that TODO
    into a pointer instead of building it. So the function *is* rewritten
    twice, and the second time is a cursor change rather than an addition.

    **Raised by the review, on the fixture** that
    `a_phase_skip_cast_during_combat_is_spent_on_the_next_combat_phase` needs:
    a registered card (Moment of Silence) has a ruling with no engine-produced
    board. Three things bound the cost of having waited, and all three were
    checked rather than assumed: `advance_turn` is named in **no other RE PR's
    sizing** (RE-6's line for it was CR 800.4j/k, which RE-1 spent), the corpus
    files 500.8–500.10 as **DEFERRED with no atom ids**, so no phase's exit
    criteria move, and CR 500.10 needs item 6's triggers whatever happens to
    500.8 (Obeka is the only card for it). So waiting costs one extra rewrite
    and no interest — which is what makes this an **ordering call rather than a
    debt**. **Called 2026-09-11 by the owner: it comes into RE as RE-10**, on
    the argument that the sentence above is unkept and that 46 cards plus a
    registered card's fixture-only ruling is more pressure than several of RE's
    nine carry. §9's RE-10 section is the design; `backlog.md` §2.17 graduates
    with it.

    Worth keeping for the general shape: **a "we built it once so nobody
    rewrites it" claim is only as wide as the cursor it is made about.** RE-1's
    was made about a queue and is true of the queue; the sentence did not say
    which level it covered, and nobody read it against the level below until
    the fixture forced it.

### Found by building RE-2 (2026-09-11)

50. **`GameState::draw_cards` has had no caller since before RA and is a draw
    path with no proposal.** A `pub fn draw_cards(player, count, ctx)` that loops
    `draw_card` — the *performer* — so an N-card draw through it would make N
    library-to-hand moves with no `DrawCard` event, no CR 121.2 instruction and
    no lineage. It was correct-and-unused while `Primitive::DrawCards` looped
    `execute_action` itself; after RE-2 it is a second spelling of the outer
    performer that skips the pipeline entirely. The census counted `DrawCard`'s
    *producers* and this is not one — it produces no action at all, which is
    exactly the shape RA's emission-walk audit missed for mana (item 43) and
    §8a's table missed for discard. **Deleted rather than routed**, because a
    plural helper beside a plural `GameAction` is the ambiguity, not the
    convenience: `Primitive::DrawCards` is the one way to ask for N draws.

51. **`Game::setup`'s opening-hand comment claims a route the code does not
    take, and has since RA-2.** The comment reads "CR 103.4 calls this drawing,
    so it goes through the chokepoint like any other draw", and RA-2's commit
    message says routing the opening hands was a deliberate deviation from the
    plan. The diff threaded an `ActionContext` and left the call at
    `state.draw_card(..)` — the performer. What *is* true is the comment's
    second half and the reason RE-2 leaves the call alone: the battlefield is
    empty and the registry has no rows, so no replacement can be gathered and no
    choice can arise (CR 103.6 puts Leylines after this point). The correction
    is one comment, and it is worth making now because "like any other draw"
    stops meaning "reaches `execute_action`" and starts meaning "proposes a
    `DrawCards` outer that decomposes" — a claim the reader can check and find
    false. **The general shape: a comment that names a mechanism ages with the
    mechanism**, and this one was written about a route the same commit chose
    not to take.

52. **CR 121.2c orders two players' draws and nothing can express it.** "If more
    than one player is instructed to draw cards, the active player performs all
    of their draws first, then each other player in turn order does the same."
    Alms Collector's rider is the first thing in the crate that instructs two
    players to draw from one effect — `Effect::Sequence([draw for you, draw for
    that player])` — and a `Sequence` resolves in text order, so when the
    affected opponent is the active player the engine draws in the wrong order.
    Observable in the event log today and in gameplay the day item 6's
    "whenever you draw" triggers land. **Not built here**: the facility is
    APNAP ordering over *an effect's recipients*, which no `Effect` arm carries
    and which CR 121.2d (shared team turns) extends; one rider is one customer
    and the rule wants two. `codebase-state.md` has the line and the sizing.
    → RE-6, which is where a lost player stops being in turn order at all.

53. **Alms Collector as `Prevent` plus riders is an infinite loop, and its own
    ruling says so.** §9's RE decision 1 kept it as the heterogeneous case on
    §3.2d's rule — "you and that player each draw a card" is two unlike things —
    and that filing is what a rider's fresh applied set punishes: an opponent's
    Thought Reflection doubles the rider's one draw back to two, Alms applies
    again, and the two effects trade cards forever. Found by
    `alms_collector_does_not_apply_again_to_the_draws_it_produced`, which
    overflowed the stack the first time it ran; the ruling that decides it is
    the same one the test is named for.

    **Third time a printed ruling has decided a rewrite's arm rather than its
    numbers** — Furnace of Rath's, then Notion Thief's (item 42), now this —
    and the first time one has decided it *against* a correction made three
    hours earlier. Worth the entry for what the two Thief/Collector corrections
    share once they are side by side: the misreading is not about "instead"
    versus "and", it is about assuming `then` is where the second clause goes.
    CR 614.5's "any modified events that may replace that event" is the test,
    and §3.2d now states it as a rule about where to split rather than as a
    description of two cards.

    **And the failure mode is worth a line of its own.** Both mis-filings
    produce a loop rather than a wrong number, both loops are the kind CR 104.4b
    calls a draw, and CR 731 loop detection is §12's — explicitly out of scope.
    So an encoding error in this corner is a crash the engine cannot diagnose
    for itself, which is the argument for the bound
    `test_two_thought_reflections_draw_four_not_infinity` carries and for
    putting one on every board in RE-2's file that can hold two of anything.

54. **A three-round median said +6.0% and the number was noise.** RE-2's middle
    arm is the engine with the pool unchanged, and `fuzz_ab.py`'s default three
    timing rounds put `main` at 14.78 / 15.97 / 20.32 ms — a 27% outlier inside
    one arm, which makes the median 15.97 and the middle arm's tight 16.67–17.05
    look like a 6% regression. Seven rounds put `main` at 15.74–16.62 and the
    delta at **+0.5%**, which is what +34 gathers a game should cost beside
    RE-1's +5.0% for +447.

    **The rule, and it is cheap:** when one arm's rounds straddle another arm's,
    raise `--rounds` before writing the delta down. `fuzz_ab.py --rounds 7
    --no-fixtures` re-runs the timing block alone in about 70 seconds, against
    35 for the whole default sitting. Worth an item rather than a footnote
    because the number would have been *published* — §8's whole discipline is
    recording measured CPU deltas, and a phase that records a wrong one poisons
    every later phase that reads it. RE-9's decision is scheduled to turn on a
    2.5-point gate, and 6 points of noise is more than that gate is worth.

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
