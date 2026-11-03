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

**Next:** the five PRs of `choices-architecture.md`'s "The build, sized", in
order. The first, `options-named-by-what-they-are`, is built on this branch
after the design; each later one on its own branch once the one before
merges.

**Where the first PR stands** (2026-11-02):
- Done: #306's 4c (`cf99bc70`, `af61f71b`: all 44 of item 361's
  order-dependent tests name what they chose, and pass with every list
  reversed) and 4a (`6a127ff3`, `58df2e7f`, `6b179335`: `ChoiceOption::Trigger`
  and `::Replacement`, a row named by the stack object whose resolution made
  it, an ability's place captured where the engine read it, the entry frame
  for an entering object). The owner's rule from this round: name by ids of
  game things (objects, the resolving stack object), never by positions or
  the engine's own bookkeeping numbers.
- Left in 4a for 4b: `expect_*_among_alike` and the log's `(n of m)`, used
  now only by the matcher's own test. CV-2b's `then_index` stays (its copy
  is made inside the call it answers; the test tries both places).
- Next: 4b (the typed matcher and costs named whole), then this PR's
  handover. Decisions 5 and 6 are `division-and-orders`, the next PR, on its
  own branch (the split, 2026-11-02).

**Traps, for `division-and-orders`:**
- 6b drops `OrderCostReductions`' prompt: a fixture migration (9 lines in 8
  test files) and a moved random stream, so the A/B runs three arms
  (`choices-architecture.md`'s "Fuzz A/B, predicted per PR").
- Decision 6's audit pays every order on a clone, so it is a `--audit` path
  in `fuzz_games` and a test over every registered cost, never a default.
