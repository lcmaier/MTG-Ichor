# "Can't" Effects — CR 101.2, 614.17, 613.11

> **Status:** design, authored 2026-08-27 in answer to PR #62's review theme A
> (`plans/handoffs/rb-review.md`). No code written yet.
> **Authority:** the restriction model — type shapes, the enforcement points,
> where a "can't" is discovered, and the phase sequencing for CR 101.2 / 614.17
> / 613.11. Where this contradicts `codebase-state.md`, that file wins on *what
> exists*; this file wins on *what is being built*. `CLAUDE.md` →
> "Critical path to v1" still owns the ordering against the other phases.
> **Companion:** `replacement-architecture.md` is the model for this document
> and owns everything CR 614–616. Read its §3.2 (the growth contracts), §3.3
> (discovery sources) and §5 (the look-ahead frame) before touching §3 and §5
> here — this design reuses all three and the reuse is the point.
> **Supersedes:** ticket **L15** in `plans/archive/implementation-plan-final.md`
> §"Post-layer pass", whose `PlayerActionRestriction` enum is a variant per card
> (§6.2). L15 was never built; nothing has to be unwound.

---

## 0. The budget — why this stays tight

`o:"can't"` is **1,857 printed cards**, which is 3.3× the corpus that
`replacement-architecture.md` §0 was written to bound. The same four
commitments apply, and each is falsifiable.

**One. A card is data, not code.** Adding a "can't" card touches
`src/cards/*.rs` and nothing else. The failure this rules out by name is the
one the archive plan already walked into: an enum with `CantCastSpells`,
`CantGainLife`, `CantAttack`, `CantActivateAbilities` as *sibling variants*,
which is a variant per card dressed as a design (§6.2).

**Two. There are exactly three growth axes, all bounded by enumerations the
engine already owns, and one of them is closed.** A "can't" can attach to an
**event** the engine proposes (bounded by `GameAction`, so `Restriction::Event`
reuses `EventPattern` and costs almost no new vocabulary), a **choice** a player
makes (bounded by `ChoiceKind`, which is what `DecisionProvider` already
enumerates), or the **application of an effect** (CR 701.19c / 615.12 — one
site, two permitted subjects, and therefore closed at one arm). Adding an arm is
a claim that the engine has a site none of the three names — the same shape of
claim as `EventPattern`'s "one arm per `GameAction` variant", and checkable the
same way. An earlier draft claimed two axes and tried to argue the third away;
§3.3 states it plainly instead.

**Three. The claim was measured, not asserted.** Every one of the 1,857 cards
was pulled and each of its 2,034 "can't" clauses classified by *where the engine
would have to enforce it* (`plans/references/cant-census.py`). The answer is
§2.1's table: six enforcement points, of which **Phase RB built one**. The
residual after classification is 4 clauses and they are printed in §2.5 rather
than rounded away; §2.6 records the one family the first pass classified wrong,
because a census's failure mode is a bucket nobody re-read.

**Four. Performance has one designated lever and a measurement gate — and this
is hotter than the replacement pipeline.** `apply_replacements` sits on
`execute_action`. A restriction check sits on `execute_action` *and* on
`candidate_priority_actions`, which runs for every player at every priority
pass and already enumerates every castable spell and activatable ability. §3.5
commits to the same instrument `gather` uses (an exact source gate, not a
heuristic) and to measuring with `fuzz_games --games 200 --seed 12345` against
the recorded RB baseline of **13.04 ms/game**.

**What this does *not* claim.** Tier 1a (combat) is not a filter and this
document does not pretend it is — CR 508.1d makes attacker declaration a
constraint-satisfaction problem, and §4.2 scopes the solver honestly rather than
hiding it. CR 614.17b's derived cost check needs a `Cost → GameAction`
projection that does not exist (§4.6). And the "unless" half of CR 508.1c —
149 clauses — needs `Effect::Conditional`, which is Phase 6. Those are budgeted,
not hidden.

---

## 1. Verdict — Phase RB was right, and it cannot grow into this

### What RB built, and why it is correct

`engine::replacement::is_blocked` is a `bool`-returning match over `GameAction`,
consulted at the top of every CR 616.1f iteration and winning over the pipeline.
Three things about it are *right* and this design keeps all three:

- **A "can't" is not a `ReplacementDef`.** Modelling indestructible as one would
  have put it in the CR 616.1 choice list, where a player could decline it.
- **It is checked ahead of the pipeline and it wins** (CR 101.2).
- **It is re-asked on every iteration**, because CR 614.17c lets a
  self-replacement change the event's *type*, and an event of a different type
  is a different "can't" question. Platinum Emperion's printed rulings confirm
  both halves of this in one card: *"Effects that would replace having you gain
  life won't be able to be applied because it's impossible for you to gain
  life"* (614.17c filters the candidates) and *"Effects that replace an event
  with having you gain life will end up replacing the event with nothing"*
  (the substituted event is re-checked). RB's loop already produces both
  answers.

### What it cannot grow into

`is_blocked` answers exactly one question — *may this proposed `GameAction`
happen?* — and §2.1 measures that at **236 of 2,034 clauses, 12%**. The other
88% never reach `execute_action` at all, because the player is never offered the
choice that would propose the event. A creature that "can't attack" is not an
attack proposal that gets rejected; it is a creature `legal_attackers` must not
return.

The review found this wearing four costumes (`rb-review.md` A1–A4) and they are
one gap:

| Costume | What it actually is |
|---|---|
| A2 — `is_blocked`'s match grows one arm per "can't" | The Tier-2 arm should be *data* (an `EventPattern`), not a match arm |
| A3 — `Primitive::CantBeRegenerated` | One of two production paths (resolution-created), needing one generic primitive rather than one per mechanic |
| A4 — `GameState::cant_be_regenerated: HashSet<ObjectId>` | A one-rule registry standing in for the general one, with a hand-rolled duration |
| A1 — no scope | This document |

### Why now

Three ordering facts, only one of which the review named:

1. **RC-4 carries CR 614.17d** ("can't" effects that modify how a permanent
   enters). It cannot ask that question without a restriction to ask it *of*.
   This is the review's stated deadline and §7 sizes exactly how much of the
   model RC-4 actually needs — which is less than the whole thing.
2. **A block of unstarted work has had no home in the critical path.**
   `cards-unlocked-ledger.md` still carries rows for `T21b` (evasion framework),
   `T21d` (the 508.1d/509.1c requirements solver) and `T22` (hexproof / shroud /
   protection targeting), all with empty status. Those ids are *dictionary
   entries* — `CLAUDE.md`'s authority table says `plans/archive/` is superseded
   and must not be acted on, so the archive plan's sequencing for them is dead
   and "waiting in a queue" would be the wrong picture. What is alive is the
   **work**, and until this document it appeared nowhere in `CLAUDE.md`'s
   critical path: items 1–7 have no entry for combat restrictions or targeting
   legality. Adding item 5b is what gives it one. The evidence that it is real
   does not depend on the archive plan at all — see point 3.
3. **`KeywordFlag` already ships four unenforced restrictions.** `Hexproof`,
   `Shroud`, `Menace` and `Intimidate` are constructible and grantable — and
   two of them are printed by `ui/display.rs` — but **no code reads any of them
   outside `display.rs`**. Grep is the whole audit: the only `KeywordFlag::`
   references to those four in `src/` are the enum definition, `display.rs`, and
   `land_types.rs`'s hexproof insertion. Cards carrying them can be registered
   today and will quietly do nothing.

---

## 2. Scope, measured

Every number below is `python plans/references/cant-census.py`, run 2026-08-27
against `o:"can't" -is:funny`. **These are clause counts, not card counts** —
one card contributes several clauses to several tiers — and card counts are
printed alongside. Neither is a work estimate; §2.4 is.

### 2.1 The tier table

| Tier | Enforcement point | CR | Clauses | Cards | Built? |
|---|---|---|---:|---:|---|
| **1a** | Declaring attackers and blockers | 508.1c, 509.1b | **1,262** | 1,195 | Flying + Defender only |
| **1b** | Beginning to cast a spell / playing a land | 601.3, 116.2a | **120** | 115 | ❌ |
| **1c** | Activating an ability | 602.5 | **35** | 35 | ❌ |
| **1d** | Choosing targets | 115.6 | **51** | 51 | ❌ (`targeting.rs` TODO) |
| **1e** | Paying a cost | 614.17b, 118 | **16** | 16 | ❌ |
| **2** | An engine-proposed event | **614.17** | **236** | 220 | ✅ **Phase RB** |
| **3** | Applying a replacement effect | 701.19c, 615.12 | **187** | 182 | Half (701.19c only) |
| **4** | Having an ability at all | 113.11 | 6 | 6 | Layer system's (§5.4) |
| **5** | *A query, not a restriction* | 101.3 | 74 | 74 | ❌ (§4.8) |
| **6** | *Not a game event* | 601.2b, 601.2f | 43 | 41 | Out of scope (§10) |
| ? | unclassified | — | 4 | 4 | printed in §2.5 |

Tier 1a is 62% of the corpus on its own, and the single largest bucket
("can't be blocked", 580 clauses) is where the engine's evasion story stops
after flying.

### 2.2 What the census cannot see

The largest populations in Tiers 1a and 2 **never print the word "can't"** —
they arrive as keyword abilities whose restriction lives in the CR. Counted
separately on Scryfall the same day:

