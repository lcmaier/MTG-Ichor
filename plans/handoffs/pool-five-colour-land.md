# Handoff — a five-colour land in the pool

**Written 2026-09-02, from CV-1's review.** Its own file rather than a section of
`cv-1-review.md`, because that file is deleted when CV-1's findings are closed
and this outlives them. Delete this one when the PR lands.

**One-line brief:** add a no-downside land that taps for any colour to the
registry and to `PERFORMANCE_POOL`, re-record the two fixture tables, and A/B it
like any pool change.

---

## Why

CV-1 measured its own reachability and it was thin: **Cytoshape resolved 16
times in 200 games** on the performance pool. `--require` (shipped in CV-1)
proves the ceiling is not the engine — forcing it into every deck and seeding
the deck's colours from its own takes it to **401**. So the gap is *deck
construction*, and specifically colour: `random_deck` picks one or two colours,
then filters the pool to cards castable in them. A three-mana two-colour instant
needs both of its colours to be the two the deck rolled, which is 1 in 10 of the
two-colour decks and none of the one-colour ones.

**This gets worse monotonically.** Every phase adds a card; the pool is 63 and
71; the per-card share falls with each one. RS-1 hit it first ("the path is open
but barely"), CV-1 hit it again, and the next phase will hit it harder.

`--require` answers *"was the path walked?"*, which is what a phase needs at its
exit. It does not fix the everyday run: the pool that measures **cost** should
also be a pool where multicolour cards get cast, or the timing arm is measuring
a board that never plays half the registry.

---

## What to build

1. **One land.** A real card is better than an invented one — the fixture table
   should be able to name it. Candidates, all "tap for any colour, no
   meaningful downside at fuzz scale":
   - **City of Brass** / **Mana Confluence** — the damage is real but 1 per tap
     and the fuzz agent does not care. Closest to "no downside" while still
     being a printed card.
   - **Command Tower** — Commander-legal, tap for any colour in your commander's
     identity; **v1 is 4-player Commander**, so this is the on-target one, but
     it needs colour identity, which the engine does not model.
   - **Gemstone Mine** — counters, so it needs CR 122 and expires.

   **Recommend City of Brass**: printed, one ability, and the "deals 1 damage to
   you" rider is a `Primitive::DealDamage` the engine already has — so it is not
   actually a downside-free land, and that is fine. If a genuinely
   downside-free land is wanted, say so in the card's doc comment and use a
   `Vec<ManaType>` producer with no rider rather than inventing a card name.

2. **`land_mana_colors` already handles it.** It reads `ProduceMana` outputs and
   returns every colour, and `random_deck`'s nonbasic filter is "produces **at
   least one** of the deck's colours" — so a five-colour land qualifies for every
   deck automatically, with no change to the filter. Check this before writing
   anything else; if it holds, the deck-construction change is *zero lines*.

3. **`NONBASIC_LANDS_PER_DECK`** is currently 5 and the ten duals compete for
   those slots. A five-colour land will be picked ~1 time in 11 per slot. **That
   may not be enough to move reachability**, and if it is not, the honest lever
   is the constant, not a weighting hack. Measure before deciding.

---

## The second half nobody would guess from the title

**The land is what lets `--require` stop seeding deck colours, and that is
arguably the bigger win.** Today `--require Mirrorweave` forces every deck to be
W/U, because that is the only way the card gets cast. So the card is exercised
against one colour pair and **never meets a black, red or green card at all**
— the reachability number goes up while board diversity collapses.

`random_deck` filters the nonland candidate pool by deck colours *before*
drawing, so a WU card is not a candidate for a mono-red deck regardless of mana.
`--require` already bypasses that by force-inserting. What it cannot bypass is
the deck having no white or blue sources — which is exactly what the land fixes.

**So this PR has two deliverables, not one:**

1. The land.
2. **Make `--require`'s colour seeding conditional or remove it**, and re-measure.
   With the land in every deck's nonbasic budget, a forced Mirrorweave in a
   mono-red deck is castable, and the run measures the card against a *random*
   board instead of a hand-picked one.

Do (2) in the same PR and report the reachability number both ways. If dropping
the seeding tanks the resolution count, the land is not being drafted often
enough and the lever is `NONBASIC_LANDS_PER_DECK` — which is the same finding
the section below asks for, arrived at from the other direction.

## What it costs, and the trap

**It changes the pool, so it invalidates both fixture tables** in
`engineering-practices.md` §3 and every A/B baseline recorded against them.
That is allowed — the pool is "representative, not frozen" as of 2026-09-01 —
but it means:

- **Three binaries, interleaved, one sitting**, as RC-3 established: `main`, the
  land registered but *not* pooled, and shipped. The middle arm should be
  identical to `main` on every counter; if it is not, the registration itself
  changed something and that is the finding.
- **Re-record both fixture tables** in the same PR. CV-1 found the previous
  table was already ~11% stale on the gather column because RC-4b moved it and
  did not re-record; do not add a second instance.
- **Re-run determinism** (three `--threads 1` runs per pool).

**The trap, stated because it is the whole reason this is a separate PR:** a
five-colour land makes *every* multicolour card more castable, so game content
moves — turns, spells cast, creatures died — and the timing arm moves with it.
That is a pool change masquerading as an engine change if it rides along with
one. **Do not land it inside a phase.**

---

## How to know it worked

Before: `--require Cytoshape` at 200 games reports **16** resolutions without the
flag and **401** with it. After: the *unforced* number should rise materially —
if it does not, the land is not being drafted often enough and the lever is
`NONBASIC_LANDS_PER_DECK`, not another land.

Report the unforced resolution count for two or three multicolour cards already
in the pool (Rhox War Monk `{G}{W}{U}`, Knight of Meadowgrain `{W}{W}`,
Cytoshape `{1}{G}{U}`) before and after. Those are the customers.

---

## Explicitly not in this PR

- **Weighting new cards up in `random_deck`.** Rejected in `cv-1-review.md` C4
  and the reason stands: it makes the pool unrepresentative in a way that
  silently distorts the timing arm, which is the one thing `PERFORMANCE_POOL`
  exists to protect. `--require` is the coverage instrument; the pool stays a
  cost instrument.
- **Colour identity** (for Command Tower). That is a Commander-track type, not a
  deck-construction one.
- **A real mana-base model.** `codebase-state.md` already records
  `NONBASIC_LANDS_PER_DECK` as "still crude, and knowingly so — replace it with a
  real picker when card breadth (Phase 8) gives it something to choose between."
  This PR does not change that verdict; it adds one card to what the crude
  picker chooses from.
