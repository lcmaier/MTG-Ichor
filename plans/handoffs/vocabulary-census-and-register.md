# Handoff — `vocabulary-census-and-register`, the design note

**Opened 2026-10-26**, at #295's second review round: the owner put this
census ahead of row 6 of the item 6 audit's cleanup (A6e), which it splits
into 6a (`resolution-recipients-and-amounts`, #295) and 6b
(`resolution-players-and-riders`). This file is the design note, reviewed
before the register and the gate are built; the landing deletes it. Every
count below was read against `ac583da4` (#294's merge).

## What it is, in plain words

A card's text is written in about forty Rust types: who an instruction acts
on (`EffectRecipient`), which player a filter names (`PlayerRef`), what a
static ability affects (`ObjectSet`), what a trigger watches
(`TriggerSubject`), how much (`AmountExpr`), and so on. Each type is a
different grammatical role, and the roles are real. But each role grew the
words it needed a phase at a time, so the same word is spelled once per
role: "this object" is spelled seven ways. A duplicate is not wrong by
itself. It goes wrong when two spellings get readers that disagree, which is
item 268: "that player" read one way by `TriggeringPlayer` and another by
the object readers. And each later fold is a sweep over card definitions,
which grow from 215 now to thousands at Phase 8.

The concept census (#287, `cr-coverage-audit.md#4c-concepts-and-their-owners`)
asked "one concept, several holders?" of every field the game holds, and
stopped at these types on purpose. This census asks the same question of
them: every word for an object, a player or an amount, every arm that spells
it, and one home for each.

Nothing changes in a game. No engine code changes here: the folds land in
the PRs decision 1 names.

## How it was read

`plans/references/vocabulary-census.py` lists every enum and struct the
register's **Stops at:** types hold, transitively, with every arm, unit arms
included (1d's reader skips those). The scope is 129 enums with 1,279 arms.
Set aside: subtypes, colors, mana types and keywords, which are values
(about 600 arms), and the event, prompt and state types the walk reaches
through a payload. That leaves 89 enums with 506 arms. 37 of them carry a
word for an object, a player or an amount, and their arms were read one by
one and grouped by the CR word each spells.

## What it found

Sites are lines in `mtgsim/src` and `mtgsim/tests` that name the arm.

| Word | Spellings | Sites | Fold |
|---|---|---:|---|
| **"This object"**, the object whose ability it is (CR 113.7a) | `EffectRecipient::ThisObject`, `Antecedent::ThisObject`, `ObjectSet::SourceOnly`, `TriggerSubject::ThisObject`, `RetargetSpec::ToEffectSource`, `CopyDonor::ThisObject`, `CostSubject::Itself` | ~300 | A |
| **"Enchanted/equipped permanent"** (CR 303.4m, 301.5a) | `EffectRecipient::Host`, `ObjectSet::Host`, `TriggerSubject::Host`, `RetargetSpec::ToHost`, `CopyDonor::Host` | ~60 | A |
| **"That creature"**, a trigger's participant (CR 608.2c) | `EffectRecipient::Triggering`, `Antecedent::Triggering` | ~55 | A |
| **"That card"**, what a delayed trigger remembers (CR 603.7c) | `EffectRecipient::Referred`, `TriggerSubject::Referred` | ~30 | A |
| **Which objects**: this, the host, a filter, any, the remembered | `ObjectSet`, `TriggerSubject`, `CostSubject`: one shape in three roles | ~265 | B |
| **"You"**, and every other player word an instruction acts on | `EffectRecipient::Controller` beside `PlayerRef::You`; `EffectRecipient::TriggeringPlayer`, `EffectRecipient::DefendingPlayer` beside `PlayerRef::DefendingPlayer` | ~275 | C |
| **A player or a permanent that qualifies** | `DamageRecipient`, `DefenderFilter`, `ObjectOrPlayerSubject` (trigger arms), `SelectionFilter` (targets): four types | ~370 | D |
| **"That much"**, the replaced event's amount (CR 615.5) | `AmountExpr::ReplacedAmount`, `TemplateAmount::ReplacedAmount`; and `Fixed` in both | ~30 | E |
| **Every permanent matching a filter** | `EffectRecipient::FilteredPermanents`, the battlefield case of `EffectRecipient::FilteredObjectsIn`, in one enum | ~110 | F |

Two that #295 already folds: an object's value (seven `AmountExpr`
variants, its decision 2) and "that player" (its decision 4).

**A gap the duplicates hid: "target opponent" can't be written.**
`SelectionFilter::Player` admits any player, while its three trigger-side
siblings each carry a `PlayerRef`. `o:"target opponent" game:paper -is:funny`
is 742 cards (2026-10-26), and v1 is four-player Commander. It is filed with
fold D, which gives it for free.

### The folds

- **A. One object word.** `Antecedent` gains `Host` and `Referred`, beside
  `ThisObject` and `Triggering(role)` and #295's `TargetOfInstance`,
  `Affected` and `PaidWith`. Each role holds it in one arm:
  `EffectRecipient::Object(Antecedent)`, `ObjectSet::Named(Antecedent)`,
  `RetargetSpec::To(Antecedent)`, `CopyDonor::Named(Antecedent)`. Each reader
  reads the word at its own moment, as `Antecedent::Triggering` already is
  read: the layer walk reads the host live, a resolution reads it as CR
  608.2h says. Seven spellings of "this object" become one. About 450 sites
  and 80 lines of readers.
- **B. One "which objects" type.** `TriggerSubject` and `CostSubject` fold
  into `ObjectSet`, which after A is a named object, a filter, a fixed set,
  or any. A trigger's subject, a static's affected set and a cost effect's
  subject are the same question in three roles. About 270 sites and 60
  lines of readers.
- **C. One player word.** `PlayerRef` becomes the player word, rather than
  #295's new `PlayerAntecedent` beside it (decision 2): it gains
  `ControllerOf(Antecedent)`, `OwnerOf(Antecedent)` (the source's `Owner`
  becomes `OwnerOf(ThisObject)`), `TheEventsPlayer` and `ForEachPlayer`.
  `EffectRecipient` names a player through one arm, `Player(PlayerRef)`,
  folding `Controller`, `TriggeringPlayer` and `DefendingPlayer`. About 275
  sites and 40 lines.
- **D. One filter for "a player or a permanent that qualifies".**
  `DamageRecipient`, `DefenderFilter` and `ObjectOrPlayerSubject` fold into
  one type whose player half holds a `PlayerRef`. Whether `SelectionFilter`
  joins it is decision 3. About 80 sites without it, 370 with it.
- **E. One amount type.** `TemplateAmount` folds into `AmountExpr`. Its doc
  keeps it apart because it is read against an event rather than a
  resolution, but `AmountExpr` already has arms read in one context only
  (`AffectedManaValue` in the layer walk, `ReplacedAmount` in a rider), each
  refusing elsewhere. About 30 sites.
- **F. One arm for "every permanent matching".** `FilteredPermanents` becomes
  a constructor of `FilteredObjectsIn(filter, BATTLEFIELD)`; the two already
  lower to one value. About 110 sites.

### What stays, and why

- `EffectRecipient::SameInstanceAs(k)`: it reuses the declaring instance's
  clause, its filter and its count, which is more than a word.
- `EffectRecipient::AttackedBy(Antecedent)` and
  `EntersAttacking::SameAs(Antecedent)`: two roles' relations ("what it is
  attacking") that already share the object word.
- `PlayerSet::You` beside `PlayerRef::You`: a set relation in the family of
  `Opponents` and `Everyone`. Spelling it as a set of one `PlayerRef` would
  spell "you" twice inside `PlayerSet`.
- `Condition::DefendingPlayer` beside `Condition::Player`: folding it needs
  `PlayerSet` to hold a `PlayerRef`, the same reason.
- `PlayerRef::Opponent` beside `PlayerSet::Opponents`: one opponent against
  every opponent.
- `ReturnUnder::Owner`: CR 610.3c's "its owner", where "it" is each returned
  object, per object, which no single antecedent names.
- `RetargetSpec::ToDamageSourceController`: the controller of the replaced
  event's source, a participant the replacement pipeline names through no
  antecedent yet. Fold A's `RetargetSpec::To(Antecedent)` leaves it as the
  one other arm.
- `ObjectSet::Fixed` beside `PlayerSet::Fixed`: objects against players.
- `Condition::HostMatches`: a filter over the host, kept until a second
  object needs a filter in a condition, when it becomes one
  `ObjectMatches(Antecedent, filter)`.
- Verbs and events (`Primitive`, `TriggerEvent`, `EventPattern`, `Rewrite`)
  are not words for a thing. Each is the CR's action, owned by its chapter,
  and `EventPattern` already grows one arm per `GameAction` variant by
  contract.

## Decisions

1. **Where the folds land.**
   - (a) **Each fold with the PR that reshapes its type.** A with 6a, whose
     decision 2 grows `Antecedent`; C with 6b, whose decision 4 is the player
     word. B, D, E and F with `names-and-hygiene-sweeps` (row 8, the
     cleanup's last PR: names that read at the call site, and the hygiene
     pass's last items), with "target opponent" riding D. 6a grows by about
     530 lines to about 1,060–1,330 sized, 6b by about 320 to about
     1,080–1,340, and row 8 by about 600–900. All three land in the band,
     where 6a and 6b alone were under its floor. The folds add sweeps to 6a
     and 6b, not concepts: A *is* decision 2's object word in every role, C
     *is* decision 4's player word.
   - (b) **One fold PR before 6a**, about 1,450 lines (1,750 with
     `SelectionFilter`). 6a and 6b then build on folded types and sweep
     nothing. It is one PR more, and 6a and 6b stay under the floor.
   - (c) **All of it in row 8, after 6b.** 6a and 6b write the old spellings,
     and row 8 sweeps them with the rest: about 100 lines more.

   **Recommended: (a).**

2. **The player word: `PlayerRef`, or `PlayerAntecedent` beside it.** This
   reopens #295's settled decision 4, which named a new `PlayerAntecedent`
   and gave `PlayerRef` a `Named(PlayerAntecedent)` arm. The census shows
   `PlayerRef` already is the player word in every other role ("you", "an
   opponent", "its owner", "defending player"), so a second type beside it
   would be a duplicate this PR's own gate would refuse.
   - (a) **`PlayerRef` gains the antecedent arms** (fold C). One player word;
     `EffectRecipient::Player(PlayerRef)`.
   - (b) **As settled at #295**: `PlayerAntecedent` beside `PlayerRef`, and
     the overlap kept with its reason.

   **Recommended: (a).** If taken, it is recorded on #295's note as a review
   round before 6b builds.

3. **`SelectionFilter`: inside fold D, or kept.** It is the targets'
   vocabulary (CR 115), and two of its arms no trigger filter has: `Spell`
   (a stack object) and `Any`, CR 115.4's "any target". It is also the
   biggest of the four, at about 290 sites.
   - (a) **Fold it in.** One type for "a player or an object that
     qualifies", with `Spell` and `Any` as arms the trigger side never uses.
     "Target opponent" is its player half's `PlayerRef`. About 370 sites in
     all.
   - (b) **Keep it, and give its player arm a `PlayerRef`** for "target
     opponent", about 40 lines. The trigger-side three still fold into one.
     Two types say "a player or a permanent", one for targets and one for
     events.

   **Recommended: (a).**

4. **The gate.** The register lands as `cr-coverage-audit.md` section 4d,
   "Words and their spellings": each word, the arm that owns it, the roles
   that hold the owner's type, and each spelling still to fold with its
   item. The question is what fails a PR.
   - (a) **Every arm of the 37 word-carrying enums has a register line**:
     it owns a word, holds one, waits for its fold's item, or is not a word
     (a value or a relation), with the reason. A new arm fails until it has
     a line, which asks the question this census asked. `check_type_surface.py`
     gains it as its third half, beside 1d's.
   - (b) **The register alone**, with no gate. Cheaper, and it goes stale the
     way the type-surface sweep did before a gate held it: `GameState` grew
     from 28 fields to 52 and the sweep read none of the new ones
     (`check_type_surface.py`'s doc records the lapse).

   **Recommended: (a).**

## Size

Docs and Python only, so none of it counts toward the band: the census
script (written), the register (about 150–250 lines of
`cr-coverage-audit.md`), the gate (about 100–150 lines), and an item per fold
filed into `codebase-state.md` with its reachability, size and slot. The
folds themselves are sized above, in the PRs decision 1 names.

## Gates

The check scripts on `CLAUDE.md`'s line, the new half among them. No
`cargo` gate can fail: nothing in `mtgsim/` changes.