| Keyword | CR | Cards | The restriction |
|---|---|---:|---|
| Flying | 702.9b | 3,250 | can't be blocked except by flying/reach |
| Menace | 702.111a | 405 | can't be blocked except by two or more |
| Reach | 702.17b | 434 | (the exception half of flying's) |
| Defender | 702.3b | 309 | can't attack |
| Ward | 702.21a | 211 | *not a restriction* — a triggered ability |
| Protection | 702.16b–f | 197 | can't be targeted / enchanted / equipped / blocked (**and 702.16e's damage half is a *prevention* effect, Phase RD, not a restriction**) |
| Landwalk | 702.14a | 122 | can't be blocked if defender controls the type |
| Fear / Intimidate / Shadow / Skulk / Horsemanship | 702.36a etc. | 147 | can't be blocked except by … |
| Hexproof | 702.11b | 96 (336 incl. grants) | can't be targeted by opponents |
| Shroud | 702.18a | 35 | can't be targeted at all |
| Indestructible | 701.8a | 115 (524 incl. grants) | can't be destroyed |

Two consequences. **Keyword-derived restrictions are a first-class discovery
source** (§3.4 source 3), the way CR 122.1c's counters are for the replacement
pipeline — synthesized during the sweep because nothing on the card says so.
And **protection is one card-facing keyword that produces four restrictions in
three different tiers plus a prevention effect** (CR 702.16b–f, taken one
subrule at a time), which is the clearest single argument that the tiers need
one shared vocabulary rather than four bespoke checks.

### 2.3 What the engine enforces today

| Restriction | Where it would be checked | Today |
|---|---|---|
| Indestructible | `is_blocked`, `pipeline.rs:53` | ✅ one hardcoded arm |
| Can't be regenerated | `push_if_applicable`, `gather.rs:228` | ✅ via a `GameState` `HashSet` |
| Defender | `validate_attackers`, `validation.rs:187` | ✅ hardcoded |
| Flying vs. non-reach blocker | `can_block`, `validation.rs:313` | ✅ hardcoded |
| Menace, Intimidate | — | ❌ flag exists, nothing reads it |
| Hexproof, Shroud | `validate_targets`, `targeting.rs:39` | ❌ flag exists, TODO comment |
| Protection | — | ❌ deliberately not a `KeywordFlag` (`keywords.rs` "Five variants used to be here") |
| Everything in Tiers 1b/1c/1e | — | ❌ |
| Damage can't be prevented | — | ❌ no prevention machinery at all (Phase RD) |

Five hardcoded checks, at five call sites, with no shared vocabulary between
them. That is the shape the model replaces.

### 2.4 The three shapes hiding in the numbers

The clause counts are not the work. Sorted by what each clause actually needs:

