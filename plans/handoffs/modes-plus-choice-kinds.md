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

**Where it stands** (2026-11-09):
- The first PR, `options-named-by-what-they-are`, landed on #307: items 336
  and 361 closed. The owner's rule from its review: name by ids of game things
  (objects, the resolving stack object), never by positions or the engine's
  own bookkeeping numbers.
- `division-and-orders` split at its brief (the owner, 2026-11-03), since the
  division and the orders share no code.
- The second, `divided-among-targets`, is handed over: its as-built is in
  `choices-architecture.md`, item 92 closed as a record.
- The dev GUI PR, `division-shares-plus-counter-words` (#312), is handed over:
  a division's shares on the stack and a counter's words beside its row, item
  362 closed.
- `board-scale`, off the route, is handed over (#313): items 376 to 378.
- `tarkir-dragonstorm-decks`' gap census landed (#315).
- `orders-the-cr-hands-a-player` is handed over: its as-built is in
  `choices-architecture.md`, items 79, 165 and "Before card breadth" 4 closed.
- `mana-provenance`'s design comes before `costs-and-choices-at-resolution`
  (the owner, 2026-11-06, #314).
- `modal-spells-and-abilities` split at its brief (the owner, 2026-11-07): its
  first PR, `modes-chosen-and-resolved` (#318), is handed over, its as-built in
  `choices-architecture.md`, items 31 and 153 closed. Prismari Command waits for
  any-color mana, Cryptic Command in its place; the Will cycle for `game-formats`.
- Its second, `mode-rules-and-costs` (#319), is handed over: items 81, 154, 345, 412
  and 413 closed, Shadrix Silverquill moved to `multiplayer-rules` (item 415),
  Vindictive Lich in its place. Past the band's top, so the picker split off at
  the point its intake set.
- `mode-picker` (#320), the dev GUI PR, is handed over: a choice of modes is
  ticked a mode at a time, a checkbox or a count per mode, read off the offered
  list alone, each row showing its mode's printed line, its as-built in
  `choices-architecture.md`. Item 417, a mode's words, found and closed in it.
- `mana-provenance`'s design is decided (#323, 2026-11-13), every decision
  the owner's after three review rounds:
  `mana-architecture.md#11-mana-provenance-what-a-unit-of-mana-keeps`. Its
  decision 9 answers no: its build does not come before
  `costs-and-choices-at-resolution`, whose `Pay` reaches the payment through
  the cast's three calls and asks nothing about mana itself. The build is
  three rows at step 7, after `mana-ability-costs`.
- `costs-and-choices-at-resolution` split at its brief (2026-11-13): the
  payment first. `pay-as-an-effect-resolves` is handed over, its as-built in
  `choices-architecture.md`: decision 4 whole, Mana Leak, Waterfall
  Aerialist and Contamination, items 418 and half of 219 closed, items 419
  to 422 found.
- Next: `choose-as-an-effect-resolves`, decision 8's `Choose` at resolution
  (item 28) and the division among objects not targeted.
