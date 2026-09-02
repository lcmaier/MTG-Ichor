# RC-4b review — answers and changes (2026-09-02)

The owner's review of PR #87, one entry per comment, in file order. A
**Changed** line means the same PR carries the change. Questions for
understanding are answered here and nowhere in source comments. Card text
quoted was fetched from Scryfall on 2026-09-02.

## `engine/actions.rs`

**`compute_characteristics(self, object).map(Box::new)` — why map Boxes over
the battlefield objects?** It is not mapping over objects. The call computes
the one object that is leaving and returns `Option<EffectiveCharacteristics>`;
`GameEvent::ZoneChange::lki` is `Option<Box<EffectiveCharacteristics>>`; the
`.map(Box::new)` turns the first into the second, boxing the value if there
is one and leaving `None` alone. The Box is the field's, and the reason is on
the field: a Rust enum is as wide as its widest variant,
`EffectiveCharacteristics` is the widest type in the engine (name, cost,
colors, types, subtypes, supertypes, keyword flags, the ability list, P/T,
controller), and without the Box every `GameEvent` in the log — a tap, a
draw, a life change — would be that wide. Behind a Box the variant carries one
pointer, and only a zone change that actually leaves the battlefield pays for
a frame. Nothing about the line changed in this PR: it moved from the
`ZoneChange` arm into `perform_zone_change`, and the one edit beside it is the
`self.battlefield.contains_key(&object)` guard, so a token in the zone with no
entity gets `None` rather than a walk.

## `tests/phase_rc4b_integration_test.rs`

**Is tap → move → `SpellCast` the right order? Wouldn't it be move → taps →
`SpellCast`?** Two orders are in play and the comment conflated them.
**Changed:** the doc comment now separates them.

- *The state order is the one you describe.* CR 601.2a puts the card on the
  stack, 601.2g activates mana abilities, 601.2i makes it cast, and the engine
  does exactly that: `move_object` at 601.2a, the taps against a card that is
  already on the stack, `SpellCast` last. A mana ability that looked at the
  stack during 601.2g would find the spell there.
- *The log order is tap → move → cast*, because the log records events and
  the move is not one until 601.2i. CR 732.1: a cast that cannot be completed
  is reversed entirely and "no abilities trigger and no effects apply as a
  result of an undone action", so a `ZoneChange` announced at 601.2a would be
  a trigger event for a cast that may not exist — item 51's phantom. The taps
  are the one thing 732.1 lets stand by default ("each player *may* also
  reverse any legal mana abilities ..."), which is why they are announced when
  they happen.

Does the difference reach a ruling? No. The two events the log orders are a
tap trigger ("whenever you tap a land for mana") and a cast trigger ("whenever
you cast"), and CR 603.3 puts both onto the stack at the same moment — the
next time a player would receive priority, which is after the cast completes
— in an order their controller chooses (603.3b). Neither placement consults
the log's order between them. A leaves-the-zone trigger on the card itself
(Syr Konrad's "whenever a creature card leaves your graveyard", for a
flashback cast) is the same: detected at 601.2i, placed after the cast, and
absent if the cast rewound — which is what 732.1 requires and what detection
at 601.2a could not give. What a reader *cannot* do is replay the state order
from the log alone; the log is the trigger record, and the state is the state.

**Manual mana management, and Arena's undo.** Agreed on both counts, and
neither is this phase's — but the phase measured the cost, so it is worth
recording. In 40 `performance` games the random agent rewound 303 casts,
seven and a half per game; every one is a decision round-trip spent on a cast
that failed at 601.2h, mostly by declining or mis-picking in the 601.2g
window. That is the CLI harness paying for manual mana. Two separate things:

1. **Undoing the tap is CR 732.1's own permission**, not an Arena
   convenience: "each player may also reverse any legal mana abilities that
   player activated while making the illegal play, unless mana from those
   abilities ... was spent on another mana ability that wasn't reversed." The
   engine takes the "may not" branch unasked. That is a `DecisionProvider`
   question at the rewind site, and for the GUI it is the expected UX.
   `test_a_rewound_cast_keeps_its_mana_abilities_and_leaves_no_zone_change`
   asserts the branch the engine takes today; **Changed:** its comment says it
   is one of 732.1's two.
2. **Auto-payment is a harness problem the engine should serve, not solve.**
   For one spell's cost, "is there a tap set that pays it" is a bipartite
   matching between pips and the colors each ability can produce —
   polynomial, and the engine already exposes the two halves
   (`enumerate_activatable_mana_abilities`, `remaining_cost_after_pool`).
   What Arena gets wrong, and what is genuinely hard, is the lookahead: which
   covering set leaves the *rest of the hand* castable, with restricted mana
   (Cavern of Souls) on top. That is a policy for the AI harness and the GUI's
   assistant, answered through the same 601.2g prompts; the engine's job is
   to make a rewind cheap and to offer 732.1's reversal.

**Changed:** `backlog.md` §2.18 records both, with the measured rewind rate.

## `plans/replacement-architecture.md`

**The token residual is not optional before card breadth.** Correct, and the
paragraph undersold it. Dour Port-Mage — "Whenever one or more other creatures
you control leave the battlefield without dying, draw a card." — and Aang,
Airbending Master — "Whenever one or more creatures you control leave the
battlefield without dying, you get an experience counter." — are exactly the
matcher that reads the residual: a leaves-the-battlefield trigger keyed on
`ZoneChange { from: Battlefield }` with a graveyard excluded. Under Hallowed
Moonlight a creature token that would enter is created in exile and never left
anything; the cheap answer's log says it left the battlefield, so both cards
fire for a token that never existed. That is the same class of wrong `from`
this phase fixed for cards, still open for tokens, and it becomes reachable
the day a token-exiling replacement and a leaves-without-dying trigger share a
pool. **Changed:** §9's token decision says so and names the cards;
`codebase-state.md` item 52 is cross-listed under "Before card breadth" as
item 8, a hard back-stop rather than an RE nicety; and RE's list entry says
which it is.
