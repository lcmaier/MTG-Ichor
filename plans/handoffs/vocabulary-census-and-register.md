# Handoff — `vocabulary-census-and-register`, the design note

**Opened 2026-10-26**, at #295's second review round: the owner put this
census ahead of row 6 of the item 6 audit's cleanup (A6e), which it splits
into 6a (`resolution-recipients-and-amounts`, #295) and 6b
(`resolution-players-and-riders`). This file is the design note, reviewed
before the register and the gate are built; the landing deletes it. Every
count below was read against `ac583da4` (#294's merge). **Revised after its
fresh read** (14 findings, each fixed here or made a decision), and **at
#296's first review round** (2026-10-26): decisions 1 and 2 settled, 3
reframed around `PlayerSet`, 4 and 5 given the ramifications and the history
the owner asked for.

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
- **H. "An opponent", one reading per word** (decision 3). A filter's player
  half takes a `PlayerSet`, so "a creature an opponent controls" is
  `ByController(PlayerSet::Opponents)` and `PlayerRef::Opponent` means only
  "one opponent, chosen as the instruction applies" (CR 608.2d). `PlayerSet`
  gains one arm, `Player(PlayerRef)`, for a filter that names one particular
  player ("a creature defending player controls", "creatures that player
  controls"); it never holds `PlayerRef::You`, which stays `PlayerSet::You`,
  and a registry test refuses it. `AnyDefendingPlayer` becomes
  `PlayerSet::DefendingPlayers`, and `Condition::DefendingPlayer` folds into
  `Condition::Player { players: PlayerSet::Player(DefendingPlayer(of)), .. }`.
  About 195 sites and 80 lines.
- **D. One filter for a player or a permanent in a trigger's arm.**
  `DamageRecipient`, `DefenderFilter` and `ObjectOrPlayerSubject` become one
  `ParticipantFilter`, named for its role: it tests the object or player
  that played a part in a trigger's event, the word `EventRole` already
  uses (#296's second round: a generic `ObjectOrPlayerFilter` read as the
  whole codebase's filter, which under decision 4(b) it isn't). A player
  half (a `PlayerSet` under H), an
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
   **Settled: (a)** (the owner, at #296's first review round, 2026-10-26).

2. **The player word is `PlayerRef` (fold C).** This reopens #295's settled
   decision 4, which named a new `PlayerAntecedent` beside `PlayerRef`. The
   census shows `PlayerRef` already is the player word in every other role
   ("you", "an opponent", "its owner", "defending player"), so a second type
   would be a duplicate this PR's gate would refuse.
   - (a) **`PlayerRef` gains the antecedent arms**, and
     `EffectRecipient::Player(PlayerRef)`.
   - (b) **As settled at #295**, the overlap kept with its reason.

   **Settled: (a)** (the owner, at #296's first review round, 2026-10-26).
   It is recorded on #295's note before 6b builds.

3. **"An opponent": one reading per word.** "A creature an opponent
   controls" means any opponent's creature; "an opponent gains control of
   it" means one opponent, chosen as the instruction applies (CR 608.2d).
   Both are `PlayerRef::Opponent` today: `object_filter.rs` reads it as
   `player != you`, and `resolve_player_ref` and `SetController` read it as
   the one (`SetController` asserts at three players or more). Each reader is
   right for its role, but one arm with two meanings is item 268's shape.

   **`PlayerSet` already says the first meaning**: `PlayerSet::Opponents` is
   "every opponent", so the fix is for a filter's player half to take a
   `PlayerSet` instead of a `PlayerRef`. The first draft muddled in a
   second need, and its `One(PlayerRef::You)` read strangely because of it
   (the owner, at #296's first review round). A filter sometimes names one
   particular player rather than a relation to you: "target creature
   defending player controls" (Ancient Hellkite; 53 cards,
   `o:"creature defending player controls"`), "that player controls" (223,
   `o:"that player controls"`, #295's for-each player among them, both
   `game:paper -is:funny`, 2026-10-26). `PlayerSet`'s arms are relations to
   you (`You`, `Opponents`, `Everyone`) and ids captured when an effect was
   made (`Fixed`), so it needs one arm that holds a named player,
   `PlayerSet::Player(PlayerRef)`. "You" stays `PlayerSet::You`, and
   `Player(PlayerRef::You)` is refused, so "you" has one spelling there.
   **What each type covers under (a)** (asked at #296's second round):

   | | `PlayerRef` | `PlayerSet` |
   |---|---|---|
   | The question it answers | "who?": exactly one player | "is this player one of them?": a test asked of a candidate |
   | Its arms | `You`, `Opponent` (one, chosen, CR 608.2d), `Player(id)`, `DefendingPlayer(of)`, and fold C's `ControllerOf(a)`, `OwnerOf(a)`, `TheEventsPlayer`, `ForEachPlayer`, `EnchantedPlayer`, `ReferredPlayer` | `Nobody`, `You`, `Opponents`, `Everyone`, `Fixed(ids)`, and fold H's `DefendingPlayers` and `Player(PlayerRef)`, the one player that word names |
   | Where it's read | the player an instruction acts on (`EffectRecipient::Player`), who gains control, who puts counters on, who chooses | a filter's "controlled by" or "owned by", a trigger's "you", "an opponent" or "a player", a replacement's or a "can't"'s affected players, a condition over players, "each opponent" |
   | "An opponent" | "an opponent gains control of it": `Opponent` | "a creature an opponent controls": `Opponents` |
   | "Defending player" | "defending player loses 2 life": `DefendingPlayer(of)` | "creature defending player controls": `Player(DefendingPlayer(of))`; with no creature to say which, `DefendingPlayers` |

   "You" is in both because it is both one player and a set of one;
   `Player(PlayerRef::You)` is refused, so a set spells it `You`.

   - (a) **Fold H, as revised**: a filter's player half takes a `PlayerSet`,
     with `Player(PlayerRef)` for a named player. `PlayerRef::Opponent` is
     then only the chosen one. About 195 sites and 80 lines.
   - (b) **Keep it**, recording on `PlayerRef::Opponent` that the role decides
     the reading, and filing a check that no filter reads it as one.

   **Recommended: (a).**

4. **`SelectionFilter`: kept, or folded into D.** The owner leans (b), and
   asked at #296's first round for its ramifications and whether the name
   is right.

   **What a target is.** CR 115.1: "The targets are object(s) and/or
   player(s)", and CR 109.1 makes an object an ability on the stack, a card,
   a copy of a card, a token, a spell, a permanent or an emblem. So a target
   is always exactly an object or a player, which is what the engine holds
   (`ObjectOrPlayerId`). What varies is which objects: a permanent ("target
   creature"), a spell ("target spell"), a card in a graveyard, an ability on
   the stack. "Any target" (CR 115.4) is creatures, players, planeswalkers
   and battles.

   **What `SelectionFilter` is today**: the criteria an object or player must
   meet to be selected, and "selected" is three things. Announced as a
   target (`EffectRecipient::Target`); chosen without targeting
   (`EffectRecipient::Choose`, CR 608.2d, and a copy's donor); and what an
   Aura may enchant (`enchant_filter`, which CR 303.4a makes its target as
   it's cast). Its arms: `Creature` (105 sites), `Player` (47), `Any` (51),
   `Permanent(ObjectFilter)` (71), `Spell` (13), and `DamageSource` (11),
   CR 609.7a's *choice* of a source of damage, which isn't a target and
   reaches past permanents and spells (any object referred to by an object
   on the stack, a face-up object in the command zone). The families it must
   hold at v1: target player 1,028, any target 841, target card in a
   graveyard 886, target opponent 742, target spell 375, target player or
   planeswalker 130, target ability 39, a source of your choice 65
   (`o:"target player"`, `o:"any target"`, the graveyard query above,
   `o:"target spell"`, `o:"source of your choice"`, and the rest as above).

   **What (b) means.**
   - Two types say "a player or an object that qualifies": `SelectionFilter`
     for a selection, and fold D's `ParticipantFilter` for a trigger's
     participant. The difference is real. A selection ranges over every zone
     a candidate can be in (the battlefield, the stack, a graveyard), so
     where the object is belongs to its criteria. An event's participant is
     placed by the event: the creature dealt damage, the planeswalker
     attacked. The register records the pair with that reason.
   - Both hold the same leaves, an `ObjectFilter` for objects and (under fold
     H) a `PlayerSet` for players, so the words inside aren't spelled twice,
     only the two wrappers are. A later merge would be a wrapper merge, not a
     respelling.
   - "Target opponent" becomes `Player(PlayerSet::Opponents)` once `Player`
     takes a `PlayerSet`, and "target player" `Player(PlayerSet::Everyone)`:
     about 50 sites.
   - `Creature` is `Permanent(ByType(Creature))` spelled twice in one enum,
     like fold F's pair, so it becomes a constructor in fold F's commit (105
     sites).
   - What it still can't say, each a target design of its own and filed with
     its count: "target player or planeswalker" (130), an ability on the
     stack (39), and cards in a graveyard (886, already `ironworks-loop`'s).

   **The name.** "Selection" is the engine's word for all three: choosing a
   target, choosing without targeting, and what an Aura may enchant.
   `validate_selection` and `enumerate_legal_selections_upto` read the type
   under that word. So `SelectionFilter` names what it holds, and a new name
   for this type alone would split that family of names. If the family is
   renamed, it's one sweep in `names-and-hygiene-sweeps`.
   - (a) **Fold it into one filter for the whole codebase**, `ObjectOrPlayerFilter` in place of `ParticipantFilter`, with `Spell`, `Any`,
     `DamageSource` and the graveyard and ability targets as arms the trigger
     side never uses. About 290 sites more, and a target-vocabulary design
     inside a fold.
   - (b) **Keep it as the selection vocabulary** (CR 115, 303.4a, 608.2d,
     609.7a), and give its player arm a `PlayerSet` (under H), so "target
     opponent" can be written. About 40–60 lines, and `Creature`'s 105 sites
     in fold F.

   **Recommended: (b), keeping the name.**

5. **The gate.**

   **How the duplicates got in** (asked at #296's first round). Three rules
   stand against respelling a thing, and none of them asks this question of
   these types:
   - **The breadth census**, `engineering-practices.md#what-a-new-type-must-express`:
     before a new type, arm or field, count what the CR and the printed
     cards ask it to hold. It asks "is this shape wide enough?", not "does
     this word already have a spelling in another type?". Every role's enum
     passed it.
   - **The concept register's gate** (#287, 2026-10-24, the second check in
     `check_type_surface.py`): a new field holding a registered concept fails
     without its line. It reads the game's state and stops at the types a
     card's text is written in, by design, since "a field there is a word on
     a card and not a fact about a game" (its section's own words).
   - **Two growth contracts** in `CLAUDE.md`: `EventPattern` grows one arm per
     `GameAction` variant, and `Rewrite` is closed. They cover two types.

   Every spelling of "this object" came with the phase that built its role,
   before #287: `ObjectSet::SourceOnly` (the first layers phase, May),
   `CostSubject::Itself` (CM-2, a spell's own cost abilities, 2026-09-07),
   `RetargetSpec::ToEffectSource` (RD-4, redirection, 09-09),
   `TriggerSubject::ThisObject` (TR-1, the trigger spine, 09-20),
   `CopyDonor::ThisObject` (CV-2a, entering as a copy, 09-28),
   `Antecedent::ThisObject` (TR-4a, the frame's facts, 10-14, as
   `ConditionObject`). Each design built its role's enum and spelled the
   words that role needed, and its review checked breadth. One commit says
   the pattern out loud: TR-5e (the attack declaration, #256, 2026-10-16)
   added "defending player" "in all three player words" (`EffectRecipient`,
   `PlayerRef`, `Condition`), taking three player vocabularies as given. The one spelling newer than #287,
   `ObjectOrPlayerSubject` (#291, 10-25), was itself a fold, of one trigger
   slot's types. So no restriction failed: none covered this vocabulary.
   This gate is the one that does.

   The register lands as `cr-coverage-audit.md` section 4d,
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

   **Settled: (a)** (the owner, at #296's second review round, 2026-10-26).

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