- **A filter over the *other* object.** 222 of the 580 "can't be blocked"
  clauses name a would-be blocker ("by creatures with power 3 or greater", "by
  artifact creatures"). There are **62 distinct tails** across all 580, and all
  but the counting ones are `PermanentFilter`-expressible today. The counting
  ones ("more than one creature", "three or more creatures" — 77 clauses) want
  one integer field, which is CR 509.1b's own menace shape.
- **A duration.** 35% of Tier 1a, 28% of Tier 1b and 11% of Tier 2 are
  turn-scoped ("can't be blocked this turn"). They come from a *resolution*, so
  they need a registry row with a `Duration`, exactly as CR 701.19a's
  regeneration shield does. This is what makes `Primitive::Restrict` the right
  answer to A3 rather than a primitive per mechanic (§6.3).
- **A condition.** 149 clauses are CR 508.1c's "or that it can't [X] unless some
  condition is met" — 138 of them in Tier 1a. These need `Effect::Conditional`,
  which is unimplemented and belongs to Phase 6. Scoped out of every phase
  below, and named so it is not rediscovered.

### 2.5 The residual

Three clauses that no bucket caught and that are about the game, printed rather
than rounded away. **None is a sixth mechanism** — each is an existing tier with
an unusual subject, which is the only claim this section makes:

| Card | Clause | Where it belongs |
|---|---|---|
| Artificial Evolution | "The new creature type can't be Wall." | CR 612 text-changing, a constraint on a *choice made during resolution* |
| Chef's Kiss | "The new targets can't be you or a permanent you control." | CR 115.7 retargeting; Tier 1d with a different subject |
| Melira, the Living Cure | "…you can't get additional poison counters this turn." | Tier 2, reached through a replacement's `then` |

The census also prints a fourth, **Sovereign's Realm** — "Your starting deck
can't have basic land cards" — which is a **Conspiracy**. Conspiracies start the
game face up in the command zone and modify deck construction (CR 113.6n,
CR 905), and the engine has no pregame procedure to modify. It is out of scope
here, out of scope for v1, and named once so a later reader does not spend a
lookup on it.

### 2.6 One clause family the census got wrong on the first pass

Recorded because a classification error is the failure mode a census has, and
finding one is evidence the buckets were read rather than trusted.

**Sigarda, Host of Herons** — "Spells and abilities your opponents control can't
cause you to sacrifice permanents" — was bucketed into Tier 1e (costs) by a
regex that saw "sacrifice". It is not a cost restriction: you never pay costs
for an opponent's spell. Diabolic Edict resolving against you is an *effect*,
and Sigarda blocks the resulting `ZoneChange { cause: Sacrificed }` **scoped by
what caused it**. It is Tier 2 with a source filter, and it is the family's
motivating card — Tajuru Preserver, Tamiyo Collector of Tales and The Master,
Multiplied are the same shape. Five clauses, moved 2026-08-27; Tier 1e fell from
21 to 16 and Tier 2 rose from 231 to 236.

**What it costs the model:** a source filter on `Restriction::Event`. The
provenance is already threaded — `ActionContext` carries the resolution — so
this is a field, not a mechanism. It is the one thing §3.3's `Event` arm needs
beyond `EventPattern` + `AffectedSet`.

---

## 3. The model

### 3.1 The claim

> **A "can't" effect is discovered exactly the way a replacement effect is, and
> differs from one only in what it is asked at.** One type, one discovery sweep,
> one predicate — and a list of *enforcement points* that call the predicate.

Everything below follows from taking that seriously. The three properties that
make `gather` work are all properties of the *source*, not of the pipeline:

- a static ability's continuous effect exists only while the source still has
  the ability (CR 613.7a), so it must be read off the **effective** ability
  list — which is what makes Humility and Blood Moon strip a restriction for
  free, at no cost;
- a resolution-created effect needs a `Duration` and a registry row;
- a rules-derived effect (a counter, a keyword) is synthesized during the sweep
  because nothing on the card says so.

All three are as true of "can't be blocked" as of "would be destroyed instead".
Splitting them would be building `gather` twice.

### 3.2 `RestrictionDef` — the type surface

```rust
/// One CR 101.2 "can't" — a prohibition on an action or an event.
///
/// Deliberately *not* shaped like `ReplacementDef`. A replacement effect is a
/// shield around a thing (CR 614.1's own words), so it splits into a `pattern`
/// and an `affected`. A restriction is a prohibition on an **action**, and
/// CR 508.1c, 509.1b and 601.3 all phrase it that way — so the subject varies
/// per arm and lives inside it.
pub struct RestrictionDef {
    pub what: Restriction,
}
```

There is no `rewrite`, no `then`, no `uses`, no `class`, no `optional`. A "can't"
does not replace, does not have a rest-of-the-effect, is never spent, is never
ordered against another "can't" (CR 101.2 has no tiebreak because it needs
none — two prohibitions agree), and is never optional. **Five fields
`ReplacementDef` needs that this does not is the evidence the two are different
types rather than one type with a flag.**

The struct is a one-field wrapper today on purpose: `unless: Option<Condition>`
(§2.4) lands in it when Phase 6 gives `Condition` a meaning, and a bare enum
would have to become a struct at that point across every reader.

### 3.3 `Restriction` — the arms, and the growth contract

```rust
pub enum Restriction {
    // ---- Axis 1: an event the engine proposes (CR 614.17) ----------------
    /// **Almost zero new vocabulary.** `EventPattern` and `AffectedSet` are the
    /// replacement pipeline's, reused verbatim: "this permanent can't be
    /// destroyed" is the same predicate over the same proposal as "if this
    /// permanent would be destroyed, instead …", minus the instead. The one
    /// addition is `by`, CR 101.2 scoped by what *caused* the event — Sigarda
    /// and the four cards §2.6 names. Provenance is already threaded through
    /// `ActionContext`, so it is a field rather than a mechanism.
    Event { pattern: EventPattern, affected: AffectedSet, by: Option<SourceFilter> },

    // ---- Axis 2: a choice a player makes (one arm per `ChoiceKind`) ------
    /// CR 508.1c. `ChoiceKind::DeclareAttackers`.
    Attack { attacker: PermanentFilter, target: Option<AttackTargetFilter> },
    /// CR 509.1b, the defending player's half. `ChoiceKind::DeclareBlockers`.
    Block { blocker: PermanentFilter },
    /// CR 509.1b's evasion half — a restriction the *attacker* imposes on who
    /// may block it. `min_blockers` is menace (702.111a) and the 77 clauses
    /// that count; `except_by` is flying, landwalk, protection and the 222
    /// that filter.
    BeBlocked {
        attacker: PermanentFilter,
        except_by: Option<PermanentFilter>,
        min_blockers: Option<u32>,
    },
    /// CR 601.3. `ChoiceKind::PriorityAction`.
    Cast { player: PlayerRef, spell: CardFilter, from: Option<Zone> },
    /// CR 116.2a — a *special action*, not a cast, and Aggressive Mining's
    /// printed ruling turns on the difference: it "doesn't stop lands from
    /// being put onto the battlefield by a spell or ability."
    PlayLand { player: PlayerRef, land: Option<CardFilter>, from: Option<Zone> },
    /// CR 602.5. `ChoiceKind::PriorityAction`.
    ActivateAbility { source: PermanentFilter, ability: Option<AbilityFilter> },
    /// CR 115.6. `ChoiceKind::SelectRecipients`.
    BeTargeted { object: AffectedSet, by: Option<SourceFilter> },
    /// CR 614.17b and CR 118. The cost-payment choice kinds.
    PayCost { player: PlayerRef, cost: CostFilter, purpose: Option<CostPurpose> },

    // ---- Axis 3: applying an effect (CR 701.19c, 615.12) -----------------
    /// Withholds a *replacement or prevention effect* rather than blocking an
    /// event or a choice. This **is** a third axis and an earlier draft of this
    /// document tried to argue it was not — the argument did not survive being
    /// written down, so it is stated plainly instead.
    ///
    /// It is a third axis because CR 101.2's "it" can be the application of an
    /// effect rather than an event: CR 701.19c says a regeneration shield is
    /// "not applied", and the destruction it would have replaced still
    /// happens. No `EventPattern` can say that.
    ///
    /// **The axis is closed at one arm, and that is what makes it safe.**
    /// Axes 1 and 2 are open because `GameAction` and `ChoiceKind` grow; this
    /// one cannot, because there is exactly one place in the engine where an
    /// effect is applied to an event (`gather`'s `push_if_applicable`) and the
    /// CR names exactly two things that may be withheld there — regeneration
    /// shields (701.19c) and prevention effects (615.12). A second arm here
    /// would be a claim that the engine applies effects somewhere else.
    ApplyReplacement { kind: ReplacementKindFilter, to: AffectedSet },
}
```

**The growth contract, meant to be enforced in review.** `Restriction` grows on
three axes and no other, and each axis is an enumeration the engine already
owns:

| Axis | What a "can't" attaches to | The enumeration that bounds it | Open? |
|---|---|---|---|
| 1 | an event the engine proposes | `GameAction` variants, via `EventPattern` | grows with the engine |
| 2 | a choice a player makes | `ChoiceKind` (`ui/choice_types.rs`) | grows with the engine |
| 3 | the application of an effect | `gather`'s one application site | **closed at one arm** |

- **`Event` never grows.** New replaceable event kinds arrive as `GameAction`
  variants and reach this arm through `EventPattern` for free. A change that
  adds an event-shaped arm here is the smell.
- **Every axis-2 arm names one `ChoiceKind`.** Adding one is a claim that the
  engine offers a player a decision that `ui/choice_types.rs` does not
  enumerate. That is a real thing to check, and usually the answer is that the
  missing item is a `ChoiceKind`.
- **Axis 3 is closed** for the reason its own doc comment gives: one application
  site, two things the CR permits withholding.
- **Per-card variety goes in the filters**, which are `PermanentFilter`,
  `CardFilter` and `PlayerRef` — vocabulary that already exists and that §2.4
  measured as sufficient for 60 of 62 observed shapes.

**Matched exhaustively; not `#[non_exhaustive]`.** Adding an arm is a normal
diff that fails to compile at every reader, which is the property
`replacement-architecture.md` §3.2 chose deliberately and for the same reason:
an arm the enforcement points do not consult is a card that silently does
nothing.

### 3.4 Where restrictions come from

The same list as `replacement-architecture.md` §3.3, in the same order, and
getting it wrong has the same failure mode — a card that silently does nothing.

1. **Static abilities of permanents**, read off each object's **effective**
   ability list during a `battlefield_ids_ordered` sweep. Not a registry scan,
   for the reason `gather` gives: it is what makes Humility strip a restriction,
   and it asks CR 614.17a's "must exist before the event" at the one instant
   that matters.
2. **Static abilities functioning in other zones.** Grafdigger's Cage is source
   1; **Conduit of Worlds' "you may play lands from your graveyard"** is the
   permission half of the same question and is source 1 too. The genuinely
   deferred case is a restriction whose *source* is in another zone. Deferred
   with the same note `gather.rs:11` carries, and blocked on the same work.
3. **Keyword abilities**, synthesized during the sweep. §2.2 is the size of this
   source and it is the largest one. Structurally identical to `gather`'s
   `counter_replacements`: derived from something the object *has* rather than
   from text it prints.
4. **Rows from resolutions with a duration** — the registry. Skullcrack's
   "Players can't gain life this turn" and Wrath of God's "They can't be
   regenerated" are both this.
5. **Rules of the game.** CR 614.17b is the whole of this source today: it is
   not a card, it is a rule that *derives* a Tier-1e prohibition from a Tier-2
   one (§4.6).

**The gate is part of the contract.** `gather` is gated by
`GameState::replacement_ability_sources`, and its doc carries the rule "add a
new gather source and you must add it to the gate, or the source is silently
dead on every board the gate skips." The restriction sweep needs its own gate
with the same rule attached, and it is a *different* set: an object can have a
restriction ability without having a replacement one.

### 3.5 The predicate, the gate, and the performance budget

```rust
/// CR 101.2 — is this prohibited right now?
///
/// The one reader of every restriction. `is_blocked` becomes
/// `is_prohibited(game, &Query::Event(action))`.
pub fn is_prohibited(game: &GameState, query: &Query) -> bool;
```

One function, so that CR 101.3's "if you can't" (§4.8) is a *caller* rather than
a parallel mechanism, and so that CR 614.17b's derived check has something to
ask.

**This is hotter than the replacement pipeline and the design has to say so.**
`apply_replacements` runs per proposed action. `is_prohibited` runs per proposed
action *plus* per enumerated priority action, and `candidate_priority_actions`
already walks every hand card and every battlefield ability for every player at
every priority pass. `gather`'s own comment prices an ungated sweep at
"~6,000 extra layer walks per `fuzz_games` game" against the untap step alone;
the Tier-1 sites are more frequent than the untap step.

Three commitments, in the order they get made:

1. **An exact gate, not a heuristic.** The same instrument as
   `replacement_ability_sources`: a set maintained at `place_on_battlefield` and
   `cleanup_zone_state`, over-approximating (a stripped ability leaves the set
   populated) and therefore costing a walk and never an answer.
2. **Keyword restrictions do not need the gate**, because
   `get_effective_abilities` is not how they are found — `has_keyword` is, and
   the combat sites already pay for that today.
3. **A measurement gate, not taste.** `fuzz_games --games 200 --seed 12345`
   against RB's recorded **13.04 ms/game**, reported per sub-phase, with the
   answer-preserving early-out (no restriction sources on the board → the whole
   sweep is skipped) as the only pre-approved optimization. Semantics-assuming
   shortcuts are ruled out in advance; `can_change_abilities()` is this
   project's cautionary tale and `layers-architecture.md` §12 has the numbers.

### 3.6 What is deliberately *not* a restriction

Three tiers are excluded, each by a rule rather than by taste. Recorded here so
the exclusions are not re-litigated:

- **Tier 4 (CR 113.11), "can't have or gain [ability]" — the layer system's.**
  CR 101.2a says outright that "adding abilities to objects and removing
  abilities from objects don't fall under this rule". So the Archetype cycle is
  not a CR 101.2 prohibition at all; it is a Layer 6 rule that a `GrantAbility`
  row does not apply. It belongs in `layers-architecture.md`, next to
  `LoseAllAbilities`, and this document hands it over. 6 clauses.
- **Tier 5 (CR 101.3), "if you can't" — a query, not a restriction.** §4.8.
- **Tier 6 — not game events.** "X can't be 0" is CR 601.2b's announcement step;
  "this effect can't reduce the cost below one mana" is CR 601.2f's pipeline and
  belongs to the cost-modification phase `replacement-architecture.md` §9 asks
  for. 43 clauses, two existing homes.

---

## 4. The enforcement points, one at a time

Each section names the rule, the engine site, the consumer cards, and what it
costs. The order is the order §7 builds them in.

### 4.1 Tier 2 — the event chokepoint (CR 614.17)

**Rule.** CR 614.17: some effects state that something can't happen; they aren't
replacement effects but follow similar rules. CR 101.2: the "can't" wins.

**Site.** `pipeline.rs`'s `is_blocked`, already in the right place and already
re-asked per iteration.

**Change.** The `match` becomes a `gather`-shaped sweep, and the arm becomes
data. Indestructible stops being `has_keyword(game, *object, Indestructible)`
inside a `GameAction::Destroy` arm and becomes a keyword-derived
`Restriction::Event { pattern: EventPattern::Destroy { source: None },
affected: AffectedSet::SourceOnly }` — which is source 3, synthesized during the
sweep, and reads identically to how `counter_replacements` synthesizes CR
122.1c's shield.

**Consumers.** Indestructible (524 cards); Solemnity and Melira ("counters can't
be put on…", and note CR 113.6i makes this function *as the object enters*);
Platinum Emperion and Teferi's Protection ("your life total can't change" — two
patterns, `GainLife` and `LoseLife`, from one clause); Abyssal Persecutor and
Platinum Angel (`PlayerLoses` / `PlayerWins`, which
`replacement-architecture.md` §8a schedules for Phase RE and which this arm
therefore inherits for free); Fear of Sleep Paralysis ("stun counters can't be
removed", which is a `RemoveCounters` block that collides deliberately with
CR 122.1d's replacement).

**Cost.** The smallest of the six. `EventPattern` already exists,
`pattern_matches` already exists, `shield_contains` already exists, and the
sweep is `gather`'s with the `Rewrite` half deleted.

### 4.2 Tier 1a — combat, and why it is a solver

**Rule.** CR 508.1c and 509.1b: the declaring player checks each creature
against **restrictions**; if any are disobeyed the declaration is illegal. CR
509.1b adds that evasion abilities are **cumulative** — "an attacking creature
with flying and shadow can't be blocked by a creature with flying but without
shadow."

**This is not a filter, and pretending otherwise is the trap.** CR 508.1d pairs
restrictions with **requirements** ("attacks if able") and its own worked example
shows why they cannot be evaluated per creature:

> A player controls two creatures: one that "attacks if able" and one with no
> abilities. An effect states "No more than one creature can attack each turn."
> The only legal attack is for just the creature that "attacks if able" to
> attack. It's illegal to attack with the other creature, attack with both, or
> **attack with neither**.

So legality is a property of the *set*, and the rule is:

> A declaration is legal iff it disobeys no restriction **and** the number of
> requirements it obeys equals the maximum obtainable over all
> restriction-legal declarations.

That is a maximisation subject to constraints, it is the `T21d` ticket, and it
is the largest single piece of work this document scopes. What follows is the
part an earlier draft left as "it is a constraint solver" and stopped — which is
the over-deferral this project has been bitten by, so it gets counted instead.

#### The decomposition, measured

The 1,262 Tier-1a clauses are not 1,262 constraints. Counted 2026-08-27:

| Shape | Clauses | Couples creatures? |
|---|---:|---|
| Per-creature restriction ("this creature can't block", "can't be blocked by artifact creatures") | **1,200** | no — a predicate on one creature |
| Per-attacker blocker count ("can't be blocked by more than one", menace's "except by two or more") | **49** | no — local to one attacker's block assignment |
| Cross-creature restriction ("can't attack alone", "can't attack unless at least two other creatures attack") | **13** | **yes** |
| Global cap ("no more than one creature can attack each combat") | **2 cards** — Silent Arbiter, Dueling Grounds | **yes**, over everything |

And the requirements side, counted on Scryfall the same day: "attacks each
combat if able" **88**, "attacks this turn if able" **18**, "blocks this turn if
able" **14**, "blocks each combat if able" **6**, lure's "must be blocked if
able" **20** — plus **goad** (83 cards), whose reminder text *is* "attacks each
combat if able". Call it ~150 cards, and note that goad is the one that matters
for v1: it is a Commander staple and it applies to *many* creatures at once.

**So 95% of Tier 1a is a predicate**, and the solver exists for ~15 printed
cards plus the requirement population. That ratio is the design.

#### The algorithm that follows from the measurement

1. **Filter by per-creature restrictions.** O(n). This is `legal_attackers` /
   `can_block` as they exist today, with the hardcoded flying and defender arms
   replaced by data.
2. **If no requirement is live on the board and no coupling constraint is
   present, stop.** The declaration is legal iff step 1 passes. This short
   circuit is the common case and must be checked first, not last — the gate is
   two set-emptiness tests.
3. **Otherwise, build the coupling graph** over the declaring player's
   creatures: an edge when a cross-creature restriction names both, and one
   component containing everything a global cap constrains. Solve each
   connected component independently and sum the obeyed-requirement counts —
   sound because requirements are independent unless a restriction couples
   them, which is exactly what the graph records.
4. **Within a component, brute-force.** Enumerate subsets, discard the ones that
   disobey a restriction, take the max obeyed count. Interchangeable creatures —
   those with no requirement and no cross-creature restriction, present only as
   *filler* to satisfy "can't attack alone" — collapse to a count rather than an
   identity, which is what keeps "goad everything, and one creature can't attack
   alone" from being exponential.

**The complexity, stated honestly.** Step 4 is `2^k` in the size of a component,
where `k` counts only creatures that carry a requirement or a cross-creature
restriction. Goad-everything gives a large `R` but an **empty** coupling graph,
so every component is a singleton and the answer is O(n) — which is the case
that would otherwise have been the disaster. A component is large only when
several of the ~15 coupling cards are on one board at once.

**The guard, and it is a real one.** CR 508.1d is NP-hard in the general case;
this design chooses a *bounded exact* algorithm over a heuristic, and takes a
hard cap on component size with a loud error rather than a silent wrong answer.
That is the same trade `MAX_616_1F_ITERATIONS` makes and it should be documented
the same way — as an invariant that has been reasoned about, not "a test
harness".

**Two things follow for the code.** First, `legal_attackers` /
`legal_blockers` keep being an **over-approximation** — that is already this
codebase's documented contract (`legality.rs:1`, "false positives are harmless,
false negatives are bugs"), and it is the right one, because the set-level check
belongs in `validate_attackers` where the CR puts it. Second, **restrictions and
requirements ship together or not at all**, because 508.1d's answer depends on
both — so `Restriction` gets a sibling `Requirement` type in RS-3, discovered by
the same sweep, and neither is useful alone.

**Consumers.** Menace and the 77 counting clauses; the fear/intimidate/shadow/
skulk/horsemanship family (147 cards) as `except_by` filters; landwalk (122);
protection's blocking half; Defender re-expressed as data rather than as
`validation.rs:187`'s hardcoded arm; and the 445 turn-scoped "target creature
can't block this turn" effects, which are registry rows.

**Deliberately not here.** The 138 Tier-1a "unless" clauses (§2.4). A creature
that "can't attack unless you pay {2}" needs `Effect::Conditional` *and* a
cost-payment prompt inside declare-attackers, and CR 508.1d says explicitly that
the player is never *required* to pay such a cost even when paying would obey
more requirements. Phase 6 or later.

### 4.3 Tier 1b — casting and playing

**Rule.** CR 601.3: "A player can begin to cast a spell only if a rule or effect
allows that player to cast it and no rule or effect prohibits that player from
casting it."

**Site.** `cast.rs`'s cast-legality check and `legality.rs::playable_lands`.
Both are already tagged `// PRE-LAYER ZONE:` for reading printed characteristics
on purpose, and that exemption stays: the restriction check is about whether the
*player may act*, which is a separate question from what the object's
characteristics are.

**The wrinkle that has to be designed for, not discovered.** CR 601.3a:

> If an effect prohibits a player from casting a spell with certain qualities,
> that player may consider any choices to be made during that spell's proposal
> that may cause those qualities to change. … Example: A player controls Void
> Winnower, which reads "Your opponents can't cast spells with even mana
> values." That player's opponent may begin to cast Rolling Thunder, {X}{R}{R},
> because the chosen value of X may cause the spell's mana value to become odd.

Read alone that looks like an existential search over an unbounded domain
(X ∈ ℕ), and an earlier draft of this document said so and called it "cheaper
than the CR 614.12 look-ahead frame" **with no evidence**. That was wrong twice
over: the cost was asserted, and the shape was misread.

**Void Winnower's printed rulings settle it.** *"For spells with {X} in their
mana costs, use the value chosen for X to determine if the spell's mana value is
even or not. For example, your opponent could cast Endless One (with mana cost
{X}) with X equal to 5, but not with X equal to 6."* So the prohibition is
evaluated **against the finished announcement**, not searched for in advance.
601.3a is a *permission to begin*, and 601.2's own rewind — "if a player is
unable to comply … the casting of the spell is illegal; the game returns to the
moment before" — is what rejects the bad choice.

So the implementation is **over-approximate at the gate, exact after
announcement**, which is this codebase's existing documented contract for
`candidate_priority_actions` and not a novel shape at all:

- `candidate_priority_actions` offers the cast if *any* announcement could
  satisfy the prohibition — and a coarse "this restriction names a quality some
  choice can change → offer it" is a legal over-approximation, because false
  positives are already harmless there;
- the real check runs at the end of CR 601.2, against the announced spell, and
  an illegal cast rewinds through the machinery `cast.rs::rollback_cast_to_hand`
  already implements.

No search, no frame, no forward-looking evaluation. The only genuinely new thing
is that a *rewind can now be caused by a restriction*, where today it is only
caused by unpayable costs.

**Consumers.** Grafdigger's Cage and Drannith Magistrate (a `from: Some(Zone)`
constraint); Rule of Law and Deafening Silence (a per-turn count, which is
player state, not a filter); Aggressive Mining and Conduit of Worlds
(`PlayLand`, and the two are opposite signs of the same predicate — Conduit
*grants* the permission Aggressive Mining removes, which is why permission and
prohibition want one query); Rakdos, Lord of Riots (a self-imposed restriction,
`AffectedSet::SourceOnly` in `spell`, and the printed ruling confirms it is a
cast restriction and not an ETB one: "Rakdos can be put onto the battlefield by
another spell or ability even if no opponent has lost life that turn").

### 4.4 Tier 1c — activating abilities

**Rule.** CR 602.5. **Site.** `mana_helpers::activatable_abilities` and
`cast.rs::activate_ability`.

**One thing to get right, and it is a `CLAUDE.md` invariant already.** Ability
*indices* are part of the layer-system invariant: `activatable_abilities`
produces an index, `priority.rs` re-derives it by id, `cast.rs::activate_ability`
consumes it, and all three must index the *effective* list. A restriction that
filters abilities filters that list, so it has to be applied at all three sites
or at none. Filtering one alone mis-activates silently — the same failure the
invariant was written for.

**Consumers.** Pithing Needle and Cursed Totem (35 clauses); Stony Silence;
Arachnus Web ("its activated abilities can't be activated", riding on a
Tier-1a restriction from the same card, which is why the two must share a type).

### 4.5 Tier 1d — targeting

**Rule.** CR 115.6 and CR 702.11b/702.18a/702.16b. **Site.**
`targeting.rs::validate_targets` — which already carries the TODO — plus
`legality.rs::enumerate_legal_selections` and `has_any_legal_choice`, because
CR 601.2c makes an illegal target an illegal *cast*, so the enumeration and the
validation have to agree.

**Consumers.** Hexproof (336), Shroud (35), Protection (197) — the whole `T22`
ticket — plus the 51 printed clauses (Canopy Cover, Anti-Magic Aura). Protection
is the card that proves the shared vocabulary earns its place: one keyword,
`BeTargeted` here, `BeBlocked` in §4.2, `Event` for the damage half in Phase RD,
and `Event { pattern: Attach }` for the enchant half.

**Note.** `by: Option<SourceFilter>` is what separates hexproof from shroud —
hexproof is "by spells or abilities your **opponents** control", shroud is
unconditional. One field, both keywords.

### 4.6 Tier 1e — costs, and the only *derived* restriction

**Rule.** CR 614.17b: "If an event can't happen, a player can't choose to pay a
cost that includes that event."

**This is the tier that is mostly not a card.** 16 clauses print it; the rest is
*derived* from Tier 2, and the printed rulings say so in as many words:

- Platinum Emperion: "If a cost would include causing you to gain life … that
  cost can't be paid." The card never says "can't pay".
- Solemnity: "If the cost of an ability … requires putting counters on an
  artifact, creature, enchantment, or land, that cost can't be paid."
- Abyssal Persecutor: "a player can't pay an amount of life that's greater than
  their life total" — and note this one is CR 118.4/119.4, *not* 614.17b, so the
  derivation has two sources.

**The machinery this needs, sized.** A `Cost → Vec<GameAction>` projection.
`types/costs.rs::Cost` is a **closed 10-variant enum** and every variant maps
onto an existing `GameAction` or onto nothing:

| `Cost` | Projects to |
|---|---|
| `Tap` / `Untap` | `GameAction::Tap` / `Untap` |
| `PayLife(n)` | `LoseLife` |
| `SacrificeSelf` / `Sacrifice(..)` | `ZoneChange { cause: Sacrificed }` |
| `Discard(..)` | `ZoneChange { from: Hand, to: Graveyard, cause: Discarded }` |
| `RemoveCounters` / `AddCounters` | `RemoveCounters` / `AddCounters` |
| `Mana(..)` | nothing — CR 106.6b mana restrictions are a different rule |

So CR 614.17b is a ten-arm match over a closed enum, and it is exact rather than
approximate. That is the whole cost of the derived path.

#### Does Yasharn need a field, or just a well-placed check?

Mostly the latter, and an earlier draft of this section muddled it by calling
the field `purpose` — a word vague enough to hide the question.

Yasharn reads "Players can't pay life or sacrifice nonland permanents **to cast
spells or activate abilities**". **Every cost payment in the CR is part of
casting a spell or activating an ability** (CR 601.2f–h and CR 602.1b–f; costs
do not exist outside those, and additional and alternative costs are paid inside
them). So the trailing clause is not narrowing anything the check could
otherwise see — it is *distinguishing costs from effects*, which is what
Yasharn's ruling spells out: "Other things may still cause players to pay life
or sacrifice creatures, such as a resolving spell or ability." **Put the check
at the cost-payment step and the clause is satisfied by where the check is.**
No field.

One card breaks that and it is worth a field rather than a special case.
**Karn's Sylex**: "Players can't pay life to cast spells **or to activate
abilities that aren't mana abilities**." The mana-ability exclusion is real
(CR 605.1a), and its ruling turns on it. So the arm carries a small set of
payment *sites*, not a `purpose`:

```rust
PayCost { player: PlayerRef, cost: CostFilter, sites: PaymentSites }
// PaymentSites: CAST | ACTIVATE_MANA | ACTIVATE_NONMANA, defaulting to all
```

Three flags, and the call site already knows which one it is — `engine/mana.rs`
distinguishes a mana ability today.

**One consequence to wire, not to model.** Both Yasharn's and Karn's Sylex's
rulings say the same thing: "that spell or ability **can't be cast or
activated**". So a Tier-1e prohibition does not merely fail the payment — it
propagates up to CR 601.3 and makes the whole cast illegal, which is the same
rewind path §4.3 needs. Tiers 1b and 1e share an exit.

**And one floor that is not a restriction at all.** "Players may always pay 0
life, even if an effect says they can't pay life" (Karn's Sylex's ruling,
CR 118.5 — paying {0} is an acknowledgement, not a resource transfer). A
`Cost::PayLife(0)` must never consult this tier.

**Authored consumers.** Yasharn, Angel of Jubilation, Karn's Sylex. The ten
"this mana can't be spent to…" cards are **not** here: they are CR 106.6b, a
property of mana in the pool, and belong with the mana system and ticket `T12d`.
**Sigarda and its family are not here either** — §2.6 moved them to Tier 2,
because they restrict an *effect* that causes a sacrifice, and you never pay
costs for an opponent's spell.

### 4.7 Tier 3 — withholding a replacement effect

**Rule.** CR 701.19c: effects that say a permanent can't be regenerated "cause
regeneration shields to **not be applied**" — they block application, not
creation. CR 615.12 is the same shape for prevention: "prevention effects
applied to unpreventable damage prevent nothing, but any additional effects they
have will take place."

**Site.** `gather.rs::push_if_applicable`, where CR 701.19c already lives.

**What changes.** The `game.cant_be_regenerated.contains(&id)` lookup becomes
`is_prohibited(game, &Query::ApplyReplacement { .. })`, and the
`GameState` `HashSet` goes away (§6.4). `ReplacementDef::is_regeneration` gains
its second reader — which its own doc calls "the smell" — and that is correct
here rather than a violation: the field's doc says it is a *rules-level
classification* that CR 701.19c needs in order to recognise a shield, and
CR 615.12 needs the same recognition for prevention. Phase RD should widen the
`bool` to a small `ReplacementKind` at that point rather than adding a second
`bool`.

**Consumers.** 155 clauses of "can't be regenerated" and 32 of "damage can't be
prevented", of which Skullcrack is both halves plus a Tier-2 restriction in one
instant: *"Players can't gain life this turn. Damage can't be prevented this
turn. Skullcrack deals 3 damage…"* — three restrictions, two tiers, one card,
one primitive.

### 4.8 Tier 5 — CR 101.3's "if you can't" is the query side

74 clauses ask whether a player *could* do something: "Each player sacrifices a
creature. **If they can't**, they discard a card" (Plaguecrafter, Doom Foretold,
Cruel Reality, Xathrid Demon).

**This is not a sixth mechanism — it is `is_prohibited` with the sign flipped**,
plus CR 101.3's "any part of an instruction that's impossible to perform is
ignored". It is listed as a tier because the census must classify it somewhere
and because it is the strongest single argument for §3.5's *one* predicate: if
each enforcement point grew its own inline check, "can they?" would have no
function to call and would be re-implemented per card.

**It is not buildable yet.** "If they can't, X" needs `Effect::Conditional`,
which `resolve.rs` currently errors on. Recorded here so the dependency is not
rediscovered; the predicate is what this design owes it, and Phase 6 owes the
rest.

---

## 5. Interaction with the replacement pipeline

### 5.1 CR 614.17a — a "can't" must exist before the event

> 614.17a "Can't" effects must exist before the appropriate event occurs — they
> can't "go back in time" and change something that's already happened.

Satisfied by construction, and for the same reason `gather` satisfies CR 614.4:
the sweep runs against live state at the moment of proposal, and there is no
"go back in time" path because there is nowhere else to ask. Solemnity's ruling
is the worked case: "If an artifact … would enter the battlefield with counters
on it at the same time that Solemnity enters the battlefield, Solemnity doesn't
stop it from getting those counters."

**One consequence for batches.** `execute_batch_inner` decides every member's
replacements against one board, before performing any of them (CR 704.3). The
restriction check has to sit on the *same* side of that split — decided against
the pre-batch board — or Solemnity's ruling inverts for a simultaneous entry.

### 5.2 CR 614.17c — the blocked event, and what may still apply

> 614.17c If an event can't happen, it can only be replaced by a
> self-replacement effect (see rule 614.15).

RB already implements this: `gather(game, &event, ctx, blocked)` discards
everything outside `ReplacementClass::SelfReplacement`, and today that means the
list is empty because nothing produces one. `rb-review.md` K2 records that the
blocked path therefore always drops the event, which is correct now and stops
being correct the moment CR 614.15 gains a producer.

**This design changes nothing here** and depends on it staying true. It is worth
recording that the interaction is *live*, not theoretical: Solemnity's printed
ruling —

> "If a replacement effect allows a player to modify or replace an event by
> putting counters on a[n] … object, that player may apply that replacement
> effect. Counters won't be put on the object, but if the original event is
> entirely replaced (such as by applying Soul-Scar Mage's replacement effect),
> the original event won't happen."

— is exactly a Tier-2 restriction on the *output* of a replacement, resolved by
re-asking `is_blocked` on the substituted event. RB's loop already produces that
answer; this design must not move the check outside the loop to get it.

### 5.3 CR 614.17d — the look-ahead frame, and what RC-4 actually needs

> 614.17d Some "can't" effects modify how a permanent enters the battlefield or
> whether it can enter the battlefield. … To determine which "can't" effects
> apply, check the characteristics of the permanent as it would exist on the
> battlefield …

This is the review's stated deadline and it is **narrower than it looks**. The
census finds six "can't enter" clauses, of which one is a dungeon (Undercity //
The Initiative, "you can't enter this dungeon unless…") and is not this rule.
That leaves the entire printed population at **five cards**:

| Card | Clause |
|---|---|
| Grafdigger's Cage | Creature cards in graveyards and libraries can't enter the battlefield. |
| Kunoros, Hound of Athreos | Creature cards in graveyards can't enter the battlefield. |
| Soulless Jailer | Permanent cards in graveyards can't enter the battlefield. |
| Weathered Runestone | Nonland permanent cards in graveyards and libraries can't enter the battlefield. |
| Worms of the Earth | Lands can't enter the battlefield. |

**And a printed ruling says four of the five do not use the frame at all.**
Grafdigger's Cage: *"Look at the card as it exists in your graveyard to
determine whether it can enter the battlefield. For example, Sculpting Steel can
be put onto the battlefield as a copy of a creature, but Phyrexian Metamorph
can't be put onto the battlefield, even if it would copy a noncreature
artifact."* The Cage's subject is the **card in the source zone**, not the
permanent as it would exist — so it is `Restriction::Event { pattern:
EventPattern::EnterBattlefield { from: Some(Graveyard | Library), object:
Some(filter) } }` evaluated against the card where it is, and it needs no
overlay.

Kunoros, Soulless Jailer and Weathered Runestone are the same shape with a
different filter. **Worms of the Earth is the one card in the population whose
subject is the entering permanent** rather than the card in its source zone, and
it is therefore the only printed customer of the frame — and even it is a
type-line check that only diverges for something like a Dryad Arbor-shaped
object whose land-ness is decided on the battlefield.

**So RC-4's dependency on this document is exactly §7's phase RS-1** — the
Tier-2 spine — and not the rest. RS-1 gives RC-4 a restriction to ask; the
*frame* half is one call site inside RC-4's own overlay, and RC-4 should build
it because it is building `compute_as_entering` anyway, not because a card is
waiting on it.

**One boundary to preserve.** CR 614.17d's "may come from the permanent itself
if they affect only that permanent (as opposed to a general subset of permanents
that includes it)" is the same `AffectedSet::SourceOnly` vs. `Filter`
distinction CR 614.12 draws, and `replacement-architecture.md` §5 already
records that collapsing those variants breaks 614.12 silently. It now breaks
614.17d silently too. Same rule, second customer.

### 5.4 CR 101.2a — where the layer system keeps its own

CR 101.2a takes ability addition and removal *out* of CR 101.2 and hands it to
CR 113.11. So Tier 4 is not this document's, and the boundary is a rule rather
than a judgement call (§3.6). The concrete consequence: an Archetype of
Endurance effect is a Layer 6 rule that a `GrantKeywordFlag(Hexproof)` row does
not apply — it is *not* `Restriction::Event`, and it must not be modelled as
one, or `compute_characteristics` acquires a reader outside the layer walk.

`ATOM-101.2a-001` is already covered by two tests in the layer suite, which is
the evidence the boundary is where the corpus puts it too.

---

## 6. The four costumes, answered

### 6.1 A1 — "can't" effects are not scoped

Answered by §2 (measured) and §3 (modelled). The review's motivating cards land
as follows, and the spread across tiers was the point it was making:

| Card | Tier | Arm |
|---|---|---|
| Abyssal Persecutor | 2 | `Event { pattern: PlayerWins/PlayerLoses }` (Phase RE vocabulary) |
| Aggressive Mining | 1b | `PlayLand` — a *special action*, per its ruling |
| Grafdigger's Cage | 1b + 2 | `Cast { from: Graveyard\|Library }` **and** `Event { EnterBattlefield }` — one card, two tiers |
| Conduit of Worlds | 1b | the permission half of `PlayLand`, plus a self-imposed `Cast` restriction |
| Rakdos, Lord of Riots | 1b | `Cast { spell: SourceOnly }` |
| Yasharn, Implacable Earth | 1e | `PayCost { purpose: CastOrActivate }` |

### 6.2 A2 — `is_blocked`'s match grows one arm per "can't"

**It does not, once the arm is data.** Tier 2's entire population is
`Restriction::Event { pattern, affected }`, and `EventPattern` is already
committed to growing on exactly one axis (one arm per `GameAction` variant).
`is_blocked`'s `match` becomes a sweep and stops growing at all.

This is also the answer to the archive plan's L15, which proposed
`PlayerActionRestriction` with variants `CantCastSpells(PlayerId)`,
`CantGainLife(PlayerId)`, `CantAttack(PlayerId)`,
`CantActivateAbilities(PlayerId, Option<String>)`, `CantDrawExtraCards(PlayerId)`.
Read against §2.1 that enum is one variant per *card* wearing a rule's name:
`CantGainLife` and `CantDrawExtraCards` are the same Tier-2 arm with different
`EventPattern`s, and `CantActivateAbilities`' `Option<String>` card-name filter
is a `PermanentFilter` spelled by hand. L15 was never built; this document
supersedes it and `codebase-state.md` records that.

### 6.3 A3 — is there a primitive per "can't" stopper?

**No — one primitive, and it is the exact analogue of `Primitive::Regenerate`.**

```rust
/// Create a restriction from a resolution.
///
/// The atom's `EffectRecipient` supplies the subject, exactly as it does for
/// every other primitive — which is what lets "Destroy target creature. It
/// can't be regenerated." name its target without a new `AffectedSet` variant.
///
/// **`Duration` is authored, never inferred** — CR 608.2c (§9 finding 1).
Restrict(Box<RestrictionDef>, Duration),
```

Three cards, three durations, one primitive — which is the test of whether the
`Duration` argument earns its place:

| Card | Restriction | `Duration` |
|---|---|---|
| Wrath of God | "They can't be regenerated" | scoped to *this resolution's* destruction (CR 608.2c) — **a variant that does not exist yet** |
| Skullcrack | "Players can't gain life this turn" | `UntilEndOfTurn` |
| Solemnity | "Counters can't be put on …" | not a resolution at all — a static ability, discovered, no primitive |

`Primitive::CantBeRegenerated` folds into the first row. Skullcrack is *three*
`Restrict` atoms in a `Sequence` — two restrictions and a damage atom, across
two tiers — with zero engine changes, which is the §0 commitment demonstrated on
the hardest printed case in the corpus.

**Why a primitive rather than an `Effect::Restriction` node.** The same reason
`Effect::Replacement` errors today: a restriction created by a resolution needs
a duration, and there is nowhere honest to read it from unless the authoring
surface carries it. CR 608.2c makes that structural rather than convenient — the
rule tells the *reader* to determine scope from English, so no node that lacks a
duration field could ever be filled in correctly by the engine.

**And most restrictions need no primitive at all.** 563 of the 2,034 clauses
carry an explicit duration and are therefore resolution-created; the other
1,471 are source 1 or source 3 — discovered off the effective ability list or
synthesized from a keyword — and never resolve. One primitive serves the 563
and nothing serves the 1,471, which is the right ratio for a design whose
premise is that a card is data.

### 6.4 A4 — a `HashSet<ObjectId>` on `GameState` for one rule

**Replaced by a `RestrictionRegistry`**, structurally the same as
`state/replacement_effects.rs`'s `ReplacementRegistry`: rows with a `source`, a
`controller`, a `Duration`, a `created_on_turn`, and the same three expiry hooks
(`remove_by_source`, `remove_expired_at_cleanup`, `remove_expired_at_turn_start`).
`GameState::cant_be_regenerated` and `turns.rs:138`'s hand-rolled `.clear()` both
go away.

**And that closes a real gap, which is the argument for doing it rather than
leaving A4 as a note.** `turns.rs:138` clears the set at cleanup, i.e. treats
"It can't be regenerated" as until-end-of-turn, under a comment asserting it is
"a this-turn fact" with no rule cited. The rule that governs it is **CR 608.2c**,
and it names this exact card text:

> 608.2c … In some cases, later text on the card may modify the meaning of
> earlier text (for example, **"Destroy target creature. It can't be
> regenerated"** …) Don't just apply effects step by step without thinking in
> these cases — read the whole text and apply the rules of English to the text.

So "It can't be regenerated" is **not a continuous effect at all**. It modifies
the meaning of the destruction instruction it follows: *destroy it in a way
regeneration cannot save it*. It is a one-shot effect scoped to that
destruction, CR 611.2 never engages, and CR 611.2a's "until end of game" is a
red herring — an earlier draft of this document reached for it and was wrong.

**RB is still wrong, in the opposite direction and by a smaller margin.** Same
scenario: Wrath of God destroys a creature carrying a CR 122.1c shield counter,
the shield replaces the destruction, the creature survives. Under 608.2c it is
not flagged at all afterwards and may be regenerated **immediately, the same
turn**. RB withholds every shield from it until cleanup. Narrow, but reachable,
and produced by exactly the hardcoded duration this section removes.

**The scope RS-1 needs, and it is a new one.** `Duration` today is
`{UntilEndOfTurn, UntilYourNextTurn, WhileSourceOnBattlefield, WhileEnchanted,
WhileEquipped, Indefinite}` — nothing says "for the event this resolution is
about to perform". `Primitive::Restrict` gets that as its first customer.
`Primitive::Regenerate`'s twin, which *is* CR 701.19a's genuinely turn-scoped
shield, keeps `UntilEndOfTurn`; the two sit next to each other in `resolve.rs`
with different durations for a reason the CR gives.

**The design consequence is bigger than the fix, and §9 finding 1 states it:**
CR 608.2c tells the reader to "apply the rules of English to the text". No
engine can derive that. Scope has to be **authored**, per card, which is the
strongest argument in this document for `Duration` being a field on
`Primitive::Restrict` rather than something the primitive infers.

---

## 7. Sizing and the phase plan

**Sized before writing, and split in the doc.** `CLAUDE.md`'s git-workflow
section is explicit that RB's failure was not keeping it whole but not counting
first, and that every PR in a split must carry at least one consumer of what it
builds. Both rules are applied below. Sub-phases are numbered `RS-1` … `RS-4`.

| PR | Shape | Measured size | Risk |
|---|---|---|---|
| **RS-0 — the shared duration registry** | Lift the ten methods `ContinuousEffectRegistry` and `ReplacementRegistry` already duplicate into one type; migrate both. No new behaviour | **10** shared methods, **2** of them (`remove_expired_at_cleanup`, `remove_expired_at_turn_start`) character-for-character identical today across **259** + **~90** lines | low — pure refactor with two existing test suites as the check. §9 finding 7 |
| **RS-1 — the spine + Tier 2** | `RestrictionDef`, `Restriction::Event`, the registry, the sweep, `is_prohibited`; `Primitive::Restrict`. Consumers: indestructible moves onto it, `CantBeRegenerated` folds in | **3** production sites replaced (`pipeline.rs:53`, `gather.rs:228`, `resolve.rs:638`) and **1** enforcement site for indestructible, which is `pipeline.rs:56` and nothing else; **1** `GameState` field + its init + **1** `turns.rs` clear deleted. New code sized against its two structural twins: `state/replacement_effects.rs` is **259** lines and `gather`'s sweep + gate is ~**120** of `gather.rs`'s 556 | low — it *deletes* two bespoke mechanisms and adds no new call site |
| **RS-2 — the read-side choice points** | `Cast`, `PlayLand`, `ActivateAbility`, `BeTargeted`. Consumers: hexproof + shroud (`T22`), Grafdigger's Cage, Aggressive Mining | **6** enforcement sites (`cast.rs` legality, `legality.rs::playable_lands`, `activatable_abilities`, `priority.rs` re-derivation, `cast.rs::activate_ability`, `targeting.rs::validate_targets`) + **2** enumeration sites that must agree (`enumerate_legal_selections`, `has_any_legal_choice`) | **medium-high** — the three ability-index sites are the `CLAUDE.md` invariant, and a restriction can now cause a CR 601.2 rewind (§4.3) |
| **RS-3a — combat, the predicate half** | `Attack`, `Block`, `BeBlocked` as per-creature restrictions. Consumers: menace, the evasion family, landwalk, protection's blocking half, Defender re-expressed as data | **2** validators rewritten (`validate_attackers`, `validate_blockers` + `can_block`), **2** enumerators (`legal_attackers`, `legal_blockers`). Covers **1,249 of 1,262** Tier-1a clauses — the 1,200 per-creature plus the 49 per-attacker blocker counts | medium — large surface, but every check is local |
| **RS-3b — combat, the solver half** | `Requirement`, the coupling graph, the bounded search (§4.2). Consumers: goad, Silent Arbiter, "can't attack alone" | **13** cross-creature clauses + **2** global-cap cards + ~**150** requirement cards. New code, not a migration | **highest** — CR 508.1d is NP-hard in general; this is where the bounded-exact choice and its cap live |
| **RS-4 — costs** | `PayCost`, the CR 614.17b derivation, the `Cost → GameAction` projection. Consumers: Yasharn, Platinum Emperion | **10**-arm projection over a closed enum; **2** payment sites (`costs.rs`, the `ChooseAdditionalCosts` path) | low, and it is last because 614.17b's derived half needs Tier 2 *and* the authored half is 16 clauses |

**Ordering, and the one hard constraint.**

> **RS-1 must land before RC-4** (§5.3). Nothing else in this document blocks
> anything in `replacement-architecture.md`.

RS-0 is a pure refactor and lands before RS-1 so that RS-1 *uses* the
abstraction rather than inventing it. RS-1 is small, deletes more than it adds,
and can go in parallel with RC-1 through RC-3. RS-2 and RS-4 are independent of
the RC/RD/RE line entirely.

**RS-3b should not start before the CR 613.8 dependency cluster** (`CLAUDE.md`
critical-path item 7): evasion is cumulative (CR 509.1b), so a solver reading
effective characteristics under timestamp-only ordering will produce answers
that change when 613.8 lands. **RS-3a does not have that problem** — a
per-creature predicate reads the same frame every other combat check already
reads today — which is the second reason to split RS-3 in two rather than the
first. The split lets 1,249 of 1,262 clauses land without waiting on item 7.

**What each PR must not do.** RS-1 must not touch a choice site; RS-2 must not
touch combat; RS-3a must not attempt CR 508.1d; and no RS phase carries the
"unless" clauses (§2.4). Each of those is the seam where this would otherwise
become one 5,000-line PR again.

### 7.1 The interleaved order, end to end

**`CLAUDE.md` → "Critical path to v1" is the authority; this is a reading of
it**, written because two phase families now run beside each other and the
useful question is "what do I do next", not "what does each doc own".

**Hold two things in your head, not eleven.**

> **Track R** turns events into things cards can modify (RC → RD → RE).
> **Track S** turns rules into things cards can forbid (RS-0 → RS-4).
> They meet **once**: RS-1 before RC-4. Everything else is free ordering.

| # | Do this | Why here |
|---|---|---|
| 1 | Finish #62 — themes G, then B | `rb-review.md`'s plan; both are on the branch already |
| 2 | **RS-0** — the shared duration registry | Pure refactor, two existing suites as the check. Cheapest thing on the list and it makes 3 and 6 smaller |
| 3 | **RS-1** — the Tier-2 spine | Net-deleting. **This is the join**: RC-4 cannot start without it |
| 4 | **RC-1** — delete the early stack pop | Independent of everything; the safest shape the project writes |
| 5 | **RC-2** — `EnterBattlefield` as an event | 773 enters-tapped cards, and the first thing that makes Track R visible in a game |
| 6 | Themes C + E — naming and hygiene | After the merge, with theme G's comment rule in hand |
| 7 | **RC-3** — the membership gate | One line, hottest path; its deliverable is the measurement |
| 8 | **RC-4** — the overlay | The join is satisfied by 3. CR 614.17d asks a question RS-1 gave it |
| 9 | **RS-2** — casting, activating, targeting | Unlocks hexproof/shroud/protection. Independent of Track R entirely; can slot anywhere from 4 onward |
| 10 | **Cost modification** — its own small phase | `replacement-architecture.md` §9 asks for it; commander tax runs through it; RS-4 wants it nearby |
| 11 | **RS-4** — costs (CR 614.17b) | Needs RS-1 for the derived half and reads better after 10 |
| 12 | **RS-3a** — combat, the predicate half | 1,249 of 1,262 Tier-1a clauses. Does **not** need item 7 |
| 13 | **CR 613.8** — the dependency cluster | `CLAUDE.md` item 7, hard back-stop before Phase 8 breadth |
| 14 | **RS-3b** — combat, the solver half | After 13, because evasion is cumulative |
| 15 | **RD**, then **RE** | Damage/prevention, then the remaining event kinds. RD gives Tier 3's other half something to withhold |
| 16 | **CR 603** — triggers | `CLAUDE.md` item 6; consumes the performed-event stream both tracks feed |

**Three sanity checks on that order.** Track R's first four steps are already
sized in `replacement-architecture.md` §9 and unchanged by this document. The
only reordering this document caused is item 3 moving ahead of item 8. And
items 9–12 are genuinely free — if a card you want to play needs hexproof more
than it needs ETB replacements, do 9 first and nothing breaks.

**What is deliberately not on this list.** The 149 "unless" clauses, CR 613.10,
CR 113.11's Tier 4, and the Conspiracy in §2.5. Each has a home elsewhere or no
home yet, and each is named in §9 or §10 so it is findable rather than lost.

---

## 8. Testing — the atoms this owes

The corpus already has the spine, and almost none of it is covered. From
`plans/atomic-tests/sessions/`:

| Atom | Rule | Tier | Status |
|---|---|---|---|
| `ATOM-614.17-001` | 614.17 | 2 | PARTIAL (`test_indestructible_is_a_cant_ahead_of_the_pipeline_not_a_replacement`) |
| `ATOM-614.17a-001` | 614.17a | 2 | uncovered — "can't" must pre-exist the event |
| `ATOM-614.17b-001` | 614.17b | 1e | uncovered |
| `ATOM-614.17c-001` | 614.17c | 2 | uncovered |
| `ATOM-614.17d-001` | 614.17d | 2 | uncovered — RS-1 / RC-4 |
| `ATOM-701.19c-001` | 701.19c | 3 | ✅ covered (RB) |
| `ATOM-615.12-001/002` | 615.12 | 3 | uncovered — Phase RD |
| `ATOM-601.3-001` | 601.3 | 1b | uncovered, ticketed **L15** |
| `ATOM-601.3a-001` | 601.3a | 1b | uncovered — Void Winnower: begin the cast, then reject the announced X (§4.3) |
| `ATOM-508.1c-001`, `ATOM-508.1c-002` | 508.1c | 1a | `-001` marked `ALREADY-IMPL`, **zero coverage**; `-002` is the "can't attack alone" pair |
| `ATOM-509.1b-001/002/003` | 509.1b | 1a | `-002` ticketed `T21b` (cumulative evasion); `-001`/`-003` marked `ALREADY-IMPL`, zero coverage |
| `ATOM-613.10-001`, `ATOM-613.11-001/002` | 613.10/613.11 | — | uncovered, ticketed **L15** |
| `ATOM-502.3-002`, `ATOM-703.4c-002` | 502.3 / 703.4c | 2 | **in `specdb owed` against shipped Phase 5-Pre** — "doesn't untap" restrictions |
| `ATOM-113.11-001` | 113.11 | 4 | uncovered — layer system's (§3.6) |

Four things to act on rather than read past:

1. **`ATOM-502.3-002` and `ATOM-703.4c-002` are in `specdb owed` against a
   shipped phase.** They are untap restrictions — Tier 2 — and RS-1 is where
   they get covered. That is a live gate, not a report.
2. **The rest of the restriction atoms escape `owed` by ticket name.** `owed`
   filters `ticket LIKE 'NEW%'` unless given `--all`: 38 rows by default, 550
   with `--all`. Every atom above ticketed `L15` or `T21b` is in the 550 and
   not in the 38. That is the filter working as designed — a ticket that names
   a future phase *is* "explicitly deferred with a reason written down" — but
   it means **`owed` alone does not show the size of this work**, and a session
   sizing RS-2 or RS-3 should run `owed --all` and grep.
3. **Three atoms marked `ALREADY-IMPL` have zero coverage** (`508.1c-001`,
   `509.1b-001`, `509.1b-003`). The implementations exist (`Defender`, flying);
   the `COVERS` links do not. RS-3 should add them, and until then the corpus
   overstates what is proven.
4. **`specdb.py`'s `SHIPPED_PHASES` gains `Phase RS` when RS-4 lands**, which is
   what arms the gate for this work.

---

## 9. Findings and open questions

1. **CR 608.2c means an engine cannot derive a restriction's scope — it has to
   be authored.** ✅ *Answered by the owner 2026-08-27; this replaces an earlier
   draft that reached for CR 611.2a and got it wrong.*

   "Destroy target creature. It can't be regenerated" is named **in CR 608.2c
   itself** as an example of later text modifying the meaning of earlier text.
   It is part of the destruction instruction — a one-shot effect scoped to that
   event — not a continuous effect, so CR 611.2 never engages. §6.4 has the
   consequence: `turns.rs:138`'s cleanup clear is too *broad*, not too narrow,
   and the fix is a resolution-scoped `Duration` variant that does not exist
   yet.

   **The general lesson is the one worth keeping.** CR 608.2c's instruction is
   literally *"apply the rules of English to the text"*. That is the CR handing
   scope determination to a human reader, and it is unusual — most of the rules
   this engine implements are mechanical. Two rules follow, and they should
   outlive this finding:

   - **Never infer a restriction's duration from its shape.** Two cards with
     identical restriction text can have different scopes because of the
     sentence before them. `Primitive::Restrict` takes a `Duration` as an
     argument for the same reason `Primitive::ModifyPowerToughness` does.
   - **Where the CR delegates to English, the card file is the only honest
     place to record the reading.** Add a rule cite in the card's comment when
     the reading is not obvious, so the next person does not re-derive it —
     this finding took a CR search and an owner correction to settle.

   The rest of §9 numbering is unchanged so citations to findings 2–6 still
   resolve.
2. **CR 601.3a is not a forward-looking check** (§4.3), and an earlier draft
   claimed it was and priced it against the CR 614.12 frame with no evidence.
   Void Winnower's rulings show it is over-approximate at the gate and exact
   after announcement, which is `candidate_priority_actions`' existing
   contract. The one new thing is that a **restriction can now cause a CR 601.2
   rewind**, where today only an unpayable cost can. RS-2 should be reviewed for
   that, not for a search.
3. **`ReplacementDef::is_regeneration` gains its second reader** (§4.7). Its own
   doc says a second reader "would be the smell". It is the right reader, but
   Phase RD should widen it to a `ReplacementKind` rather than adding a parallel
   `is_prevention: bool`.
4. **Permission and prohibition are the same query with opposite signs.**
   Conduit of Worlds *grants* the land play Aggressive Mining removes; CR 601.3
   is phrased as "a rule or effect allows … and no rule or effect prohibits".
   This document models only the prohibition half. Whether the permission half
   should share the type is open, and the answer probably arrives with the
   `PlayLand` consumers rather than before them. Do not build both speculatively.
5. **`lands_per_turn` is still a raw field** (`player.rs:23`, read directly by
   `player.can_play_land()`). It is L15's step 4 and it is *not* a restriction —
   it is a computed player-scoped value. Recorded here only so that superseding
   L15 does not lose it; it belongs with the cost-modification phase, which is
   the other CR 613.11 consumer.
6. **CR 613.10 (player-affecting continuous effects) has no home doc.** It is
   adjacent to this work — "an effect might give a player protection from red" —
   and neither `layers-architecture.md` nor this file owns it. Whichever phase
   builds `PlayerRef`-scoped continuous effects should claim it.

7. **RS-1 makes this the third duration registry, and that is the moment to
   abstract the shared half.** Not the whole registry — the *expiry* half, which
   is a rules concern rather than a storage one.

   **Measured.** `ContinuousEffectRegistry` (`state/continuous_effects.rs`) and
   `ReplacementRegistry` (`state/replacement_effects.rs`) already share ten
   methods: `new`, `add`, `remove`, `remove_by_source`, `iter`, `is_empty`,
   `len`, `retain_effects`, `remove_expired_at_cleanup` and
   `remove_expired_at_turn_start`. The last two are **character-for-character
   identical logic** — same `Duration` match, same `active_player` /
   `created_on_turn` guard, same "suppress unused until multi-turn durations
   arrive" note. A fourth is coming with delayed triggers (CR 603.7).

   **Why this clears the two-customers guard.** The project's rule is "don't
   refactor speculatively", and this is not speculation: three concrete
   customers, identical semantics, and the semantics are **CR 514.2 and
   CR 613.7**, not an implementation detail. The failure mode duplication has
   here is specific — a duration bug fixed in one registry and not the other two
   is invisible, because each has its own tests and each passes.

   **What to abstract, and what not to.** A `DurationRegistry<T>` (or a shared
   `expiry` module over `&mut Vec<Row<T>>`) carrying `id` allocation, `source`,
   `controller`, `duration`, `created_on_turn`, and the three expiry hooks.
   **Not** the query side: `effects_in_layer`'s layer-indexed cache,
   `summary()`'s scope gate, and `Uses::Once`'s row removal are each specific to
   one customer and are where the registries genuinely differ.

   **When.** Its own small PR, immediately before RS-1, migrating the two
   existing registries first so RS-1 is a *user* of the abstraction rather than
   the thing that invents it. That ordering is what keeps it from becoming a
   generic nobody else fits — the same reason `replacement-architecture.md`
   insists every PR in a split carries a consumer.

8. **`Restriction` needs a `Requirement` sibling and RS-3 cannot ship without
   it** (§4.2). CR 508.1d's maximisation is defined over both, so a restriction
   solver that cannot see "attacks each combat if able" produces wrong answers
   rather than incomplete ones. ~150 printed cards, goad the important one for
   Commander. Named here because it is the one place this document's scope has
   to grow beyond its title.

---

## 10. Explicitly out of scope

- **Tier 4 (CR 113.11)** — `layers-architecture.md`'s, by CR 101.2a (§3.6).
- **Tier 6** — CR 601.2b announcement caps and CR 601.2f cost floors; two
  existing homes (§3.6).
- **The 149 "unless" clauses** — need `Effect::Conditional` (Phase 6) (§2.4).
- **CR 106.6b mana-spend restrictions** — the ten "this mana can't be spent
  to…" cards belong with the mana system and ticket `T12d`, not here.
- **Requirements ("attacks if able")** are *in* scope for RS-3 and only RS-3,
  because CR 508.1d makes them inseparable from restrictions. They are named
  here so that no earlier phase picks them up.
- **Restrictions whose source is in another zone** — source 2, deferred on the
  same terms and blocked on the same work as `gather.rs:11`'s.

---

## 11. Documents this owes

Written as part of the PR that lands this file, not after:

- `plans/handoffs/rb-review.md` — theme A rows A1–A4 closed, pointing here.
- `CLAUDE.md` — one authority-table row. **One row and no invariant**: the
  restriction invariants belong here until something is built, and
  `rb-review.md` G1 is trying to shrink that file, not grow it.
- `plans/codebase-state.md` — the L15 supersession, §9 finding 1's duration
  gap, and the unenforced `KeywordFlag`s from §2.3.
- `plans/cards-unlocked-ledger.md` — a `Part 4` block, one row per RS phase.
- `plans/replacement-architecture.md` §4.4 — a pointer here, and the RC-4
  dependency named as RS-1 rather than as "theme A".

When RS-4 lands: add `Phase RS` to `specdb.py`'s `SHIPPED_PHASES`, and delete
`plans/references/cant-census.py` if §2 has no remaining customer.
