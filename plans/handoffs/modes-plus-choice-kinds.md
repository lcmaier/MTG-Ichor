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

**Where the first PR stands** (2026-11-02):
- `cf99bc70`: 43 of item 361's 44 order-dependent tests name what they
  chose (`RecordingDecisionProvider::choosing`, `allocating`). The 44th,
  `what_a_waiting_delayed_trigger_names_is_a_source_wherever_it_has_gone`
  (tr3c), orders two alike Flickerwisp triggers and names them once 4a's
  options are complete.
- `6a127ff3`: 4a's two arms, `ChoiceOption::Trigger` and `::Replacement`,
  their log words and labels, and a script's shorthand naming an effect by
  its source. Open in it:
  1. **An entering object's ability is read in the zone it leaves**: the
     place is computed off the object's current abilities, so Thunder-Thrash
     Elder's devour granted by Jund logs as "an ability it no longer has"
     (`phase_rc5_integration_test`'s devour tests). The place has to come
     from the frame the pipeline gathered the effect in, captured into the
     option where it is built.
  2. **Two registry rows of one source read alike** (Words of Worship's two
     shields): #306's design names a row by "the stack object whose
     resolution made it", but a resolution has no id of its own here
     (`ResolutionContext::source` and the stamp are the permanent's). Put to
     the owner: give a resolution a number of its own, or name a row by what
     it does.
  3. Then: the 13 place-based answers named outright, `expect_*_among_alike`,
     CV-2b's `then_index` and the log's `(n of m)` removed (or kept for rows
     alike in every word, per item 2's answer), and the 44th test.
- Next after 4a: 4b (the typed matcher and costs named whole), decision 5
  (the division, Arc Lightning), decision 6 (the two orders, 6b, the audit).

**Traps, for the first PR:**
- The 33 tests that answer by place (`codebase-state.md` item 361) move to
  answers named by identity in its first commit, before any option's wording
  changes; the other 143 `picking(` uses stay.
- 6b drops `OrderCostReductions`' prompt: a fixture migration (9 lines in 8
  test files) and a moved random stream, so the A/B runs three arms
  (`choices-architecture.md`'s "Fuzz A/B, predicted per PR").
- Decision 6's audit pays every order on a clone, so it is a `--audit` path
  in `fuzz_games` and a test over every registered cost, never a default.
