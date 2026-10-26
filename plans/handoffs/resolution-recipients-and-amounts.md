# Handoff — `resolution-recipients-and-amounts`, the design note

**Opened 2026-10-26**, A6e's cleanup PR 6 (`item-6-audit.md`'s cleanup
plan, row 6, and its entry under "What each PR takes"). This file is the
design note, reviewed before the build; the landing deletes it. Every size
and count below was read against `ac583da4` (#294's merge). The intake,
`check_state_of_play.py --slotted resolution-recipients-and-amounts`, lists
14 open items; with them go the duplicate inventory's rows 38 and 39.

## Where the work sits

Most of it is inside one stack object's resolution (`engine-map.md` §1.1's
RESOLVE step): the walk over the effect tree that hands each instruction what
it acts on, the player it names and its amounts. Four items sit elsewhere on
the path:

- **Announcing, CR 601.2c.** Item 155 (castability finds an assignment for
  every instance of "target" at once) runs at every priority, and decision 4
  says where per-player targets would go.
- **Paying, CR 601.2h.** Item 30 records what a payment chose (decision 2).
- **RESOLVE.** Item 334 (a target an earlier instruction moved is not the
  object chosen, decision 1); items 269 and 299 (an object's value, decision
  2); items 268, 272 and 292 (which player an instruction names, decisions 3
  and 4); items 94 and 281 (what a group recipient holds, decision 6); row 38
  (one function for what a non-targeting recipient names) and row 39 (two
  more "no player" sites, with item 292).
