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

#### `value-chosen-as-an-effect-resolves` — a value or objects chosen as an effect resolves, kept, and used — ✅ landed 2026-11-15

*Evicted 2026-11-15 from `plans/choices-architecture.md` §5, where the heading
and a stub stay.*

PR #330, the first of `values-chosen-at-resolution`'s three: every kind of decision 7 a printed
card can use as the effect resolves, items 425, 426 and 292, and most of item 394. Its design
note, with the six review rounds that settled it, is `handoffs/values-chosen-at-resolution.md`.
- **The model**: choose, keep, use. `Primitive::ChooseValue { name, what, how }` chooses and does
  nothing else. The answer is kept on `ResolvingObject::choices` under a `ChoiceName`, beside the X
  chosen as the object resolves, and is gone with the object (CR 608.2n). A name is a kind
  (`Color`, `CreatureType`, `CardType`, `Number`, `Player`, `Objects`); a card that keeps two of a
  kind at once adds a second name with that card.
- **What is chosen** (`ValueChoice`): a color (the five, CR 105.4), a creature type (CR 205.3m's
  316, `CreatureType::ALL`, fifteen added and a test reading the rule's list off the rules file), a
  card type or one of a printed few, a number between printed bounds, a player in a `PlayerSet`,
  and objects by a filter and a count: #326's choice and division, now behind the step.
- **Who chooses** (`ChosenHow`): a recipient naming one player (you, a target as its instance,
  "that player" an earlier target or step named), or at random from `GameState::rng`, nobody
  asked: a player or objects, which cards print; a color, a type or a number at random is refused.
- **A choice never made** (its chooser not there) leaves nothing under the name, never an earlier
  answer: a verb on it acts on nothing, a filter naming it describes no object, a later choose
  step chooses nothing (CR 608.2b). An amount or a player word that reads it errs (item 435).
- **How a step reads it**: `EffectRecipient::ChosenAs` (objects by existence, CR 400.7, each with
  its share for a verb that reads one, `resolve_divided`'s arms the one list; a player while in
  the game), `AmountExpr::ChosenAs`, and
  `ObjectFilter::ChosenAs`, which the resolution writes into the filter just before it matches it
  (`filter_as_read_now`: a color, a type, or `OneOf` the chosen objects, and an amount read once,
  CR 608.2h). The filter table and its referents are untouched: the table refuses a name nobody
  read (`ChoiceNotRead`), loudly.
- **One filter for a number compared**: `ObjectFilter::WithValue { of, is }` over power and
  toughness, at least, at most, exactly or less than an amount; `PowerLE` folded in.
- **One shape for a choice of objects**: Final Showdown and Feast of the Victorious Dead are two
  steps each, asking what they asked. `EffectRecipient::Choose` is a question's label, refused as
  what a verb acts on (item 434); the builder's refusal of a chain on a choice went with it.
- **A player verb with no player does nothing** (`player_acted_on`; CR 608.2b, 800.4a): it fell to
  the controller, which Vindictive Lich showed at three seats.
- **The kinds**: `ChoosePlayer`, widened from `ChooseOpponent`, and `ChooseColor`,
  `ChooseCreatureType`, `ChooseCardType`, `ChooseNumber`, each with its words, why, setup line
  and replay, and each naming the object resolving. The dev GUI needed no case: buttons, a seat,
  the number field. A creature type is 316 buttons, which nobody has looked at (item 436).
- **The registry's checks**: a kept name is read after the step that keeps it, never from another
  mode, from outside the "may" that kept it, or by anything that outlasts the resolution; a verb
  acts on a chosen player or objects and a chooser can be found; a player verb is written on a
  recipient that can name a player. Each has a test that it refuses. The card words refuse a
  name where its kind cannot be read (an amount, a filter).
- **The cards**: Expel the Interlopers (pooled, 128 → 129); ten castable fixtures, six the text
  of a printed card (Wash Out, Kindred Dominance, World Queller, By Invitation Only, The Nipton
  Lottery, Duneblast).

Where it differs from the note's words:
- **A choose word takes its chooser first** (`choose_number(you(), 0, 10)`), as every verb takes
  its recipient, where the note's sketch left "you" out; `at_random()` goes in the same place.
- **Three fixtures are invented cards**: Gluntch's and Truth or Consequences' sentences are not
  those cards' text alone, and a target opponent's number (on Choice of Damnations) was not listed.
- **A divided choice's verb prints the total** its choose step divides, as a targeted division's
  atom does, kept one number by the registry's test.
- **A divided choice is kept as pairs**, an object and its share (`ChosenValue::Divided`), where
  the note drew two lists side by side.
- **"At random" is narrower than the note's `ChosenHow`**: a player and objects, the two a card
  prints.
- **`Effect::Remember` does not see a choose step**: a later trigger naming a chosen object is
  the third row's (item 428).
- Filed with their homes: items 428 to 437, and a line on item 396.

#### `several-choosers-at-one-step` — a choice each player makes, and its answers — ✅ landed 2026-11-18

*Evicted 2026-11-18 from `plans/choices-architecture.md` §5, where the heading
and a stub stay.*

PR #334, the second of `values-chosen-at-resolution`'s three and the first of the three PRs it
split into at its design stop: the walk and what it keeps. Items 435 and 14 closed, 429 and 437
narrowed to `secret-choices-and-votes`. Its design note, with the three review rounds that
settled its thirteen decisions, is the second row's section of
`handoffs/values-chosen-at-resolution.md`.
- **Every choice made is a list of answers, each with its player** (decision 1):
  `ChoiceMade { kind, answers }` of `ChoiceAnswer { for_player, value }`, on
  `ResolvingObject::choices`. One chooser leaves one answer. A filter reads any answer ("of a
  type chosen this way" is an `Or` of them, "the rest" every answer's objects); a verb acts on
  every answer's objects, each once, and a draw or a life loss on each player chosen; an amount,
  a player word and a verb that takes one player read exactly one answer.
- **The walk** (decision 2): `ChosenHow { walk, answered_by }`. `ChoiceWalk { players }` asks each
  player of a group still in the game, the active player first and then in turn order (CR 101.4,
  608.2e); one order, no `WalkOrder`. `AnsweredBy` is one player, each player for themselves, or
  at random. The choice is opened empty before the first question and each answer added as it is
  given, so a later player's question reads the answers before theirs (CR 101.4b). A walk that
  reaches nobody chose nothing, so "the rest" is everything. Built here: the active player
  starting, one player, each for themselves. A walk answered by one player for each, or at random
  for each, is refused by the registry until `secret-choices-and-votes` and `separate-into-piles`.
- **"They"** (decision 3): `PlayerRef::Named(PlayerAntecedent::ChoosingFor)`, the player an
  answer is for: each player in turn under a walk, the one chooser without one. Written into a
  choice's filter as the question is asked; anywhere else it errs.
- **An open number** (decision 8): `ValueChoice::Number { least, most: Option }`, asked as
  `XOffer::NoTop { at_least }` through `pick_number(least, u64::MAX)`. The random agent draws
  within 20 of the least; the dev GUI's field has no top.
- **A second name for a second choice of one kind**: `ChoiceKind::SecondPlayer`, `ThirdPlayer`
  and `ObjectsTheOthersChose`, and `ValueChoice::Player { among, other_than }`, the players no
  earlier name kept.
- **Each step's list of what it reads** (item 435): `engine/choices_read.rs`, every match
  exhaustive, asked before a step acts. A step reading a choice never made does not happen (CR
  608.2b): an "if you do" after a passed-over instruction reads `Cant`; a "may" on one, or on a
  player never chosen, is not offered; and an "if" that cannot be read is false, leaving the
  answer before it. A choose step reads its own count and total. Each read says whether it needs
  the one answer, and one that follows a walk is refused by the registry's test and, for a card
  it never saw, by the resolution before the verb acts. The registry reads the same list; a
  definition that outlasts the resolution (a granted ability, a token's, a copy's exceptions) is
  scanned in its printed form.
- **A pick limited by a total** (decision 12): `ValueChoice::Objects { among, count, total }`
  with `PickTotal { of, is }`. The question is `DecisionKind::SelectWithinATotal { source, of,
  limit, numbers }`: the limit read (`TotalLimit::AtMost` or `AtLeast`), and each option's number
  as the layers give it, by object. The engine refuses an answer outside the limit
  (`Rejection::TotalOutsideTheLimit`, naming the total) and asks again. `TotalLimit` decides
  whether any pick is inside by the pick furthest inside it: none is a choice never made (CR
  608.2d), none but the empty pick is not asked. The random agent keeps each option unless it
  moves the pick outside the limit; the dev GUI shows each option's number and the running total
  and offers Done only inside the limit.
- **Each player's own, all of them**: `PickCount::All` on the edict's pick, written `all_their`.
  "Sacrifice [those]" is the effect's controller's (CR 701.21a), so "each player sacrifices all
  other creatures they control" is an edict with nothing to choose, in its one batch.
- **"It can't be regenerated"** (decision 13, item 14): `Primitive::Destroy { cant_be_regenerated
  }`, carried on `GameAction::Destroy`, where a regeneration shield is passed over for that
  destruction only. Written `.cant_be_regenerated()`; a registry test holds every card printing
  the clause to a marked destroy.
- **The cards**: Slaughter the Strong (a Tarkir: Dragonstorm Commander deck card), Harsh Mercy,
  Fortunate Few and Choice of Damnations registered with their rulings; Plague Spores moved onto
  the mark. Slaughter the Strong is pooled (129 → 130), in Divine Reckoning's place.

Where it differs from the note's words:
- **Divine Reckoning and Gluntch, the Bestower are not registered** (item 438). The note called
  flashback built, and it is an arm no cast reads; Gluntch's Treasure tokens need any-color mana.
  Each one's rule is on an invented instant that cites its rulings.
- **"Each player sacrifices all other creatures they control" was not in the note**, which wrote
  "the rest" as a filter on a sacrifice. A sacrifice of a named permanent is the controller's
  alone, so `PickCount::All` was added at the build, on the edict, where who sacrifices already
  lives.
- **The total's question shows `TotalLimit`, two arms**, where the note put `ValueIs` on it.
  "Less than" is at most one fewer; "exactly" is refused (item 439). It is a question kind of its
  own, which the note left open.
- **`chosen_this_way()`** and its two siblings are words for the same placeholder
  `one_of_the_chosen()` writes, as the note said; Fortunate Few's "hasn't been chosen this way"
  is `one_of_the_chosen()` and `one_the_others_chose()` together, not one word.
- **The prompt walk's fourth window** was re-picked (0xA4_0004): the registrations moved every
  window of the sorted registry.
- **The pre-review's fixes**, each with a board that failed first: a life total takes an amount
  past its range as its edge (Choice of Damnations' number has no top); the read list's gaps
  above; a pick with nothing inside its limit; and a draw for each player chosen, where it drew
  for one.
- Filed with their homes: items 438 to 441; 429 and 437 to `secret-choices-and-votes`.
