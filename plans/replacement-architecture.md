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
    Fizzled,         // 608.2b — countered by game rules

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

### 3.4 `ReplacementRegistry`

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
                  .filter(applies_to(ev))                # EventPattern + AffectedSet
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
- **CR 614.6/614.7** — `None` drops the event, and an event that was never
  proposed never reaches this function (CR 614.7a's zero damage is already
  short-circuited in `perform_action`).

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

CR 614.17: **"can't" effects follow similar rules but are not replacement
effects.** They are checked before the pipeline and they win (CR 101.2). Two
consequences for this design:

- Indestructible (CR 701.8a) is a "can't", not a replacement. The check that
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
  concrete state in five places, audited 2026-08-24: four battlefield reads
  (`compute.rs:113` the `control_since_turn` seed, `:180` the counter entry,
  `:408` `base_controller`, `:623` `effect_applies_to`'s membership gate) and
  one registry read (`:242`, `effects_in_layer`). Route those through one
  accessor pair that a caller can perturb. A `GameState` clone would work here on
  budget grounds — §11 item 5 prices both call sites — but is the wrong
  instrument: it duplicates the object store and `GameState.rng`, the latter
  against the determinism doctrine outright, and it produces a second live copy
  of every `ObjectId`, which is a v4 UUID and therefore *aliased* rather than
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
Devotion is not implemented yet (no `AmountExpr` counts mana symbols), so today
this costs one line and a test; discovered later it would be a silent wrong
answer on 15 gods plus every `CountOf`-driven ETB replacement.

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
battlefield"). Write `test_grist_entering_is_not_a_creature` first: it isolates
the clause with no devotion arithmetic in the way.

**It also bounds the risk this section opened with.** The gods looked like
evidence the look-ahead is unboundedly hairy. What they actually produced is a
single sentence with a table behind it and a test:
`test_god_entering_does_not_count_itself_for_devotion`. That is the shape to
insist on for the rest of RC-B — an unintuitive ruling that reduces to a
mechanical rule is fine; one that does not is a signal to stop and re-read the
CR before writing code.

### 5b. Three corrections from a judge-corpus pass (2026-08-26)

Five card interactions were put to this design by the owner. Two confirm it, one
moves the RC Part A/B seam, and two add rules it did not state.

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
which is about CR 614.5 and stays per-event.

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
copy-on-enter in scope — re-check it when Layer 1 lands and RC-B fills the
CR 616.1c bucket.

### 5c. Dress Down moves the Part A/B seam

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

**Why this is not just another RC-B card.** §5 split RC on the claim that Part A
handles "ETB replacements whose applicability does not depend on the frame —
`AffectedSet::SourceOnly`, unconditional… 'this land enters tapped'". That claim
is false, and Dress Down is the proof: whether the entering land *has* its
enters-tapped ability is itself a frame question. The design collapses two
questions into one:

1. **Does the entering object still have its own replacement ability?** Answered
   by the frame's Layer 6 output under clause (3). Prior to everything else.
2. **Which replacements apply to this event?** §5's clauses, the question the
   overlay was designed for.

Question 1 needs exactly the piece Part B was going to build: the membership gate
at `compute.rs:622`, which returns `false` for any `AffectedSet::Filter` effect
against an object not in `game.battlefield` — so today an entering Clone matches
no filter, Dress Down included, and keeps its ability.

**The seam moves, the split survives.** Part A takes the membership gate and the
frame's ability list — enough to ask "what abilities would this object have on the
battlefield", which is the whole of question 1. Part B keeps clause (1)'s pending
`EnterMods`, the filter-based *applicability* of other permanents' replacements
(Orb of Dreams), and the 614.13a/b exclusion sets. Part A is still the smaller,
lower-risk half; it is just not overlay-free, and shipping it overlay-free would
enter a Dress Downed Clone as a copy.

**De-risking split.** RC Part A implements ETB replacements whose applicability
does not depend on the frame — `AffectedSet::SourceOnly`, unconditional. That is
"this land enters tapped" and "this enters with N +1/+1 counters", which is the
overwhelming bulk of the 773 + 580, **plus the membership gate and the frame's
ability list, per §5c**. RC Part B builds the rest of the overlay and turns on
filter-based ETB replacements (Orb of Dreams, Blood Moon interactions) and the
614.13a/b exclusion sets. Same Part A/B shape as Phase LD.

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
| `engine/layers/compute.rs` | battlefield reads go through one accessor (overlay seam) | RC-B |
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
"this permanent", regeneration is `SourceOnly`, and RC Part A is `SourceOnly`
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

#### RA-1 — the plumbing (tickets 1–2)

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

#### RA-2 — routing the silent sites (tickets 3–5, 10–12)

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
2. `ReplacementRegistry` with duration expiry.
3. `apply_replacements` — the §4.1 loop, including 616.1a–f, 614.5, 616.1g
   recursion, 614.17c's blocked-event path, and APNAP.
4. `GameAction::Destroy`; `Primitive::Destroy` lowers to it; indestructible
   moves to the "can't" check (CR 701.8a/614.17).
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

- **`EventPattern` ships five arms, not one per `GameAction` variant.**
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

- **Part A:** `GameAction::EnterBattlefield { mods: EnterMods }`;
  `place_on_battlefield` becomes its performer; `AffectedSet::SourceOnly`
  unconditional ETB replacements — enters tapped (CR 110.5b), enters with
  counters (CR 122.6a), CR 614.12a choice-before-entry. ~1,350 cards.
  **Plus the membership gate and the frame's ability list (§5c)** — without them
  a Dress Downed Clone still copies, and an enters-tapped land still enters
  tapped after losing the ability that says so.
  **Ride along: delete the early stack pop** (`codebase-state.md` Deferred
  Migrations 7). Part A rewrites `init_zone_state`, which is one of
  `GameState::resolving`'s two readers; removing the pop deletes the other.
  Audit the five production `stack.is_empty()` readers first — none is reachable
  during a resolution today, but CR 608.2g's "unless an effect instructs" case
  makes `cast.rs:576` reachable once RC-era cards arrive.
- **Part B:** the §5 hypothetical overlay; CR 614.12 clauses (2) and (3);
  CR 614.17d; CR 614.13/613a/b auxiliary zone changes and exclusion sets;
  CR 616.1b/c classes (control-changing and copy-on-enter get their buckets even
  though the copy system itself is Layer 1 work).

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
step/phase/turn begin), token and counter doublers (614.16), life-gain
replacement (119.10), mana replacement (106.6a), CR 608.3e (permanent spell
whose controller can't put it onto the battlefield), and the three kinds §8a's
audit added: `PlayerLoses` / `PlayerWins` (CR 104, 6 cards) and the discard
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
| 3 | Self-replacement (CR 614.15) plumbing | **Deferred past RB, as planned.** RB gave `SelfReplacement` its CR 616.1a bucket and no producer; `ResolutionContext` still has three fields. The field lands with the first card that needs it |
| 4 | Replacement effects outside the battlefield | **Still open — RB wrote the sweep but did not size it.** Now that `gather` exists the shape question is answerable: a zone parameter on one loop plus a timestamp on `GameObject`. The card count is still owed |
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
   in `layers-architecture.md` §9/§15.2 when RC-B lands, and measure the RC-B
   overlay with `fuzz_games --games 200 --seed 12345` against the pre-RC-B
   baseline — a look-ahead that runs a few times per turn should not move the
   number, and if it does, the accessor indirection has leaked into the hot walk
   and that is the bug to find.

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

---

## 12. Explicitly out of scope

- **Layer 1 / the copy system (CR 707).** 23 Phase-6 atoms, a separate system.
  CR 616.1c gets its ordering *bucket* in RC-B so the classification is complete,
  but nothing produces a copy-on-enter replacement until Layer 1 lands.
- **CR 614.14 / 607 linked abilities.** Needs the CR 607 work (T20).
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
- `layers-architecture.md` §9 / §15.2 item 3 — the overlay decision, once made.
  **Untouched by RA**, correctly: RA-3's LKI frame is a `compute_characteristics`
  call taken *before* a mutation, not a hypothetical about a perturbed board, so
  it needed neither the accessor pair nor a clone. The overlay is still RC-B's,
  and §11 item 5's decision still stands unrecorded there.
- `cards-unlocked-ledger.md` — ✅ RB's entry is in. The ETB unlock is the largest
  single entry the ledger will take; add it with RC.
