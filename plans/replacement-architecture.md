# Replacement & Prevention Effects — CR 614–616

> **Status:** design, authored 2026-08-24. No code written yet.
> **Authority:** type shapes, the pipeline algorithm, the event vocabulary, and
> the phase sequencing for CR 614/615/616. Where this contradicts
> `codebase-state.md`, that file wins on *what exists*; this file wins on *what
> is being built*. `CLAUDE.md` → "Critical path to v1" still owns the ordering
> of this phase against the others.
> **Companion:** `layers-architecture.md` is the model for this document and the
> owner of everything CR 613. Read §5.2 (acyclicity) and §9 (hypothetical check)
> before touching §5 here — the look-ahead frame is the same machinery.

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
before Phase 8 card breadth, not before this. And §5 below shows this phase
*pre-pays* part of it: the CR 614.12 look-ahead frame is the same hypothetical
overlay CR 613.8's step-4 check needs (`layers-architecture.md` §9).

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
| `CreateTokens { def, controller, n }` | 614.16 | RE | |
| `BeginStep` / `BeginPhase` / `BeginTurn` | 614.10 | RE | skips replace these |
| `ProduceMana` | 106.6a | RE | |

`ZoneChangeCause` is the semantic carrier that makes CR 701.8b answerable:

```rust
pub enum ZoneChangeCause {
    /// CR 701.8b, way 1 — an effect that uses the word "destroy".
    Destroyed,
    /// CR 701.8b, ways 2 and 3 — the SBAs that check lethal damage / deathtouch.
    DestroyedBySba,
    Sacrificed,          // CR 701.21 — NOT destruction
    ZeroToughness,       // CR 704.5f — NOT destruction
    LegendRule,          // CR 704.5j
    AuraOrEquipmentSba,  // CR 704.5m/n/p
    Discarded, Milled, Drawn,
    Cast, Resolved, Countered, Fizzled,
    Returned, Exiled, PutOntoBattlefield,
}
```

Two rules for `cause`, both learned the hard way elsewhere in this tree:

- **The caller sets it.** `Primitive::Sacrifice` knows it is sacrificing;
  `perform_action` cannot recover that from `(from, to)`.
- **Nothing may branch on `cause` outside the replacement pipeline and the
  trigger matcher.** It is not a general-purpose tag; a third reader is a third
  place for it to drift.

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
// types/effects.rs — replaces the commented-out `ApplyReplacement` line
pub enum Effect {
    // ...
    /// CR 614/615. On a *static* ability, this ability generates a replacement
    /// effect and produces no layer rows (`register_static_effects` skips it,
    /// without tripping the loud-lowering assert). On a *resolving* spell or
    /// ability, this registers a shield in the replacement registry.
    Replacement(Box<ReplacementDef>),
}

pub struct ReplacementDef {
    /// Which proposed events this watches (CR 614.1, 615.1).
    pub pattern: EventPattern,
    /// Which objects/players it shields. Reuses the layer system's
    /// `AffectedSet` — `SourceOnly` is CR 614.12's "affects only that
    /// permanent" test, and that is exactly the distinction 614.12 draws.
    pub affected: AffectedSet,
    /// How it rewrites a matching event.
    pub modification: EventModification,
    /// CR 616.1a–d — which forced-choice bucket this falls in.
    pub class: ReplacementClass,
    /// How many times it can fire (CR 615.7 shields, 701.19a "next time").
    pub uses: Uses,
    /// CR 903.9b is an explicit exception to CR 614.5 and is the only one in
    /// the rules. Default `false`.
    pub exempt_from_614_5: bool,
}

/// CR 616.1a–e. `Other` is 616.1e — free choice.
pub enum ReplacementClass { SelfReplacement, ControlChanging, CopyOnEnter, BackFaceUp, Other }

