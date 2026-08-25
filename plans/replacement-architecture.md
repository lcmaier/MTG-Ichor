# Replacement & Prevention Effects — CR 614–616

> **Status:** design, authored 2026-08-24. No code written yet.
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
algebra — `plans/references/replacement-census.py` re-runs both passes and the
answer is a number.

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
    Resolved,        // 608.2m — stack → battlefield or graveyard
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
    /// CR 122.1c/d/h -- backed by counters on a permanent. Applying removes one
    /// counter; the effect exists while at least one remains. Note the CR's
    /// wording: one *or more* counters create **a single** replacement effect,
    /// so two shield counters do not give two applications to one event.
    CounterBacked(CounterType),
}
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
python plans/references/replacement-census.py clauses
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

`Rewrite::apply` still needs `&mut GameState` and a `DecisionProvider`, because
`then` resolves against live state — but that is the existing `resolve_effect`
path, not new machinery.

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
fn apply_replacements(game, action, ctx, inherited) -> Option<GameAction>
    applied: HashSet<ReplacementInstanceId> = inherited   # 3.2d lineage
    ev = action

    loop:                                                # CR 616.1f
        cands = gather(game, ev)                         # 3.3, five sources
                  .filter(applies_to(ev))                # EventPattern + AffectedSet
                  .filter(|c| c.exempt_from_614_5        # CR 903.9b
                              || !applied.contains(c.instance))   # CR 614.5
        if cands.is_empty(): return Some(ev)

        bucket  = forced_bucket(cands)                   # CR 616.1a -> b -> c -> d -> e
        chooser = affected_chooser(game, ev)             # CR 616.1 / 400.6
        chosen  = if bucket.len() == 1 { bucket[0] }
                  else { ask_choose_replacement(dp, chooser, bucket) }

        if chosen.optional && !ask_apply(dp, chooser, chosen):
            applied.insert(chosen.instance)              # opportunity taken (CR 614.5)
            continue                                     # ... but no `consume_use`
        if !chosen.exempt_from_614_5: applied.insert(chosen.instance)
        chosen.consume_use(game)                         # Uses::Once / Shield / CounterBacked

        match chosen.apply(game, ev, ctx)?:              # Rewrite + `then`
            None     => return None                      # CR 614.6 -- never happens
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

**`ask_choose_replacement` is called only when `bucket.len() >= 2`.** That is
CR-correct (there is no choice to make with one candidate), it is what keeps the
existing `ScriptedDecisionProvider` tests green rather than drowning them in
unexpected prompts, and it is what loop-detection Tier 1 counts as "forced".

### 4.2 Batches and simultaneity

Three rules need an event *set*, not an event:

- **CR 704.3** — "performs all applicable state-based actions **simultaneously
  as a single event**". `engine/sba.rs` currently performs them one
  `change_zone` at a time.
- **CR 704.7** — "if multiple state-based actions would have the same result at
  the same time, a single replacement effect will replace all of them."
  Unreachable without the batch.
- **CR 615.7** — "if damage would be dealt … by two or more applicable sources
  at the same time, the … controller chooses which damage the shield prevents."
  `apply_combat_damage` currently loops one assignment at a time, so this choice
  cannot exist.

So `execute_action` gains a batch sibling:

```rust
pub fn execute_actions(&mut self, batch: Vec<GameAction>, ctx: &ActionContext)
    -> Result<Vec<GameAction>, String>;
```

The batch shares one `applied` set and one batch id (which lands on every
emitted `GameEvent`, satisfying the CR 603.2c/603.6a "one or more" trigger
requirement that Phase 7 will need). `execute_action` becomes
`execute_actions(vec![action])`.

Callers that must batch: `apply_combat_damage` (CR 510.2), the SBA sweep
(704.3), and any "each player …" effect (CR 101.4).

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

**But "as a permanent" is narrower than "on the battlefield", and getting that
boundary wrong is the failure this section exists to prevent.** See §5a, which
was written after review flagged the Theros gods.

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

**It also bounds the risk this section opened with.** The gods looked like
evidence the look-ahead is unboundedly hairy. What they actually produced is a
single sentence with a table behind it and a test:
`test_god_entering_does_not_count_itself_for_devotion`. That is the shape to
insist on for the rest of RC-B — an unintuitive ruling that reduces to a
mechanical rule is fine; one that does not is a signal to stop and re-read the
CR before writing code.

**De-risking split.** RC Part A implements ETB replacements whose applicability
does not depend on the frame — `AffectedSet::SourceOnly`, unconditional. That is
"this land enters tapped" and "this enters with N +1/+1 counters", which is the
overwhelming bulk of the 773 + 580. RC Part B builds the overlay and turns on
filter-based ETB replacements (Orb of Dreams, Blood Moon interactions) and the
614.13a/b exclusion sets. Same Part A/B shape as Phase LD.

---

## 6. Engine interaction points

