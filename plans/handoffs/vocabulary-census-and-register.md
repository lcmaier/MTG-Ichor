# Handoff — `vocabulary-census-and-register`, the design note

**Opened 2026-10-26**, at #295's second review round: the owner put this
census ahead of row 6 of the item 6 audit's cleanup (A6e), which it splits
into 6a (`resolution-recipients-and-amounts`, #295) and 6b
(`resolution-players-and-riders`). This file is the design note, reviewed
before the register and the gate are built; the landing deletes it. Every
count below was read against `ac583da4` (#294's merge). **Revised after its
fresh read** (14 findings, each fixed here or made a decision).

## What it is, in plain words

A card's text is written in a few dozen Rust types, each a grammatical
role: who an instruction acts on (`EffectRecipient`), which player a filter
names (`PlayerRef`), what a static ability affects (`ObjectSet`), what a
trigger watches (`TriggerSubject`), how much (`AmountExpr`). The roles are
real. But each grew the words it needed a phase at a time, so a word is
spelled once per role: "this object" seven ways. A duplicate isn't wrong by
itself. It goes wrong when two spellings get readers that disagree, which is
item 268 ("that player" read two ways), and this census found a second
(`PlayerRef::Opponent`, decision 3). Each later fold is a sweep over card
definitions, which grow from 215 to thousands at Phase 8.

The concept census (1d, `concept-census-and-register`, #287,
`cr-coverage-audit.md#4c-concepts-and-their-owners`) asked "one concept,
several holders?" of every field the game holds, and stopped at these types
on purpose. This asks the same of them. Nothing changes in a game and no
engine code changes here; decision 1 says where the folds land.

## How it was read

`plans/references/vocabulary-census.py` walks every enum and struct that the
concept register's **Stops at:** types hold, transitively, and lists every
arm, unit arms included (1d's reader skips them). It reaches 129 enums, and
every one is in exactly one class in the script's `CLASSES`, each class with
its reason; `--classes` fails a reached enum in no class, so a new type
cannot slip past:

| Class | Enums | Arms | Why its arms spell no word, or what was read |
|---|---:|---:|---|
| words | 29 | 158 | read arm by arm and grouped by the CR word each spells |
| values | 25 | 639 | a printed value (a type, a color, a counter's kind, a zone) |
| verbs | 23 | 242 | an action or event the CR names, owned by its chapter |
| facts | 15 | 60 | a fact or predicate asked of what a word names |
| settings | 19 | 47 | how many, how often, how long, how rounded |
| keys | 4 | 11 | a thing named by identity |
| record | 14 | 122 | the game's record and its observers, the concept census's |

Sites below are lines in `mtgsim/src` and `mtgsim/tests` naming the arm.

## What it found

**Object words**, with the role each spelling sits in:

| Word | Spellings | Sites |
|---|---|---:|
| "This object" (CR 113.7a) | `EffectRecipient::ThisObject`, `Antecedent::ThisObject`, `ObjectSet::SourceOnly`, `TriggerSubject::ThisObject`, `CostSubject::Itself`, `RetargetSpec::ToEffectSource`, `CopyDonor::ThisObject` | ~300 |
| "Enchanted/equipped creature", the object half of CR 303.4m | `EffectRecipient::Host`, `ObjectSet::Host`, `TriggerSubject::Host`, `RetargetSpec::ToHost`, `CopyDonor::Host` | ~60 |
| "That creature", a trigger's participant (CR 608.2c) | `EffectRecipient::Triggering`, `Antecedent::Triggering` | ~55 |
| "That card", what a delayed trigger remembers (CR 603.7c) | `EffectRecipient::Referred` (objects and players), `TriggerSubject::Referred` | ~30 |
| "That target" | `EffectRecipient::SameInstanceAs`; #295 adds `Antecedent::TargetOfInstance` | — |
| One object's fact in a condition | `Condition::HostMatches`, `Condition::SourceTapped`, `Condition::SourceUntapped`, `Condition::CounterCount { object: Antecedent }` | ~50 |
| Every permanent matching a filter | `EffectRecipient::FilteredPermanents`, the battlefield case of `FilteredObjectsIn`, in one enum | ~110 |

**Player words:**

| Word | Spellings | Sites |
|---|---|---:|
| "You" | `EffectRecipient::Controller`, `PlayerRef::You`, `PlayerSet::You`, `ReturnUnder::You` | ~620 |
| "Defending player" (CR 508.5) | `EffectRecipient::DefendingPlayer`, `PlayerRef::DefendingPlayer`, `PlayerRef::AnyDefendingPlayer`, `Condition::DefendingPlayer` | ~35 |
| "That player" | #295's decision 4 | — |
| "Enchanted player", the player half of CR 303.4m | **none**: Curses can't be written | — |
| "That player" on a delayed trigger | `EffectRecipient::Referred`'s players | — |
| "An opponent" | `PlayerRef::Opponent`, read as *any* opponent in a filter and as *one chosen* opponent in an instruction; `PlayerSet::Opponents` | ~80 |
| "Its owner" | `PlayerRef::Owner` (the source's), `ReturnUnder::Owner` (each returned object's) | ~40 |

**A player or a permanent that qualifies**: four types. `DamageRecipient`,
`DefenderFilter` and `ObjectOrPlayerSubject` (trigger arms, each with a
`PlayerRef` player half, about 80 sites) and `SelectionFilter` (targets and
CR 609.7a's choice of a source, about 290 sites, its player arm with no
filter).

**Amounts**: an object's value (#295's decision 2); "that much"
(`AmountExpr::ReplacedAmount`, `TemplateAmount::ReplacedAmount`); a fixed
number (`AmountExpr::Fixed`, `TemplateAmount::Fixed`, `PerChosen::Fixed`,
and `TargetCount::Exactly` beside `ExactlyAmount(Fixed(n))`).

**Three gaps the duplicates hid**, each counted on Scryfall under
`game:paper -is:funny` (2026-10-26):
- "target opponent", 742 cards (`o:"target opponent"`): `SelectionFilter::Player`
  admits any player, while its three trigger-side siblings carry a
  `PlayerRef`;
- "enchanted player", 48 cards (`o:"enchanted player"`): `Host` is an object
  word in every role;
- "target player or planeswalker", 130 (`o:"target player or planeswalker"`),
  and "target activated or triggered ability", 39
  (`o:/target (activated|triggered|activated or triggered) ability/`):
  `SelectionFilter` has no arm for either. Targets in a graveyard (886,
  `o:/target [a-z ]*card (in|from) (a|your|an opponent.s|that player.s) graveyard/`)
  are `ironworks-loop`'s already.

### Which role answers which object word

A role that holds the one object word must refuse the words it can't answer,
and an arm a reader must refuse is worse than a missing one (`CLAUDE.md`'s
growth contract). So the table decides where the word goes. ✓ answers,
— meaningless there, ✗ refused today.

| Role | This | Its host | A participant | The remembered | A target | The affected | Paid with |
|---|---|---|---|---|---|---|---|
| An instruction's recipient (`EffectRecipient`, at resolution) | ✓ | ✓ | ✓ | ✓ | ✓ as `SameInstanceAs` | — | ✓ (6a) |
| A trigger's subject (`TriggerSubject`, at the match) | ✓ | ✓ | — | ✓ | — | — | — |
| A condition's object (`Antecedent`, layer walk and resolution) | ✓ | ✓ as `HostMatches` | ✓ | ✗ | ✓ (6a) | — | ✗ |
| A static's affected set (`ObjectSet`, layer walk) | ✓ | ✓ | — | — | — | — | — |
| A cost effect's subject (`CostSubject`, CR 601.2f) | ✓ | — | — | — | — | — | — |
| A redirect (`RetargetSpec`, a `Rewrite` payload) | ✓ | ✓ | — | — | — | — | — |
| A copy's donor (`CopyDonor`, a `Rewrite` payload) | ✓ | ✓ | — | — | — | — | — |

### The folds

- **A. The object word in the three roles that answer several of it.**
  `Antecedent` gains `ThisObjectsHost` (named so it never reads as
  `Triggering(EventRole::Host)`, an attach event's host) and `Referred`'s
  object half, beside `ThisObject`, `Triggering(role)` and #295's
  `TargetOfInstance`, `Affected` and `PaidWith`. One wrapper name in every
  role: `EffectRecipient::Object(Antecedent)` folds `ThisObject`, `Host`,
  `Triggering` and `Referred`'s objects, and refuses `Object(TargetOfInstance)`
  and `Object(Affected)`, so `SameInstanceAs` stays the recipient's one
  spelling of "that target" (Act of Treason's "untap that creature" is
  `SameInstanceAs(0)`); `TriggerSubject::Object(Antecedent)` folds
  `ThisObject`, `Host` and `Referred`, and its `Any` folds into
  `Filter(ObjectFilter::All)`; `Condition::HostMatches` becomes
  `Condition::ObjectMatches(Antecedent, ObjectFilter)`, the shape
  `CounterCount` already has. About 390 sites and 95 lines of readers.
- **C. The player word is `PlayerRef`** (decision 2). It gains
  `ControllerOf(Antecedent)`, `OwnerOf(Antecedent)` (`Owner` becomes
  `OwnerOf(ThisObject)`), `TheEventsPlayer`, `ForEachPlayer`,
  `EnchantedPlayer` (CR 303.4m's player half, the Curses) and
  `ReferredPlayer` (CR 603.7c's players). `EffectRecipient::Player(PlayerRef)`
  folds `Controller`, `TriggeringPlayer`, `DefendingPlayer` and `Referred`'s
  players. About 295 sites and 50 lines.
- **H. "An opponent", one reading per word** (decision 3). Filters and
  player conditions take a `PlayerSet`, which gains `One(PlayerRef)`;
  `PlayerSet::You` becomes `One(PlayerRef::You)`, `AnyDefendingPlayer`
  becomes `PlayerSet::DefendingPlayers`, and `Condition::DefendingPlayer`
  folds into `Condition::Player { players: One(DefendingPlayer(of)), .. }`.
  Then `PlayerRef::Opponent` means only "one opponent, chosen as the
  instruction applies" (CR 608.2d), and "any opponent" is
  `PlayerSet::Opponents`. About 300 sites and 80 lines.
- **D. One filter for a player or a permanent in a trigger's arm.**
  `DamageRecipient`, `DefenderFilter` and `ObjectOrPlayerSubject` become one
  `ObjectOrPlayerFilter` (the name pairs with `ObjectOrPlayerId` and
  `ObjectOrPlayer`, item 291's): a player half (a `PlayerSet` under H), an
  object half (a `TriggerSubject`), and their `Or`. About 80 sites and 60
  lines. `SelectionFilter` is decision 4.
- **F. One arm for every permanent matching.** `FilteredPermanents` becomes a
  constructor of `FilteredObjectsIn(filter, BATTLEFIELD)`, and the
  resolution's four refusals of `FilteredObjectsIn` (`resolve.rs`) become a
  refusal of `zones != BATTLEFIELD`. About 115 sites and 20 lines.

Dropped from the first draft, by the table: folding `TriggerSubject` and
`CostSubject` into `ObjectSet` would give a trigger's subject a fixed set and
a zone field that only the layer walk reads; folding `TemplateAmount` into
`AmountExpr`, and the object word into `RetargetSpec` and `CopyDonor`, would
widen `Rewrite`'s closed payloads to arms the pipeline must refuse.

### What stays, each with its reason

- `ObjectSet::SourceOnly` and `ObjectSet::Host`: a static's affected set
  answers only these two object words.
- `CostSubject::Itself`: the cost role answers only "this spell" (CR 113.6d).
- `RetargetSpec`, `CopyDonor`, `TemplateAmount` and `PerChosen`: `Rewrite`'s
  payloads are a closed algebra, each arm one the pipeline applies.
  `RetargetSpec::ToDamageSourceController` is among them: CR 614.9's
  redirect to the replaced event's source's controller.
- `Condition::SourceTapped` and `SourceUntapped`: the feeds table reads them
  as entry state (`Condition::reads_entry_state`), which a general
  `ObjectMatches` would hide.
- `ObjectFilter::NotSource` and `OtherThanInstance`: relations to an object
  ("another"), not words for one.
- `ReturnUnder`: CR 610.3c's "its owner" is each returned object's, which no
  antecedent names; under H its `You` could be `PlayerRef::You`, but the
  pair is the rule's own default and its one alternative.
- `Choice.chooser` and `EntersAttacking::Player`'s player, both
  `EffectRecipient`: "target player sacrifices" declares an instance of
  "target", which only a recipient can; a `PlayerRef` names a player.
- `TargetCount::Exactly` and `UpTo` beside their `Amount` arms: castability's
  `fewest()` reads the fixed count without an evaluator, at every priority.
- `EffectRecipient::AttackedBy` and `EntersAttacking::SameAs`: two roles'
  relations, already holding `Antecedent`.
- `ObjectSet::Fixed` beside `PlayerSet::Fixed`; `AffectedSet` (CR 614.1's
  union of the two); `DiscardChooser` (CR 701.9b's who picks); `EventRole`
  and `IdentityRef` (the role inside `Triggering`, and "this ability");
  `Duration`'s arms (relations of time to the source).

## Decisions

1. **Where the folds land.** Sizes are carried (code at 1.5–2×, a counted
   sweep as counted, plus tests).
   - (a) **One fold PR, `card-vocabulary-folds`, before 6a.** A, C, H, D and
     F, one fold per commit, about 1,600–1,800 lines carried, in the band.
     6a and 6b then build on folded types and write each word once, at about
     665–1,210 and 930–1,490 carried, under the floor as the owner accepted
     for the split. Each PR does one kind of thing: the fold PR renames,
     the builds add behavior. One PR more for the detour.
   - (b) **Each fold with the PR that reshapes its type**: A with 6a
     (about 1,200–1,790), C and H with 6b (about 1,720–2,350), D and F with
     `names-and-hygiene-sweeps` (row 8, the cleanup's last PR: names that read
     at the call site, and the hygiene pass's last items; about 1,520–2,560).
     No PR more, but 6b and row 8 near or cross the ceiling, and 6a and 6b
     each mix a sweep with new behavior.
   - (c) **All of it in row 8, after 6b**, which then crosses 2,500 and
     splits.

   Either (a) or (b) reopens #295's settled decision 2's filing:
   `EffectRecipient::Object(Antecedent)` was filed at row 8, and moves.
   **Recommended: (a).**

2. **The player word is `PlayerRef` (fold C).** This reopens #295's settled
   decision 4, which named a new `PlayerAntecedent` beside `PlayerRef`. The
   census shows `PlayerRef` already is the player word in every other role
   ("you", "an opponent", "its owner", "defending player"), so a second type
   would be a duplicate this PR's gate would refuse.
   - (a) **`PlayerRef` gains the antecedent arms**, and
     `EffectRecipient::Player(PlayerRef)`.
   - (b) **As settled at #295**, the overlap kept with its reason.

   **Recommended: (a).** It is recorded on #295's note before 6b builds.

3. **"An opponent": one reading per word.** `PlayerRef::Opponent` is read
   as any opponent in a filter (`object_filter.rs`, `player != you`) and as
   one chosen opponent in an instruction (`resolve_player_ref`, and
   `SetController`, which asserts at three players or more). Each reader is
   right for its role, but one arm with two meanings is item 268's shape.
   - (a) **Fold H**: filters take a `PlayerSet`, which gains `One(PlayerRef)`,
     so "any opponent" is `PlayerSet::Opponents` and `PlayerRef::Opponent` is
     only the chosen one. It also folds `PlayerSet::You`,
     `AnyDefendingPlayer` and `Condition::DefendingPlayer`.
   - (b) **Keep it**, recording on `PlayerRef::Opponent` that the role decides
     the reading, and filing a check that no filter reads it as one.

   **Recommended: (a).**

4. **`SelectionFilter`: kept, or folded into D.** Its arms are `Creature`,
   `Player`, `Any` (CR 115.4's "any target"), `Permanent(filter)`, `Spell`,
   and `DamageSource`, which is CR 609.7a's *choice* of a source (65 cards,
   `o:"source of your choice"`), not a target. The families it must hold at
   v1: target player 1,028, any target 841, target card in a graveyard 886,
   target opponent 742, target spell 375, target player or planeswalker 130,
   target ability 39 (`o:"target player"`, `o:"any target"`, the graveyard
   query above, `o:"target spell"`, and the rest as above).
   - (a) **Fold it into `ObjectOrPlayerFilter`**, with `Spell`, `Any`,
     `DamageSource` and the graveyard and ability targets as arms the trigger
     side never uses. About 290 sites more, and a target-vocabulary design
     inside a fold.
   - (b) **Keep it as the target and choice vocabulary** (CR 115, 609.7a),
     and give its player arm a `PlayerSet` (under H), so "target opponent"
     and "target player or planeswalker" can be written. About 40–60 lines.
     Two types say "a player or a permanent", one for targets and one for
     events, each with its reason on the register.

   **Recommended: (b).** The target gaps are a design of their own, filed
   with their counts; graveyard targets stay `ironworks-loop`'s.

5. **The gate.** The register lands as `cr-coverage-audit.md` section 4d,
   "Words and their spellings", holding the walk's classes and, for each
   `words` enum, a line per arm: it owns a word, holds the word type, is a
   kept spelling with its reason, or waits for its fold's item.
   - (a) **A third check in `check_type_surface.py`**, beside 1d's two: every
     enum the walk reaches is classed, so a new type fails until it is; and
     every arm of a `words` enum has its line, so a new arm fails until it
     does. The script's `CLASSES` moves into the register, which both read.
   - (b) **The register alone**, with no gate. It goes stale the way the
     type-surface sweep did before a gate held it: `GameState` grew from 28
     fields to 52 unread (`check_type_surface.py`'s doc records it).

   **Recommended: (a).**

## Filed at the build, each with its reachability, size and slot

- Folds A, C, H, D and F, one item each, at the PR decision 1 names.
- "Target opponent" and "target player or planeswalker", with decision 4.
- "Target activated or triggered ability", with decision 4's target design.
- "Enchanted player", with fold C.
- `PlayerRef::Opponent`'s two readings, with fold H or decision 3's check.

Each is unreachable today, since no registered card prints it.

## Size

Docs and Python only, none of it in the band: the census script (written,
with its classes), the register (about 200–300 lines of
`cr-coverage-audit.md`), the third check (about 100–150 lines), and the
items above. The check scripts on `CLAUDE.md`'s line are its only gates.
