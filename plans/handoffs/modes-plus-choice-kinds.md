# Handoff — `modes-plus-choice-kinds`, the build

**Opened 2026-11-01** on `choices/modes-plus-choice-kinds` off `d00a2abd`
(#306's merge), as `roadmap-v2.md` step 5 of "From triggers to v0.5". The
design is `choices-architecture.md`, every decision the owner's after three
review rounds on #307 (2026-11-02). This file says where the build stands;
the landing of the last PR deletes it.

**Done, on #307:** the design and its census
(`plans/references/choice-kinds-census.py`, `tdc-names.txt`), and two
amendments to `setup-architecture.md#7e-pausing-at-a-decision`'s decision 1:
a land tapped for mana, and a setup line naming an earlier step of the turn.

**Next:** the six PRs of `choices-architecture.md`'s "The build, sized", in
order. The first, `options-named-by-what-they-are`, is built on this branch
after the design; each later one on its own branch once the one before
merges.

**Where it stands** (2026-11-02):
- The first PR, `options-named-by-what-they-are`, is handed over on #307:
  its as-built is in `choices-architecture.md`, items 336 and 361 closed.
  The owner's rule from its review: name by ids of game things (objects,
  the resolving stack object), never by positions or the engine's own
  bookkeeping numbers.
- `division-and-orders` split at its brief (the owner, 2026-11-03), since the
  division and the orders share no code.
- Next: `divided-among-targets` (decision 5) on
  `choices/divided-among-targets`, then `orders-the-cr-hands-a-player`
  (decision 6 and 6b) on its own branch once that merges.

**First commits of `divided-among-targets`, from #307's review (the owner,
2026-11-03):**
1. **A row's label shows its targets.** Two rows of one source can differ by
   nothing else: Nomads en-Kor activated twice at two creatures, where the row
   picked decides which creature takes the damage. `ui::display::resolution_words`
   reads `RegisteredReplacementEffect::targets`, about 10 lines, and that
   field's "no reader yet" goes.
2. **What a counter does, as data beside its label.** The label stays "its
   finality counter". CR 122.1c, d and h's own words sit beside it for a client
   that chooses to show them, as Arena does on a hover (`backlog.md` §2.38's
   "What a client is handed"). About 15 lines and a test; the dev GUI draws it
   in its own PR.

**Traps, for `orders-the-cr-hands-a-player`:**
- 6b drops `OrderCostReductions`' prompt: a fixture migration (9 lines in 8
  test files) and a moved random stream, so the A/B runs three arms
  (`choices-architecture.md`'s "Fuzz A/B, predicted per PR").
- Decision 6's audit pays every order on a clone, so it is a `--audit` path
  in `fuzz_games` and a test over every registered cost, never a default.
