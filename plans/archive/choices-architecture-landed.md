# choices-architecture.md — landed phases, evicted

**A record of finished work, not a plan.** Every section here sat in
`plans/choices-architecture.md` and was moved out when its phase shipped,
leaving the ✅ heading and a stub in the live doc. Nothing here is owed and
nothing here should be acted on; what each section is *for* is the
reasoning — the build as sized, what the building changed. Later phases are
appended by the PR that lands them.

#### `mode-rules-and-costs` — the node's rules, its other choosers and a mode's own cost — ✅ landed 2026-11-07

*Evicted 2026-11-07 from `plans/choices-architecture.md` §5, where the heading
and a stub stay.*

**The intake** (2026-11-07, against `354270f6`). The slotted list read items
81, 154, 158's modal half, 412 and 413, and all five were taken. Each card's
clauses read against the tree:
- **Arc Trail**'s "any other target" is item 154's first half, and nothing more.
- **Demonic Pact**'s "target opponent" is item 345, pulled forward from
  `multiplayer-rules`: `SelectionFilter::Player` takes a `PlayerSet`, whose
  `Opponents` already reads "every other player", and the targeted opponent is
  the one who discards, so nothing reads "an opponent" two ways (item 342 stays
  where it is). "Choose one that hasn't been chosen" is the record.
- **Shadrix Silverquill**'s third mode, "puts a +1/+1 counter on each creature
  they control", was sized as a `PlayerRef` arm for "the player an instance of
  target chose", about 80–150 lines. Building it found the larger half: "target
  player" is the sentence's subject, which no atom acts on, so nothing declares
  the instance. That is a recipient's shape, not an arm, so the brief's
  fallback held: the card moved to `multiplayer-rules` (item 415), Divided
  Envoy carrying its node's rules, and Vindictive Lich, whose three modes are
  written today, "each mode must target a different player" as printed.
- **Spree** (item 81): Explosive Derailment, the first `keyword:spree` card by
  mana value whose modes are both written today (Abrade's two), cast from hand
  at {R} plus {2} a mode.

**The re-count**, code and tests touched: item 413, 250–400 (its 150–250 plus
the 43 test reads of `chosen_targets`); item 154's two halves, 220–360; 700.2d
with item 412, 120–200; pawprints, 60–120; 700.2e, 60–120; the record of modes
chosen before, 150–250 (a scenario save line among them); a mode's own cost
with spree, 200–350; item 345, 120–180; Shadrix's player word, 80–150; the four
cards and their rulings, 150–250. **About 1,410–2,380 without the picker, and
1,710–2,980 with it**, whose 300–600 was a guess from #312. So the picker was
the last commit and the split point: if the engine half passed about 2,000 by
the cards, the picker would go to a PR of its own, as the design first had it.

**As measured**, at each commit: item 413, 599 touched, its renames through
every test's reads; item 154's first half 328, item 345 167, the node's rules
with item 412 968 (about 400 of it tests and four fixtures), the record 169, a
mode's own cost 195: the engine half 2,426, past the split point, so the picker
went to its own PR. The cards and their rulings 628, 2,970 in all, past the
band's top, and the pre-review's fixes 121 more, 3,091. The save word was not
built (item 414), nor Shadrix's player word.
