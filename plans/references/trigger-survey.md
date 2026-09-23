# The trigger survey — what triggers watch, and what the performed stream carries

**What this is.** Step 1 of `roadmap-v2.md` row A6: before the triggers
architecture doc decides a shape, it has to know what it is deciding for —
which events triggered abilities watch, how often, in what shapes, and which
of them the engine's performed-event stream (`codebase-state.md` main item 42)
can already carry. That is `codebase-state.md` "Before Triggered abilities"
item 2, the event-shape audit, written on 2026-08-24 as a three-bullet
checklist and never run against the stream RA built. This is the run.

**What it is not.** Research tooling, not rules authority: the CR decides what
a trigger is, the triggers architecture doc decides the types, and this file
records what was counted and what was found. It proposes no trigger AST and
registers no card — the pools stay frozen until a dispatcher exists, since
`tests/determinism_test.rs` plays `default_registry` and a card the engine
cannot play is a card the fuzz harness would build.

**Every number is generated.** The tables between the `trigger-survey:` markers
are written by `plans/references/trigger-survey.py --write`; the script prints
the Scryfall query beside every count, reads the corpus atoms out of
`plans/atomic-tests/spec.sqlite`, parses the `GameEvent` and `GameAction` enums
out of the tree, and **asserts every field a row names against the enum** — so
a rename, a new field or a closed gap fails the script rather than leaving this
doc quietly wrong. The prose outside the markers is authored: the decisions,
the gap list and the questions.

<!-- trigger-survey: begin DATE -->
Counted 2026-09-23 (Scryfall fetch date from the cache; the tree and the corpus as of the run).
<!-- trigger-survey: end DATE -->

---

## 1. What was asked, and the four decisions

The brief named four decisions and a recommendation for each; all four were
taken as recommended.

1. **What counts as one trigger event: the CR's phrasing, not Scryfall's.**
   "Dies" and "is put into a graveyard from the battlefield" are one row,
   because CR 700.4 makes them one event ("the term dies means 'is put into a
   graveyard from the battlefield'"), and CR 603.6c names both spellings of the
   same leaves-the-battlefield trigger. Where Scryfall's vocabulary is finer
   than the CR's (the four "becomes" transitions, the five attack shapes of
   CR 508.3a–e) the row is the CR's and the note says what it folds.
2. **A missing field is a row in table two and an item, never a patch.** The
   doc owns the event stream's shape (row A6: "what waits for the doc, because
   the doc names its fields"). Every gap below is filed in `codebase-state.md`
   with a reachability line and a size and built nowhere.
3. **No trigger AST.** A survey names what must be expressible; the doc decides
   the type. Table two's "what the condition reads" column is that list.
4. **No cards registered.** See "What it is not" above.