pub enum Uses {
    /// CR 614.1a static abilities, 615.10, 701.19b — every time, forever.
    Static,
    /// CR 701.19a regeneration shield, CR 615.8 "next time [source] would deal
    /// damage" — one application, then the effect is gone.
    Once,
    /// CR 615.7 — "prevent the next N damage"; each point prevented decrements.
    Shield(u64),
    /// CR 122.1c/d/h — backed by counters on a permanent. Applying removes one
    /// counter; the effect exists while at least one remains. Note the CR's
    /// wording: one *or more* counters create **a single** replacement effect,
    /// so two shield counters do not give two applications to one event.
    CounterBacked(CounterType),
}
```

`EventPattern` is a predicate over a proposed `GameAction` plus the affected
object's *effective* characteristics — never its printed ones. It is written as
data, not a closure, for the same reason `PermanentFilter` is: closures cannot
be compared, cloned cheaply, or inspected by the loop detector.

`EventModification` is the rewriter. It needs `&mut GameState` and a
`DecisionProvider`, because CR 615.5 ("the rest of the effect takes place
immediately afterward"), CR 701.19a (regeneration taps and removes from combat)
and CR 122.1c (remove a counter) all perform work beyond rewriting the event.
Its return type is the whole of CR 614.6 and 616.1g:

```rust
pub enum ReplacementOutcome {
    /// CR 614.6 — the event never happens. Skips, "prevent that damage",
    /// "instead do nothing".
    Nothing,
    /// One modified event, which re-enters the pipeline (CR 616.1f).
    Modified(GameAction),
    /// Several — Doubling Season's "creates twice that many instead". Each
    /// branch inherits the applied-set (CR 614.5's "an event or any modified
    /// events that may replace that event").
    Split(Vec<GameAction>),
}
```

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

```
fn apply_replacements(game, action, ctx) -> Vec<GameAction>   // the performed set
    applied: HashSet<ReplacementInstanceId> = {}
    work: VecDeque<GameAction> = [action]
    out: Vec<GameAction> = []

    while let Some(ev) = work.pop_front():
        loop:                                            # CR 616.1f
            cands = gather(game, ev)                     # §3.3, five sources
                      .filter(applies_to(ev))            # EventPattern + AffectedSet
                      .filter(|c| c.exempt_from_614_5    # CR 903.9b
                                  || !applied.contains(c.instance))   # CR 614.5
            if cands.is_empty(): break

            bucket = forced_bucket(cands)                # CR 616.1a → b → c → d → e
            chooser = affected_chooser(game, ev)         # CR 616.1 / 400.6
            chosen = if bucket.len() == 1 { bucket[0] }
                     else { ask_choose_replacement(dp, chooser, bucket) }

            if !chosen.exempt_from_614_5: applied.insert(chosen.instance)
            chosen.consume_use(game)                     # Uses::Once / Shield / CounterBacked

            match chosen.apply(game, ev, ctx)?:           # CR 614.6
                Nothing      => { ev = DROPPED; break }
                Modified(e)  => ev = e
                Split(es)    => { work.extend(es[1..]); ev = es[0] }
        if ev != DROPPED: out.push(ev)
    out
```

Six things this encodes, each with its rule:

- **CR 614.4** — `gather` runs against live state at the moment of proposal.
  There is no "go back in time" path because there is no other place to ask.
- **CR 614.5** — the `applied` set is keyed on the *effect instance*, and it
  follows the event through every modification. `exempt_from_614_5` exists for
  exactly one rule (903.9b) and must not grow a second user without a CR cite.
- **CR 616.1a–e** — `forced_bucket` returns the highest-priority non-empty class
  and only that class; 616.1e is the fallthrough.
- **CR 616.1f** — the outer `loop` re-gathers after every application, so an
  effect made newly applicable by the modification is picked up (CR 616.2).
- **CR 616.1g** — `Split` pushes to the *back* of `work`, so the outer event
  finishes its own replacement chain before an inner one starts. Doubling Season
  before Voice of All's "choose a color"; CR 121.2a's "draw N" before the
  individual draws.
- **CR 614.6/614.7** — `Nothing` drops the event, and an event that produced no
  proposal was never in `work` to begin with (CR 614.7a's zero damage is already
  short-circuited in `perform_action`).

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
`compute_to_ceiling` sees a battlefield that contains the entering object under
the proposed controller, plus the registry rows its own statics would generate,
plus the pending mods.

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

**Two notes that decide how to build it.**

- **Its second customer is CR 613.8.** `layers-architecture.md` §9 step 4 is a
  hypothetical check — "temporarily apply B to a frame snapshot, recompute A's
  `affected`, compare" — and §9 leaves the snapshot shape open ("clone vs. CoW
  overlay"). Same machinery. Build the overlay here, and the 613.8 cluster
  inherits it. Design it as a read-side view, not a `GameState` clone;
  `compute.rs` reads the battlefield in four places and they all become one
  accessor.
- **Self does not mean self-affecting.** CR 614.12's own Orb of Dreams example:
  a permanent's *replacement* effect applies to itself only if it "affects only
  that permanent", i.e. `AffectedSet::SourceOnly` — a filter-based one
  (`Permanents enter tapped`) does not. But clause (2) puts **no such
  restriction on the characteristics computation**: an entering creature with
  "Creatures you control get +1/+1" does get its own anthem in the look-ahead
  frame. The self-only test belongs to the replacement's applicability, not to
  the frame. Two different questions, one rule number.

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

**Exit criterion:** every state mutation observable by CR 614 or CR 603 is
emitted from exactly one place, and an event log replay can distinguish drawn
from tutored, destroyed from sacrificed, and countered from resolved.

### Phase RB — the pipeline, with counters and regeneration as consumers

1. `ReplacementDef`, `EventPattern`, `EventModification`, `ReplacementOutcome`,
   `ReplacementClass`, `Uses`; `Effect::Replacement`;
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

### Phase RE — the remaining event kinds

Draw replacement (614.11, 614.11a/b, 121.2a's outer event, 121.6a empty
library), skips (614.10/a/b — per-player consumable `pending_skips` consulted at
step/phase/turn begin), token and counter doublers (614.16), life-gain
replacement (119.10), mana replacement (106.6a), CR 608.3e (permanent spell
whose controller can't put it onto the battlefield).

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

Test cards go in `src/cards/phase_r*_cards.rs`; integration tests in
`tests/phase_r*_integration_test.rs`.

---

## 11. Findings and open questions

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

5. **Open — the overlay's shape.** `layers-architecture.md` §15.2 item 3 left
   "clone vs. CoW overlay" open for the dependency algorithm and it is still
   open. This phase forces the decision (§5). Whichever is chosen, record it in
   *both* documents — a divergent answer between the 614.12 frame and the 613.8
   check is two implementations of the same idea.

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
