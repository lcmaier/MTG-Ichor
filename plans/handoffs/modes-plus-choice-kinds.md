# Handoff — `modes-plus-choice-kinds`, the build

**Opened 2026-11-01** on `choices/modes-plus-choice-kinds` off `d00a2abd`
(#306's merge), as `roadmap-v2.md` step 5 of "From triggers to v0.5". The
design is `choices-architecture.md`, every decision the owner's after three
review rounds on #307 (2026-11-02). This file says where the build stands;
the landing of the fourth PR deletes it.

**Done, on #307:** the design and its census
(`plans/references/choice-kinds-census.py`, `tdc-names.txt`), and two
amendments to `setup-architecture.md#7e-pausing-at-a-decision`'s decision 1:
a land tapped for mana, and a setup line naming an earlier step of the turn.

**Next:** the four PRs of `choices-architecture.md`'s "The build, sized", in
order. The first, `options-named-plus-division`, is built on this branch
after the design; each later one on its own branch once the one before
merges.

**Traps, for the first PR:**
- The 33 tests that answer by place (`codebase-state.md` item 361) move to
  answers named by identity in its first commit, before any option's wording
  changes; the other 143 `picking(` uses stay.
- 6b drops `OrderCostReductions`' prompt: a fixture migration (9 lines in 8
  test files) and a moved random stream, so the A/B runs three arms
  (`choices-architecture.md`'s "Fuzz A/B, predicted per PR").
- Decision 6's audit pays every order on a clone, so it is a `--audit` path
  in `fuzz_games` and a test over every registered cost, never a default.