| Site | Change | Phase |
|---|---|---|
| `engine/actions.rs::execute_action` | the pipeline call; gains `ActionContext`; batch sibling | RA/RB |
| `engine/actions.rs::change_zone` | gains `cause` and `ActionContext` | RA |
| `engine/zones.rs::move_object` | performer only; emission moves here | RA |
| `engine/zones.rs::draw_card` | routes through `execute_action(DrawCard)`; emits `CardDrawn` | RA |
| `engine/stack.rs` × 3 `// REPLACEMENT-BYPASS:` | pop-aware `ZoneChange` dispatch that skips the stack-Vec removal; also where the LKI frame is captured | RA |
| `engine/turns.rs::process_draw_step` | stop calling `draw_card` directly (CR 614.11, 614.10) | RA |
| `engine/turns.rs::process_untap_step` | route through `Untap`; emit `Untapped` (CR 122.1d) | RA |
| `engine/costs.rs:150,165` | `Cost::Tap`, `{Q}` — emit `Tapped` | RA |
| `engine/combat/steps.rs:70` | attackers tap — emit `Tapped` | RA |
| `engine/combat/resolution.rs::apply_combat_damage` | batch, not a loop (CR 510.2, 615.7) | RA/RD |
| `engine/cast.rs::activate_ability` | emit `AbilityActivated`; resolution emits identity-bearing `AbilityResolved` | RA |
| `state/game_state.rs::StackEntry` | add `cast_from: Zone` — §8c, two customers | RA |
| `engine/sba.rs` | batch sweep (CR 704.3/704.7); `cause` on each move | RA/RB |
| `engine/resolve.rs::Primitive::Destroy` | lowers to `GameAction::Destroy`, not to `ZoneChange` | RB |
| `state/game_state.rs::place_on_battlefield` | becomes the *performer* of an already-replaced `EnterBattlefield` | RC |
| `state/game_state.rs::register_static_effects` | skip `Effect::Replacement` bodies without tripping the loud-lowering assert | RB |
| `engine/layers/compute.rs` | battlefield reads go through one accessor (overlay seam) | RC-B |
| `engine/turns.rs::advance_turn` | gains `dp`; consults pending skips (CR 614.10) | RA / RE |

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
10 core mutations were queried against Scryfall (2026-08-24). Reproduce with:

```bash
python plans/references/replacement-census.py sizing
```

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
RC and RE split into Parts A/B if they run long, as LD did.

### Phase RA — the event spine (no replacement behavior)

The Deferred Migrations item 3 ticket list, verbatim, plus the DP plumbing.
**Whole suite stays green with no behavior change** except newly-emitted events.

1. `ActionContext` threaded through `execute_action` / `change_zone` /
   `advance_turn` / `apply_combat_damage` / the SBA sweep.
2. `ZoneChangeCause` on `ZoneChange`; every caller sets it.
3. Route the draw-step draw through the chokepoint; add `CardDrawn` (CR 121.5 —
   106 cards say "whenever you draw", 54 say "your second card").
4. Tap/untap through the chokepoint with `Tapped`/`Untapped` (CR 603.2e); the
   four silent sites in `costs.rs`, `turns.rs`, `combat/steps.rs`.
5. `AbilityActivated` + identity-bearing `AbilityResolved` (CR 603.7h).
6. Payload upgrades: layer-computed LKI frame on battlefield-leaving zone
   changes (CR 603.10a), `cause`, batch id, resolution context.
7. Close the three `// REPLACEMENT-BYPASS:` sites with the pop-aware dispatch.
8. Demote `CreatureDied` / `PlaneswalkerDied` / `LegendRuleSacrificed` to
   display sugar.
9. `execute_actions` batch form; `apply_combat_damage` and the SBA sweep use it.
10. `StackEntry.cast_from: Zone` — the origin a spell was cast from (§8c). Two
    customers: Don't Blink's "cast from exile", and CR 903.8's commander tax.

**Exit criterion:** every state mutation observable by CR 614 or CR 603 is
emitted from exactly one place, and an event log replay can distinguish drawn
from tutored, destroyed from sacrificed, and countered from resolved.

### Phase RB — the pipeline, with counters and regeneration as consumers

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
7. **CR 704.6d — commander in graveyard/exile → command zone.** See §11 item 1:
   this is an SBA, *not* a replacement, and `check_state_based_actions` already
   takes a DP. It can ship here or earlier.
8. CR 903.9b — commander to hand/library, with `exempt_from_614_5: true`.

### Phase RC — ETB replacements (the big unlock)

- **Part A:** `GameAction::EnterBattlefield { mods: EnterMods }`;
  `place_on_battlefield` becomes its performer; `AffectedSet::SourceOnly`
  unconditional ETB replacements — enters tapped (CR 110.5b), enters with
  counters (CR 122.6a), CR 614.12a choice-before-entry. ~1,350 cards.
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
  seed from the shell; every line must match except the two wall-clock lines.
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
| 3 | Self-replacement (CR 614.15) plumbing | **Defer to RB.** The `ActionContext::resolution` hook lands in RB; the `ResolutionContext` field lands with the first card that needs it. Low breadth, and building a general mechanism first is what §0 commitment one warns against |
| 4 | Replacement effects outside the battlefield | **Defer, but size it in RB.** Per `dont-over-defer`: count the cards during RB rather than at the end of RE, because the answer (zone parameter vs. separate registry) changes the sweep's shape and the sweep is written in RB |
| 5 | Overlay shape | **Answered** — read-side accessor, closed on measurement |
| 6 | Skips are not `execute_action` events | **Answered.** A design note; the work is in RE |
| 7 | `ScriptedDecisionProvider` blast radius | **Answered.** The mitigation is §4.1's two-candidate rule; watch it, do not re-decide it |
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
call sites** to label. An afternoon of reading, no query required.

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

- `codebase-state.md` — the CR 614–616 row, the CR 9 table (item 11.1 above),
  and Deferred Migrations: item 3 closes with RA, item 2's three bypasses close
  with RA, and every new stub gets a line at commit time.
- `CLAUDE.md` — an authority-table row for this file; a new invariant if the
  chokepoint discipline needs one stated (it probably does: *"never mutate
  observable state outside `perform_action`"* is this phase's analogue of the
  layer-system invariant).
- `layers-architecture.md` §9 / §15.2 item 3 — the overlay decision, once made.
- `cards-unlocked-ledger.md` — the ETB unlock is the largest single entry the
  ledger will take; add it with RC.
