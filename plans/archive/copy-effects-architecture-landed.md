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
