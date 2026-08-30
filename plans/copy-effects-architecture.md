# Copy Effects — CR 707, 712, 708, 729, and Layer 1

> **Status:** design, authored 2026-08-29 in answer to `codebase-state.md` →
> "Was the critical path complete? — audited 2026-08-27", which ran the "can't"
> discovery backwards as a detector and found this gap. No code written yet.
> **Authority:** the copy model — what a copiable value *is* in this engine,
> which mechanism each population uses, where a copy is produced, and the phase
> sequencing for CR 707 / 712 / 708 / 729 and CR 613.2's Layer 1. Where this
> contradicts `codebase-state.md`, that file wins on *what exists*; this file
> wins on *what is being built*. `CLAUDE.md` → "Critical path to v1" still owns
> the ordering against the other phases (this document proposes item **5c**).
> **Companions:** `layers-architecture.md` owns CR 613 and this document is a
> customer of its §5.2 termination argument and its §6 CDA provenance table —
> read both before §3. `replacement-architecture.md` §2a is as-built truth for
> every type Phase RB shipped and nothing here contradicts it; §5/§5c own the
> ETB look-ahead frame that §5.1 below re-scopes.
> **Supersedes:** ticket **D5** ("copy system") in
> `plans/archive/implementation-plan-final.md`, which 20 corpus atoms still cite.
> D5 was never built; nothing has to be unwound.

---

## 0. The budget — why this stays tight

The copy corpus is **1,093 clauses over 752 cards** that print the word, plus
**517 double-faced cards, 304 face-down producers and 34 mutate cards** that do
not. Four commitments, each falsifiable.

**One. A card is data, not code.** Adding a copy card touches `src/cards/*.rs`
and nothing else. The failure this rules out by name is a `Primitive::Clone`,
`Primitive::TokenCopy`, `Primitive::CopySpell` family — one primitive per
mechanic, which §3.3 replaces with one capture and four *producers* that all
carry the same payload.

**Two. There is one payload and it is a snapshot.** CR 707.2b and 707.2c make
copying a point-in-time capture, not a live reference. That single fact decides
the type (§3.2), refutes the CR 613.8 blocker (§5.2), keeps
`layers-architecture.md` §5.2's termination argument intact (§5.2), and is the
one invariant whose violation would move this whole track behind critical-path
item 7. It is stated as a rule, not an implementation detail.

**Three. The claim was measured, not asserted.** Every card was pulled and each
copy clause classified by *which engine mechanism would have to produce it*
(`plans/references/copy-census.py`, run 2026-08-29). §2.1's answer is six
mechanisms, of which the engine has **none**. The residual after classification
is 2 clauses and they are printed in §2.6. The script also audits its own rule
labels against `MTG-Rules/versions/tmnt.txt` at startup, which is the direct
answer to `rb-review.md` theme J1 — a census whose labels are wrong is worse
than no census, because it is citable.

**Four. Performance has one designated lever, and it is a gate that already
exists and is about to become unsound.** A Layer 1 row sits at the top of
`compute_characteristics`, the hottest path in the engine. It also defeats two
ETB-time scans that today decide what `gather` and the continuous-effect
registry ever look at (§4.7). Fixing those is not a follow-up; it is CV-1's
deliverable, because the failure mode is a card that silently does nothing.

**What this does *not* claim.** CR 712 (double-faced cards) is a second card
model, not a copy mechanism, and §4.5 scopes it as such rather than pretending
Layer 1 absorbs it. CR 729 (merging) and CR 708 (face-down) are named, sized and
given phases of their own (**CV-6**, **CV-7**) rather than a type surface here
— with the one shape constraint they impose recorded in §3.2 so the door stays
open. And CR 707.10's
spell copies touch no layer at all; putting them in a "copy phase" because they
share a word would be the same error the tier table exists to prevent.

---

## 1. Verdict — the seam, answered first

### 1.1 The CR decides it, and the answer is "Layer 1 owns application, CR 707 owns the payload"

The question the audit asked was whether Layer 1 or CR 707 owns copying. It is
not a choice; CR 613.2c settles it in one sentence:

> **613.2c** After all rules and effects in layer 1 have been applied, the
> object's characteristics are its copiable values. (See rule 707.2.)

So **copiable values are the *output* of layer 1, not an input to it.** CR 707
defines what a copiable value is; CR 613.2a is where a copy effect is applied
and ordered. The engine's copiable-values object follows directly:

> **The copiable values of an object are `compute_to_ceiling(game, id, END_OF_LAYER_1, cache)`,
> materialized into an owned snapshot at the moment the copy effect starts to
> apply.** That is `compute.rs`'s existing frame-cache entry point, at the
> ceiling the existing `LAYER_ORDER` array already indexes.

This is worth stating as the doc's headline because it means the engine does
**not** need a new concept. It needs one new `EffectModification` arm, one
capture call at an existing entry point, and a rule about what the arm may
store.

### 1.2 One mechanism, not two — and the `CardData` swap is rejected

The audit's framing offered three candidates: a `CardData` swap at ETB, a Layer
1 row, or both, per population. **The answer is a Layer 1a row for every
population that has copiable values at all**, and the `CardData` swap is wrong
for three independent reasons. Recording all three because each kills it alone:

1. **It breaks the graveyard, via CR 400.7.** `move_object` preserves the
   `ObjectId` and the `GameObject`, so a swapped `Arc<CardData>` follows the
   permanent out of play. A Clone copying Grizzly Bears would be a *Grizzly
   Bears card* in its owner's graveyard, castable and searchable as one. CR
   707.2's copiable values are values of the permanent; the card is still Clone.
   `card_data` is documented as "the immutable printed card definition" and that
   doc comment is load-bearing.
2. **It has no timestamp, so it cannot be ordered.** CR 613.2a applies copy
   effects "in timestamp order". A base-characteristics swap is applied before
   layer 1 by construction and therefore sits outside the ordering that CR 707.3
   ("Objects that copy the object will use the new copiable values") requires
   when two copy effects meet.
3. **It cannot end.** 68 of Tier C's 81 clauses are `until end of turn`
   (§2.1). A swap has no `Duration` and nothing to expire.

The cost of the single mechanism is one indirection on the hottest path, and
§7's CV-1 measures it rather than assuming it.

### 1.3 The one rule that makes all of this hold

> **A copy row stores *values*, never a reference.** `CopyFrom(Box<CopiableValues>)`,
> captured once. Never `CopyOf(ObjectId)` resolved during the layer walk.

Three consequences, each argued where it belongs, and each of which flips if the
rule is broken:

- CR 707.2b ("changing the copiable values of the original object won't cause
  the copy to change") and 707.2c ("determined only at the time that effect
  first starts to apply") are *satisfied by construction* rather than by an
  explicit severing step.
- Under CR 613.8a(b), a value-carrying copy row is **independent of every other
  layer 1 effect** — nothing another effect does can change its text, its
  existence, what it applies to, or what it does. So copy work does **not**
  queue behind critical-path item 7. A reference-carrying row would be dependent
  and it would. → §5.2.
- A reference-carrying row would ask for another object's ceiling-1 frame from
  *inside* layer index 0, which is not a strict descent and therefore breaks
  `layers-architecture.md` §5.2's termination argument outright — two permanents
  copying each other is a real board, and CR 613.8b's dependency-loop fallback
  has no counterpart in the frame cache. → §5.2.

### 1.4 Why now

Three ordering facts.

1. **RC-4 is scheduled to produce a `CopyOnEnter` and it must not.**
   `replacement-architecture.md` §9 lists "the CR 616.1b/c buckets" in RC-4's
   scope. 616.1c cannot be produced without the whole capture-and-row spine
   below. §5.1 resolves this and the answer moves the split — which is the
   deadline this document was written against, and the reason it lands before
   RC-4 starts rather than after.
2. **`rb-review.md` I9 attached an obligation to this document by name**, and
   it turns out to have a twin nobody had found (§4.7). Two ETB-time scans read
   printed abilities; a Layer 1 copy defeats both; the failure mode of each is a
   card that silently does nothing, which is this project's named worst outcome.
3. **Two shipped enum arms have no producer and one shipped comment is
   wrong about who will give them one.** `ReplacementClass::CopyOnEnter` and
   `BackFaceUp` (`types/replacement.rs:324,326`) ship under a doc comment saying
   they arrive "in Phase RC-B". They do not; they arrive here. `Layer1Copy`
   (`engine/layers/types.rs:43`) is documented as "still a stub: the variant
   exists, nothing produces an effect in it".

---

## 2. Scope, measured

Every number below is `python plans/references/copy-census.py`, run 2026-08-29.
The mechanism table counts **clauses**; the population table counts **cards**.
They overlap by construction and are never summed. Neither is a work estimate —
§7 is.

### 2.1 The mechanism table

**Cards is the unit that matters here; clauses are shown for provenance only.**
The clause count is how the classifier partitions the corpus — that is what
produced the six mechanisms — but it is a poor size estimate for copying, and
that is a real difference from `cant-census.py`. A "can't" card can carry several
*independent* restrictions needing different enforcement points, so clauses are
the work. A copy card carries one mechanism and then a rider sentence about it
("You may choose new targets for the copy"), so **clauses count sentences, not
work**. Tier D is the extreme case: 542 clauses over 306 cards, because 186 of
those clauses are one shared CR 707.10c prompt.

| Tier | Mechanism the engine must build | CR | Cards | Clauses | Built? |
|---|---|---|---:|---:|---|
| **A** | Token copy — `CreateToken` carrying copiable values | 707.1, 111.1 | **306** | 318 | ❌ |
| **B** | Enters as a copy — a CR 616.1c replacement | 707.5, 616.1c | **69** | 69 | ❌ (bucket ships, no producer) |
| **C** | Becomes a copy — a Layer 1a continuous effect | 707.4, 707.2c | **81** | 81 | ❌ (`Layer1Copy` is a stub) |
| **D** | Spell / ability copy — a stack object, **no layer** | 707.10 | **306** | 542 | ❌ |
| **E** | Cast a copy — create in the zone, then cast | 707.12–14 | **47** | 73 | ❌ |
| **F** | *Copy as subject* — the rules read a copy, nothing is produced | 707.1, 707.11 | 8 | 8 | n/a |
| ? | unclassified | — | 2 | 2 | printed in §2.6 |

Tiers A and D are the largest, and **D needs no layer system at all** — a copy
of a spell is a new object on the stack (CR 707.10), and its hard parts are the
retargeting rules rather than characteristics. That inversion — biggest
population, least coupling — is why §7 can ship it independently of everything
else.

Tier C is 81 cards and is the one that needs Layer 1, a timestamp and a
duration. **Size and difficulty run in opposite directions here**, which is the
one thing this table is shaped to make visible.

### 2.2 The population table — what the word search cannot see

Half the copiable-values corpus never prints "copy". A transforming DFC is a
copiable-values question (CR 712.2 → 613.2a) that says *transform*; a morph
creature is one (CR 708.2) that says *face down*; a mutate creature is one (CR
729.2a) that says *mutate*. Card counts, Commander-legal alongside:

| Population | CR | Cards | Commander-legal |
|---|---|---:|---:|
| Nonmodal DFC (transform) | 712.2 | **396** | 388 |
| Modal DFC | 712.3 | **100** | 98 |
| Meld | 712.4 | 21 | 21 |
| Flip card | 710.1 | 25 | 19 |
| Face-down producers (morph/megamorph/manifest/disguise/cloak) | 708.2a | **304** | 294 |
| Mutate — merges with a permanent | 729.1 | 34 | 34 |
| Says "transform" | 701.27 | 286 | 285 |
| Says "face down" | 708.2 | 113 | 105 |

**120 double-faced cards can *be* a commander** (88 transform + 32 modal). That
is the single number that decides §6.

### 2.3 Three things the detector's table got wrong

The 2026-08-27 audit is the reason this document exists and its conclusion is
correct. Three of its cells are not, and they are corrected here and in
`codebase-state.md` in the same PR.

| Claim | Correction |
|---|---|
| "CR 707 (copying), 712 (DFC/meld), 613.2 (Layer 1), **706**" | **706 is Rolling a Die.** The intended section is **708, Face-Down Spells and Permanents** — which is Layer 1b, has 10 uncovered atoms, and is a genuine member of the cluster |
| "**2,890** double-faced cards" | `is:dfc` is 2,890 and **2,243 of them are `layout:art_series`** — art cards with a signature on the back, not playable Magic and with no copiable values to model. Add `double_faced_token` (80) and `reversible_card` (71) and the rules-relevant population is **517**: 396 transform + 100 modal + 21 meld. The conclusion survives at 517; the *scoping* argument does not survive at 2,890, because a 5.6× overstatement is what turns "schedule it" into "schedule it first" |
| The cluster is 707 + 712 + 613.2 + 708 | **CR 729, Merging with Permanents, is a fourth member and it is named by CR 613.2a itself** — "This includes copy effects (see rule 707) *and changes to an object's characteristics determined by merging an object with a permanent (see rule 729)*". 19 uncovered atoms, 34 mutate cards, and CR 729.2a is explicitly "a copiable effect whose timestamp is the time the objects merged". Out of scope for v1 (§6), but it is Layer 1a and the detector missed it |

**The detector itself is not impeached by this.** It found a real gap with the
right shape; what it got wrong is what a headline Scryfall count always gets
wrong, and the fix is the one this project already wrote down: *check breadth
with counts, and check what the count includes*. The census script now prints
the decomposition (`--decompose`) so the next reader cannot repeat it.

### 2.4 The `AffectedSet` split — 17 clauses out of 150

Which copy rows are scoped to their own source, and which to a filter? This
decides exactly one thing, and it is a scheduling question, not a modelling one
(§5.3). `copy-census.py --scope`:

| Tier | Source-scoped | Filter-scoped |
|---|---:|---:|
| B — enters as a copy | 65 | **4** (Essence of the Wild, Infinite Reflection, Mystic Reflection, Thunderbond Vanguard) |
| C — becomes a copy | 68 | **13** (Mirrorweave, Cytoshape, Hall of Mirrors, Sakashima's Will, …) |

**133 of 150 copy clauses are `AffectedSet::SourceOnly`.** Those are torn down
by machinery that already exists — `cleanup_zone_state` →
`replacement_effects.remove_by_source` / `continuous_effects.remove_by_source`
(`engine/zones.rs:366–374`) — so they are not blocked on CR 400.7. The 17 are.
The script prints all 17 by name rather than by count, because a bucket nobody
re-read is a census's failure mode.

### 2.5 What the engine has today

| Piece | Where | State |
|---|---|---|
| `Layer::Layer1Copy` | `engine/layers/types.rs:43` | Variant exists; **nothing produces an effect in it** (its own doc says so) |
| `LAYER_ORDER` slot | `engine/layers/compute.rs:35` | Present, index 0. Sublayer split 1a/1b missing |
| `ReplacementClass::CopyOnEnter` / `BackFaceUp` | `types/replacement.rs:324,326` | Ship; ordered correctly by `forced_bucket`; **no producer** |
| `GameObject.is_copy: bool` | `objects/object.rs:30` | Field exists; **written nowhere**, read nowhere, asserted false by two tests |
| `Primitive::CreateToken` | `engine/resolve.rs:575` | Implemented (RB), but builds `CardData` from a `TokenDef` — no path takes copiable values |
| Copiable-values capture | — | ❌ |
| CR 712 faces on `CardData` | — | ❌ — `CardData` has one face |
| CR 708 face-down state | — | ❌ — no `BattlefieldEntity.face_down` |

Nothing here is wrong; it is all correctly-shaped scaffolding with no producer.
That is the same position `ReplacementClass` was in before RB.

### 2.6 The residual

Two clauses no bucket caught, printed rather than rounded away. Neither is a
seventh mechanism:

| Card | Clause | Where it belongs |
|---|---|---|
| God-Eternal Kefnet | "That copy costs {2} less to cast." | Tier E with a cost modification — CR 707.9's exception shape, but applied to a *cast* copy rather than to characteristics |
| Spellweaver Helix | "…you may copy the other." | Tier E; "the other" is an anaphor to an exiled card and no regex over one clause can resolve it |

Two named oddities the CR itself singles out are *not* residual — they are
bucketed, and worth naming because the CR gives each its own rule: **Garth
One-Eye** (CR 707.13, a copy of a card defined by name, created outside the
game) and **Magar of the Magic Strings** (CR 707.14, a copy of a card by noted
name using CR 608.2h's last-existed characteristics). Both are Tier E, both are
out of v1 scope, and both are the kind of card that is cheap once the capture
exists and impossible before it.

---

## 3. The model

### 3.1 The claim

> **A copy is a snapshot of an object's post-layer-1 characteristics, stored as
> a value in a Layer 1a row.** Every producer — a replacement as a permanent
> enters, a resolution, a static ability, a token creation — differs only in
> *when it captures* and *what carries the result*. There is one capture
> function and one modification arm.

Three properties follow, and all three are properties the CR states rather than
conveniences:

- **The capture is at the moment the effect starts to apply** (CR 707.2c),
  never re-derived. This is the opposite of how every other continuous effect in
  this engine works — a Layer 6 grant is re-evaluated at every walk — and the
  asymmetry is CR 707.2b, not an optimization.
- **What is captured is post-layer-1, not printed** (CR 613.2c). So a Clone
  copying a Clone copying a Bear gets a Bear, and a copy of a face-down creature
  gets 708.2a's 2/2, for the same one reason.
- **Status, counters, stickers and non-copy effects are not captured**
  (CR 707.2's last sentence). Tapped-ness, control, damage, +1/+1 counters and
  every Layer 2–7 row on the copied object are excluded — which falls out of
  capturing at ceiling 1 and is worth an assertion, because a future
  `compute_to_ceiling` change that raised the default ceiling would silently
  start copying anthems.

### 3.2 `CopiableValues` — the type surface

```rust
/// CR 707.2 — the values a copy acquires, captured once (CR 707.2b/2c).
///
/// This is `EffectiveCharacteristics` as of the end of layer 1 (CR 613.2c),
/// minus the fields CR 707.2 excludes because they are status or control
/// rather than characteristics. It is a *value*: nothing in it points at the
/// object it was captured from, which is what makes a copy row independent
/// under CR 613.8a(b) (see the architecture doc §5.2).
pub struct CopiableValues {
    pub name: String,
    pub mana_cost: ManaCost,
    pub colors: HashSet<Color>,
    pub types: HashSet<CardType>,
    pub subtypes: HashSet<Subtype>,
    pub supertypes: HashSet<Supertype>,
    pub keywords: HashSet<KeywordFlag>,
    /// CR 707.2a — a copy acquires abilities because they derive from rules
    /// text, not by copying "the abilities" separately. Carries each
    /// `AbilityDef.is_characteristic_defining` verbatim: CR 604.3a(2)'s third
    /// clause (§3.4).
    pub abilities: Vec<AbilityDef>,
    pub power: Option<i32>,
    pub toughness: Option<i32>,
    /// CR 712 — `None` for a single-faced object. Present so that CR 707.8a's
    /// "the resulting token is a double-faced token" has somewhere to live
    /// without a second capture type (§4.5). Unpopulated until CV-5.
    pub back_face: Option<Box<CopiableValues>>,
}
```

Deliberately **not** `EffectiveCharacteristics` reused: that type carries
`controller` and `control_since_turn`, and CR 707.2 excludes both. A copy that
carried a controller would be a Layer 2 effect wearing a Layer 1 costume, and
`compute.rs`'s `any_control_changing` fast path (`compute.rs:389`,
`characteristics.rs:51`) proves its correctness on the claim that **Layer 2 is
the only channel that writes controller**. Reusing the bigger type would make
that claim false by accident. This is the one place where a smaller type is
protecting an existing optimization's soundness argument, so it is worth the
duplication.

**The one arm this adds to `EffectModification`:**

```rust
    // --- Layer 1a (CR 613.2a) ---
    /// Boxed for the same reason `GrantAbility` is: this is the largest
    /// payload in the enum and the enum is matched at every layer.
    CopyFrom(Box<CopiableValues>),
```

**Growth contract.** `EffectModification` grows one arm per *characteristic
channel*, and `CopyFrom` replaces every channel at once, which is what layer 1
means. A second layer-1 arm is a claim that CR 613.2 has a third sublayer; the
two it has are `CopyFrom` (1a) and face-down (1b), and §4.6 says why face-down
should be derived from `BattlefieldEntity` state rather than added here.

### 3.3 The four producers

All four build the same `CopiableValues`. They differ in what carries it.

| # | Producer | CR | Carrier | Tier | Phase |
|---|---|---|---|---|---|
| 1 | A resolution or an activated ability | 707.4, 707.2c | A `ContinuousEffect` row with a `Duration` | C | CV-1 |
| 2 | A replacement as a permanent enters | 707.5, 616.1c | `Rewrite::Instead` → an `EnterMods` field, then a row with `Duration::Indefinite` at placement | B | CV-2 |
| 3 | Token creation | 707.1, 111.1 | The new object's own `Arc<CardData>`, built from the capture | A | CV-3 |
| 4 | A copy of a spell or ability | 707.10 | A new stack object; **no row, no layer** | D, E | CV-4 |

Producer 3 is the exception that proves §1.2's rule: a token has **no printed
card**, so writing its capture into `Arc<CardData>` is not a swap — it is the
token's only identity, exactly as `token_card_data` already builds one from a
`TokenDef` (`engine/resolve.rs:1450`). A token copy that dies goes nowhere (CR
111.7 / SBA 704.5d), so §1.2's graveyard objection does not arise.

Producer 2 needs the row anyway, and not only for uniformity: **CR 707.4** lets
a permanent that is copying a permanent copy a *different* object while
remaining on the battlefield, without re-entering. That is a row being replaced,
which a base-characteristics swap could not express without a second mechanism.

### 3.4 CDA provenance — where the handing happens

`layers-architecture.md` §6's provenance table already has the row: *"Copy
effect (Layer 1), text-changing effect (Layer 3) — rides along on the def — CR
604.3a(2), third clause."* This document states where "rides along" is
implemented, and the answer is **the capture, and nowhere else**:

- `CopiableValues.abilities` is cloned out of the source's ceiling-1
  `EffectiveCharacteristics.abilities`, `is_characteristic_defining` flags
  included. That is the whole of the normal case, and it is free.
- **The trap is CR 707.9d**, and it is the one place the capture must actively
  *drop* a flag:

  > **707.9d** When applying a copy effect that doesn't copy a certain
  > characteristic, retains one or more original values for a certain
  > characteristic, or provides a specific set of values for a certain
  > characteristic, any characteristic-defining ability … of the object being
  > copied that defines that characteristic is not copied.

  So "…as a copy of that creature, except it's a 0/1" must **remove the copied
  object's P/T-defining CDA**, not merely set P/T afterwards — otherwise the CDA
  re-derives at Layer 7a and overwrites the exception. `139 of 331` printed
  "as a copy"/"that's a copy" cards carry the word "except" (`--decompose`), so
  this is the common case, not a corner.
- **707.9d's own carve-out**: exceptions of the form "in addition to its other
  types" do *not* drop the CDA. One `if` in the exception applier, and it needs
  a test, because getting it backwards makes Sakashima-style cards silently
  wrong rather than loudly broken.
- A **Layer 6 `GrantAbility` still clears the flag**, unchanged. Copying is
  layer 1 and grants are layer 6; they meet only in the sense that a copied
  ability list can later be added to.

### 3.5 What is deliberately not a copy effect

- **CR 712 transform** is not a copy effect. It changes *which face* is up; the
  copiable values are then read off that face (CR 707.8). Modelled as card data
  with two faces plus a status bit, not as a Layer 1 row (§4.5).
- **CR 708 face-down** is Layer **1b**, a sibling sublayer, and is derived from
  `BattlefieldEntity` state rather than registered (§4.6).
- **CR 729 merging** *is* a Layer 1a copiable effect by CR 729.2a and would use
  `CopyFrom` — but it needs a permanent represented by several components, which
  is a `BattlefieldEntity` change, not a copy change. That is **CV-7** (§6).
- **`GameObject.is_copy`** is a *provenance* flag, not a mechanism. It should be
  set by producers 3 and 4 (a token copy, a spell copy) and stay false for 1 and
  2, whose objects are ordinary permanents with a row on them. It has no reader
  today and CV-1 should not invent one.

---

## 4. The production points, one at a time

### 4.1 Tier B — enters as a copy (CR 707.5 / 616.1c)

**69 cards, 65 of them source-scoped.** Clone's "You may have this creature
enter as a copy of any creature on the battlefield" is a replacement effect
that modifies how the permanent enters, and CR 707.5 is explicit that "It
doesn't enter the battlefield, and then become a copy."

The pipeline shape is already built: it is a `ReplacementDef` with
`class: ReplacementClass::CopyOnEnter` (**a name CR 707.5 contradicts — rename
it to `CopyAsEnters` while it still has no producer**, §9 item 9),
`pattern: EventPattern::EnterBattlefield`
(RC-2's), `optional: true`, and a `Rewrite::Instead` producing an
`EnterBattlefield` whose `EnterMods` carries the captured values. `forced_bucket`
already orders 616.1c ahead of 616.1e. **The choice of what to copy is CR
707.6's** — "the object's controller will get to make any 'as [this] enters'
choices for it" — which is a `DecisionProvider` prompt, and the CR 616.1
two-candidate short-circuit does *not* apply to it: it is a choice *within* one
replacement, not a choice *among* replacements.

Three things this must get right, each with a rule behind it:

- **The row is created at placement, before ETB triggers are collected**
  (CR 707.5's last sentence: "any enters-the-battlefield triggered abilities of
  the copy will have a chance to trigger" — *of the copy*, meaning the copied
  object's, meaning the row exists first).
- **Copied "enters with" abilities take effect** (CR 707.5, same sentence). So
  the captured ability list must be re-scanned for ETB replacements — which is
  §4.7's problem, appearing here for the first time.
- **CR 707.6: choices made for the copied permanent are not copied.** The
  capture is characteristics only; `BattlefieldEntity` is not consulted.

### 4.2 Tier C — becomes a copy (CR 707.4 / 707.2c)

**81 cards.** Two sub-shapes with different `Duration`s and one shared row:

- **From a resolution** — Cytoshape, Mirrorweave, Polymorphous Rush.
  `Duration::UntilEndOfTurn` overwhelmingly. `Primitive::Copy { source, target,
  duration }` registers the row; this is the *only* new primitive the whole
  design asks for, and it is generic across all 81 cards for the same reason
  `Primitive::Restrict` is generic across the "can't" corpus.
- **From an activated or static ability** — Vesuvan Doppelganger, Shapesharer.
  CR 707.2c's "determined only at the time that effect first starts to apply"
  is what makes the static case a capture rather than a live view, and it is the
  clause that makes a static copy ability behave unlike every other static
  ability in the engine. Worth a comment at the capture site; it is not
  recoverable from the code.

**CR 707.4's re-copy**: "Some effects cause a permanent that's copying a
permanent to copy a different object while remaining on the battlefield. The
change doesn't cause enters-the-battlefield or leaves-the-battlefield abilities
to trigger. This also doesn't change any noncopy effects presently affecting the
permanent." All three clauses are free under the row model — replacing a row
fires no zone change, and Layer 2–7 rows are untouched — which is the clearest
single piece of evidence that the row is the right carrier.

### 4.3 Tier A — token copies (CR 707.1, 111.1)

**306 cards, the largest layer-touching bucket and the cheapest.**
`Primitive::CreateToken` exists and builds an `Arc<CardData>` from a `TokenDef`;
this adds a second constructor that builds one from a `CopiableValues`. No row,
no layer, no duration.

Two rules that are easy to miss and cheap to honour:
- **CR 707.2's exclusions apply**: the token copy of a creature with three +1/+1
  counters has none.
- **CR 707.8a**: a token copy of a double-faced permanent is a *double-faced
  token* with both faces, which is what `CopiableValues.back_face` exists for.
  Unreachable until CV-5, and the field is the reason CV-5 does not have to
  revisit CV-3.

### 4.4 Tiers D and E — the stack copy (CR 707.10, 707.12)

**306 + 47 cards, and they touch no layer, no registry and no characteristic.**
CR 707.10: "To copy a spell … means to put a copy of it onto the stack" — the
copy carries the original's characteristics *and all decisions made for it*
(modes, targets, X, costs). In this engine that is a `StackEntry` clone plus a
new `ObjectId`, and the interesting parts are all decision-shaped:

- **CR 707.10c** — "may choose new targets for the copy": 186 clauses, the
  single largest sub-bucket in the census. A `DecisionProvider` prompt reusing
  `enumerate_legal_selections`, with the rule's own leniency ("may leave any
  number of the targets unchanged, even if those targets would be illegal").
- **CR 707.10d/e** — copy-per-target and copy-with-a-named-target (Zada, Ink-
  Treader, Precursor Golem). N copies, ordered by their controller's choice.
- **CR 707.10a** — a copy in any zone but the stack ceases to exist, as an
  SBA. New SBA, and it is the reason `is_copy` should finally get a writer.
- **CR 707.10f** — a copy of a *permanent* spell becomes a token as it resolves,
  which is where Tier D rejoins Tier A.
- **CR 707.10b** — a copy of an ability has the same source as the original, and
  is "the same ability" for effects that count resolutions. That is a CR 603.7h
  identity question and it lands with triggers (critical-path item 6), so CV-4
  covers spell copies and defers *ability* copies (39 clauses) with a cite.

This tier is genuinely independent of the rest of this document, which is why
§7 lets it float.

### 4.5 CR 712 — faces (transform, modal, meld)

**517 cards; 496 of them transform or modal; 120 can be a commander.** This is
not a copy mechanism — it is a **second card model** — and the honest framing is
that CR 712 lands *beside* copy work and shares one seam with it.

What it needs: `CardData` gains a back face; `GameObject` or `BattlefieldEntity`
gains "which face is up"; CR 712.2's transform is a status change, not a
characteristic effect; CR 712.3's modal faces are chosen at cast time and never
change; CR 712.8's rules on which face's values apply in which zone.

The seams with this document, both narrow:
- **CR 707.8** — copying a DFC uses the copiable values of the face that is
  currently up. One line at the capture, once faces exist.
- **CR 616.1d / `ReplacementClass::BackFaceUp`** — "enters transformed" is a
  replacement, and it is the *only* place CR 712 touches the pipeline.

`BackFaceUp` therefore gets its producer in CV-5 and not in RC-4 (§5.1).

### 4.6 CR 708 — face-down, and Layer 1b

**304 producers; CV-6, and undesigned here beyond the one constraint it puts on
the capture.** CR 708.2 makes the face-down characteristics *be* the copiable
values, so it is inside layer 1 by definition, and CR 613.2b puts it in sublayer
1b — after 1a.

The shape `layers-architecture.md` §7 already specifies is right and should be
kept: `BattlefieldEntity.face_down: bool` is canonical, and layer 1b
*synthesizes* 708.2a's 2/2 colorless no-name characteristics from the flag
rather than registering a row. Deriving from state the engine already owns is
the same call the 7c counter path made.

**One consequence for this document and it is the whole reason face-down is
mentioned at all:** a capture at ceiling "end of layer 1" gets 708.2a's 2/2 for
free, with no special case, *provided the ceiling is the end of 1b and not the
end of 1a*. That is one integer, and §5.4 is about the fact that three files
currently name the sublayers in the wrong order.

### 4.7 The three ETB scans a copy defeats — rb-review I9, and its twin

`rb-review.md` I9 attached one obligation to this document. Auditing it found a
second instance of the same bug, in the layer system rather than the pipeline.

**The general form**, which is the part worth keeping:

> **A fast-path gate must be derived from the same place the sweep reads.**
> `gather` reads the **effective** ability list; its gate reads two *sources* of
> ability. Every new way an ability can reach the effective list is a new leg on
> every such gate. Layer 1 (copy) and Layer 3 (text-change) are the two that do
> not exist yet, and **both must add a leg to every gate below.**

**Leg 1 — `gather`'s gate (I9).** `game_state.rs:249` claims "between them the
gate is sound", resting on `replacement_ability_sources` (printed abilities,
inserted at ETB, `game_state.rs:760`) and
`RegistryScopeSummary::any_granted_replacement` (Layer 6 grants,
`continuous_effects.rs:73`). A Layer 1 copy of a permanent with a printed
replacement ability changes the effective ability list through neither. The
copy's replacement is silently dead on every board where the gate short-circuits
(`gather.rs:143`).

**The fix, specified: extend `RegistryScopeSummary` with a third flag.**

```rust
    /// True iff some row is an `EffectModification::CopyFrom` whose captured
    /// values carry a replacement ability. Third leg of `gather`'s gate: a
    /// copy reaches the effective ability list through neither of the other two.
    pub any_copied_replacement: bool,
```

Chosen over the alternative (inserting the copy's object into
`replacement_ability_sources` when the row is created) for one reason that is
not taste: a copy row can **expire** without a zone change, and
`replacement_ability_sources` is removed only at `cleanup_zone_state`
(`zones.rs:374`). The set would drift high. Drifting high is the *safe*
direction — it costs a wasted layer walk, never a wrong answer — but the field's
own doc comment says "a set rather than a count, **so it cannot drift**", and
quietly making that sentence false is worse than the extra field. The summary is
recomputed from the registry on every mutation, so it cannot drift at all.

**Leg 2 — `register_static_effects`, and this one nobody had found.**
`game_state.rs:737` "Reads printed abilities on purpose" — deliberately, to
avoid circularity inside `place_on_battlefield`, and that reasoning is sound for
the case it was written for. But it means **a copy of a permanent with a static
continuous ability registers no row for that ability.** A Clone of a Glorious
Anthem is a Glorious Anthem that pumps nothing. Same failure mode as leg 1,
different registry, and it is *more* likely to be hit: static abilities are far
more common on creatures than replacement abilities.

The fix is not symmetric with leg 1, because here the *row itself* is missing
rather than a gate being wrong. CV-1 must re-run static registration against the
captured ability list at the moment a `CopyFrom` row is created or replaced, and
remove those rows when it expires. Sized in §7; it is the reason CV-1 is not a
one-arm PR.

**Leg 3 — anything Phase 6 adds.** Triggered-ability registration will want the
same ETB scan and must take the effective list or a copy's triggers will be
dead. Named here because critical-path item 6 has not started and this is
cheaper to say now than to find later.

**The other two summary flags, audited and clean.** `any_multi_row_group`
(`compute.rs:203`) counts rows in the registry, and a copy row is a registry
row, so it is correct by construction. `any_control_changing`
(`compute.rs:389`, `characteristics.rs:51`) rests on "Layer 2 is the only
channel that writes controller", and CR 707.2 excludes control from copiable
values, so a `CopyFrom` row cannot write one — **which is exactly why
`CopiableValues` must not reuse `EffectiveCharacteristics`** (§3.2).

---

## 5. Joins with what is already built

### 5.1 What RC-4 actually needs — the split moves

**The finding: RC-4's scope does not survive, and the fix makes RC-4 smaller.**

`replacement-architecture.md` §9 lists "the CR 616.1b/c buckets" in RC-4. Taken
literally that means RC-4 produces a `CopyOnEnter`, and producing one requires
`CopiableValues`, the capture, the Layer 1a arm, the row, and both legs of §4.7
— the whole of CV-1 plus CV-2. That is not a scope adjustment inside RC-4; it is
a different phase wearing RC-4's name.

**The resolution, and it needs no new machinery:**

| Bucket | CR | Producer | Where |
|---|---|---|---|
| `ControlChanging` | 616.1b | **stays in RC-4** | Layer 2 is shipped (critical-path item 4 ✅), so 616.1b is producible today |
| `CopyOnEnter` | 616.1c | **moves to CV-2** | needs the copy spine |
| `BackFaceUp` | 616.1d | **moves to CV-5** | needs CR 712 faces |

This is precisely the position `ReplacementClass` was designed for: RB shipped
all five arms with one producer, on the stated reasoning that "a bucket that
does not exist cannot be ordered". A bucket with no producer is the normal
state, not a debt. **RC-4 keeps 616.1b, drops 616.1c, and `types/replacement.rs:316`'s
"`ControlChanging` and `CopyOnEnter` in Phase RC-B" becomes half-true and should
be corrected to name this document.**

**So: is RC-4 blocked on this design? No — and that is the useful answer.**
RC-4 is blocked on RS-1 (CR 614.17d) as `cant-effects-architecture.md` §7 says,
and nothing here adds a second block. What this document buys RC-4 is
permission to *not* build a copy system, which is worth more than an unblock.

**One thing RC-4 does inherit**, and it is small: CR 707.5's "if the text that's
being copied includes any abilities that replace the enters-the-battlefield
event … those abilities will take effect" means the ETB look-ahead frame must
eventually read a *copied* ability list. RC-4's frame reads
`get_effective_abilities`, which is the right side of §4.7's rule, so this is
free at the frame and costs only the row existing first — i.e. CV-2 orders after
RC-4, never before.

### 5.2 The CR 613.8 join — refuted

**The claim to test:** Layer 1 copies generate dependency-ordering-sensitive
boards, so copy implementation queues behind critical-path item 7 the way RS-3b
does.

**Refuted, and the reason is one rule.** CR 707.2b: "Once an object has been
copied, changing the copiable values of the original object won't cause the copy
to change." A copy is a snapshot. CR 613.8a(b) asks whether applying one effect
would change what another one does — and nothing can change what a snapshot
already holds. Copy rows are independent of each other, and of everything else
in layer 1a. **Item 7 is not a prerequisite for any CV phase.**

Two footnotes, and neither changes the answer:

- **CR 613.8 in `tmnt.txt` carries no examples** — 613.8a–c are three sentences
  and the example block older printings had is gone. So the paragraph above is
  read off 613.8a's criteria rather than off a worked case. Criterion (a) would
  confine the question to layer 1a anyway, and (c) is satisfied because a copy
  effect is never a CDA.
- **The snapshot is also what keeps `layers-architecture.md` §5.2's termination
  argument true.** That proof is "computing an object at ceiling C only ever
  requests ceilings < C". Resolving a copy *during* the walk would ask for the
  copied object's ceiling-1 frame from inside layer index 0 — not a strict
  descent, and two permanents copying each other is a legal board. Capturing
  once, outside the walk, means the recursion never happens. This is the engine
  half of §1.3 and the only part of it that is not just restating the CR.

**What this does not claim.** CV-5 (faces) and any future CR 729 work are *not*
covered by this argument, because a merged permanent's characteristics come from
its topmost component and that is a live relationship, not a snapshot. If
merging is ever built, re-ask 613.8a(b) for it specifically.

### 5.3 The CR 400.7 trap — what ends a copy effect

**The question:** `move_object` keeps `ObjectId` across zones and CR 400.7 is
unimplemented (`codebase-state.md` Deferred Migrations item 10). What ends a copy
effect when the permanent leaves the battlefield, and does the answer depend on
item 10 landing first?

**The answer is measured, and it splits 133/17** (§2.4).

- **For the 133 source-scoped clauses: nothing new is needed, and the reason is
  structural rather than lucky.** A Clone's copy row and a Vesuvan
  Doppelganger's have the copied permanent as *both* source and subject. When it
  leaves the battlefield, `cleanup_zone_state` calls
  `continuous_effects.remove_by_source(id)` (`zones.rs:366`) and the row is gone
  before the object reaches the graveyard. This is already implemented, already
  tested for Layer 2/6/7 rows, and needs no change. It is also the second reason
  §1.2 rejects the `CardData` swap: the swap has no source to be removed by.
- **For the 17 filter-scoped clauses: item 10 is a genuine prerequisite.**
  Mirrorweave's row is sourced by a spell that is already in the graveyard and
  scoped by a filter; `cleanup_zone_state` "removes only effects *sourced by*
  the leaving object, never effects *targeting* it" (item 10's own words). A
  creature caught by Mirrorweave that dies and returns the same turn would come
  back as a copy — a new object wearing the old object's effect, which is what
  CR 400.7 exists to forbid. This is the general item-10 bug appearing in layer
  1; it is not new and it is not worse here.
- **CR 400.7's exception list does not reach copy effects.** 400.7a–c preserve
  effects that changed a *permanent spell's* characteristics or controller, and
  prevention effects. A copy row on a permanent is none of those. So copy work
  neither needs nor risks the exception half.

**Scheduling consequence, stated so it cannot be missed:** CV-1 and CV-2 ship
`AffectedSet::SourceOnly` only, and the filter-scoped 17 wait for item 10. That
is a phase boundary, not a caveat — §7 draws it as CV-1b.

**And a fact worth recording while here**, because it is the kind that gets
expensive late: item 10's fix must run *before* `remove_by_source`, not instead
of it. A copy row is torn down by source today; a CR 400.7 implementation that
severed effects by subject and left source-teardown alone would double-remove
harmlessly, but one that replaced source-teardown with subject-teardown would
leave a Clone's own row alive on a Clone that changed zones. Whoever implements
item 10 needs that sentence.

### 5.4 Layer 1's sublayers are inverted in three places

**Found while writing §4.6, and it is a fact rather than a preference.**
`tmnt.txt` says:

> **613.2a** Layer 1a: **Copiable effects** are applied. This includes copy
> effects (see rule 707) and changes … determined by merging …
> **613.2b** Layer 1b: **Face-down** spells and permanents have their
> characteristics modified as defined in rule 708.2.

Three places in this repository say the opposite — that 1a is face-down and 1b
is copy:

| Where | Text |
|---|---|
| `plans/layers-architecture.md` §7 | "**1a — face-down effects** … **1b — copy effects**" |
| `plans/codebase-state.md`, Layers item 10 | "CR 613.2 splits layer 1 into face-down effects (1a) and copy effects (1b)" |
| `mtgsim/src/engine/layers/compute.rs:27` | "CR 613.2 splits layer 1 into face-down effects (1a) and copy effects (1b)" |

**No code is wrong**, because `Layer1Copy` collapses both and nothing produces
an effect in either. What is wrong is a derivation the docs share: all three
justify the order with *"a Clone copying a face-down creature must copy the 2/2
colorless characteristics, not the printed card (CR 707.2)"* — a **correct
conclusion from a wrong premise**. The 2/2 is not the result of face-down
applying first; it is CR 708.2's own sentence, "Any listed characteristics are
the copiable values of that object's characteristics", reinforced by CR 708.10
for the copy-of-a-face-down case specifically. The right order is copy then
face-down, and the 2/2 answer is unaffected.

**Why it matters at exactly one point:** §4.6's claim that a capture gets
708.2a's 2/2 for free depends on capturing at the end of **1b**. A reader who
believes 1b is copy would take the ceiling one sublayer too early and the bug
would appear only on boards with a face-down creature being copied — which is to
say, in Phase 8 and not in any test written before it.

`codebase-state.md` is corrected in this PR. `layers-architecture.md` §7 and
`compute.rs:27` are named in §11 as owed, and are deliberately not touched here:
this pass ships no `.rs` changes, and §7 of that document is its own authority.

---

## 6. v1 scope — everything, in an order

v1 is **4-player Commander through a GUI, and highly parallel AI games over the
CLI** (owner, 2026-08-24), and the longer-range target is **every card but the
genuinely obscure** (owner, 2026-08-30). An earlier draft of this section
scoped meld, mutate, face-down and flip cards *out* of v1 on population size and
effort. **That was wrong twice over and is withdrawn.**

**Why it was wrong on the merits.** Deferring these does not defer their cost,
it *raises* it, and this project already has the vocabulary for why. Meld and
mutate both need a permanent represented by **several components** (CR 729.2,
712.4), and "is this one object or several?" is a **fact**, not a feature —
`codebase-state.md`'s own triage. Facts are unrecoverable if not captured when
they exist, and adding one late means re-threading every system built in
between. `BattlefieldEntity` is single-component today and every phase on the
critical path writes more code against that assumption. **The cheapest moment to
decide whether a permanent can be several objects is before Phase 8 card
breadth, not after** — the same back-stop CR 613.8 has.

**Why it was wrong on the population.** 21 meld cards is not 21 cards' worth of
demand: Brisela, Voice of Nightmares is a played Commander card, and the meld
pairs are the kind of card people build decks around rather than the kind they
cut. Mutate is 34 cards and a whole mechanic with an identity. Neither is
Camouflage or Panglacial Wurm, which is the standard this project actually
holds — "most obscure", not "smallest bucket".

**So nothing here is out of v1.** What is left is an *order*, and the ordering
argument is about coupling, not worth:

| Population | Cards | Phase | Why here in the order |
|---|---:|---|---|
| Tier D/E — spell and cast copies | 306 + 47 | **CV-4** | Zero coupling to layers, registry or pipeline. Can land first or last; nothing waits on it |
| Tier C — becomes a copy | 81 | **CV-1** | The spine every other CV phase carries |
| Tier B — enters as a copy | 69 | **CV-2** | Needs RC-2's `EnterBattlefield` event |
| Tier A — token copies | 306 | **CV-3** | Needs CV-1's capture; nothing needs it |
| CR 712 transform + modal DFC | 496 | **CV-5** | A second card model. 486 Commander-legal and **120 can be a commander** |
| **Face-down** (CR 708) | **304 producers** | **CV-6** | Layer 1b, and CR 707.2 makes copiable values depend on face-down status — so it is coupled to CV-1's capture ceiling and to nothing else. Its bulk is a *casting* mechanism (CR 708.4: alternative costs, a turn-face-up special action), which is why it is its own phase rather than a rider on CV-1 |
| **Merging** (CR 729) + **meld** (CR 712.4) | 34 + 21 | **CV-7** | The multi-component `BattlefieldEntity`. Last in this track because it is the largest structural change, **and it is the one with a back-stop**: before Phase 8 card breadth, for the fact reason above |
| **Flip cards** (CR 710) | 25 | **CV-5** | Rides along once faces exist; CR 729.2h already couples them to merging |
| Garth One-Eye (707.13), Magar (707.14) | 2 | **CV-4** | Cheap once the capture exists; the CR names each individually |

**There is no single total, and an earlier draft's "~1,300 cards" was a sum of
sets that overlap** — which §2's own header forbids. The two honest headline
numbers, and the measured overlap between them:

- **752 cards print a copy clause** (the mechanism table).
- **901 cards carry a face or state that is a copiable-values question** and
  mostly print no copy clause at all: 517 double-faced (§2.3's correction, not
  2,890) + 304 face-down producers + 55 merging/meld + 25 flip.
- **They overlap by 25 cards** — the transform/modal/meld cards that also print
  the word, printed by `copy-census.py`'s table-overlap line rather than assumed
  negligible. So the union is **1,628**, and that is the only figure in this
  document that is a sum of anything.

**One thing this section deliberately does not do: design CV-5 through CV-7.**
Their rows above are populations, ordering and the coupling argument — not a
type surface. DFC is a second card model and merging is a `BattlefieldEntity`
change; each earns its own design pass **when it is next**, on the evidence
available then. Committing a shape for them now would be designing three phases
ahead of the first line of code, which is the failure this document is already
close enough to. → §9 item 10.

---

## 7. Sizing and the phase plan

Sized before writing and split in the doc, per `engineering-practices.md` §4.
Every PR carries at least one consumer of what it builds. Sub-phases are
numbered **CV-1 … CV-7** ("copiable values"). `CP` and `LC` were both taken —
`cards-unlocked-ledger.md`'s Running Totals uses `CP-A`…`CP-D` for checkpoints,
and `layers-architecture.md` §13 uses Phase `LC`.

| PR | Shape | Measured size | Risk |
|---|---|---|---|
| **CV-1 — the capture, the row, and the two legs** | `CopiableValues`, `EffectModification::CopyFrom`, the ceiling-1 capture, `Primitive::Copy`, `RegistryScopeSummary::any_copied_replacement`, and static re-registration off the captured list. **Consumer: Cytoshape** (`AffectedSet::SourceOnly`, `UntilEndOfTurn`) plus a Clone-of-an-Anthem probe for leg 2 | **1** new `EffectModification` arm; **1** apply site (`compute.rs` layer index 0, which today applies nothing); **1** new field on `RegistryScopeSummary` + its recompute (`continuous_effects.rs:114`); **1** gate leg (`gather.rs:143`); **1** re-registration path against `register_static_effects` (`game_state.rs:737`). New type sized against `EffectiveCharacteristics` (12 fields) | **medium-high** — it is one line in the hottest path in the engine, and `layers-architecture.md` §12 measured an ungated existence check at 5.2×–8.0×. The deliverable is as much the `fuzz_games --games 200 --seed 12345` measurement as the behaviour |
| **CV-1b — filter-scoped copies** | `AffectedSet::Filter` on `CopyFrom`. **Consumer: Mirrorweave** | **17** clauses (§2.4), printed by name by `--scope` | low mechanically; **blocked on `codebase-state.md` item 10 (CR 400.7)** and that is the entire content of the block |
| **CV-2 — enters as a copy** | The `CopyOnEnter` producer: a `ReplacementDef` with `class: CopyOnEnter`, the CR 707.6 choice prompt, CR 707.9d's CDA drop, and the 707.9a–c exception applier. **Consumer: Clone, then Sakashima-shaped "except it isn't legendary"** | **69** cards, **139** of the 331 printed "as a copy" cards carry an "except"; **0** new pipeline machinery (`forced_bucket` already orders 616.1c) | medium — **blocked on RC-2** (`EventPattern::EnterBattlefield` must exist) and ordered after RC-4 (§5.1) |
| **CV-3 — token copies** | A second `Arc<CardData>` constructor from `CopiableValues`; CR 707.10f's permanent-spell-copy-becomes-a-token path. **Consumer: "create a token that's a copy of target creature"** | **1** constructor beside `token_card_data` (`resolve.rs:1450`); **1** `Primitive::CreateToken` arm | low — no row, no layer, no duration |
| **CV-4 — spell copies** | `StackEntry` copy, CR 707.10c's retarget prompt, 707.10d/e's per-target copies, 707.10a's cease-to-exist SBA, and `is_copy`'s first writer. **Consumer: Fork, then Zada** | **542** clauses but **1** new object path; **186** clauses are the 707.10c prompt alone; **1** new SBA. Defers CR 707.10b ability copies (**39** clauses) to critical-path item 6 | medium — largest population, and the retarget prompt reuses `enumerate_legal_selections` rather than inventing a path |
| **CV-5 — faces (CR 712)** | A back face on `CardData`, a face-up-side bit, CR 712.2 transform as a status change, 712.3 modal cast-time choice, 707.8's capture-the-up-face line, and the `BackFaceUp` producer for 616.1d. **Consumer: one transform creature and one modal DFC land** | **496** cards; touches `CardData`, the cast path (712.3's face choice), and one line of CV-1's capture | **highest** — it is a second card model, and it is the phase most likely to want its own split once someone counts `CardData`'s readers |
| **CV-6 — face-down (CR 708)** | Layer 1b, `BattlefieldEntity.face_down`, 708.2a's synthesized 2/2, the CR 708.4 cast-face-down path and the turn-face-up special action. **Consumer: one morph creature, cast and turned up** | **304** producers; **1** new sublayer in `LAYER_ORDER`; the bulk is the *casting* path, not the layer | medium-high — **unsized here on purpose.** The layer half is small and known; the casting half needs its own count of `cast.rs`'s alternative-cost sites before anyone commits a number |
| **CV-7 — merging + meld (CR 729, 712.4)** | A multi-component `BattlefieldEntity`, 729.2a's topmost-component characteristics as a Layer 1a copiable effect, 729.3's component separation, 729.3b's exile timestamp ordering, and 729.3d's replacement-applies-to-all-components | 34 + 21 cards; **the largest structural change in this document** and the only one that touches a type every phase reads | **highest, and it has a back-stop** — before Phase 8 card breadth (§6). **Unsized here on purpose**; it earns its own design pass |

**The spine, named.** **CV-1 is this track's RS-1**: small, one arm, and it is
what every other phase is a consumer of. Unlike RS-1 it does not delete anything
— there is no bespoke mechanism to fold in, because nothing produces a layer 1
effect today — so it is net-adding and its risk is concentrated in one line on
the hot path rather than spread across call sites.

**Ordering, and the hard constraints.**

> **CV-1 before CV-2, CV-3, CV-5 and CV-6.** All four carry `CopiableValues`.
> **RC-2 before CV-2**, which needs `EnterBattlefield` to be an event at all.
> **`codebase-state.md` item 10 before CV-1b**, and nothing else in this
> document is blocked on it.
> **CV-4 is free** — it touches no layer, no registry and no replacement, and
> can land at any point from today onward.
> **CV-7 before Phase 8 card breadth**, because a multi-component
> `BattlefieldEntity` is a fact and every phase in between writes code against
> the single-component assumption (§6).

**What each PR must not do.** CV-1 must not touch the ETB path; CV-2 must not
touch tokens; CV-3 must not register a row; CV-4 must not touch the layer
system; CV-5 must not attempt meld; and **CV-1 through CV-5 must not touch CR
708 or CR 729** — those are CV-6's and CV-7's, and reaching for either early is
how CV-1 becomes a `BattlefieldEntity` rewrite. Each is the seam where this
becomes one 5,000-line PR again.

### 7.1 Where this sits in the interleaved order

`cant-effects-architecture.md` §7.1 holds the end-to-end reading of
`CLAUDE.md`'s critical path, and this document changes **three** of its rows and
adds a track. Its own summary framing extends cleanly:

> **Track R** turns events into things cards can modify (RC → RD → RE).
> **Track S** turns rules into things cards can forbid (RS-0 → RS-4).
> **Track V** turns objects into things cards can become (CV-1 → CV-7).
> Track V meets Track R **twice**: RC-2 before CV-2, and RC-4 keeps 616.1b while
> giving 616.1c up to CV-2.

| Change to §7.1's list | Why |
|---|---|
| Step 8 (**RC-4**) loses 616.1c | §5.1 — it cannot produce one |
| **CV-4** may be inserted anywhere from step 4 onward | No coupling to either existing track |
| **CV-1 → CV-2 → CV-3**, after RC-2 and RC-4 | The spine, then its consumers |
| **CV-5** after CV-1, before Phase 8 card breadth | 496 cards, 120 of them potential commanders |
| **CV-1b** after item 10 | §5.3 |
| **CV-6**, **CV-7** after CV-5; CV-7 before Phase 8 card breadth | §6 — CV-7's multi-component permanent is a *fact*, and the back-stop is the same one item 7 has |

**Everything this document names now has an owner**, which was not true of the
earlier draft: CR 708 and CR 729 were "real, sized, and belong to nobody", which
is exactly the condition the detector exists to find. They are CV-6 and CV-7.
What is deliberately *not* here is their design (§6's closing note) — an owner
and an ordering is what a plan owes them; a type surface is what the phase
itself owes, on the evidence available then.

---

## 8. Testing — the atoms this owes

**101 atoms, 101 uncovered**, printed by `copy-census.py --atoms` (which reads
`spec.sqlite` so the number cannot drift from prose):

| CR | Section | Atoms | Uncovered | Corpus phase tags |
|---|---|---:|---:|---|
| 707 | Copying Objects | 30 | **30** | Phase 6 ×23, Phase 7 ×4, Phase 9 ×1, UNKNOWN ×2 |
| 712 | Double-Faced Cards | 36 | **36** | Phase 9 ×34, UNKNOWN ×2 |
| 729 | Merging with Permanents | 19 | **19** | Phase 9 ×19 |
| 708 | Face-Down Spells and Permanents | 10 | **10** | Phase 9 ×9, UNKNOWN ×1 |
| 613.2 | Layer 1 | 3 | **3** | Phase 6 ×2, Phase 8 ×1 |
| 710 | Flip Cards | 3 | **3** | Phase 9 ×3 |

**The phase vocabulary these atoms need**, which is the point of this section —
the corpus files CR 707 with *replacement effects* (Phase 6 ×23) and everything
else in Phase 9, and neither tag names a phase this project schedules:

| Corpus tag | Maps to | Count |
|---|---|---:|
| `D5`, `D5 (copy system)`, `D5 + Phase 7/9` | **CV-1 … CV-4** | 26 |
| `NEW — DFC *` (712) | **CV-5** | 34 |
| `NEW — Meld *` (712) | **CV-7** | ~7 |
| `NEW — Face-down *` (708) | **CV-6** | 10 |
| `NEW — Merged permanent *` (729) | **CV-7** | 19 |
| `NEW — Layer 1 sublayer / copiable values` (613.2) | **CV-1**, and §5.4 | 3 |

Four things to act on rather than read past:

1. **None of these atoms is in `specdb owed`'s default 38, and that is now a
   pattern worth fixing rather than working around.** `owed` filters
   `ticket LIKE 'NEW%'`, and the 707 atoms carry `D5` — so 26 of the most
   load-bearing atoms in this cluster sit only in `owed --all`'s 551. The same
   filter hid the "can't" cluster behind `L15` and `T21b`. **Two for two on the
   large gaps**, which is §9 item 11's subject and a one-line fix in a file this
   pass does not touch. Until then: `owed --all` and grep, or
   `copy-census.py --atoms`. The 712 / 708 / 729 atoms *do* say `NEW` but are
   tagged `Phase 9`, so those are correctly deferred rather than hidden.
2. **The corpus files CR 707 under Phase 6**, i.e. with triggered abilities and
   replacement effects. That is defensible — CR 616.1c is a replacement — but it
   is why the critical path never saw it, and re-tagging is *not* proposed here:
   `specdb.py build` regenerates from the session files, and rewriting 23
   authored atoms to fit a phase name is exactly the kind of edit that makes a
   corpus agree with a plan instead of with the rules.
3. **`specdb.py`'s `SHIPPED_PHASES` gains nothing until CV-5 lands**, and it
   should not gain `Phase 9` even then — Phase 9 also holds meld, face-down and
   merging, which §6 scopes out. The honest gate for this track is the CR
   707 / 613.2 slice, checked by hand at each CV phase's exit.
4. **Two atoms are marked `UNKNOWN` phase in 707 and two in 712.** Worth a read
   during CV-1 sizing; an `UNKNOWN` phase tag is how the corpus records that the
   author could not place it, which is a weak signal that the atom is
   cross-cutting.

---

## 9. Findings and open questions

1. **`register_static_effects` has the same hole `gather`'s gate has, and
   nobody had found it** (§4.7 leg 2). A copy of a permanent with a static
   continuous ability registers no row for it. This is the most consequential
   thing in this document and it is a *feature* gap rather than a fact gap, so
   it costs nothing today and CV-1's price to fix.

2. **`ReplacementClass`'s doc comment names the wrong phase.**
   `types/replacement.rs:316` says `ControlChanging` and `CopyOnEnter` arrive
   "in Phase RC-B". Per §5.1, `ControlChanging` arrives in RC-4 and `CopyOnEnter`
   in CV-2. A one-line correction owed by whichever PR touches that file next;
   deliberately not made here (no `.rs` changes in this pass).

3. **Layer 1's sublayers are named backwards in three places** (§5.4), with a
   correct conclusion drawn from a wrong premise in all three. Corrected in
   `codebase-state.md` here; owed by `layers-architecture.md` §7 and
   `compute.rs:27`.

4. **Open: does `CopiableValues` want to be `Arc`, not `Box`?** Infinite
   Reflection and Essence of the Wild apply one capture to every creature a
   player controls, and Mystic Reflection to a whole batch of entering
   permanents. Under `Box`, each row clones the capture. Under `Arc`, one
   capture is shared and the row is a pointer — still a *value* in §1.3's sense,
   because the target of the `Arc` is immutable and detached from the copied
   object. **Not decided here**: it is 17 cards, the `Box` form is simpler to
   reason about, and the answer should arrive with CV-1b's measurement rather
   than before it. Named so CV-1 does not accidentally foreclose it — which it
   would, if `CopiableValues` were made mutable-in-place.

5. **Open: what should `is_copy` mean?** CR 707.10a's cease-to-exist SBA needs
   "is a copy of a card" and "is a copy of a spell", which are different
   questions, and CR 707.11 ("an effect that refers to a permanent by name still
   tracks it even if it becomes a copy of something else") needs neither. The
   field exists, has no writer, and the honest options are: give it a precise
   CR 707.10a meaning in CV-4, or delete it and let CV-4 introduce what it
   actually needs. **Recommend deciding in CV-4, not before** — a flag with no
   reader has no cost, and inventing a meaning in CV-1 would be speculative.

6. **Open: does the capture belong at ceiling "end of layer 1" or at a
   dedicated `copiable_values(game, id)` entry point?** They are the same
   computation; the difference is whether the ceiling constant is spelled at
   each call site. A named function is better documentation and gives §3.1's
   third property ("status, counters and non-copy effects are not captured") one
   place to assert itself. Weakly recommend the named function; it is a naming
   call and CV-1's author should make it.

7. **CR 707.10b's ability copies are a CR 603.7h identity question, not a
   copy question.** "The copy is considered to be the same ability by effects
   that count how many times that ability has resolved during the turn" is the
   same durable `(source, ability)` identity RA-2 built `AbilityIdentity` for.
   39 clauses, deferred to critical-path item 6, and named here so the triggers
   phase does not discover it.

8. **The clause is the wrong unit for copying, and `cant-census.py`'s success
   with it does not transfer.** A "can't" card can carry several *independent*
   restrictions needing different enforcement points, so a clause is a unit of
   work. A copy card carries one mechanism plus rider sentences about it, so a
   clause is a unit of *sentence*. §2.1 now leads with cards and keeps clauses
   only as provenance for how the six mechanisms were partitioned — which is the
   one job the clause classifier genuinely did. **The census remains worth its
   cost for the partition and for the population table; its headline number
   should not have been a clause count.** Recorded rather than quietly fixed,
   because the next census in this project will face the same choice and the
   deciding question is "can one card need this mechanism twice, independently?"

9. **`ReplacementClass::CopyOnEnter` is named backwards and now is the cheapest
   moment to rename it.** It reads as "enters, then copies", which is the exact
   reading **CR 707.5 goes out of its way to disclaim**: "It doesn't enter the
   battlefield, and then become a copy of that permanent." The CR's own phrasing
   is *as it enters*, so **`CopyAsEnters`** (or `CopyAsItEnters`) says what the
   bucket is. Not renamed here — this pass ships no `.rs` changes — but the
   argument for doing it soon is that the variant has **no producer, no test and
   no card**: it costs one `match` arm today and grows a consumer in CV-2.
   Sibling: 616.1d's `BackFaceUp` is fine, because "enters with its back face up"
   is the CR's own wording.

10. **This document should not design CV-5, CV-6 or CV-7, and §6 deliberately
    does not.** They have populations, an owner and an ordering; they do not have
    a type surface, and giving them one now would be designing three phases
    ahead of the first line of code. **The honest risk this document carries is
    not that it is wrong — it is that it is the third design pass in a row with
    no implementation between them.** RB merged 2026-08-26; since then the tree
    has gained two architecture docs and no engine code. Nothing on the critical
    path is *blocked* on more design: RC-1 is a pure deletion sized at 11 sites,
    RS-0 is a pure refactor, RS-1 is net-deleting, and none of the three needs
    this file. **The recommendation, stated where a later reader will find it:
    build the next thing, and let CV-5–CV-7 be designed when they are next.**

11. **`specdb owed`'s default filter has now failed to surface both of the two
    large gaps anyone has gone looking for**, and that is a pattern rather than
    bad luck. `owed` filters `ticket LIKE 'NEW%'`, which is a filter on *how an
    atom was annotated*, standing in for a question about *what the plan
    schedules*. The "can't" cluster escaped it (tickets `L15`, `T21b`); the copy
    cluster escapes it (ticket `D5`). Both were found by `owed --all` plus a
    grep, which is the query `owed` should have been.

    **This is not an argument against the spec database.** The corpus is
    authored from a close CR read and is the reason both gaps were *findable* at
    all; `orphans` and `suspicious` catch real annotation errors and cost
    nothing; and `stats` answers the coverage question it was built for. The
    defect is one default in one subcommand.

    **The fix is small and belongs in its own PR** (out of scope here: this pass
    touches `plans/references/` only). Either drop the ticket filter and let
    `owed` group by rule prefix, or keep the filter and make `owed` print the
    `--all` count beside the filtered one, so a reader can never see "38" without
    also seeing "551". The second is a one-line change and would have surfaced
    both clusters.

12. **CR 729.3d reaches into the RB pipeline, and CV-7 is where it lands.**
    "If multiple replacement effects could be applied to the event of a merged
    permanent leaving the battlefield … applying one of those replacement effects
    to the object applies it to all components" — that is a CR 614.5 applied-set
    question about an object that is several objects, and it is a sibling of
    `rb-review.md` H9's per-member-vs-per-affected-object question. Named so that
    whoever answers H9 for Phase RD knows a second customer exists, and so CV-7
    does not discover it.

---

## 10. Explicitly out of scope

**Out of scope for this document is not out of scope for v1** — §6 withdrew that
framing. CR 708 (CV-6), CR 729 + meld (CV-7) and CR 710 are *scheduled and
undesigned*: they have an owner, an ordering and a population here, and they get
their type surfaces from their own passes. What follows is out of scope
outright.

- **CR 613.8 itself** — critical-path item 7, and §5.2 argues this track does
  not need it.
- **Text-changing effects (Layer 3)** — deferred indefinitely at 25 cards
  (`engine/layers/types.rs`), but they are §4.7's *other* missing gate leg, so
  the rule in §4.7 is written to cover them in advance.
- **Re-tagging the corpus** — §8 item 2.
- **`plans/archive/*`** — `CLAUDE.md`'s authority table says it is superseded;
  ticket `D5` is named in §0's header only so the 26 atoms citing it resolve.

---

## 11. Documents this owes

Written as part of the PR that lands this file:

- `CLAUDE.md` — critical-path item **5c** and one authority-table row. **No new
  invariant**: the copy invariants belong here until something is built, and the
  file is at its budget.
- `plans/codebase-state.md` — close its own 2026-08-27 demand for this document;
  fix the `706` → `708` cite; correct 2,890 → 517 with the decomposition; add
  CR 729 to the cluster; fix the Layers item-10 sublayer inversion (§5.4).
- `plans/cards-unlocked-ledger.md` — a Part 5 block, one 📋 row per CV phase.

Owed but deliberately **not** written here, because this pass ships no `.rs`
changes and does not edit documents it is not correcting:

- `plans/layers-architecture.md` §7 — the sublayer inversion (§5.4), and a
  pointer here from §6's provenance table for CR 707.9d (§3.4).
- `mtgsim/src/engine/layers/compute.rs:27` — the same inversion, in the comment
  that `LAYER_ORDER` carries.
- `mtgsim/src/types/replacement.rs:316,324` — `CopyOnEnter`'s phase cite (§9
  item 2) and the rename to `CopyAsEnters` (§9 item 9), which is cheapest while
  the variant still has no producer.
- `plans/specdb.py` — `owed`'s default ticket filter (§9 item 11). Its own PR.
- `mtgsim/src/state/game_state.rs:249` — scope the "between them the gate is
  sound" claim, per `rb-review.md` I9, to "sound until Layer 1 or Layer 3 exist".
- `plans/replacement-architecture.md` §9 — RC-4's bucket list, per §5.1.

When CV-5 lands: delete `plans/references/copy-census.py` if §2 has no remaining
customer.
