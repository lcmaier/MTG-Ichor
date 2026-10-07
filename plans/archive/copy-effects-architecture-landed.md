# copy-effects-architecture.md — landed phases, evicted

**A record of finished work, not a plan.** Every section here sat under a ✅
heading in `plans/copy-effects-architecture.md` and was moved out on 2026-09-11 once the phase had shipped,
leaving the heading, a stub and a pointer in the live doc. Nothing here is
owed and nothing here should be acted on; what each section is *for* is the
reasoning — the design as sized, what the building changed, the measurement.
`check_state_of_play.py` reads the ✅ headings that stay, and fails when a
landed section keeps more than 40 lines in the live doc
(`engineering-practices.md` §4). Later phases are appended by the PR that
lands them.

### 7a. CV-1 — the capture, the row and the two legs — ✅ 2026-09-02

*Evicted 2026-09-11 from `plans/copy-effects-architecture.md`, where the heading and a stub remain.*


**What landed**, against the row above: `CopiableValues` and one capture point
(`engine/layers/copy.rs`, with `END_OF_LAYER_1` and its `debug_assert`);
`EffectModification::CopyFrom(Box<CopiableValues>)` applied at `LAYER_ORDER[0]`;
`Primitive::Copy(CopyRoles, Duration)`; `ChoiceKind::ChooseCopySource` and
`ask_choose_copy_source`; **both** gate legs; `register_copied_static_effects`;
Cytoshape and Mirrorweave, with Cytoshape in `PERFORMANCE_POOL` (62 → 63).
1,711 additions — inside §4's 1,500—2,500 band, sized before the first line.
18 integration tests; the CR 707 / 613.2 slice goes 0 → **6 atoms partially
covered** (613.2a-001, 613.2c-001, 707.2-001, 707.2-003, 707.2b-001, 707.4-001),
all partial because every one of their boards is a Clone entering and CV-1 has no
entry producer. `owed` clean; suite green; zero warnings.

**Five findings.**

1. **There are two gates, not one** (§4.7's correction). RS-1 built a second to
   the same recipe and its own comment named CV-1 as the owner of its third leg.
   The rule generates a leg per gate per new route to the effective ability list,
   and the *number of gates* is the term that grows.
2. **Leg 2 needs no teardown path, and that is CR 604.2 paying for itself.** A
   derived row whose copy expired or was superseded stops applying on the next
   walk, because the existence check reads the source's frame and the frame
   includes layer 1. Removal is hygiene, bought by giving each derived row the
   copy row's own `Duration`. What is left is a re-copy within one turn, whose
   superseded rows sit inert until CR 514.2 — recorded in Deferred Migrations,
   because with `Duration::Indefinite` (CV-1b) they would accumulate unbounded.
3. **`Primitive::Copy` needs a role binding the sketch did not have** (§4.2).
   Cytoshape and Mirrorweave attach the atom's target to opposite ends of the
   same sentence, so `CopyRoles` has two arms and a phase that shipped one card
   would have found the second binding in CV-2. **Review found a third thing:**
   the arm's donor exclusion was structural and Mirrorform prints the same shape
   without the word "other", so it is now `exclude_donor: bool` with Mirrorform
   registered as its consumer. The census cannot see a one-word difference
   inside one mechanism — `plans/handoffs/cv-1-review.md` A1.
4. **`--dump-events` is blind to a copy.** Registering a row is not an observable
   action, so the event stream sees a Cytoshape reach the graveyard and nothing
   else — in both arms. First-divergence attribution, which was RC-3 and RC-4's
   sharpest instrument, is a **weak** one for every CV phase: it can only see a
   copy's downstream consequences, and only if they change an outcome before the
   game ends. The counters (`Frames/walk`) are the instrument that works here.
5. **The `PERFORMANCE_POOL` fixture table in `engineering-practices.md` §3 was
   already stale**, by ~11% on the gather column, and the cause is RC-4b rather
   than this phase. Re-recorded; the miss is recorded there.

**The measurement.** Three release binaries, interleaved in one sitting, 200
games / seed 12345 / `--threads 1`, medians of five rounds: `main` at 103acf1
(A), CV-1's engine with `PERFORMANCE_POOL` exactly as `main` had it (B), and CV-1
shipped (C). On `stress` B and C are the same binary by construction, which the
event streams confirm.

| | A: main | B: engine, pool unchanged | C: shipped |
|---|---|---|---|
| performance walks | 100,496 | **100,496** | 99,952 |
| performance frames | 136,402 | **136,402** | 136,726 |
| performance frames/walk | 1.36 | **1.36** | **1.37** |
| performance ms/game | 117.5 | 113.2 | 118.1 |
| performance ms / 1,000 walks | 1.169 | 1.126 | 1.181 |
| stress walks | 92,139 | 97,230 | 97,230 |
| stress frames/walk | 1.31 | 1.31 | 1.31 |
| stress ms/game | 98.3 | 99.8 | 102.6 |
| stress ms / 1,000 walks | 1.067 | 1.026 | 1.055 |

**B against A is exact, not approximate.** Every counter is identical to the
digit — walks, frames, frames/walk, turns, spells cast, gathers, restriction
queries — and the event streams are **40/40 byte-identical** on `performance`.
The engine adds an `EffectModification` arm to a `match`, two flags to a summary
recomputed on mutation, and a leg to each of two gates; on a board with no copy
row none of it runs, and the measurement says so rather than arguing it.

**The timing column separates nothing, and the honest thing is to say so.** The
run-to-run spread on this machine is about —4% to +6% for one binary at 200
games (`main` read 115.2 to 124.3 ms across five rounds), and B reads *faster*
than A on both pools while playing byte-identical games. Anything inside that
band is jitter. **This is what `layers-architecture.md` §12's 5.2—8.0→ figure
predicts, read correctly**: that multiplier is the CR 604.2 existence check
*without its gate*, and a copy row never pays it — `EffectOrigin::Resolution`
returns `true` before any frame is computed. The phase was sized expecting its
risk in the copy row on the hot path, and the copy row is the cheap half.

**What did move is `Frames/walk`, 1.36 → 1.37 on `performance`**, and it is the
*derived* rows, not the copy row: a copied static ability is an
`EffectOrigin::StaticAbility` row like any other and pays exactly what a printed
anthem pays. Citanul Hierophants is the pool's only creature carrying one, so the
+0.7% is one card's worth. On `stress` `Frames/walk` is flat at 1.31 while walks
rise 5.5% — that is two more cards in a 70-card pool, i.e. game content.

**Reachability, counted rather than assumed** (40 games, `--dump-events`):
Cytoshape resolved **once** on `performance`, Mirrorweave **twice** on `stress`.
Thin, and worth naming as thin — a 3-mana instant in a 63-card random pool is
not Keldon Warlord. It is enough to open the path (the `Frames/walk` delta is
the evidence) and not enough for the pool to be where a copy regression would be
caught; the tests are.

**Event streams.** 40 games / seed 12345 per pool, canonicalized — and the
canonicalizer needed a **third** mask, because `SpellFizzled` and the ETB
fallback print a full UUID rather than the 8-hex prefix every other line uses.
Without it two identical games read as divergent at the first fizzle.
`performance` A vs B: **40/40 identical**. `stress` B vs C: **40/40 identical**,
which is the pool separation checking itself. `performance` A vs C: 38/40, and
both divergences are the first differing *draw* — the pool grew 62 → 63, so
`random_deck` builds a different deck. `stress` A vs B: 33/40, all seven at a
draw (68 → 70). **No divergence anywhere traces to a copy resolving**, for
finding 4's reason.

**Determinism holds.** 200 games / seed 12345 per pool, three `--threads 1` runs
each, byte-identical outside `=== Timing ===`. 0 errors, 0 panics, 0 turn-limit
hits on both pools.

**What CV-1 did not build**, deliberately: `Duration::Indefinite` (CV-1b, blocked
on item 10); the entry producer and CR 707.9's exceptions (CV-2); tokens (CV-3);
`is_copy` (§9 item 5, decided in CV-4); `CopiableValues.back_face`, which §3.2
specifies and which CV-1 **omitted** — a field no producer writes and no reader
reads is a Deferred Migrations line bought for nothing, and CV-5 adds it in the
same commit that populates it.

---

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
> `PermanentState` is a fact and every phase in between writes code against
> the single-component assumption (§6).

**What each PR must not do.** CV-1 must not touch the ETB path; CV-2 must not
touch tokens; CV-3 must not register a row; CV-4 must not touch the layer
system; CV-5 must not attempt meld; and **CV-1 through CV-5 must not touch CR
708 or CR 729** — those are CV-6's and CV-7's, and reaching for either early is
how CV-1 becomes a `PermanentState` rewrite. Each is the seam where this
becomes one 5,000-line PR again.

### 7b. CV-2a — enters as a copy (CR 707.5, 616.1c) — ✅ 2026-09-28

*Evicted 2026-09-28 from `plans/copy-effects-architecture.md`, where the
heading and a stub remain. CV-2b's half of the design, D6 with its tables and
CV-2b's sites, size, tests and arms, stayed live there as §7c.*

**What landed**, in four commits against the design below: copiable loyalty
(`2306ac3`); the rewrite, the choice and the carrier (`28c0217`); Clone's
card, 32 tests and the gates' docs (`c11748a`, the A/B's engine arm); and
Clone registered and pooled, 98 → 99, with its eight rulings linked
(`014544b`). 1,551 added lines of code and tests against the ~1,700–2,200
sized: engine 401 against ~600, cards 75 against ~70, tests 1,075 against
~1,000. Atoms covered: ATOM-613.2a-001, 613.2c-001, 707.2-001, 707.2-003,
707.2b-001 and 613.1a-001, all partial since CV-1, and 707.5-001, 707.5-002,
616.1c-001, 608.3e-001, 903.3-002 and 306.5a-001; partially, 707.6-001 and
COMP-9A-002. Closed by hand against the table, since `owed` cannot list this
cluster (§9 item 11).