**Two traps the brief named, and what happened to each.** Scryfall's `o:` is a
substring match, so the tables use the substring form and print the query;
the over-count is a known one (`o:"when "` matches "whenever ", and a card
with two triggers counts once per phrase it contains). The brief also reported
that the regex form did not honor word boundaries through the API in its own
test; on 2026-09-18 it did (§7), and the tables still use the substring form
on purpose — a number anyone can reproduce beats a cleaner one that rests on
a feature the API does not document as stable. And the CR is the customer: an
event the CR names with no printed card today still gets a row (603.6c's
owner-leaves clause, 603.7b–g's provenance rules), since a fixture is its test
until a card is. Two rows were first drafted that way and the owner's review
found the cards — Avatar Aang prints CR 603.1b's form, and Wan Shi Tong's
family prints 603.10a's third class — so both carry queries now. That is the
trap in the other direction: the phrase the CR uses is not always the phrase a
card uses, and a "not searched" needs a second search before it stands.

---

## 2. The stream and the corpus today

The counts the tables are read against. The stream side is parsed from the
tree; the corpus side is `plans/atomic-tests/spec.sqlite`, which
`python plans/specdb.py build` regenerates from the session files.

<!-- trigger-survey: begin STREAM -->
| what | count | reproduced by |
|---|---:|---|
| `GameEvent` variants | 32 | `pub enum GameEvent` in `mtgsim/src/events/event.rs`, parsed |
| ... emitted somewhere | 32 | an emit call naming the variant within four lines |
| ... never emitted | 0 |  |
| emit call sites | 35 | `emit_event(` and `emit_event_unstamped(` in `mtgsim/src`, the definitions and comment lines excluded |
| `GameAction` variants | 23 | `pub enum GameAction` in `mtgsim/src/engine/actions.rs`, parsed |
| `ZoneChangeCause` arms | 22 | `pub enum ZoneChangeCause` in `mtgsim/src/types/zones.rs`, parsed |
| registered cards with a triggered ability | 3 | `AbilityType::Triggered` in `mtgsim/src/cards/*.rs` |
<!-- trigger-survey: end STREAM -->

On 2026-09-18 three variants — `PhaseEnd`, `StepEnd`, `TurnEnd` — were
declared and emitted nowhere. No printed trigger reads an end: "at end of
combat" is the end-of-combat step *beginning* (CR 511.2) and "at end of turn"
was errata'd to "at the beginning of the end step" (CR 513.1a). TR-1 deleted
the three (item 18 below) and added `AbilityTriggered`, the one record emitted
through the second door, `emit_event_unstamped`, which the site count reads
since 2026-09-23.

<!-- trigger-survey: begin CORPUS -->
| what | count | reproduced by |
|---|---:|---|
| Phase 7 atoms | 133 | `SELECT COUNT(*) FROM atoms WHERE phase='Phase 7'` |
| ... CR sections they span | 36 | `COUNT(DISTINCT substr(rule_num,1,3))`, same filter |
| ... under CR 603 | 52 | `rule_num LIKE '603%'`, same filter (54 in the corpus at any phase) |
| ... fully covered | 44 | `coverage.partial = 0` |
| ... partially covered, by a test that does not build the atom | 2 | `coverage.partial = 1` and no full row |
| | | `ATOM-121.5-001` -- `test_library_to_hand_without_drawing_is_not_a_draw` |
| | | `ATOM-608.2d-001` -- `test_a_player_with_no_creatures_is_asked_nothing_and_no_error_is_raised` |
<!-- trigger-survey: end CORPUS -->

On 2026-09-18 no Phase 7 atom was fully covered, and the six partials were
each a test that proved the *event* half of an atom whose trigger half is
critical-path item 6's — the `ATOM-615.6-001` test's name says so in as many
words. TR-1 took four of the six to full; the two left are TR-2's
(`ATOM-121.5-001`) and a deferral (`ATOM-608.2d-001`,
`triggers-architecture.md` §13).

---

## 3. Table one — the CR's own vocabulary

Every trigger event, shape or gate that CR 603.1b through 603.12a names, in
the CR's order, with the corpus atoms that exercise it and a printed count
where a phrase isolates it. "Shape" is the brief's axis: per event,
one-or-more, state, delayed, intervening-if, look-back — plus reflexive
(603.12), the two count modifiers (603.2d, 603.2h), 603.2h's uncodified
sibling "this ability triggers only once each turn", and 603.7c–g's identity
and provenance rules, which the CR names and the axis did not.

<!-- trigger-survey: begin TABLE-1 -->
| CR | the event, as the CR words it | shape | corpus atoms | cards | Scryfall query (plus `game:paper -is:funny`, `unique=cards`) |
|---|---|---|---|---:|---|
| 603.1b | more than one trigger condition, and "all" of them in a period -- Avatar Aang (2025) prints the form: four bending conditions and "if you've done all four this turn", which its ruling reads over the whole turn whether or not Aang was there -- so the tracker is per player per turn, not per source | multi-condition (turn history) | 603.1b-001 | 1 | `o:"done all"` |
| 603.2 | a game event or a game state matches the trigger event -- every card that carries a trigger; the base every other row is a share of | umbrella | 603.2-001 | 14,149 | `(o:"when " or o:"whenever " or o:"at the beginning of")` |
| 603.2b | a phase or step begins -- "at the beginning of" -- the step rows in table 2 split it | per event | 603.2b-001 | 2,566 | `o:"at the beginning of"` |
| 603.2c | one event with several occurrences -- once per occurrence, or once for "one or more" -- the batch (`BatchId`) is the boundary | one-or-more | 603.2c-001 | 443 | `o:"whenever" o:"one or more"` |
| 603.2d | a triggered ability triggers additional times -- not an event: a multiplier read as the ability triggers. Panharmonicon's ruling draws its edges -- the object's own triggered abilities only, never CR 603.6d's "enters" statics or a replacement effect; the rule's last sentence excludes the delayed and reflexive triggers those abilities create; and an ability that "triggers only once each turn" is not doubled (the row below 603.2h) | count modifier | 603.2d-001 | 37 | `o:"triggers an additional time"` |
| 603.2e | "becomes" -- tapped, untapped, attached, blocked: the transition only -- "becomes the target" and "becomes unattached" have rows of their own below | per transition | 603.2e-001, 603.2e-002 | 328 | `(o:"becomes tapped" or o:"becomes untapped" or o:"becomes attached" or o:"becomes blocked")` |
| 603.2f | the object with the ability is at no time visible to all players -- it does not trigger -- not an event; a gate on every row. It answers `atomic-tests/supplemental-docs/603-2f-complexity.md`: Guerrilla Tactics discarded onto the library under Library of Leng is never visible and does not trigger, and under Future Sight the top card is revealed and it does -- visibility is per object, not per zone (S1). The gate is a *global* bit, "visible to all players" at the instant after the event, a subset of `backlog.md` §2.9's per-viewer query; that entry weighed moving up at RE-8's close and stayed as `roadmap-v2.md` B4 (1-2 PRs, anywhere in A or B, back-stopped before Phase 8's reveal cards), and until a reveal exists no hidden-zone object is visible, so `Zone::is_public()` is exact today and the doc names the predicate per object for B4 to fill | visibility gate | 603.2f-001 | -- | not searched |
| 603.2g | a prevented or replaced event never happened -- why the matcher reads the *performed* stream and nothing upstream of it | stream property | 603.2g-001 | -- | not searched |
| 603.2h | "Do this only once each turn" -- the action-taken gate, per source object: Nykthos Paragon's rulings have every life gain trigger it until the action is taken, one of two instances on the stack act (ATOM-603.2h-002), and two Paragons act twice; Panharmonicon can double it, since the extra instance just does nothing | per-turn action gate | 603.2h-001, 603.2h-002 | 34 | `o:"do this only once each turn"` |
| (no rule; CR 702.179d's speed is the baseline CR's one use) | "This ability triggers only once each turn" -- a cap on triggering, per source object -- no rule of its own in the baseline CR, so the earliest printing's ruling is the definition -- Elvish Warmaster (Kaldheim, found with `order:released direction:asc`): "Once the triggered ability has triggered once during a turn, it can't trigger again, even if the triggered ability is still on the stack, has been countered, or has otherwise left the stack." The later rulings fill in the edges (Jin-Gitaxias: once, not once per opponent; Tyvar: once per creature it is granted to; Fang and Stonebinder's Familiar: once for a batch) and the judge literature adds that Panharmonicon cannot double it. A flag set as the ability triggers: a second tracker beside 603.2h's, written by the dispatcher rather than by the resolution -- question 16 | per-turn trigger gate | none | 128 | `o:"triggers only once each turn"` |
| 603.3b | another ability triggering -- the second APNAP tier -- Strict Proctor's shape: the trigger event is a trigger | per event | 603.3b-001, 603.3b-002 | 1 | `o:"causes a triggered ability to trigger"` |
| 603.4 | intervening "if" -- the condition read as the event happens and again at resolution -- a comma-if anywhere behind a trigger word; the regex reading is §7's pair 6 | intervening-if | 603.4-001, 603.4-002, 603.4-003 | 1,338 | `(o:"when " or o:"whenever " or o:"at the beginning of") o:", if "` |
| 603.5 | "may" and "unless" -- the choice is made at resolution, the ability stacks regardless -- not an event; a `ChoiceKind` the phase adds (A4j) | optional | 603.5-001 | -- | not searched |
| 603.6a | a permanent enters the battlefield -- `o:"enters"` alone also matches CR 603.6d's "enters" statics, which are not triggers -- the line Panharmonicon's ruling draws, since CR 603.2d doubles the triggers and never the statics | per event (zone change) | 603.6-001, 603.6a-001 | 5,701 | `(o:"when " or o:"whenever ") o:"enters"` |
| 603.6c | a permanent leaves the battlefield, including "dies" (CR 700.4: put into a graveyard from the battlefield) -- one row, not two, because CR 700.4 makes the phrases one event | look-back (603.10a) | 603.6c-001, 603.6c-002 | 1,688 | `(o:"when " or o:"whenever ") (o:"leaves the battlefield" or o:"dies" or o:"put into a graveyard from the battlefield")` |
| 603.6c | a phased-in permanent leaves the game because its owner leaves -- no printed phrase to search; `GameEvent::LeftTheGame` carries the frame and item 6 owns the qualifier | look-back (603.10a) | 603.6c-001, 603.6c-002 | -- | not searched |
| 603.6c | put into a zone "from anywhere" -- never a leaves-the-battlefield ability -- the row that must *not* look back through 603.10a's first class: Guile's ruling has it trigger from the graveyard even when Lignify took the ability on the battlefield, and not when Yixlid Jailer takes it in the graveyard. A library or hand destination is 603.10a's third class instead, and that one does look back | per event (zone change) | 603.6c-001, 603.6c-002 | 94 | `(o:"when " or o:"whenever ") o:"from anywhere"` |
| 603.6e | the enchanted permanent leaves the battlefield -- an Aura's own trigger -- finds both new objects: the card and the Aura in its graveyard | look-back (400.7e/f) | 603.6e-001, 603.6e-002 | 98 | `(o:"when " or o:"whenever ") o:"enchanted" (o:"dies" or o:"leaves the battlefield")` |
| 603.7 | a delayed triggered ability -- "at the beginning of the next ...", "when this creature becomes untapped" -- created at resolution, never retroactive (603.7a); CR 513.2's next-turn rule | delayed | 603.7-001, 603.7a-001 | 391 | `o:"at the beginning of the next"` |
| 603.7b | once -- the next time its event occurs -- unless a stated duration; simultaneous events, the controller chooses -- not searched: the first draft's `o:"the next time"` counts CR 615's shields ("the next time ... would deal damage ... prevent"), not delayed triggers, and was withdrawn; ATOM-603.7b-002 is the simultaneous case (Tatsumasa under a doubler) | delayed | 603.7b-001, 603.7b-002 | -- | not searched |
| 603.7c | a delayed trigger tracks its object through characteristic changes, and loses it at a zone change (CR 400.7) -- an object reference, not a filter -- the same identity question as 603.6's zone-change triggers | delayed: identity | 603.7c-001 | -- | not searched |
| 603.7d | created by a spell: the source is the spell, the controller whoever controlled it as it resolved -- the spell's stack object is gone by the time the trigger fires (CR 608.2n), so the source is a remembered identity | delayed: provenance | 603.7d-001 | -- | not searched |
| 603.7e | created by an activated or triggered ability: the source is that ability's source -- inherits `AbilityIdentity`'s source half | delayed: provenance | 603.7e-001 | -- | not searched |
| 603.7f | created by a static ability's replacement effect: the source is the object with the static ability, the controller its controller as the replacement applied -- the pipeline is the producer and carries no resolution stamp -- question 14 | delayed: provenance | 603.7f-001 | -- | not searched |
| 603.7g | created by a static ability that let a player take an action: the source is that object, the controller its controller as the action was taken -- a special action is the producer -- question 14 | delayed: provenance | 603.7g-001 | -- | not searched |
| 603.7h | the ability that created it has resolved N times this turn -- Ashling's shape; the count is per instance or per ability (S3) | delayed, counted | 603.7h-001 | 27 | `o:"time this ability has resolved this turn"` |
| 603.8 | a game state matches -- a state trigger -- no event by definition; P1's mid-resolution check | state | 603.8-001, 603.8-002 | 29 | `(o:"when you control no" or o:"whenever you control no" or o:"when there are no" or o:"whenever there are no" or o:"when you have no" or o:"whenever you have no")` |
| 603.9 / 603.10f | a player loses the game, or leaves it other than by a draw -- over-count: also matches "you lose the game" effects behind an unrelated trigger | look-back | 603.9-001 | 21 | `(o:"when " or o:"whenever ") o:"loses the game"` |
| 603.10a | a card leaves a graveyard -- the second of 603.10a's three classes | look-back | 603.10a-001, 603.10a-002 | 38 | `(o:"when " or o:"whenever ") (o:"leaves your graveyard" or o:"leaves a graveyard" or o:"leave your graveyard" or o:"leave a graveyard")` |
| 603.10a | an object all players can see is put into a hand or library -- the third class, and printed: Wan Shi Tong and Dutiful Knowledge Seeker ("put into a library from anywhere"), Golgari Brownscale (into your hand from your graveyard), Stormfront Riders (returned to your hand from the battlefield -- its ruling has it trigger for itself when bounced with another). A custom card can name the class outright, which is why the row carries a query rather than "not searched" | look-back | 603.10a-001, 603.10a-002 | 8 | `(o:"put into a library from" or o:"put into your hand from" or o:"is returned to your hand" or o:"is returned to its owner's hand" or o:"are put into a library") (o:"when " or o:"whenever ")` |
| 603.10b | a permanent phases out -- phasing is unbuilt | look-back | none | 37 | `o:"phases out"` |
| 603.10c | an object becomes unattached | look-back | 603.10c-001, 603.10c-002, 603.10c-003 | 4 | `o:"becomes unattached"` |
| 603.10d | a player loses control of an object, or an opponent gains control of it from them | look-back | 603.10d-001 | 126 | `(o:"when " or o:"whenever ") (o:"gains control" or o:"gain control" or o:"loses control" or o:"lose control")` |
| 603.10e | a spell is countered -- over-count: "can't be countered" behind an unrelated trigger | look-back | 603.10e-001 | 19 | `(o:"when " or o:"whenever ") o:"countered"` |
| 603.10g | a player planeswalks away from a plane -- Planechase is excluded | out of scope | none | -- | not searched |
| 603.11 | a static ability linked to a triggered one -- "whenever you reveal ... this way" -- CR 607's linkage; the trigger condition sits mid-paragraph | per event | none | 6 | `o:"whenever you reveal"` |
| 603.12 | reflexive -- "when you do", "when [something happens] this way" -- checked immediately after creation, against the creating resolution's own events | reflexive | 603.12-001 | 258 | `o:"when you do"` |
| 603.12a | "when you pay [that cost] one or more times" -- once, however many times | reflexive | 603.12a-001 | 7 | `o:"one or more times"` |
<!-- trigger-survey: end TABLE-1 -->

**Reading it.** The atom column is the corpus's coverage of the *rule*, not of
the card population — 603.6a has two atoms and the largest population in the
game. Rows with "not searched" are the CR-is-the-customer rows: the rule names
an event or a gate that no oracle phrase isolates, and the atom (or, for
603.10a's third class, a fixture yet to be written) is its test.

---

## 4. Table two — the printed distribution against the performed event

Each trigger event printed cards use, most common first, against the
performed event that would carry it today, the facts its condition reads and
where each fact is: **on the record** (a named field, asserted to exist),
**live** (read off `GameState` at dispatch — which the resolved design allows,
since detection is synchronous at the mutation instant and "that instant is
now"), **a turn tracker** (item 42's per-player summaries, which wait for the
doc), **the envelope** (`EventStamp`'s batch and resolution), or **not on the
record** (a field the performer drops or a frame that is not captured).
"**none**" in the performed-event column is a gap; "(any)" means the row is a
shape every event can wear.

<!-- trigger-survey: begin TABLE-2 -->
| trigger event | CR | cards | performed event today | what the condition reads, and where it is | note |
|---|---|---:|---|---|---|
| enters the battlefield | 603.6a | 5,701 | `PermanentEnteredBattlefield`, `ZoneChange`, `TokenCreated` | the permanent: `PermanentEnteredBattlefield.object_id`; under whose control: `PermanentEnteredBattlefield.controller`; its types, the moment it is there (603.6b): live (the layer walk); the zone it came from: `ZoneChange.from`; created rather than moved (111.13): `TokenCreated.zone` | carried across two records: the zone change or the creation, then the entry; a token's entry has no `from` |
| attacks / is attacked / attacks with / attacks alone | 508.3a-e | 1,695 | `AttackersDeclared` | which creatures: `AttackersDeclared.attackers`; whom each attacks: **not on the record** (`AttackersDeclared` has no `defender`); whom each attacks (today): live (`AttackingInfo.target`); the attacking player: live (the active player); alone: `AttackersDeclared.attackers` | **field gap**: 508.3a's "attacks [a player]", 508.3b's "is attacked" and 508.3e's "attacks another player" read the defender, which the record does not carry |
| casts a spell | 601.2i | 1,546 | `SpellCast` | the spell: `SpellCast.spell_id`; who cast it: `SpellCast.caster`; creature spell, mana value, colors: live (the stack object); the zone it was cast from: live (the stack entry's `cast_from`); first / second spell this turn: a turn tracker (item 42) | carried; a copy is not cast (CR 707.10) and CV's copy path must emit no `SpellCast` |
| dies | 700.4 / 603.6c | 1,241 | `ZoneChange` | from the battlefield to a graveyard: `ZoneChange.from`; why: `ZoneChange.cause`; what it was, and whose (603.10a): `ZoneChange.lki` | carried; the frame is CR 603.10a's look-back, captured before CR 611.2a drops the registry rows |
| draws a card | 121.1 | 1,216 | `CardDrawn` | who: `CardDrawn.player_id`; which card: `CardDrawn.card_id`; first or second card this turn (miracle, 702.94a): a turn tracker (item 42) | carried; CR 121.5 is why this is not the library-to-hand zone change. Transcendent Archaic is the subtlety: its ETB draws X, the colors spent to cast the spell that became it (CR 400.7d, information the permanent keeps about its own casting), and "if you draw one or more cards this way" reads the count the performed draws returned, not the stream |
| at the beginning of upkeep | 603.2b / 500.6 | 1,167 | `StepBegin` | which step: `StepBegin.step`; whose turn: `StepBegin.player` | carried since TR-1 (item 10): the performer writes the `player` `GameAction::BeginStep` carries -- "your upkeep" and "each opponent's upkeep" read it |
| deals damage / deals combat damage | 120.4b | 1,127 | `DamageDealt` | the source: `DamageDealt.source_id`; the recipient: `DamageDealt.target`; how much: `DamageDealt.amount`; combat or not: `DamageDealt.is_combat`; the source's controller and types: live (the source, still on the battlefield until SBAs) | carried since TR-1 (item 10); the next row is the share that reads `is_combat` |
| deals combat damage | 510.2 / 120.4b | 804 | `DamageDealt` | combat or not: `DamageDealt.is_combat` | the share of the row above that reads the field item 10 added |
| at the beginning of the end step | 513.1 / 513.2 | 976 | `StepBegin` | which step: `StepBegin.step`; whose turn: `StepBegin.player` | carried, as the upkeep row is; CR 513.2's next-turn rule is §14's question 1 |
| intervening "if" | 603.4 | 1,338 | (any) | the condition, as the event is performed: live (the board then); the condition again, as the ability resolves (608.2a): live (the board then) | no event of its own: a predicate the matcher evaluates twice |
| sacrifices | 701.17 | 555 | `ZoneChange` | that it was a sacrifice: `ZoneChange.cause`; who sacrificed it: `ZoneChange.lki` | carried; `ZoneChangeCause::Sacrificed`, and the frame's `controller` is the player who sacrificed |
| one or more (a batch) | 603.2c | 443 | (any) | the events performed as one: the `EventStamp` envelope | carried by the envelope: every record in a batch carries one `BatchId` |
| discards | 701.8 | 436 | `ZoneChange` | that it was a discard: `ZoneChange.cause`; who: `ZoneChange.owner` | carried; "cycles or discards" (702.29d) waits for cycling |
| at the beginning of the next ... (delayed) | 603.7 | 391 | `StepBegin`, `PhaseBegin`, `TurnBegin` | the step: `StepBegin.step`; whose turn: `StepBegin.player`; "next" -- not this one (513.2): live (when the ability was created) | carried by the step rows; the creation instant is the delayed ability's own field |
| dies with or without counters, or while attached -- the frame's status | 702.79a / 702.93a / 603.6e / 603.10c | 109 | `ZoneChange` | what it was, and whose: `ZoneChange.lki`; the counters it had, what it was attached to, whether it was tapped: **not on the record** (`EffectiveCharacteristics` carries characteristics and the controller, no status) | **field gap**: persist's and undying's intervening-if read the counters the permanent had as it died; an Aura's 603.6e trigger reads what it enchanted |
| leaves the battlefield | 603.6c | 330 | `ZoneChange`, `LeftTheGame` | from the battlefield: `ZoneChange.from`; what it was (603.10a): `ZoneChange.lki`; left the game with its owner: `LeftTheGame.lki` | carried on both routes; the phased-in qualifier is item 6 |
| at the beginning of combat | 603.2b / 506.1 | 313 | `StepBegin`, `PhaseBegin` | the step: `StepBegin.step`; whose turn: `StepBegin.player` | carried, as the upkeep row is; "at end of combat" (511.2) is the end-of-combat step beginning |
| blocks / blocks a creature / becomes blocked / becomes blocked by | 509.3a-d | 210 | `BlockersDeclared` | the (blocker, attacker) pairs: `BlockersDeclared.blockers` | carried; the four shapes are four readings of one list (CR 700.1's example: one event or two) |
| gains life | 119.9 | 140 | `LifeChanged` | who: `LifeChanged.player_id`; how much: `LifeChanged.old`; the source: `LifeChanged.source` | carried; one record per source event, which is CR 702.15e's two lifelink triggers |
| loses life | 120.3a | 51 | `LifeChanged` | who: `LifeChanged.player_id`; how much: `LifeChanged.new`; from damage, a payment or an effect (727.1a): `LifeChanged.cause` | carried; one record per `LoseLife` proposal -- combat damage from two attackers is two (§14's question 3: per record); `cause` since TR-1 (item 10) |
| becomes the target | 115.1 / 603.2e | 117 | **none** | the object targeted: no event; by which spell or ability, whose: no event | **no event**: targets are chosen at CR 601.2c and announced to nothing |
| ward -- becomes the target of an opponent's spell or ability | 702.21a | 195 | **none** | the object targeted, and who controls the targeting spell: no event | **no event**: the same gap, with a keyword's population behind it |
| becomes tapped | 603.2e | 114 | `Tapped` | the permanent: `Tapped.object_id` | carried; emitted on the transition only (CR 603.2e), never for a redundant tap or an entry |
| becomes untapped | 603.2e | 33 | `Untapped` | the permanent: `Untapped.object_id` | carried, as tapping is |
| is dealt damage | 120.4b | 108 | `DamageDealt` | the recipient: `DamageDealt.target`; excess damage (120.10): live (toughness and marked damage) | carried |
| tapped for mana | 106.12a | 31 | `ManaAdded` | that the source was tapped for it: `ManaAdded.tapped_for_mana`; what mana: `ManaAdded.mana` | carried; CR 605.1b makes the trigger a mana ability that skips the stack (main item 11) |
| counters are put on / the Nth counter | 122.6 / 122.7 | 72 | `CountersChanged` | the subject: `CountersChanged.subject`; how many: `CountersChanged.added`; fewer than N before, N or more after: live (the count now, minus `added`); counters it entered with (122.6): **not on the record** (the entry announces no `CountersChanged` and `PermanentEnteredBattlefield` carries no `mods`) | carried for a permanent on the battlefield; CR 122.6 makes entry counters "put on" it too -- a question |
| activates an ability | 602.2a | 39 | `AbilityActivated` | which ability: `AbilityActivated.identity`; who: `AbilityActivated.controller` | carried |
| an ability resolves / has resolved N times | 608.2p / 603.7h | 27 | `AbilityResolved`, `ZoneChange` | which ability, durably: `AbilityResolved.identity`; a spell resolving: `ZoneChange.cause`; this instance or this ability (S3): a turn tracker (item 42) | carried; the per-turn count is the tracker's and its key is S3's decision |
| is countered | 603.10e | 19 | `SpellCountered`, `AbilityCountered`, `SpellFizzled` | the spell: `SpellCountered.spell_id`; by what: `SpellCountered.countered_by` | carried; CR 608.2b (tmnt) says a spell whose targets are all illegal "doesn't resolve" -- `SpellFizzled` is not this trigger's event, a question for the doc |
| loses the game | 603.9 | 21 | `PlayerLost` | who: `PlayerLost.player_id`; why: `PlayerLost.reason` | carried; the 800.4d refusal at placement is item 7 |
| creates a token | 701.7a / 111.13 | 14 | `TokenCreated` | the token, where: `TokenCreated.zone`; whose: `TokenCreated.owner` | carried; item 8 -- keyed on this event, never on `is_token` at entry |
| scries / surveils | 701.22d / 701.25d | 31 | `Scried` | who: `Scried.player_id`; how many, and how many really (Elrond): `Scried.looked_at` | carried for scry; surveil is unbuilt |
| shuffles a library | 701.24e | 168 | `LibraryShuffled` | whose: `LibraryShuffled.player_id` | carried; one record per shuffle (701.24f) |
| mills / put into a graveyard from anywhere / leaves a graveyard | 701.17a / 603.6c / 603.10a | 406 | `ZoneChange` | the move and why: `ZoneChange.cause`; what it was, off the battlefield (603.10a's second class): **not on the record** (`lki` is `None` unless `from` is the battlefield) | carried by cause; the `lki` frame is captured only for a battlefield departure, so 603.10a's graveyard-leaving class has no frame |
| is exiled | 701.13 | 21 | `ZoneChange` | the move and why: `ZoneChange.cause` | carried; `ZoneChangeCause::Exiled` |
| plays a land | 305.1 | 16 | `ZoneChange`, `PermanentEnteredBattlefield` | played rather than put: `ZoneChange.cause` | carried; `ZoneChangeCause::PlayedAsLand` |
| becomes attached | 603.2e / 701.3a | 10 | `Attached` | the attachment and its host: `Attached.host`; moved from another host: `Attached.former_host` | carried on the transition only (701.3b) |
| becomes unattached | 603.10c | 4 | `Attached`, `EquipmentDetached`, `ZoneChange` | left a host for another: `Attached.former_host`; dropped by CR 704.5n: `EquipmentDetached.former_host`; the host left the battlefield: live (the host's `ZoneChange` and the attachment's own frame) | three routes and no one record; the third is a look-back with no "unattached" line -- a question for the doc |
| gains control / loses control | 603.10d | 126 | **none** | the object, the old and the new controller: no event | **no event**: control changes are Layer 2 continuous effects and a control-changing `Primitive` writes a registry row, which is not a performed event |
| phases out | 603.10b / 702.26 | 37 | **none** | the permanent: no event | **no event, and no action**: phasing is unbuilt (`codebase-state.md`, "Phasing") |
| searches a library | 701.23f | 21 | **none** | who searched which library: no event | **no event, and no action**: a search is not a `GameAction`; the found card's move is |
| damage is prevented | 615.13 | 15 | **none** | that a prevention effect applied: no event | **no event**: a shield applying is a CR 616.1 trace record, not a performed event (615.13 wants one per prevention applied) |
| state triggers | 603.8 | 29 | **none** | the state: live (a predicate at every dispatch (P1)) | no event by definition |
| reflexive -- "when you do" | 603.12 | 258 | (the creating resolution's own records) | the action, inside the creating resolution: the `EventStamp` envelope | carried: the records a resolution performed carry its `ResolutionStamp` |
| turn history: first / second spell, second card, last turn, this game | 603.1b / item 42 | 237 | **none** | what a player did this turn, last turn, this game: a turn tracker (item 42) | item 42's per-player turn summaries, which wait for the doc (P2-P4). "This game" is a scope, not a window: Commander's Insight counts commander casts from the command zone this game, which is CR 903.8's own counter -- question 15 |
| keyword actions with no engine action: turned face up, transforms, cycles, explores, crews, expends, commits a crime | 701 / 702 / 700.13-14 | 318 | **none** | the keyword action: no event | **no action, so no event**: Phase 8's breadth, one event per keyword as the keyword lands |
| inherent triggers with no source: the monarch, the initiative, rad counters | 724.2 / 725.2 / 727.1 | 113 | `StepBegin`, `DamageDealt` | the events: `StepBegin.step`; the source: no source -- CR 113.8's exception | the events exist; the *source* does not, and `AbilityIdentity` has no arm for a rule-owned ability -- a question for the doc |

Queries, in row order (each plus `game:paper -is:funny`, `unique=cards`):

- enters the battlefield: `(o:"when " or o:"whenever ") o:"enters"`
- attacks / is attacked / attacks with / attacks alone: `(o:"when " or o:"whenever ") o:"attacks"`
- casts a spell: `o:"whenever" o:"cast"`
- dies: `(o:"when " or o:"whenever ") o:"dies"`
- draws a card: `o:"whenever" o:"draw"`
- at the beginning of upkeep: `o:"at the beginning of" o:"upkeep"`
- deals damage / deals combat damage: `(o:"when " or o:"whenever ") (o:"deals damage" or o:"deals combat damage")`
- deals combat damage: `(o:"when " or o:"whenever ") o:"deals combat damage"`
- at the beginning of the end step: `o:"at the beginning of" o:"end step"`
- intervening "if": `(o:"when " or o:"whenever " or o:"at the beginning of") o:", if "`
- sacrifices: `o:"whenever" o:"sacrifice"`
- one or more (a batch): `o:"whenever" o:"one or more"`
- discards: `o:"whenever" o:"discard"`
- at the beginning of the next ... (delayed): `o:"at the beginning of the next"`
- dies with or without counters, or while attached -- the frame's status: `(kw:persist or kw:undying or o:"enchanted creature dies" or o:"becomes unattached")`
- leaves the battlefield: `(o:"when " or o:"whenever ") o:"leaves the battlefield"`
- at the beginning of combat: `o:"beginning of combat"`
- blocks / blocks a creature / becomes blocked / becomes blocked by: `(o:"when " or o:"whenever ") o:"blocks"`
- gains life: `o:"whenever" o:"gain life"`
- loses life: `o:"whenever" (o:"lose life" or o:"loses life")`
- becomes the target: `o:"becomes the target"`
- ward -- becomes the target of an opponent's spell or ability: `kw:ward`
- becomes tapped: `o:"becomes tapped"`
- becomes untapped: `o:"becomes untapped"`
- is dealt damage: `(o:"when " or o:"whenever ") o:"is dealt damage"`
- tapped for mana: `o:"tapped for mana"`
- counters are put on / the Nth counter: `o:"whenever" (o:"counter is put" or o:"counters are put" or o:"put one or more")`
- activates an ability: `(o:"when " or o:"whenever ") (o:"activate an ability" or o:"activates an ability" or o:"activate a loyalty" or o:"activates a loyalty")`
- an ability resolves / has resolved N times: `o:"time this ability has resolved this turn"`
- is countered: `(o:"when " or o:"whenever ") o:"countered"`
- loses the game: `(o:"when " or o:"whenever ") o:"loses the game"`
- creates a token: `(o:"whenever you create" or o:"whenever a player creates" or o:"whenever an opponent creates" or o:"token is created" or o:"tokens are created")`
- scries / surveils: `(o:"whenever you scry" or o:"whenever you surveil" or o:"scries" or o:"surveils")`
- shuffles a library: `(o:"whenever you shuffle" or o:"shuffles")`
- mills / put into a graveyard from anywhere / leaves a graveyard: `(o:"whenever you mill" or o:"mills" or o:"from anywhere" or o:"leaves your graveyard" or o:"leaves a graveyard")`
- is exiled: `(o:"whenever you exile" or o:"whenever a player exiles" or o:"is exiled from" or o:"are exiled from" or o:"card is exiled" or o:"cards are exiled")`
- plays a land: `(o:"when " or o:"whenever ") o:"play a land"`
- becomes attached: `o:"becomes attached"`
- becomes unattached: `o:"becomes unattached"`
- gains control / loses control: `(o:"when " or o:"whenever ") (o:"gains control" or o:"gain control" or o:"loses control" or o:"lose control")`
- phases out: `o:"phases out"`
- searches a library: `(o:"whenever you search" or o:"whenever a player searches" or o:"searches")`
- damage is prevented: `(o:"when " or o:"whenever ") o:"prevented"`
- state triggers: `(o:"when you control no" or o:"whenever you control no" or o:"when there are no" or o:"whenever there are no" or o:"when you have no" or o:"whenever you have no")`
- reflexive -- "when you do": `o:"when you do"`
- turn history: first / second spell, second card, last turn, this game: `(o:"first spell" or o:"second spell" or o:"second card" or o:"last turn" or o:"this game")`
- keyword actions with no engine action: turned face up, transforms, cycles, explores, crews, expends, commits a crime: `(o:"when " or o:"whenever ") (o:"turned face up" or o:"transforms" or o:"cycle" or o:"explores" or o:"becomes crewed" or o:"expend" or o:"commit a crime")`
- inherent triggers with no source: the monarch, the initiative, rad counters: `(o:"the monarch" or o:"the initiative" or o:"rad counter")`
<!-- trigger-survey: end TABLE-2 -->

**What the table says in one paragraph.** The stream carries the big
populations: enters, dies, casts, draws, sacrifices, discards, the step
beginnings, the combat declarations, life and counters and tokens — each has a
record, a cause where the CR distinguishes causes, and CR 603.10a's frame
where the CR looks back. What it does not carry falls into four kinds. A
**dropped field**: the proposal knew and the performer did not write it down
(whose step, combat or not, the life loss's cause, each attacker's defender).
A **frame too narrow**: `lki` is characteristics and a controller, captured
only when a permanent leaves the battlefield, while persist and undying read
the counters it had and CR 603.10a's other two classes read cards leaving
graveyards. A **missing event** for something the engine already does:
choosing a target, changing control, applying a prevention shield, entering
with counters. And **no action at all**: phasing, searching, the keyword
actions of Phase 8 — not this phase's, and named so the dispatcher is built to
take a new event kind without a redesign.

---

## 4a. Table three — what conditions and resolution clauses read, and where the answer lives

Tables one and two count what triggers *watch*. This one counts what they
*read*: every trigger condition and every resolution clause that asks a
question — the intervening "if", "if you do / don't / can't", "when you
do", "this way", "that many", "for each", X, a delayed trigger's "it" — and
the place the answer lives. Table two gave the intervening "if" one row and
one place, "live (the board then)"; the owner's Vibrance question on
2026-09-23 found about 200 of them reading a fact about the spell the
permanent was, which is not on the board at all. Six places:

- **the board now** — `Condition`, read by `settled_holds` at either instant;
- **the spell it was** — CR 400.7d: "what costs were paid to cast that spell
  or what mana was spent to pay those costs";
- **a history** — `triggers-architecture.md` §3.10's trackers;
- **the bound object's last known information, or the matched record** — the
  binding (§3.4) and the frame (§3.11), CR 608.2h and 113.7a;
- **the resolution's own choices** — CR 118.12: "whether the player chose to
  pay an optional cost or started to pay a mandatory cost, regardless of what
  events actually occurred";
- **the resolution's own events** — CR 603.12: "earlier during the resolution
  of the spell or ability that created them".

The last column says who holds each place today: the engine (named), the
design (the section and the phase), or **neither**. That column is a reading
of the tree and the design by hand; the counts and queries are generated.

<!-- trigger-survey: begin TABLE-3 -->
| clause, and what it reads | CR | cards | where the answer lives | who has that place |
|---|---|---:|---|---|
| intervening "if", every one -- the survey's one bucket, which table two modeled as "live (the board then)"; the rows below split it | 603.4 | 1,338 | the board now; the spell it was (400.7d); a history (§3.10); the bound object's LKI, or the matched record | design: §6.1, both instants through `settled_holds` |
| ... a presence or a count on the board: "if you control", "if you have", "if an opponent", "if there are", "if that player" -- the place the design assumed for all of them | 603.4 / 608.2a | 398 | the board now | engine: `Condition` and `settled_holds`, a leaf per card |
| ... this turn or last turn: morbid, raid, revolt, "no spells were cast last turn" -- over-count: "this turn" anywhere in the text | 603.4 / 603.1b | 408 | a history (§3.10) | design: `PlayerHistory` (§3.10, TR-2); engine: none |
| ... the spell it was: mana spent to cast it -- adamant, "if mana from a Treasure was spent", the evoke Incarnations -- main item 9's 2026-09-22 note: "mana spent is recorded nowhere yet, so that one needs its capture at payment first" | 400.7d / 601.2h | 58 | the spell it was (400.7d) | **neither**: no payment records which mana paid it (item 30), and `CastFacts` has no field for it |
| ... the spell it was: kicked or bargained -- an additional cost | 400.7d / 702.33d | 107 | the spell it was (400.7d) | engine: `StackEntry.additional_costs_paid`, never carried to the permanent; `Condition::SpellWasKicked` is declared and its only evaluator asserts; design: a `CastFacts` field when a card reads one (main item 9) |
| ... the spell it was: an alternative cost -- prowl, surge, spectacle, madness, emerge, sneak | 400.7d / 118.9 | 13 | the spell it was (400.7d) | engine: `StackEntry.chosen_alternative_cost`, never carried; design: a `CastFacts` field (main item 9) |
| ... the spell it was: cast at all, and by whom -- "you" is the ability's controller, so a copy of the trigger controlled by another player fails it (main item 9) | 400.7d / 603.4 | 72 | the spell it was (400.7d) | engine: `PermanentState.cast` (`CastFacts { by, from }`, TR-1); no `Condition` leaf yet |
| ... the spell it was, read again after the source is gone: evoke beside a mana-spent "if" -- Vibrance and its four siblings: the evoke sacrifice and the ETB trigger go on the stack together, and if the sacrifice resolves first the ETB's recheck reads a gone source; CR 113.7a and 608.2h answer it from last known information | 603.4 / 113.7a / 702.74a | 5 | the spell it was (400.7d); the bound object's LKI, or the matched record | **neither**: item 169's evaluator answers false for a source that has left, and §3.11's frame carries no cast facts |
| ... an object the event moved, as it last existed: "if it had counters on it", "if it was a Human" | 603.10a / 608.2h | 46 | the bound object's LKI, or the matched record | engine: the frame's characteristics (`ZoneChange.lki`); design: counters on §3.11's `Status` (TR-4) |
| ... persist and undying: the reminder text's "if it had no ... counters on it" -- `o:` excludes reminder text, so the keyword query | 702.79a / 702.93a | 48 | the bound object's LKI, or the matched record | design: `Status.counters` (§3.11, TR-4); engine: none (item 14) |
| ... a leaves-the-battlefield trigger whose source is gone: "you" and the board at both instants | 603.4 / 603.10a / 109.5 | 105 | the board now; the bound object's LKI, or the matched record | design: §6.1's `TriggerContext` evaluator (TR-2); engine: false for a departed source (item 169) |
| ... the source's own zone: "if this card is in your graveyard", CR 113.6b's statement | 113.6b / 603.4 | 17 | the board now | engine: `Condition::SourceInZone`; **neither** reads it as the trigger's zone statement (item 173, unscheduled) |
| ... a choice made as it entered: tribute -- not a cast fact, but the same kind of place: something the permanent remembers about arriving | 702.104b / 614.12 | 11 | the spell it was (400.7d) | **neither**: tribute is Phase 8's keyword; the choice is made as the permanent enters and kept on it |
| X, the value the spell that became it was cast with -- over-count: an X spell with any enters trigger, whether or not that trigger reads X | 107.3m | 55 | the spell it was (400.7d) | engine: `PermanentState.x_value`; no reader (ATOM-107.3m-001, deferred to Phase 8) |
| "that many": the matched record's amount, or the resolution's own count | 608.2c / 603.2c | 260 | the bound object's LKI, or the matched record; the resolution's own events (603.12) | engine: `AmountExpr::TriggeringAmount` (TR-1) for the record's; **neither** for the resolution's own ("discard any number, then draw that many") |
| "if you do": the resolution's own choice (optional) or its own action (mandatory) -- §6.2 describes the optional form only | 603.5 / 118.12 | 1,019 | the resolution's own choices (118.12) | design: `last_optional_taken` (§6.2, TR-2) for the optional form; ATOM-118.12-001 (TR-2); ATOM-118.12-002, the outcome altered, is Phase 8's |
| "if you don't", "if you can't": the choice declined, or an instruction that could not be followed | 118.12 / 101.3 | 105 | the resolution's own choices (118.12); the resolution's own events (603.12) | design: the same field, negated, for "don't"; **neither** for "can't" |
| "if they do", "if a player does": another player's choice | 101.4 / 608.2d | 53 | the resolution's own choices (118.12) | design: §6.2's `OptionalEffect` asks one player; "unless [a player] pays" is CP-1's; **neither** for "any player may ... if a player does" |
| "when you do": reflexive, against the resolution's own events | 603.12 | 258 | the resolution's own events (603.12) | design: §4.6 (TR-3), whose window is "since `created.record`" where CR 603.12 says "earlier during the resolution" |
| "when [something happens] this way": reflexive, the second form -- the one regex row: a substring cannot tie "when" to "this way" in one sentence | 603.12 | 55 | the resolution's own events (603.12) | design: §4.6 (TR-3) |
| "when you pay ... one or more times" | 603.12a | 7 | the resolution's own events (603.12) | design: §4.6, `OncePerEvent` over the reflexive window; the payment loop is CP-1's |
| "this way": a count or a fact of the resolution's own events | 608.2c / 603.12 | 480 | the resolution's own events (603.12) | engine: every record carries `EventStamp.resolution`; **neither** reads it outside §4.6's reflexive window |
| "for each": a count at resolution -- the largest mixed row | 608.2c / 608.2h | 785 | the board now; a history (§3.10); the resolution's own events (603.12); the spell it was (400.7d) | engine: the board's counts; the rest by the three rows below |
| ... "for each ... this way" | 608.2c | 122 | the resolution's own events (603.12) | **neither**, as "this way" above |
| ... "for each ... this turn" | 608.2c / 603.1b | 106 | a history (§3.10) | design: `PlayerHistory` (§3.10, TR-2) |
| ... "for each color of mana spent to cast it" | 400.7d / 702.44a | 5 | the spell it was (400.7d) | **neither** (item 30) |
| a delayed trigger's "it", "them", "that token": the objects the creating resolution made or moved | 603.7c / 707.10e | 144 | the resolution's own events (603.12) | design: `DelayedTrigger.refs` (§3.9, TR-3), filled by the producer -- from the instruction or from the records it performed, unstated |
| a token-creation trigger under a doubler: "twice that many" -- Ajani, Nacatl Avenger's ruling: under Doubling Season its reflexive ability triggers once per Cat Warrior made | 614.1a / 603.12a | 26 | the resolution's own events (603.12) | design: `CreatesToken` (§3.3, TR-5), whose `kind` is the token's type, not instructed or added |
| a token-creation trigger under a substitute: "would be created" -- Ajani, Nacatl Avenger's ruling: under Divine Visitation the Angels made instead do not trigger it | 614.1a / 614.6 | 18 | the resolution's own events (603.12) | design: `CreatesToken` (§3.3, TR-5); nothing tells an instructed token from a substitute |

Queries, in row order (each plus `game:paper -is:funny`, `unique=cards`):

1. `(o:"when " or o:"whenever " or o:"at the beginning of") o:", if "`
2. `(o:"when " or o:"whenever " or o:"at the beginning of") (o:", if you control" or o:", if you have" or o:", if an opponent" or o:", if there are" or o:", if that player")`
3. `(o:"when " or o:"whenever " or o:"at the beginning of") o:", if " (o:"this turn" or o:"last turn")`
4. `(o:"when " or o:"whenever " or o:"at the beginning of") o:"was spent to cast"`
5. `(o:"when " or o:"whenever " or o:"at the beginning of") (o:"was kicked" or o:"was bargained")`
6. `(o:"when " or o:"whenever " or o:"at the beginning of") o:"cost was paid"`
7. `(o:"when " or o:"whenever " or o:"at the beginning of") (o:"if you cast it" or o:"if it was cast" or o:"if it wasn't cast" or o:"if you didn't cast it")`
8. `(o:"when " or o:"whenever " or o:"at the beginning of") o:"was spent to cast" kw:evoke`
9. `(o:"when " or o:"whenever ") (o:", if it had" or o:", if that creature had" or o:", if it was a" or o:", if it wasn't a" or o:", if that creature was")`
10. `(kw:persist or kw:undying)`
11. `(o:"when " or o:"whenever ") (o:"dies" or o:"leaves the battlefield") o:", if "`
12. `o:"if this card is in your graveyard" (o:whenever or o:"at the beginning" or o:"when ")`
13. `(o:"when " or o:"whenever " or o:"at the beginning of") o:"tribute wasn't paid"`
14. `m:X (o:"when " or o:"whenever ") o:"enters" -o:"enters with X"`
15. `(o:"when " or o:"whenever " or o:"at the beginning of") o:"that many"`
16. `(o:"when " or o:"whenever " or o:"at the beginning of") o:"if you do"`
17. `(o:"when " or o:"whenever " or o:"at the beginning of") (o:"if you don't" or o:"if you can't")`
18. `(o:"when " or o:"whenever " or o:"at the beginning of") (o:"if they do" or o:"if that player does" or o:"if a player does" or o:"if no one does" or o:"if they don't" or o:"if that player doesn't")`
19. `o:"when you do"`
20. `o:/\bwhen [^.]*this way/`
21. `o:"one or more times"`
22. `(o:"when " or o:"whenever " or o:"at the beginning of") o:"this way"`
23. `(o:"when " or o:"whenever " or o:"at the beginning of") o:"for each"`
24. `(o:"when " or o:"whenever " or o:"at the beginning of") o:"for each" o:"this way"`
25. `(o:"when " or o:"whenever " or o:"at the beginning of") o:"for each" o:"this turn"`
26. `(o:"when " or o:"whenever " or o:"at the beginning of") (o:"colors of mana spent" or o:"color of mana spent")`
27. `(o:"it at the beginning of the next" or o:"them at the beginning of the next" or o:"token at the beginning of the next" or o:"tokens at the beginning of the next")`
28. `o:"twice that many"`
29. `o:"would be created"`
<!-- trigger-survey: end TABLE-3 -->

**Reading it.** The rows overlap by design: the first is the whole
intervening-"if" population, the next twelve split it by place, and a card
with two clauses is in two rows. The sizes quoted when this table was
commissioned (14,880 triggered cards, 1,399 intervening "if"s) were counted
with no `game:paper -is:funny`; five of them reproduce exactly that way, and
this table keeps the survey's filter so its rows are shares of table one's.
Every count in tables one and two was the same on 2026-09-23 as on
2026-09-18.

---

## 5. The gaps — filed as `codebase-state.md` "Before Triggered abilities" items 10–18

Each is a row in table two and an item there, with the item's reachability
line and size. None is built here (decision 2); `triggers-architecture.md`
§3.12 decides each and schedules it, and TR-1 closed items 10 and 18. The reachability verdict is
the same for all nine — **unreachable**: no dispatcher reads any event, so no
fuzz game can produce a wrong answer from a missing field today — and it
stops being so the day the matcher lands, which is why each is filed before
the doc rather than after it.

- **Item 10 — three performers drop a proposal field the record needs.**
  `GameAction::BeginStep` and `BeginPhase` carry `player` and `StepBegin` /
  `PhaseBegin` do not (the upkeep, end-step, beginning-of-combat and delayed
  rows: "your upkeep", "each opponent's upkeep"); `DealDamage` carries
  `is_combat` and `DamageDealt` does not (the "deals combat damage" row);
  `LoseLife` carries `LifeLossCause` and `LifeChanged` does not (CR 727.1a's
  "from radiation", one card). Live-derivable at dispatch for the first —
  the active player owns every step — and not for the second: a triggered
  ability resolving during the combat damage step deals noncombat damage in
  that step, so the step does not say. Sized: three fields, three performers,
  `format_event`'s three arms, and the literal sites that name the shapes
  (`StepBegin {` at 5 in `src` and 12 in `tests`, `DamageDealt {` 10 and 7,
  `LifeChanged {` 10 and 9), ~40 lines; the shape of each field is the doc's.
- **Item 11 — `AttackersDeclared` carries no defender.** CR 508.3a's "attacks
  [a player]", 508.3b's "is attacked" and 508.3e's "attacks another player"
  read whom each creature attacks; the record is the attacker list, and the
  defender is on `AttackingInfo.target` — live, but a record that cannot say
  whom an attack was declared against is a trace that cannot answer the
  question A4c built it for. Sized: `Vec<(ObjectId, AttackTarget)>` at the one
  site, ~10 lines.
- **Item 12 — no event announces a target being chosen.** CR 601.2c chooses
  targets, 601.2i says abilities that trigger on the cast trigger then, and
  nothing in between emits: "becomes the target" is 117 cards and ward
  (CR 702.21a, "whenever this permanent becomes the target of a spell or
  ability an opponent controls") is 195 more, the largest population with no
  record at all. Sized: a variant carrying the targeting object, its
  controller and the target, emitted where the cast and the activation
  announce themselves and where CR 603.3d puts a trigger on the stack, ~30
  lines; whether one spell targeting one permanent twice is one event or two
  (CR 115.9a counts instances) is question 11.
- **Item 13 — no event announces a control change.** CR 603.10d's two
  look-back triggers (126 cards) watch an event the engine performs as a
  Layer 2 registry row: `Primitive::GainControl` writes the row and
  `get_effective_controller` answers differently from then on, and a row
  expiring at cleanup changes control back with no proposal anywhere. This
  is the one gap that is not a field: control is a *computed* value, and
  "gains control" is a "becomes" on the layer walk's output, the shape
  CR 603.2e gives tapping. Sized: unknown until the doc says whether a
  change in a computed value is detected at the registry write (and its
  expiry), by the walk, or as a state trigger's cousin; the doc's.
- **Item 14 — the LKI frame carries characteristics and no status.**
  `EffectiveCharacteristics` is the frame, and it has no counters, no
  attachment link and no tapped bit. Persist (CR 702.79a) and undying
  (702.93a) are dies-triggers with an intervening-if on the counters the
  permanent *had*; an Aura's CR 603.6e trigger reads what it enchanted; the
  four "becomes unattached" Equipment (603.10c) read the host they left — 109
  cards on the row's query. Sized: three fields copied from `PermanentState`
  at the capture site in `perform_zone_change` and its `LeftTheGame` twin,
  ~15 lines; whether a frame typed as *characteristics* should carry status
  at all is the doc's, and CR 603.10's word is "appearance".
- **Item 15 — the frame is captured only for a battlefield departure.**
  CR 603.10a names three look-back classes and the engine captures one: a
  card leaving a graveyard (38 cards) and a visible object put into a hand or
  library (8) get `lki: None`. It matters because a continuous effect's filter
  can reach a graveyard (`ZoneSet`, `layers-architecture.md` §13c) — under
  Yixlid Jailer, registered since LJ, a "when this card leaves your
  graveyard" ability must not trigger, and the only way to know is the frame
  from before the move. Sized: the capture condition widened from
  `from == Battlefield` to the three classes, ~10 lines, once the doc says
  which zones the frame is computed for.
- **Item 16 — no event for a prevention effect applying.** CR 615.13:
  "such an ability triggers each time a prevention effect is applied to one
  or more simultaneous damage events" — 15 cards, Selfless Squire the plain
  one. The pipeline knows (A4c's `pipeline` record is written at exactly that
  iteration) and the stream does not. Sized: an event emitted by the shield
  path RD-2 built, one per prevention applied, ~10 lines; its fields are the
  doc's.
- **Item 17 — counters a permanent enters with announce nothing, and the
  entry record carries no `mods`.** CR 122.6: "counters being put on an
  object … refers to putting counters on that object while it's on the
  battlefield and also to an object that's given counters as it enters" —
  so "whenever one or more +1/+1 counters are put on a creature you
  control" (the counters row, 72 cards) triggers for a creature entering with
  them, and today's stream has no `CountersChanged` for the entry (by design:
  CR 614.16's doublers replace the `EnterMods`, so the replacement side needs
  no event) and no `mods` on `PermanentEnteredBattlefield`. Live-derivable at
  the entry's dispatch — every counter on the permanent then is an entry
  counter — and not from the record. Sized: the counter rows on the entry
  event or a `CountersChanged` per row after it, ~10 lines; which, is
  question 4.
- **Item 18 — three `GameEvent` variants are never emitted.** `PhaseEnd`,
  `StepEnd`, `TurnEnd`, declared with the enum and written by no performer.
  No trigger reads an end (§2), so the choice is emit or delete, and the
  recommendation is delete: an arm the stream cannot carry misleads the
  reader the way `replacement-architecture.md` §3.2a says a pattern arm the
  pipeline cannot apply does.
  Sized: three variants and their `format_event` arms, ~15 lines.

**What is not a gap, and why it is listed.** The turn-history row (237 cards)
and 603.7h's count are item 42's trackers and wait for the doc by that item's
own text; the state-trigger row is no event by definition (P1); the keyword-
action row and phasing and searching are Phase 8's, one event per keyword as
the keyword lands. Each is in table two so the dispatcher's design knows the
kinds of thing it will be asked to take later.

---

## 6. Corner cases the CR names — questions for the doc, not answered here

The table surfaced these; each is a decision the doc owns, and this file
records the rule and the row rather than an answer.

1. **CR 513.2 and "next".** A permanent with "at the beginning of the end
   step" entering during the end step, or a delayed trigger "at the beginning
   of the next end step" created during it, waits for the *next turn's* — the
   step does not back up. The delayed row carries `StepBegin` and nothing on
   the record says when the ability was created; that instant is the delayed
   ability's own field, and the doc names it.
2. **CR 608.2b and `SpellFizzled`.** Under the baseline CR a spell whose
   targets are all illegal "doesn't resolve. It's removed from the stack" — the
   text does not say countered, so "whenever a spell is countered" (603.10e,
   19 cards) reads `SpellCountered` and `AbilityCountered` and not
   `SpellFizzled`. The doc should say so in the matcher's terms, because
   "countered on resolution" is still the phrase players use for it, and a
   reader who learned it that way will expect the third event to count.
3. **Life loss per source or per batch.** `LifeChanged` is one record per
   `LoseLife` proposal, and CR 120.3a makes each source's damage its own loss,
   so two attackers' combat damage is two records in one batch; CR 702.15e
   settles the *gain* side the same way ("separate life gain events"). The
   doc decides whether "whenever you lose life" (51 cards) is per record or
   per batch, and reads the printed rulings before it does — this is the
   shape `engineering-practices.md` §8 warns about, a rule on one side and
   the ruling on the other.
4. **CR 122.6's entry counters — which record.** Item 17's shape: counter
   rows on the entry event, or a `CountersChanged` per row announced after
   the entry, inside its batch. The second reads the same to a "counters are
   put on" matcher and differently to a "the Nth counter" one (CR 122.7 reads
   *before* and *after*), and to the replacement side, which must not see it
   as a second event CR 614.16 could double.
5. **"Becomes unattached" has three routes and no one record.** An
   attachment moved to another host (`Attached.former_host`), one dropped by
   CR 704.5n (`EquipmentDetached`), and one whose host or self left the
   battlefield (the zone change, with the frame item 14 would widen). CR
   603.10c makes the trigger a look-back; the doc says which routes are the
   event and whether an Aura put into the graveyard by CR 704.5m "becomes
   unattached" on its way.
6. **Triggered abilities with no source.** The monarch (CR 724.2), the
   initiative (725.2) and rad counters (727.1) are "inherent triggered
   abilities" that "have no source", an explicit exception to CR 113.8 —
   113 cards mention them and Phase 7 carries six CR 724 atoms and three CR
   725. `AbilityIdentity` is a `(source, ability)` pair; the doc decides the
   arm, beside S3's per-instance question.
7. **When a state trigger is re-checked.** CR 603.8's condition can become
   true with no performed event: an "until end of turn" effect ends at
   cleanup (CR 514.2, a turn-based action), a registry row expires, a layer
   epoch bump changes what is a creature. The resolved design checks "at
   every dispatch"; the doc says what a dispatch is when nothing was
   performed, and CR 702.131d (continuous effects reapplied before the
   check) is the rule that watches it.
8. **CR 603.6a and the batch boundary.** "Each time an event puts one or
   more permanents onto the battlefield, all permanents on the battlefield
   (including the newcomers) are checked" — the *event* is the batch, so two
   permanents entering together each see the other's static abilities
   (603.6b) before either is checked. The entry record is emitted per
   permanent after `register_static_effects`; whether the matcher runs per
   record or once per batch against the settled board is the doc's, and RC-5's
   "applying an entry can move the board" is the finding to reread first.
9. **`PermanentEnteredBattlefield` has no `from`.** "Enters from a
   graveyard" reads the zone change announced just before, in the same batch;
   a token has no zone change and a `TokenCreated` instead. The doc decides
   whether the matcher joins the two records or the entry carries the zone.
10. **CR 707.10 and 707.12 — which copies are cast.** Two shapes, and
    `copy-effects-architecture.md` §4.4 already holds them as tiers D and E.
    A copy of a *spell* is put onto the stack and "isn't cast" (707.10, tier
    D): no `SpellCast`, so "whenever you cast" stays quiet and storm's copies
    do not storm again. A copy of a *card* that an effect lets a player cast
    (707.12, tier E — Isochron Scepter, Mizzix's Mastery, the Elite Arcanist
    of Panharmonicon's ruling) "follows the rules for casting spells": the
    copy is created in the card's zone and cast through CR 601.2a–h while
    another spell or ability is resolving (CR 117.2a), so `SpellCast` fires
    for it and every cast trigger sees it. `SpellCast` has one emitter today,
    inside `cast_spell`, and neither tier is built; the doc says which door
    each tier takes, and the copy track builds it so.
11. **One spell, one permanent, two instances of "target".** CR 115.9a counts
    instances separately for "with N targets"; whether "becomes the target"
    and ward trigger once or per instance is a rulings question for item 12's
    event, and CR 603.2c ("once each time its trigger event occurs … repeatedly
    if one event contains multiple occurrences") is the rule it turns on.
12. **CR 605.1b — a triggered mana ability skips the stack.** "Whenever you
    tap a permanent for mana, add an additional …" (the tapped-for-mana row)
    resolves "immediately after the mana ability that triggered it, without
    waiting for priority" (605.4a). Main item 11; the dispatcher's pending
    queue has one class of member that never waits.
13. **CR 603.6c's "from anywhere" must not look back.** The row is a
    zone-change trigger with no frame, by rule — a "put into a graveyard from
    anywhere" ability on a card whose abilities Layer 6 removed in the
    graveyard reads the board *after* the event. The doc states it as the
    matcher's rule, because it is the one case where a `ZoneChange` with a
    frame must be matched without one.
14. **Delayed triggers' identity and provenance (603.7c–g).** 603.7c tracks
    an object rather than a filter and loses it at a zone change (CR 400.7);
    603.7d–e name the source and controller a spell or an ability hands down,
    where the spell's stack object is gone by the time the trigger fires
    (CR 608.2n); 603.7f–g name what a replacement effect and a static
    ability's special action hand down "at the time", and neither producer
    has a resolution stamp to inherit. The doc decides who registers a delayed
    trigger, what it records about its birth, and what its object reference
    is. Each of the five has its atom in the corpus (table one).
15. **Game-scoped lookback and the two-turn window.** Item 42 bounds the
    in-state *event* window to each player's current and previous turn and
    materializes what the rules read as per-player counters; a "this game"
    quantity is a *scope* on a counter, not a window on the log — Approach of
    the Second Sun's casts, or Commander's Insight's commander casts from the
    command zone, which is CR 903.8's own tax counter. What the doc decides
    is how a new game-scoped quantity gets its counter. Three shapes are on
    the table. A tracker field per quantity, bumped at the chokepoint — item
    42's design, one field and one update arm per card family. The per-turn
    summaries kept for the whole game rather than for two turns: they are a
    few dozen counters per player per turn, so a hundred-turn four-seat game
    holds kilobytes, "this game" becomes a fold over them and "since the
    beginning of your last turn" a range — for the quantities the summary
    already carries. And a card-registered counter: a tracker the ability
    declares the way a trigger declares its event (a filter, a scope, a key),
    fed by the same matcher, for a quantity no summary anticipated. The sink
    stays outside the engine under all three. **A fourth, the owner's
    (2026-09-18), is a decider over the three rather than a mechanism**: read
    every card that can enter the game before it starts and switch on only
    the counters those cards read — the "pregame sweep" that
    `state-tracking-architecture.md` already records with its failure mode
    (conjure, wishes, Momir-shaped formats: fall back to track everything).
    What it fixes is the third shape's real weakness: a counter registered
    when its card first appears has no past, and a "this game" quantity
    counts from turn one. And its holes close if the sweep is taken over the
    **registry** rather than the decklists: every card this engine can ever
    play is a registered card, conjured and wished ones included, and a
    custom card joins the registry at load, before any game — so the union
    of counters any registered card reads is a static property of the
    build, the decklist sweep is an optimization over it that pays only if
    measured, and Momir is just a deck that draws from the whole registry.
16. **Two once-per-turn gates, not one.** CR 603.2h's "Do this only once
    each turn" is an *action-taken* gate: the ability keeps triggering until
    the action is taken, only one of several instances on the stack acts, and
    Panharmonicon can double it (Nykthos Paragon's rulings). "This ability
    triggers only once each turn" — 128 cards and no rule of its own in the
    baseline CR — is a *triggered* gate. Its definition is the earliest
    printing's ruling, Elvish Warmaster's: "Once the triggered ability has
    triggered once during a turn, it can't trigger again, even if the
    triggered ability is still on the stack, has been countered, or has
    otherwise left the stack" — set as the ability triggers, once per source
    object, once for a batch, and not doubled (Jin-Gitaxias, Tyvar, Fang,
    Stonebinder's Familiar; the judge literature). The first is written
    by the resolution and read at trigger time and again at resolution; the
    second is written by the dispatcher and read before CR 603.2d's
    multiplier applies. The doc names both trackers and where each is written.

---

## 7. The boundary check

Scryfall's regex form (`o:/.../`) honored `\b` on 2026-09-18. The pairs below
are the rows where a word boundary could change the count; the substring
column is what the tables use.

<!-- trigger-survey: begin BOUNDARY -->
| pair | substring query | cards | cards, regex reading |
|---:|---|---:|---:|
| 1 | `o:"dies"` | 1,246 | 1,245 |
| 2 | `o:"when "` | 6,181 | 6,181 |
| 3 | `(o:"when " or o:"whenever " or o:"at the beginning of")` | 14,149 | 14,149 |
| 4 | `(o:"when " or o:"whenever ") o:"enters"` | 5,701 | 5,461 |
| 5 | `(o:"when " or o:"whenever ") o:"attacks"` | 1,695 | 1,611 |
| 6 | `(o:"when " or o:"whenever " or o:"at the beginning of") o:", if "` | 1,338 | 1,329 |
| 7 | `o:"whenever" o:"one or more"` | 443 | 454 |

The regex readings, by pair (each plus `game:paper -is:funny`, `unique=cards`):

1. `o:/\bdies\b/`
2. `o:/\bwhen\b/`
3. `o:/\b(when|whenever|at the beginning of)\b/`
4. `o:/\b(when|whenever)\b[^.]*\benters\b/`
5. `o:/\bwhenever\b[^.]*\battacks\b/`
6. `o:/(when|whenever|at the beginning of)[^.]*, if /`
7. `o:/\b(when|whenever)\b[^.]*one or more/`
<!-- trigger-survey: end BOUNDARY -->

Two things to read off it. `o:"when "` and `\bwhen\b` agree exactly: the
trailing space already excludes "whenever", so the base row's over-count is
zero on that axis. And the regex forms that anchor the trigger word to the
phrase in one sentence (`[^.]*`) differ from their substring pairs by a few
percent either way — below where the substring lets the trigger word and the
phrase sit in different sentences (enters, attacks), above where the
substring row asked for "whenever" and the regex admits "when" (one or more).
That spread is the over-count decision 1 accepted, and it is small.

---

## 8. Regenerating

```bash
python plans/references/trigger-survey.py --write
```

Fetches every query not in the gitignored cache (`.census-triggers.json`,
~80 requests at a courteous rate), rebuilds `spec.sqlite` if it is missing,
parses the enums, asserts the field claims, and rewrites the seven marked
blocks in place; the prose is untouched. `--refresh` drops the cache first,
`--boundary` prints §7 alone, `--names Q` prints the first page of names
behind a query (how the small buckets in table two were read), `--no-fetch`
runs from the cache and fails on a query it lacks. The script's docstring
says when to delete both files.
