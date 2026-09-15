# Handoff — RE-9, mana (the last of RE's ten PRs)

**Opened 2026-09-15**, because RE-9 is the first RE PR to span a session: its
design check is written and waits on the owner's review before code starts
(`replacement-architecture.md` §9, "RE-9 — mana", **"The design check —
thirteen decisions"**). Exit criterion 5 of the RE section says this file is
opened by that PR and **deleted by the PR that lands it** — that is RE-9's own
landing commit, and it is a gate.

## Where things stand

- Branch `replacement/re-9-mana`, off `main` at 95d9e92 (RE-10's merge).
- Draft PR **#140**. One commit, the doc alone (0b25935). No engine code yet.
- `plans/replacement-architecture.md`: RE-9's design check (thirteen
  decisions) and §11 items 93–96. Every count in it was read from the tree
  and from Scryfall on 2026-09-15.

## What the owner is being asked

The three questions at the end of the design check, with the recommendation
on each: *(a)* ship the type-changing `Instead` leg with Deep Water (decision
7) or record it; *(b)* `Mana productions` in the fixture table or a
`fuzz_games` line only; *(c)* the name `ProduceMana` is kept.

## To resume, on the go-ahead

1. Re-read the design check; every decision names its file and its test.
2. Build in this order, one commit each where the seam allows: the event and
   its three exhaustive arms; `resolve_mana_effect`'s signature and both
   proposers; the performer and `ManaAdded`'s reshape; the pattern arm and
   `pattern_watches`; the `Amount(Multiplier)` leg; the template leg and the
   three predicate arms; the counter; the cards (Mana Reflection, Nyxbloom
   Ancient, Deep Water) with their rulings passes; the tests with their
   `COVERS:` lines; then the pool move and the docs.
3. Measure per the section's "Measure" — four arms (`main`, engine,
   registered, pooled), both pools, two seats and four, `--rounds 7`, the
   `main` worktree at `../mtgsim_v2_main` rebuilt at 95d9e92 first, each
   arm's `Card pool: (N cards)` header read before trusting it.
4. Apply decision 11's rule to the number, and record the outcome either way.
5. Land: `✅` heading, the body evicted to
   `plans/archive/replacement-architecture-landed.md` with a ≤40-line stub,
   §9's row updated with the shipped size, `codebase-state.md` item 111
   closed and evicted, `fuzz-record.md` re-recorded if the pool moved,
   `specdb build`, the four checks each on its own exit code, and **this
   file deleted**.