**What the building changed.** Nothing in the design: every decision held as
reviewed. Four things surfaced in the tests:
1. **Two fixture facts, neither a CV-2 defect.** The registered Grizzly Bears
   carries no Bear subtype, and Master Biomancer's Mutant clause is
   `backlog.md` §2.30's. The tests dropped those assertions rather than bend
   them.
2. **A donor's own entry can end a test's game.** A Wall of Omens placed for a
   player with an empty library draws as it enters, and CR 704.5b ends the
   two-player game before the Clone's copied trigger resolves. The test
   stocks both libraries.
3. **A fixture's rulings cannot be linked.** The gate refuses a `RULING:` line
   for a card the ledger does not hold, so Essence of the Wild's rulings are
   cited in the tests' prose. Clone's links waited for the commit that
   registered it.
4. **"About half" was read, not assumed.** Clone's `(0, 1)` pick copied in 27
   of the 50 choices a 200-game trace asked, and the other 48
   `ChooseCopySource` prompts in it were Cytoshape's forced picks.

**The measurement.** `fuzz-record.md`, CV-2a's block: every engine counter
file byte-identical to `main`'s outside the timing, on both pools at two
seats and four; +0.01% instructions per decision; Clone reached in 61% of
two-seat games and 46% of four-seat ones.