- **DECIDE and RIDERS, CR 614–615.** Item 106 (a redirect to a target chosen
  when its effect was created) in `apply_rewrite`; items 90 and 329 (a
  rider's view of the resolution that created its row) in `resolve_rider`
  (decision 5).
- **PERFORM.** Item 307 (entering attacking is asked of the attacking
  player) at the entry.
- **DETECT.** Item 268 changes what a trigger's binding names, so the
  dispatch's ordering elision reads it. That is why the close-out runs the
  big audited smoke.

No card is registered. Each item's card is a fixture in its test file, which
is how both pools stay identical.

## The re-count

Code and tests apart, since the plan carries code at 1.5–2×. Read against
the tree on 2026-10-26; item 155 had no size line and is sized here.

| Item | What it builds | Code | Tests |
|---|---|---:|---:|
| 334 | Targets held as the existence chosen; the walk drops a target that is no longer it, and a back-reference follows this resolution's own move into a public zone (decision 1). About 70 test literals stage a `ChosenTargets` by id | 60–90 | 70–100 |
| 269, 299 | `AmountExpr::ValueOf(Antecedent, ObjectValue)`; seven variants folded at about 40 sites in `src` and 17 in tests; the source's power at resolution (decision 2) | 80–120 | 50–80 |
| 30 | What a payment chose, beside `CostChoices`, its sacrifice recorded and read by decision 2's word | 40–60 | 30–50 |
| 268 | `ThatPlayer(PlayerWord)` and its `PlayerRef` twin; `TriggeringPlayer`'s 13 sites migrated; a zone change names no player (decision 3) | 50–80 | 60–90 |
| 272 | `Effect::ForEach` over players, for a body with no target, as one event (decision 4) | 60–90 | 60–90 |
| 292, row 39 | No player is an answer that does nothing: `resolve_player_for_self`'s 13 calls, `named_player`, `EntersAttacking::Player` | 60–80 | 60–80 |
| row 38 | One `subjects_from_context` | ~40, net −15 | — |
| 90 | The row's targets read by a rider, through one rider-only recipient, with the rulings' existence-and-type check (decision 5) | 80–110 | 70–100 |
| 106 | A redirect to a target chosen when its effect was created, CR 614.9's check at application | 20–30 | 30–40 |
| 329 | A rider's ability source, so its "this" names its replacement's source | 10–20 | 30–40 |
| 94, 281 | `Group` holds `EachOf` and a target member; a row per player; the member's instance in the announcement (decision 6) | 40–70 | 70–100 |
| 155 | Castability searches for an assignment where a clause reads an earlier instance; greedy stays where none does | 50–70 | 40–60 |
| 307 | `is_attacking_player` at both sites | ~10 | ~30 |
| **Total** | | **600–870** | **600–860** |

That's **about 1,200–1,730 sized** and **1,500–2,600 carried**, against the
brief's 1,350–1,430 and 1,640–2,080. What grew: decision 2's sweep, decision
3's migration, item 155's search, and decision 1's test literals. The top
of the carried range crosses 2,500 only if every item overruns at 2×; the
band is tracked at each commit and a crossing flagged at once. If it
crosses, the split line is the replacement half (items 90, 106, 329), which
shares no type with the rest. Decision 4's option (b) adds 250–400 and
would need that split.

## The census

Every count is Scryfall's `total_cards` for the query shown, under
`game:paper -is:funny`, read on **2026-10-26** by
`census.py` and `sample.py` (a scratchpad copy; the queries are below each
table). `o:` skips reminder text. A family's examples are the first cards
Scryfall lists. Two families came back far wider than their items assumed:
"each creature and each player" is 65 damage cards, not one prevention
card (decision 6), and the commonest "for each opponent" body is a target
per opponent, 57 cards (decision 4).

## Decisions

1. **Item 334: targets held as the existence chosen.**

   | What the shape must do | Count | Example | Holds today |
   |---|---:|---|---|
   | A separate instance of "target" naming an object an earlier instruction moved finds nothing (CR 400.7, 608.2b) | 16 by a loose query, few in fact | Withdraw, Closing Statement | no: the later instruction acts on the new object |
   | A back-reference ("that card", "it") to a target this effect moved into a public zone finds the new object (CR 400.7j) | 60 | Cloudshift | yes, by id, and it also follows a move into a hidden zone, which 400.7j does not |
   | Information about a target that moved is its last known information (CR 608.2h) | 25 | Swords to Plowshares, Lie in Wait | no reader: `TargetPower` refuses |
   | An object a cost moved is found by the effect (CR 400.7j's second sentence) | 34 + 51 | Fling | no record (item 30, decision 2) |

   Queries: `o:/(destroy|exile|return|tap|untap) target creature[^.]*\.[^|]*target creature/`;
   `o:/exile [^.]*target [^.]*, then return (it|that card|those cards|them)/`;
   `o:/exile target [^.]*\. (its|that [a-z]+'s) controller/`; and decision 2's
   payment rows. Most of the first query's 16 are kept apart by "another"
   or by whose creature it is (Withdraw, Closing Statement), so the printed
   boards are few and the item's fixtures are the CR's (CR 115.3 lets one
   object be chosen for two instances). Cloudshift's ruling (2018-03-16) states the first two rows
   together: "The returned card won't be the target of any spells or abilities
   that targeted it before." Swords to Plowshares' (2022-12-08) states the
   third: its power "from when it was last on the battlefield".

   Today `ChosenTargets` holds each target by id (`targeting.rs`), CR
   608.2b's check runs once as the resolution begins, and every instruction
   after that reads the id. The concept register calls an `ObjectId` where
   the CR keys by existence a narrow key
   (`cr-coverage-audit.md#4c-concepts-and-their-owners`).

   - (a) **`ChosenTargets` holds `ObjectOrPlayer`, the existence chosen.**
     The walk's one index into it (`resolve_effect_step`), and
     `ExileUntil`'s second instance, hand an instruction only the targets
     that are still the existence chosen. A back-reference
     (`SameInstanceAs`) also finds an object this resolution's own records
     moved into a public zone, as the object that move made (CR 400.7j). A
     declaring instance does not, since "target creature" names the object
     chosen. A player is gone once they have left the game (item 292). The
     castability fold keeps an id list of its own, since nothing moves during
     an announcement, so the hot path is untouched. *Code*: the type, the
     walk's filter, the follow through `resolution_records`, about 70 test
     literals given the game to read epochs from. *Performance*: one filtered
     list per instruction at resolution, never at priority. *Upkeep*: one
     holder, keyed as the CR keys it; a new reader gets the check for free.
   - (b) **Ids stay; the walk compares each against the announcement.** The
     walk already holds the announced `TargetInstance`s, which carry the
     existence, so the filter is about 30 lines with no type change.
     *Upkeep*: a rider or a bare effect has no announcement, so its targets
     are never checked; the narrow key stays, a register finding with no fix.

   **Recommended: (a).**

2. **Items 269, 299 and 30: the object word, and one `AmountExpr::ValueOf`.**

   | What the shape must do | Count | Example | Holds today |
   |---|---:|---|---|
   | Power of this object | 26, and "its power" 265 shared with the next four rows | Dreadhorde Arcanist | `SourcePower`, at an entry and in a cost only; refused at resolution (item 299) |
   | Power, toughness of a target, or of "that creature" | 3 + 28 power, 0 + 26 toughness | Soul's Grace; Agonizing Demise | `TargetPower`, `TargetToughness`, both refused |
   | Power, toughness of a trigger's participant | inside "that creature's" and "its" | Paladin of Atonement | `TriggeringPower(role)`, `TriggeringToughness(role)` |
   | Mana value of an object | its 66, that card's 43, that spell's 27, that creature's 2 | Ad Nauseam | `AffectedManaValue` in the layer walk only |
   | Counters of a kind on an object | on this 65, on that or target 2 | Aether Vial | `SourceCounters(kind)` |
   | A value of an object a cost paid with: sacrificed, discarded, revealed | power 28, toughness 15, mana value 12 + 4 + 3 | Fling, Animal Boneyard | nothing (item 30) |
   | A value of an exiled or chosen object | 3 + 3 exiled; 0 chosen | Corpse Explosion | nothing |
   | "Enchanted/equipped creature's power" | 0 as an amount | — | not needed |
   | "The greatest power among" a set | 77 | Alena, Kessig Trapper | out: a rank over a set, `CountOf`'s sibling, not one object |

   Queries: `o:"equal to this creature's power"`, `o:"equal to its power"`,
   `o:"equal to that creature's power"`, `o:"equal to target creature's power"`,
   `o:/equal to the sacrificed [a-z]+'s power/`, the same five for toughness,
   `o:"equal to its mana value"`, `o:"equal to that card's mana value"`,
   `o:"equal to that spell's mana value"`, `o:/equal to the (sacrificed|discarded|revealed|exiled) [a-z]+'s mana value/`,
   `o:/equal to the number of [^.]*counters on (it|this [a-z]+)\b/`,
   `o:"greatest power among"`. Fling's ruling (2024-04-12): "Use the
   sacrificed creature's power as it last existed on the battlefield."

   **What a payment chose (item 30)**, which the paid-with word reads:

   | What the record must hold | Count | Example |
   |---|---:|---|
   | An object sacrificed as an additional cost, read back | 34 spells, 51 activated | Fling; Altar of Dementia |
   | An object exiled, discarded, revealed or tapped as a cost, read back | 9 + 2 + 7 + 3 | Champion of the Path |
   | An object that convoked | 6 | Ancient Imperiosaur |
   | An object beheld | 24 | Caustic Exhale |
   | A player a payment chose (gift) | 25 | Into the Flood Maw |

   Queries: `o:"as an additional cost to cast this spell, sacrifice" (o:"sacrificed creature" or o:"sacrificed permanent" or o:"sacrificed artifact")`,
   `o:/sacrifice (a|another) [a-z]+: [^\n]*sacrificed [a-z]+/`,
   `o:"as an additional cost" o:<verb> o:"<verb>ed card"` for exile, discard
   and reveal, `o:/tap an untapped [a-z]+[^:]*:[^\n]*tapped [a-z]+/`,
   `o:"convoked"`, `o:"behold"`, `kw:gift`.

   A reader names which object (this one, a target, a participant, the object
   a continuous effect is applied to, an object a payment used) and which
   value (power, toughness, mana value, counters of a kind). Today each
   pairing is its own variant, and printed cards fill most of the grid.

   - (a) **`AmountExpr::ValueOf(Antecedent, ObjectValue)`**, where
     `Antecedent`, today "this" and a trigger's participant, gains the
     instance of "target" (counted as `SameInstanceAs` counts), the object
     a continuous effect is applied to (the layer walk's only), and an object
     a payment used, by the act that used it (`PaidWith(PaymentAct)`). Every
     word is read as CR 608.2h reads an object: current while it is in the
     public zone it was expected in, its last known information once it has
     left. `ObjectValue` is power, toughness, mana value, or counters of a
     kind. The seven variants fold in; decision 3's "its controller" and
     "its owner", `AttackedBy` and `DefendingPlayer` share the word. Item
     30's record is `StackEntry.paid_with`, beside `CostChoices` (the side a
     copy keeps, CR 707.10): each object or player a payment chose, tagged by
     its act, carried to the resolving object and the permanent as the cost
     decisions are. Built here: this, a target, a participant, the affected
     object, and the sacrifice. Other acts land with their costs. *Code*: one
     arm and one evaluator, the sweep. *Performance*: the layer walk's
     mana-value read becomes one match arm deeper; the budget reads it.
     *Upkeep*: a new pairing is free; a new object word is one `Antecedent`
     arm, read by every reader at once.
   - (b) **One variant per pairing, adding the two this PR needs**
     (the source's power at resolution, a sacrificed creature's power).
     *Upkeep*: the grid above, a variant at a time.
   - (c) **`ValueOf` with an object word of its own**, separate from
     `Antecedent`, so a condition never meets a word it can't answer (the
     affected object, an object a payment used). *Upkeep*: two vocabularies
     for "which object", and decision 3's "its controller" chooses one.
     `Antecedent::Triggering` already answers "no" on a static ability, so
     the precedent is there for (a).

   **Recommended: (a).**

3. **Item 268, with 272's and 292's readers: "that player" names its
   participant.**

   | What "that player" means | Count | Example | Holds today |
   |---|---:|---|---|
   | The one player the event names: damaged, casting, drawing, attacked | 194, 76, 33, 65; 630 trigger cards say "that player" | Abyssal Specter | `TriggeringPlayer`, one fixed player per arm (`player_of`) |
   | The controller of a participant as it last existed | dies under an opponent's control 4, "dies … its controller" 13 | Massacre Wurm | no: a zone change's player is its owner |
   | The owner of a participant | into an opponent's graveyard 4 | Bloodchief Ascension | `player_of`'s zone-change arm |
   | The controller or owner of a target | 107 controller, 12 owner | Swords to Plowshares | nothing |
   | The player a "for each opponent" is on | 59 | Adeline, Resplendent Cathar | nothing (decision 4) |

   Queries: `(o:"when" or o:"whenever") o:"opponent controls dies" o:"that player"`,
   `(o:"when" or o:"whenever") o:"into an opponent's graveyard" o:"that player"`,
   `(o:"when" or o:"whenever") o:"dies" o:"its controller"`,
   `o:/target [^.]*\. its controller/`, `o:/target [^.]*\. its owner/`,
   `o:"deals combat damage to a player" o:"that player"`, `(o:"when" or o:"whenever") o:"<verb>" o:"that player"`
   for casts, draws and attacks, `(o:"when " or o:"whenever ") o:"that player"`,
   `o:/for each opponent[^.]*that player/`. Massacre Wurm's ruling: it triggers
   for a creature an opponent controls that dies at the same time as the
   Wurm, which is the controller as it left.

   Since TR-5a each instruction names the object participant it reads
   (`Triggering(EventRole)`, CR 608.2c). The player half still reads one
   player per arm, and a zone change's is its owner.

   - (a) **One arm, `EffectRecipient::ThatPlayer(PlayerWord)`**, with
     `PlayerWord` the event's player, the controller of an `Antecedent`, the
     owner of an `Antecedent`, or the player a `ForEach` is on. `PlayerRef`
     gains the same word, for a filter's "that player controls" and a
     `by`. `TriggeringPlayer` migrates to the event's player, and a zone
     change names no player: the card says owner or controller. The attack
     arm's player is each occurrence's attacked player. The controller of
     an object that has left is its last known information's (CR 608.2h).
     One reader answers every word, with "nobody" an answer that does
     nothing (item 292). *Code*: one arm on each enum, 13 sites migrated, no
     card uses `TriggeringPlayer`. *Performance*: none; the binding's reads
     change what the ordering elision compares, which the audited smoke
     checks. *Upkeep*: a new antecedent is free; one exhaustive match.
   - (b) **Separate arms**: `TriggeringPlayer` stays; `ControllerOf`,
     `OwnerOf` and an iterated player are three new arms on each enum.
     *Upkeep*: four arms each reader matches, and four places for item
     292's "nobody".

   **Recommended: (a).** `DefendingPlayer(Antecedent)` could fold into the
   word later; folding it is a sweep and not this PR's.

4. **Item 272: what `ForEach` iterates, and the target per player.**

   | What the body does, "for each opponent/player, …" | Count | Example | Holds today |
   |---|---:|---|---|
   | A target per player, "target … that player controls" or "from that player's graveyard", announced at cast (CR 601.2c) | 54 + 3 | Blatant Thievery, Hapatra | nothing |
   | Choose an object of that player's as it resolves (CR 608.2d) | 17 | Benthic Anomaly | nothing |
   | Create a token per player, 13 of them attacking that player | 13 | Adeline, Resplendent Cathar | nothing |
   | Encore: a copy per opponent that attacks it if able | 26 | Belonging | the attack requirement is `combat-restrictions`' |
   | Unless that player pays or sacrifices | 3 | Acererak the Archlich | nothing |
   | Friend or foe | 5 | Pir's Whim | nothing |
   | "For each opponent who …": a count (25), or per a player's answer (Tempt with …, 10) | 40 | Tempt with Vengeance | a count is an amount; the answer is not recorded |
   | "For each [object], …" | 83 | Brass's Bounty | `ForEach(ObjectSet, ..)`, refused at resolution |

   Queries: `o:/for each (opponent|player), [^.]* target [^.]*(that player|that opponent|they) controls/`,
   `o:/for each (opponent|player), [^.]* target [^.]*from that player's graveyard/`,
   `o:/for each (opponent|player), [^.]*choose/ -o:/for each (opponent|player), [^.]* target /`,
   `o:/for each (opponent|player), (you )?create/`, `kw:encore`,
   `o:/for each (opponent|player), [^.]*unless (that player|they)/`,
   `o:"choose friend or foe"`, `o:/for each (player|opponent) who/`,
   `o:/\bfor each (creature|artifact|permanent|land|card|spell)[^.,]*, (create|put|destroy|exile|return|draw|you|that|it)/`.
   Blatant Thievery's ruling: "You gain control of only one permanent from
   each player." Adeline's: tokens for each opponent, "not just opponents
   that you attacked", each choosing that player or a planeswalker of theirs
   as it enters.

   **Not a choice: one event, not one per player.** CR 608.2f processes an
   action taken on several players at once, so "for each opponent, create a
   token" is one creation: a "whenever one or more tokens enter" triggers
   once, and 608.2e asks each player's choices in APNAP order first. The
   body runs once per player, in APNAP order, inside one batch window that
   is dispatched once.

   - (a) **`Effect::ForEach(Iteration, body)`**, `Iteration` being players
     (a `PlayerGroup`) or objects (an `ObjectSet`, still refused).
     Players are built for a body that declares no target: the tokens, the
     choices, "unless that player". A body that declares an instance of
     "target" is refused when the card is built. The 57 per-player-target
     cards are filed with their reachability and size, slotted at
     `multiplayer-rules`. *Code*: the enum, the walk's loop, the window,
     the registration refusal. *Upkeep*: the target half is one item with
     its own design.
   - (b) **Build the target per player too.** The announcement expands the
     body's instances once per player in the group at cast; castability, CR
     608.2b's check and the walk's instance cursor follow; whether a player
     with no legal target blocks the cast is a ruling to read first. About
     250–400 more lines, so the PR splits at decision 5's line.

   **Recommended: (a).** v1 is four-player Commander, where the 57 are
   staples (Hapatra, Grasp of Fate, Elminster's Simulacrum), so the
   item's slot is on the route before Phase 8, not after it.

5. **Items 90, 106 and 329: a rider's view of the resolution that made its
   row.**

   | What the rider or redirect must reach | Count | Example | Holds today |
   |---|---:|---|---|
   | "Deals that much damage to any target", the target chosen when the shield was made | 7 | Divine Deflection, Acolyte's Reward | `RegisteredReplacementEffect.targets`, read by nothing |
   | The whole next damage event redirected to a target chosen when the effect was made | 16 (26 with "deal … instead") | General's Regalia, Glarecaster | nothing; `RetargetSpec` has three arms |
   | Part of an event redirected ("the next 2 damage") | inside the 16 | Harm's Way | `backlog.md` §2.25, unchanged |
   | A rider's "this", the object whose replacement it is (CR 603.7f) | 0 | — | `ability_source: None`, so `ExileUntil` errors |

   Queries: `o:prevent o:/that much damage to (any target|target)/`,
   `o:/is dealt to (any target|target [a-z ]+) instead/`,
   `o:/(dealt|deal) [^.]*to (any target|target [a-z ]+) instead/`,
   `o:/instead[^.]*exile[^.]*until (this|that)/`. Harm's Way's ruling: after
   it resolves, the target is not re-checked, but damage can't be
   redirected to a permanent no longer on the battlefield or a creature, or
   to a player who left the game (CR 614.9). Divine Deflection's three
   rulings are in item 90.

   A rider is a resolution of its own, with the replaced event's subject as
   its one "target" (`resolve_rider`). The row's targets were chosen when the
   row was made, and are kept on the row.

   - (a) **One rider-only recipient, `EffectRecipient::TargetedAtCreation`**,
     the row's targets captured when the rider is queued, read off the row
     by its `ReplacementInstanceId`, with the rulings' existence-and-type
     check and no CR 608.2b re-check. Anywhere else it refuses, as
     `ReplacedAmount` does. `RetargetSpec::ToTargetedAtCreation` reads the
     same row at application, with CR 614.9's destination check. The
     rider's ability source is its replacement's source: the object for a
     static ability (CR 603.7f), the row's source for a registry row.
     `ReplacementInstance` gains no field, so the gather, which builds one
     per candidate per proposal, does no more work. *Upkeep*: a rider's
     "target" stays the replaced event's subject, as today.
   - (b) **The row's targets as the rider's later instances**, after the
     subject at instance 0, read by `SameInstanceAs(k)`. No new arm.
     *Upkeep*: a rider's `Target` already means "the event's subject", not
     a target; this widens that overload, and a rider author counts
     instances across two resolutions.

   **Recommended: (a).**

6. **Items 94 and 281: what a `Group` holds.**

   | What the recipient must hold | Count | Example | Holds today |
   |---|---:|---|---|
   | Damage to each creature and each player (or each opponent), one event (CR 608.2f) | 65 | Pestilence, Earthquake | nothing: `Group` refuses `EachOf` |
   | A prevention shield on each creature and each player (CR 615.11) | 1 | Kitsune Palliator | nothing |
   | This object and a target, together | 11 | Hunting Kavu | nothing: `Group` refuses a target member |

   Queries: `o:/deals? [^.]*damage to each (other )?creature[^.]* and each (player|opponent|other player)/`,
   `o:prevent o:/each creature[^.]* and each player/`,
   `o:/(exile|destroy|return|tap|untap|sacrifice) (this [a-z]+|it) and (up to one )?(another )?target/`.
   Pestilence's ruling: "Each activation is considered a new damage
   effect." Kitsune Palliator's: shields for every creature and player as it
   resolves, none for one that enters later. Hunting Kavu's: if its target
   gains flying, the ability doesn't resolve, so this creature isn't exiled
   either (CR 608.2b's "all its targets").

   - (a) **`Group` admits `EachOf` and an instance of "target" or
     "choose".** A group's damage is already one batch; `CreateReplacement`
     makes a row per creature and per player; the instance walk descends
     into a group, so a target member takes its place in the announcement,
     and CR 608.2b's ordinary check drops it alone or ends the ability.
     *Upkeep*: `Group` keeps meaning "each member found as it would be
     alone, acted on at once".
   - (b) **A recipient of its own for "each creature and each player"**
     (a filter and a player set), with `Group` taking only the target
     member. *Upkeep*: a second way to say a union, which `Group`
     already says.

   **Recommended: (a).**

## What builds without a decision

- **Item 155.** Where a clause reads an earlier instance, castability
  searches the earlier instances' candidates for an assignment that leaves
  the later clause one (a backtracking search over a handful of instances),
  instead of feeding the first legal choices forward. Where no clause
  reads one, which is every spell but the "another target" family, nothing
  changes, so the priority path's cost is the budget's to read on those
  spells only. The fixture: "target creature" then "another target creature
  you control", the caster's one creature first in timestamp order.
- **Item 292 and row 39.** `resolve_player_for_self` returns no player for a
  recipient whose player is gone, and the instruction does nothing: a
  "target player" CR 608.2b dropped, a "that player" whose binding finds
  nobody (CR 603.6), a player who has left the game (CR 800.4a). Every
  recipient is matched, as `DealDamage`'s are. `named_player` and
  `EntersAttacking::Player` answer the same way. If a registered card leans
  on today's fallback to the controller, that is a finding of the build,
  shown in the PR.
- **Row 38.** One `subjects_from_context` for what a non-targeting recipient
  names from the resolution alone (this object, a participant, a player
  word, each player of a group, the permanents a filter matches). The walk,
  a group's members and `players_named_by` each filter it.
- **Item 307.** `is_attacking_player` beside `is_defending_player`, read by
  `decide_entry_attacks` and `entering_attack` off the entering creature's
  effective controller (CR 506.3b).

## Gates and predictions

- `cargo build --all-targets` with zero warnings, `cargo clippy
  --all-targets -- -D warnings`, `cargo test` with the whole-game and
  `clone_bound_test` tests in release, and the check scripts on
  `CLAUDE.md`'s line.
- The close-out (`plans/close_out.py`): both pools at two seats and four,
  **predicted identical**, since no registered card exiles, returns or
  shuffles a target, reads a player whose binding finds nobody, or uses a
  word this PR adds; and the callgrind budget, whose hot path is item 155's
  search on the "another target" spells.
- The big audited smoke (decision 3 changes what a binding names):
  `fuzz_games --audit`, 1,000 games at seed 777, both pools.
- `specdb owed --phase "Phase 7"` clean.
- The dev GUI's tests only if a `ChoiceKind` changes; none is planned.

## Filed at the close-out, if the decisions stand

Each with its reachability, size and slot:
- the target per player (decision 4), at `multiplayer-rules`;
- "for each [object]" at resolution, if no item already holds it;
- "for each opponent who does": a player's answer to an earlier "may",
  recorded per player (Tempt with …);
- the exiled and chosen object words (decision 2's table), each with its
  first card.