**Trace-page decision: no** (`engineering-practices.md` §7's test). The reads
CV-2a adds ride structures two pages already walk: the would-be entity is
RC-4b's page's, and the snapshot applied at layer 1a is CV-1's. The frame now
answers "a Colossus" for an entering Clone, where it answered "a 0/0
Shapeshifter", through the same accessor.

#### The design, as reviewed


**Design, re-derived 2026-09-28 against 861a812, for review before any code.**
The CV-2 row in §7 was sized on 2026-09-02, before RC-4's look-ahead frame,
TR-1's dispatcher and #188's shared payloads. Built as written, it would have
put a copy on the wrong CR 616.1 step (fact 1 below).

#### The four facts the row predates

1. **The class is derived from the rewrite.** `ReplacementClass::from_rewrite`
   (`types/replacement.rs:1687`) files every `Rewrite::Instead` under 616.1e's
   `Other`. §4.1's "a `Rewrite::Instead` producing an `EnterBattlefield`" would
   have put Clone's copy in the same bucket as "enters tapped". That is the
   board CR 616.1f's own example (Essence of the Wild and Rusted Sentinel)
   exists to order. 616.1c gets a producer only from a rewrite that names it,
   as 616.1b's `EnterUnderControlOf` does.
2. **Every gate's printed leg is filed at the entry and cleared at the
   departure.** `register_static_effects` (`state/game_state.rs:1639`) fills
   `trigger_sources`, `replacement_ability_sources`,
   `restriction_ability_sources` and `cost_modification_ability_sources` from
   the ability list as the permanent is placed. `cleanup_zone_state`
   (`engine/zones.rs:444`) empties them as it leaves. §4.7 put copies on the
   summary's legs because a Tier C row "can expire without a zone change". An
   entry copy cannot, so it can use the legs a printed ability uses.
3. **The look-ahead builds a would-be entity from the proposal.**
   `Lookahead::new` (`engine/layers/lookahead.rs:58`) seeds the entering
   object's `PermanentState` from `EnterMods`: tapped, and CR 122.6a's
   counters, which the board pass applies at layer 7c at the entity's timestamp
   (`engine/layers/board.rs:936`). A value that rides the proposal and lands on
   the entity is already a shape the frame reads.
4. **`CopyFrom` has held an `Arc` since #188** (244c64a, 2026-09-25), because
   the registry is cloned with every fork and a `Box` cost ten or more
   allocations per copy row. §9 item 4 still says `Box`.

#### The decisions

**D1. The carrier is state on the permanent, not a registry row.** The shared
problem: between the CR 707.6 choice and the performer, the copy has to be in
CR 614.12's frame, so the loop's next iteration sees the copied abilities
(707.5). On the battlefield it has to apply at layer 1a in timestamp order
with any later Tier C copy (707.4). Every gate has to see the copied abilities
before the entry is dispatched. And the copy has to end exactly when the
permanent leaves (400.7). All four hold for both producers: the entering
permanent's own ability (Clone, 59 of the 62 printed cards) and another
permanent's (Essence of the Wild, Infinite Reflection, Mystic Reflection).

| | A: a `CopyFrom` row at placement (§3.3 as written) | **B: `PermanentState.entered_as_copy` (recommended)** |
|---|---|---|
| Code | the row; a new `EffectOrigin` arm, since `Resolution` would be a lie (no resolution made it) and `StaticAbility` would fail CR 604.2's check the moment the copy removes the ability that made it; a would-be row in the look-ahead; `register_copied_static_effects` at placement | one field, written by `place_on_battlefield`; one layer-1 application in the board pass, a `Kind::Own` beside the counters'; the would-be entity carries it; registration reads it |
| Gates | CV-1's summary legs, automatically. Every sweep then widens to every permanent (`gather.rs:282`, `dispatch.rs:747`, the restriction and cost sweeps) for as long as the permanent lives. CV-1 accepted that for turn-bounded rows | the printed legs, per object and exact: registration files the copied list, and `cleanup_zone_state` clears it |
| Teardown | `remove_by_source`, with `source` the entering permanent. Exact for Clone. For Essence it is §5.3's refused overload, and making Essence the `source` ends every copy when Essence leaves, which its rulings say does not happen | the entity: `remove_from_zone_collection` drops the `PermanentState`, so the copy has no row, no duration and no source |
| Timestamp | allocated at placement | the entity's (CR 613.7d), re-stamped with it under 613.7e |
| Cost | nothing on a board without such a permanent; on a board with one, every sweep walks every permanent for as long as it lives | one field read per member in `Board::seed`, which already reads the entity, and an empty list at layer 1; on a board with one, the sweeps visit it the way they visit a printed ability |

B also follows a rule the tree is already converging on. **How a permanent
entered is state on it**: `tapped`, the counters, `face_down` (CV-6's layer
1b, §4.6), and the entered-as type `backlog.md` §2.30 designs for Master
Biomancer ("no row, because it is not an effect with a source"). **What an
effect later does to it is a row.** A Tier C copy over an entry copy is
exactly that pair. The row applies after the state at layer 1a by timestamp,
and when it expires the entry copy shows again. §1.2's three objections to a
`CardData` swap all fail against B: the card is still Clone in the graveyard,
the state has the entity's timestamp, and a later row supersedes it.

**Tier C is unchanged, and Mirrorweave shows why the line is the entry and not
the source.** A copy an effect makes after a permanent has entered is CV-1's
row. Mirrorweave's source is the spell, its subjects are every other
creature, and CR 611.2c locks that set as the row begins; the row lasts its
own duration and expires at CR 514.2. Neither the spell leaving nor the donor
leaving ends it, since the values are a snapshot, and a subject leaving is
main item 10's (CV-1b). Essence of the Wild's copies are the entry-time case
of the same thing, a source that is not the affected permanent, and they are
state because they are made as the permanent enters. **What decides the carrier is
when the copy is made, never who made it.**

**D2. The rewrite is `Rewrite::EnterAsCopy(EntryCopyTemplate)`, the eighth
arm.** CR 614.1c permits it ("[This permanent] enters as . . ."), and CR
616.1c names its class by what the effect does, so `from_rewrite` maps it to
`CopyAsEnters`. That is §9 item 9's rename, made in the same commit. It is not
an `Instead`: fact 1, and an `Instead` overwrites the event that CR 616.1f
accumulates. It is not a field of `EnterModsTemplate` either. `is_fixed` and
`classify`'s `ModsAdding` cell rest on an `EnterWith` writing counters and a
status that two applications merge in either order. A copy is neither: it
prompts, and the later of two copies wins (Essence of the Wild's third
ruling). So `classify` answers `None` for it, a real choice beside anything.

The template is the authored half and `EnterMods.copy` (D3) the evaluated
one, as `EnterModsTemplate` is to `EnterMods`. CV-2a's template is
`{ donor: CopyDonor }`, and CV-2b adds `except: Vec<CopyException>`. The donor
has three arms, one per printed binding:
- `Chosen(SelectionFilter)`: "any creature on the battlefield" (Clone,
  `SelectionFilter::Creature`) or "a creature or planeswalker you control"
  (Spark Double, a `Permanent` filter over `Or` and `ByController`). 52 of the
  62 printed cards.
- `ThisObject`: Essence of the Wild's "this creature".
- `Host`: Infinite Reflection's "enchanted creature".

**D3. Between the choice and placement, the values ride `EnterMods.copy`.** It
is an `Option<Arc<CopiableValues>>`, written by the arm and by nothing else,
and read three ways:
- The look-ahead's would-be entity carries it. So every later iteration's
  frame sees the copy: source 1a, `set_affects`, and the "can't" predicate.
  Worms of the Earth refusing a Clone that chose Dryad Arbor, CR 608.3e's own
  example, needs no new code.
- `would_be_rows` lowers the copied static abilities instead of the printed
  ones (CR 614.12's clause 2, for the object as it would exist).
- `place_on_battlefield` moves it onto the entity.

The `Arc` is #188's reason again: the pipeline clones the event every
iteration and a fork clones the entity, and a `Box` would allocate at each
(floor 2). A second copy in one entry replaces the first, never merges with
it. Essence of the Wild's third ruling ("the one whose copy effect you apply
last") and CR 707.9e's cancellation say the same thing from two sides, so
`EnterMods::merge` asserts that it is handed no copy. CV-2b widens the field
to `EntryCopy { values, added }` for 707.9e (D6).

**D4. The CR 707.6 choice.**
- **Who chooses:** the entering object's controller. That is the proposal's
  `controller`, after any CR 616.1b effect has applied, since 616.1b is the
  step before. For Clone it is also the instance's controller, but 707.6 names
  the object's controller, so that is the value the arm reads.
- **The candidates:** `enumerate_legal_selections` over the battlefield, in
  timestamp order. It is a choice, not a target, so hexproof and shroud do
  not apply (Clone's first ruling). The entering object is not on the
  battlefield while its entry is decided, and neither is anything entering in
  the same batch, so Clone's fifth ruling holds by construction (main item 46's
  one-board decision). A candidate that is itself a copy gives what it copied,
  because the capture is `copiable_values` at `END_OF_LAYER_1` (§3.1; 613.2c).
- **The "may":** the def's `optional` stays the text's "you may". It is asked
  inside the choice, as a `(0, 1)` pick over the candidates, so declining is
  picking none. That is devour's precedent: `ChooseAuxiliaryZoneChange`'s
  "declining is a count rather than a separate optional-replacement prompt".
  The pipeline skips its own yes/no (`pipeline.rs:500`) for a rewrite that
  asks the "may" itself. CR 616.1's two-candidate rule does not apply, because
  this is a choice within one replacement. The one-outcome rule does apply:
  one candidate is two outcomes and is asked; no candidate is nothing and is
  not asked; a mandatory chosen copy (none is printed) over one candidate is
  forced. A decline applies the effect and changes nothing, and CR 614.5's
  applied set already spends it.
- **The prompt:** `ChoiceKind::ChooseCopySource`, reused, with `spell_id`
  renamed `source`: the object whose copy effect is choosing. For CV-1 that is
  the resolving spell or ability; for Clone it is the entering permanent, whose
  own ability it is. The question is the same one, and main item 68 forbids
  a second variant for it.

**D5. CR 707.5's last sentence is an order inside `place_on_battlefield`.** The
entity is inserted with `entered_as_copy` set, then the counters go on, then
`register_static_effects` runs, then the entry is announced. Registration
reads the abilities the permanent entered with: the copy's, or else the
printed ones. So the copied triggered abilities are in `trigger_sources` before
the batch closes and dispatches the entry (CR 603.6a, "including the
newcomers"). The copied "enters with" and "as enters" abilities have already
applied during the loop, found by source 1a off the frame (`gather.rs:234`).
The comment that registration "reads printed abilities on purpose"
(`game_state.rs:1633`) becomes "reads the abilities the permanent entered
with". That is still not a frame, so it is still not circular.

*D6 stayed live as `copy-effects-architecture.md` §7c.*

**D7. Loyalty is a copiable value (CV-2a).** CR 707.2 lists it and CR 109.3
makes it a characteristic, but `EffectiveCharacteristics` has no such field. A
Clone of a creature that is also a planeswalker would enter with none (no such
card is registered today), and CV-2b's Spark Double copying Grist would enter
with only its one additional counter. `loyalty: Option<i32>` joins the frame.
It is seeded from `CardData.loyalty`, written by `CopyFrom` and carried by
`CopiableValues`, and `default_enter_mods` reads CR 306.5b's "printed loyalty
number" off the frame.

A copy's application then rebuilds the counters the rules gave:
`mods.counters = 306.5b(result)`, plus this copy's 707.9e counters in CV-2b,
each through `strip_prohibited_counters`. The rebuild is exact because CR
616.1's ladder puts every 616.1c application before any 616.1e one. At a
copy's application, the only counters in the mods are the seed's and a
previous copy's. That is asserted in debug builds, because the premise is a
property of the printed pool: a copy that becomes applicable only after an
"enters with" has applied is unprinted.

**D8. Teardown, and what CV-1b must leave alone.** The copy leaves with the
entity. The copied static abilities' rows are `register_static_effects`' own
(`EffectOrigin::StaticAbility`, `WhileSourceOnBattlefield`, with the permanent
as source), torn down by `remove_by_source` like every printed static's. Tier
B never uses `Duration::Indefinite`, because there is no row to give it to.
What main item 10's subject-keyed teardown must leave alone is §5.3's fact, now
general rather than Tier B's: `remove_by_source` stays beside it for every
static row, printed or entered-as.

**D9. §9 item 4 is answered: `Arc`, already.** #188 moved the row's payload to
`Arc`, and D3 carries the same `Arc` on the proposal and the entity.

**D10. Two PRs, not one** (the size is below). **CV-2a** is the carrier, the
rewrite, the choice, the frame, registration, copiable loyalty and Clone.
**CV-2b** is CR 707.9's applier and Spark Double. Each carries its consumer
(§4), and A6c becomes three PRs.

#### The standing question (§4.1), asked of the new arm

*What does it check?* Nothing after the choice. It reads the board twice at
the choice (the candidates, then the donor's values), and the values are never
re-read, so the arm has nothing to compare later. *What happens if it runs
twice?* A second copy replaces the first (D3), and the loop terminates. Each application spends one CR 614.5 entry keyed
`(entering object, ability id)`. A copied copy ability is a new key only when
it comes from a different card. The printed ids on the battlefield are
finite, so a chain of copies ends.

One consequence of that key, recorded rather than fixed: a Clone that copies a
Clone which copied nothing is not re-offered Clone's ability, because the ids
are equal (`AbilityId::printed` hashes the card name). The CR would re-offer
it, as in 707.9e's Altered Ego example. It is unobservable, because the second
offer's candidates and exceptions are the first's.

`filter_is_mods_invariant` rests on a premise Master Biomancer's doc comment
states: `ByType` and `BySubtype` are invariant "only because no `EnterMods`
field feeds a type". `copy` feeds every characteristic. The premise survives
in the form the predicate actually needs: no member of a *suppressible* bucket
writes `copy`. Only `EnterAsCopy` writes it, `classify` never admits
`EnterAsCopy`, and CR 616.1's ladder never puts a 616.1c effect in the same
bucket as a 616.1e one. The comment says so in CV-2a.

#### The sites, counted

| Site | CV-2a |
|---|---|
| `types/replacement.rs` | `CopyOnEnter` → `CopyAsEnters`; `Rewrite::EnterAsCopy`, `EntryCopyTemplate`, `CopyDonor`; `EnterMods.copy` (the const, 2 constructors, `is_none`, `merge`); 2 exhaustive `Rewrite` matches (`from_rewrite`, `is_prevention`) |
| `types/effects.rs` | none |
| `engine/replacement/pipeline.rs` | `apply_rewrite`'s arm (donor, choice, capture, the D7 rebuild); `classify` (1 arm); the optional branch's skip (1); `strip_prohibited_counters`' literal (1); `filter_is_mods_invariant`'s comment |
| `engine/layers/lookahead.rs` | the would-be entity carries the copy (1); `would_be_rows` reads the entered-with list (1) |
| `engine/layers/board.rs` | `Board::seed` notes each member's entry copy off the entity it already reads (1); a layer-1 `Kind::Own` per noted member (1); `Tiebreak::EntryCopy` (1) |
| `engine/layers/types.rs`, `compute.rs`, `copy.rs` | `EffectiveCharacteristics.loyalty`, `seed_frame`, `from_frame`, `apply_to`; `land_types.rs`' test literal |
| `state/battlefield.rs` | `PermanentState.entered_as_copy` and `new` |
| `state/game_state.rs` | `place_on_battlefield` writes it (1); `register_static_effects` reads the entered-with list (1); `default_enter_mods` reads the frame's loyalty (1) |
| `ui/` | `ChooseCopySource`'s `spell_id` → `source` (`choice_types.rs`, `ask.rs`, `cli.rs`, `resolve.rs:1810`, `tests/prompt_subject_test.rs:241`); `ask_choose_copy_source` takes its bounds |
| the four gates | **no code**: the printed legs (`gather.rs:284`, `restriction/predicate.rs:82`, `cost_determination/gather.rs:64`, `dispatch.rs:530`) read the sets registration files; their comments name the entry copy |
| cards | Clone; `PERFORMANCE_POOL` 98 → 99 |

#### Size against §4's band

Calibrated on CV-1, which shipped 1,711 lines for a comparable surface (a
payload type, one producer, a prompt, two cards, 18 tests in 854 lines), and
on §4's warning that tests run about twice their row.

| | Engine | Cards | Tests | Code and tests | Docs |
|---|---:|---:|---:|---:|---:|
| CV-2a | ~600 | ~70 | ~1,000 | **~1,700–2,200** | ~250 |
| as one PR | | | | ~2,800–3,700 | |

One PR runs over the band. The split is where the cards already divide the
work: Clone needs no exception, and Spark Double needs little else.

#### The tests, and the atom each claims

All tests are in `tests/phase_cv2a_integration_test.rs` and
`tests/phase_cv2b_integration_test.rs`. A fixture is a card built inline in
the test, named for the printed card whose board it stands in for, and never
registered.

| Test | Atom | Claim |
|---|---|---|
| Clone cast from hand (`cast_spell` under `ManaWindowStop`, exact `{3}{U}`) copies a 5/5 | ATOM-613.2a-001 | COVERS (CV-1 had it partial) |
| Clone B copies Clone A, a copy of Grizzly Bears | ATOM-613.2c-001 | COVERS (was partial) |
| the donor's +1/+1 counters, tapped status, pump and animation are not copied; a Clone of an animated noncreature artifact is that artifact, with its ability | ATOM-707.2-001, ATOM-707.2-003 | COVERS both (were partial) |
| Cytoshape makes the donor a copy of something else, and the Clone is unchanged | ATOM-707.2b-001 | COVERS (was partial) |
| -3/-3 on a Clone of a 5/5 leaves a 2/2 | ATOM-613.1a-001 | COVERS (was partial) |
| a Clone of a Skyshroud Behemoth-shaped fixture enters tapped with two counters; a Clone of Chainbreaker, with two -1/-1 counters | ATOM-707.5-001 | COVERS |
| a Clone of a Wall of Omens-shaped fixture draws a card | ATOM-707.5-002 | COVERS |
| a Clone of Thunder-Thrash Elder: its controller makes the devour choice, and the Elder's counters are not copied | ATOM-707.6-001, COMP-9A-002 | PARTIAL both. Their "choose a creature type" and "choose a color" boards are `backlog.md` §2.2's "as it enters" choice record |
| an Essence of the Wild-shaped fixture and a Rusted Sentinel-shaped fixture make an untapped copy | ATOM-616.1c-001 | COVERS |
| a lands-can't-enter fixture refuses a Clone that chose Dryad Arbor, which goes to the graveyard | ATOM-608.3e-001 | COVERS (RC-4b had it partial) |
| a Clone of a commander is not a commander and deals no commander damage | ATOM-903.3-002 | COVERS |
| a planeswalker card in hand has its printed loyalty | ATOM-306.5a-001 | COVERS |

Without an atom, in CV-2a: declining gives a 0/0 that dies; one candidate asks
once; no candidate asks nothing; a creature entering in the same batch is not
a candidate; a token donor, and the Clone is not a token; the legend rule; a
Clone in the graveyard is a Clone card and its derived rows are gone; one test
per gate (Clones of Master Biomancer, Sigarda, Thalia and Soul Warden); a
Clone of Citanul Hierophants grants the mana ability; Root Maze taps a Clone of
Chainbreaker (616.1f through the frame); Cytoshape over a Clone, then expiry
restores the entry copy; two Essence-shaped fixtures give the last one applied;
a Clone beside one gives the Essence; the `Host` donor; a Clone of a creature
planeswalker fixture enters with its loyalty, doubled under Doubling Season.

#### The A/B arms

**CV-2a.**
- `engine`, the last commit before Clone is registered, against `main`:
  every gameplay and diagnostic counter `IDENTICAL` on both pools at two seats
  and four. Instructions per decision within +0.3 points: one field read per
  member in `Board::seed`, an empty list at layer 1, and one entity read per
  registration.
- `shipped`, HEAD with Clone pooled (98 → 99), against `main`: every row
  moves, since both pools changed. Clone copies in about half its entries (the
  random provider's `(0, 1)` pick declines the other half), `Decisions` rise
  by that prompt, and there are 0 errors, panics and turn-limit hits. No
  scripted fixture migrates, because no existing test casts Clone.

#### Findings, and the review

1. **`owed` misses this cluster twice over** (§9 item 11). The ticket filter
   hides the `D5` atoms, and the phase filter hides `ATOM-707.5-002` even from
   `--all`.
2. **Main item 40's `PendingReplacement` omits the group's members.** Its sizing
   holds the three sets, but a fork at a CR 616.1 prompt after an application
   also needs the rewritten events, and CV-2's captured copy is the largest
   thing they carry. One line on the item, with CV-2a.
3. **RS-2 must keep hexproof out of `validate_selection`.** Clone's and
   Cytoshape's choices both enumerate through it, and neither targets.
4. **Essence of the Wild beside Clone asks a 616.1c prompt whose board outcome
   is fixed** (Essence's seventh ruling). It is accepted: `classify` cannot
   prove it, and it is a real CR 616.1 choice.
5. **The engine cannot say "enters untapped"** (D6): `backlog.md` §2.40.
6. **Every elision is argued at its own site, and nothing checks them
   together.** The premise `filter_is_mods_invariant` rests on had to be
   re-argued here (the standing question above). The owner's note from the
   review: formalize exactly what the engine elides and why, and show the
   system sound. That is `codebase-state.md` main item 185, not CV-2's.

**The review (the owner, 2026-09-28).** D1 to D5 and D7 to D10 are agreed,
with the Essence board a fixture and main item 66 (`--dump-state`) moved to
CV-1b, where main item 10 is its other customer. D6 was reworked in answer to the
review's two questions: whether a new category of exception forces a
retrofit, and what a custom card can say. The superseded text elsewhere in
this document is corrected where it stands, in §1.3, §3.1, §2.4, §3.3, §4.1,
§5.3 and §9, so each fact is stated once. The wider drift is for the docs
audit the owner has in mind after the triggers phase.

### 7c. CV-2b — CR 707.9's exceptions (Spark Double) — ✅ 2026-09-29

*Evicted 2026-09-29 from `plans/copy-effects-architecture.md`, where the
heading and a stub remain. The section's three standing subsections, the
arms, the Kaito board and the printed population, moved to that doc's
§4.1a, live, where CV-1b and CV-3 read them; what is here is the record.*

**What landed**, in five commits against the design below: the design
re-derived against RG and the Kaito board decided (`abd60be`); the engine
map and the feedback-loop census on the route, and this phase's trace page
decided (`9967841`); the vocabulary, the applier and Spark Double's card
(`d970c35`); 33 tests (`eccfc6b`, the A/B's engine arm); and Spark Double
registered and pooled, 100 → 101, with its nine rulings linked (`e1bc838`,
the shipped arm). +1,901 added lines of code and tests against the
~1,300–1,600 re-derived: engine 520 against ~480, cards 103 against ~90,
tests 1,278 against ~850. Atoms covered: ATOM-707.9f-002, 707.9f-001,
707.9e-001, 707.9b-001 and 707.9d-001; partially, 707.9d-002, 707.9a-001,
707.3-001 and COMP-9A-006. Closed by hand against the table below, since
`owed` cannot list this cluster (§9 item 11).

**What the building changed.** No decision; four things surfaced:
1. **Where the applier lives.** The sites named `pipeline.rs` for 9e and 9f.
   They went to a new `engine/replacement/entry_copy.rs`, which the
   `EnterAsCopy` arm calls and to which `pipeline.rs` lends
   `evaluate_enter_template` and `strip_prohibited_counters`. Nothing
   reached `compute.rs` or `layers/types.rs`: made at the capture, the
   exceptions leave layer 1a a finished snapshot.
2. **The builder had to walk exceptions.** An ability a copy exception gives
   is nested in the copy ability's replacement, and `CardDataBuilder::build`
   stamps ids only on the defs `Effect::for_each_ability_def_mut` reaches.
   `CopyException::for_each_ability_def_mut` joined the walk, and a test
   asserts the stamp.
3. **A status claim needed an exact release.** `EnterMods::take_back`
   restores the status an addition replaced, and `EnterMods::merge` drops
   that claim when a later effect sets a status. A board with a copy made
   applicable by a status write pins it. Counters are exact in CR 616.1's
   order; `codebase-state.md` main item 189 records the multiplier case,
   which no registered card reaches.
4. **The tests outgrew the estimate**, 1,278 lines against ~850: a third of
   the file is its own provider and fixtures, as CV-2a's is, and the rulings
   and the Kaito board added boards the table did not list.

**The measurement.** `fuzz-record.md`, CV-2b's block. The engine arm
`IDENTICAL` to `main` on every counter at two seats and four, +0.12%
instructions per decision, 0.02 over the predicted ±0.1: a size probe on
both trees attributes it to `EnterMods` growing 56 → 88 bytes (`GameAction`
80 → 112), and `Option<Arc<EntryCopy>>` is the lever. Spark Double reached
in 66% of two-seat games and 48% of four-seat ones.

**Trace page: `plans/traces/cv-2b-an-exception-is-checked-without-itself.html`**,
decided yes by the owner at the design review, ahead of the close.

#### The section as re-derived and reviewed

**Designed and reviewed with CV-2a on 2026-09-28** (§7b, landed), and
**re-derived on 2026-09-29 against RG**, which landed under it
(`replacement-architecture.md` §3.5). CV-2b builds CR 707.9's applier on
CV-2a's entry copy, and three of CV-2a's shapes grow for it:
- `EntryCopyTemplate` gains `except: Vec<CopyException>`.
- `EnterMods::copy` becomes `Option<EntryCopy { values, added }>`, so a later
  copy in the same entry removes exactly what a 707.9e exception added.
- A copy's application adds this copy's 707.9e counters, through
  `strip_prohibited_counters` as an `EnterWith`'s are, into
  `EnterMods::counters`, where a multiplier (CR 616.2), the performer and the
  look-ahead already read an entry's counters; `added` records them. It
  rebuilds nothing: since RG, CR 306.5b is an ability the gather finds on the
  copy's frame after the copy applies.

The consumer is Spark Double, a `CopyDonor::Chosen` over "a creature or
planeswalker you control" whose exceptions are 707.9b's "isn't legendary" and
two of 707.9f's conditions on 707.9e's additional counters.

#### What the re-derivation changed (2026-09-29)

RG changed the ground under four of this section's assumptions, and one of
them changes a decision:
1. **A decision: the frame CR 707.9f's condition is read against.** As
   reviewed, each condition was read "with every unconditional exception
   applied and no conditional one". Since RG the look-ahead counts the counters
   an entry already carries (the register row `lookahead-entry-counters`), and
   on Kaito's board that rule contradicts 707.9f's own words (the Kaito board,
   below). The design now reads them as written: each conditional exception is
   judged against the copy without it, with every other exception that applies
   there, and a conditional one is judged the same way, without both. Each
   judgment leaves out the exception it is about, so the recursion ends, at a
   depth of at most the number of conditional exceptions: two on Spark Double.
2. **The feeds table's `copy` row covers what a copy's 707.9e exceptions
   write** (`replacement-architecture.md` §3.5): counters and a status on the
   entry, written only at CR 616.1c's step, as the copiable values are. So the
   row's exemption holds for them for the same reason, and
   `EntryWrites::of_candidates` keeps its empty `EnterAsCopy` arm.
3. **A status is set, not accumulated, since RG** (`EnterMods::status`), so
   "removes exactly what it added" means, for a status, putting back the one
   the copy replaced. `EnterMods::merge` drops the copy's claim to the status
   when a later effect sets one, so a later copy never restores over that
   effect's status.
4. **`Additionally` carries an `EnterModsTemplate`, to which RG added
   `edits`.** CR 707.9e defines its exception as "an additional effect rather
   than a modification of the affected object's characteristics", so an edit
   there is `Modifies` written in the wrong arm, and the applier refuses it.

And two stale facts: the performance pool is 100 since Archelos (RG), so
Spark Double makes it 101; and Grist is not registered (Loyalty Probe is the
only registered planeswalker), so the A/B's planeswalker path is a test
board's, not a pooled card's.

*The design, the Kaito board and the printed population stood here; they
are the live doc's §4.1a.*

#### The sites

- `types/replacement.rs`: `EntryCopyTemplate.except`; `copy` becomes `Option<EntryCopy>`, and `merge` drops a copy's claim to the status it set
- `types/effects.rs`: `CopyException` (4 arms, one per CR 707.9 sub-rule); `CharacteristicEdit` gains the copy placement's arms its fixtures use (`GainsAbility`, `GainsKeyword`, `PowerToughness`, `Name`), each refused at the entry placement until an entry card needs it
- `engine/replacement/pipeline.rs`: the arm's exceptions; 9e's `EntryCopy.added`, and a later copy taking it back; 9f's recursive frame
- `engine/layers/copy.rs`: the modifications made to the captured values, and 9d's drop; the CDA classifier in `cda.rs`. Nothing in `compute.rs` or `layers/types.rs`: the exceptions are made once, at the capture, so layer 1a applies a finished snapshot as it does today
- the readers of `EnterMods::copy` (`PermanentState::entering`, the look-ahead's would-be rows): `.values`
- cards: Spark Double; 100 → 101

#### Size against §4's band

| | Engine | Cards | Tests | Code and tests | Docs |
|---|---:|---:|---:|---:|---:|
| CV-2b, as reviewed | ~450 | ~80 | ~700 | ~1,100–1,500 | ~150 |
| CV-2b, re-derived 2026-09-29 | ~480 | ~90 | ~850 | **~1,300–1,600** | ~250 |

Calibrated on CV-2a, which was sized at ~1,700–2,200 and shipped 1,551
added lines: engine ~600 against 401, tests ~1,000 against 1,075. The
re-derivation adds 707.9f's recursion and the status bookkeeping (~30), and
tests: the Kaito board, four boards for rulings #7 and #8, and the six other
rulings. The band's floor is 1,500, so this sits at its bottom edge, and the
docs grow with the two register rows.

#### The tests, and the atom each claims

| Test | Atom | Claim |
|---|---|---|
| Spark Double cast from hand copies a legendary creature you control: a +1/+1 counter, not legendary, no legend rule | ATOM-707.9f-002 | COVERS |
| Spark Double copies a noncreature artifact a resolution animated: no counter | ATOM-707.9f-001 | COVERS |
| Spark Double copies a Clone that copied nothing (Glorious Anthem keeps it alive), then Clone's copied ability copies a Bear: no +1/+1 counter | ATOM-707.9e-001 | COVERS |
| a Copy Artifact-shaped fixture copies Darksteel Myr, and a Clone of it is an artifact creature enchantment | ATOM-707.9b-001 | COVERS |
| a Quicksilver Gargantuan-shaped fixture copies Tarmogoyf: 7/7, with no CDA | ATOM-707.9d-001, COMP-9A-006 | COVERS; PARTIAL (changeling) |
| a Glasspool Mimic-shaped fixture keeps a subtype CDA | ATOM-707.9d-002 | PARTIAL: changeling is unbuilt |
| a gains-an-ability fixture, and a Clone of it has the ability | ATOM-707.9a-001 | PARTIAL: the atom's Unstable Shapeshifter is Tier C |
| a Vesuvan-shaped fixture copies Culling Drone and keeps its own color with no devoid, and a Clone of it is the same | ATOM-707.3-001 | PARTIAL: the upkeep copy is CV-1b's |

Also in CV-2b, with no atom:
- **Spark Double's rulings** (§3.4), one linked test each: what is not copied
  (#1), a copy of it is not legendary either (#2, the first row above), X is 0
  (#3), a copy's copy (#4), a token's values without being a token (#5), the
  copied "enters" abilities and triggers (#6), and nothing entering beside it
  (#9).
- **Ruling #7:** Spark Double copying Loyalty Probe, and Grist's clause, gets
  printed plus one and no +1/+1; copying a creature that "enters with" two
  +1/+1 counters, under Master Biomancer, it gets 1 + 2 + 2. Beside Doubling
  Season the planeswalker's count is an order: CR 306.5b first reaches 8,
  Doubling Season first 5 (premise (d)'s shape, on the copy's counter). The
  `// RULING-DEVIATION:` test is the Kaito board's.
- **Ruling #8:** March of the Machines makes a copied Sol Ring a creature as
  it enters, so it gets the counter; a Gideon Jura-shaped planeswalker a
  resolution animated is copied as a noncreature planeswalker, with no +1/+1;
  a Gideon Blackblade-shaped planeswalker creature gets both counters and CR
  306.5b's.
- **The Kaito board**, beside Oath of Gideon: 1 loyalty and one +1/+1
  (`lookahead-entry-counters`, `copy-exception-conditions`).
- A "can't have counters" fixture strips the +1/+1; `Modifies(Name)` and
  `Modifies(GainsKeyword)` fixtures; the gray area's two readings, on the
  entry state's "enters untapped" (`backlog.md` §2.40), and a later copy
  putting back the status a 707.9e exception set; a declined Spark Double is
  a 0/0 Illusion, and CR 704.5f takes it; an edit in `Additionally`, and an
  `If` in an `If`, are refused.

**The gate:** `specdb owed` cannot close this phase, and it is two short, not
one. Its default filter still hides the 707 atoms ticketed `D5` (§9 item 11),
and `ATOM-707.5-002` is filed under Phase 7, which `SHIPPED_PHASES` leaves
out, so even `owed --all` never lists it. CV-2 closes by hand, against this
table.

#### The A/B arms

- `engine` against `main`: `IDENTICAL`. No registered card prints an
  exception before Spark Double, so the applier never runs, and the only
  change on the hot path is the `EntryCopy` a Clone's entry clones instead of
  a bare `Arc`. Instructions per decision within ±0.1 points (added at the
  build, before any arm ran).
- `shipped`, Spark Double pooled (100 → 101): every row moves. `--require
  "Spark Double"` reads its +1/+1 path on `performance`. The planeswalker path
  is the test boards': Loyalty Probe is the only registered planeswalker, a
  fixture that stays out of `performance`, and a stress game reaches it only
  if one player draws both. Spark Double is reached in about as many games as
  Clone was at CV-2a (61% at two seats), copies less often than Clone, since
  its donors are its controller's own creatures, and adds two to four look-ahead
  walks per copy (707.9f's judgments), so `Layer walks` rises a little. Zero
  errors, panics and turn-limit hits (added at the build, before any arm ran).

**Trace page: yes**, decided by the owner at the design review (2026-09-29)
rather than at the close. 707.9f adds a read no page walks: a look-ahead of
a copy that has not been made, taken once per judgment, inside the CR 616.1c
application. The page walks Spark Double's entry read by read, on the Kaito
board and on a plain creature.

### 7d. CV-1b — copies that last, and CR 400.7's new object — ✅ 2026-10-06 (PR #230)

*Evicted 2026-10-06 from `plans/copy-effects-architecture.md`, where the
heading and a stub remain.*

**The scope, as briefed** (#229's close). Item 10's rule, with its three
reference kinds and the fourth the type-surface re-sweep found
(`RegisteredReplacementEffect.targets`) and the 400.7a–c carve-out; CV-1b's
`Duration::Indefinite` row and its teardown, with Mirrorform spelled as
printed and one registered consumer; item 189's fixture; and the atoms. The
open decisions went to the owner at the start, with what the hunt below had
found, and each took the recommendation: Cryptoplasm with "another target"
built here, the combat fix in this PR as its own commit and arm, one PR split
at 2,500 if it crossed, and the applied set's key included.

**What the hunt changed**, card by card before the count
(`engineering-practices.md` §4):
1. **Two reference kinds item 10 did not list.** CR 609.7a's chosen source
   lives in a replacement's pattern, not its affected set, and is pruned at
   the same point, with 400.7c's exception. Combat pairings were never
   cleaned: a dead blocker stayed in `blocked_by`, and the division prompt
   offered it. That was reachable and wrong.
2. **The carve-out, narrowed.** Item 10's "a move from the stack to the
   battlefield prunes nothing keyed on the mover" became 400.7a and 400.7c
   as written: every continuous row, and a prevention effect watching damage
   from the spell. A restriction on the spell does not carry over.
3. **"Another target"** (`ObjectFilter::NotSource` in a selection) was
   refused, and `triggers-architecture.md` §12 had given it to TR-3b. Six of
   the 25 indefinite clauses print it.
4. **"Except it has this ability"** cannot be card data, since the def would
   contain itself. `GainsThisAbility` is made the resolving ability's def at
   the capture, off `GameState::resolving`.
5. **Item 16b, the brief's unlisted prerequisite**, was worse than recorded.
   Copying one donor twice applied its statics twice, and the fix the item
   sized would have dropped an indefinite copy's statics for good once a
   turn's copy over it expired. Tagging copied abilities per copy row fixes
   both.

Two more surfaced in the build: `Effect::instances` did not walk into
`Optional`, so Cryptoplasm's target was never announced; and a declined
"may" left the resolution's instance cursor behind.

**The commits.** The rule: the registries' prune (`0ed8c6d`), targets by
identity (`c933609`), the applied set's key (`9ae388e`, with the trace
test's expectation `3092567`), and the combat removal (`6415df8`). The copy:
per-row tags (`2580f4d`), Mirrorform and the exceptions on the primitive
(`0911ee5`), "another target" (`62061b2`), Cryptoplasm unregistered with its
engine pieces (`d8be020`), its registration (`a42f078`), item 189
(`0478fc9`), the composite atom's test (`8cbf132`), and a perf fix the
close-out's callgrind found (`84b999c`, the gather's lookup off its hot
path).

**The arms**, predicted before any ran:
- `rule` against `main`: every gameplay row identical, cost rows a little
  lower, ±0.3% instructions. Read **IDENTICAL** at two seats and four on
  both pools, and +1.29%. The callgrind diff put nearly all of it in
  `replacement::gather`, the permanent's epoch looked up per permanent per
  gather for the applied set's key. Moved into the counter loop, the same
  arm reads +0.23%.
- `combat`: moves a few games. Read identical at two seats, moving at four.
- `engine` (Cryptoplasm unregistered): `performance` identical where combat
  is, `stress` moving through Mirrorform. As read.
- `shipped`: a pool change, so a re-record (`engineering-practices.md` §3).

**The tests.** 29 in `phase_cv1b_integration_test.rs`, in eight sections:
the registries' half, the announcement's, combat's, a re-copy, copies that
last, "another target", Cryptoplasm, and item 189. Two RB tests now assert a
regeneration shield unspent before its creature dies, not after. Every
ruling of Mirrorform's four and Cryptoplasm's four is a test.

**The review round** (the owner, 2026-10-07). Two items this PR had filed
were finished in it. Item 218: a copy that lasts hides every earlier copy of
its object for as long as both exist, so it retires them and the rows their
copied statics generated (`retire_earlier_copies_of`); the owner's "overwrite
the line item" is that, with a turn's copy over a lasting one kept apart,
since the lasting one shows again. Item 221: CR 707.9c over several objects
is a row per object. Its test found that one timestamp per effect made the
rows one `EffectGroup`, which CR 613.6 held to the first row's object, so
each row takes its own. The one-word names this PR added were renamed for
their call sites (`refers_to`, `remove_references_to`, `from_announcement`,
`for_text_of`, `as_resolved_targets`, `chosen_damage_source`), and the
609.7a read lost its nineteen `None` arms: `PatternFill`, the closed list of
what a resolution writes into a pattern, is where the guard belongs. CR
400.7's twelve exceptions were tabled with an owner each (§5.3): five are
PM-0's (`permission-architecture.md` §5 question 6), and stickers have none.
