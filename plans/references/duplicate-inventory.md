# One fact computed in more than one place — the engine's first inventory

> **Status:** run 2026-10-10 against `main` at 45a1d0e (#239's merge), for
> `roadmap-v2.md` row A6l, which this closes (#PRNUM). **Authority:** the
> entries are `python plans/similar_functions.py`'s at that commit; the
> verdicts are this file's. Each merge's owner is a `codebase-state.md` item
> (§3), and on what exists, `codebase-state.md` wins. **Re-run:** the report
> takes about 6 seconds, and `--against main` about 13.

## 0. The question, and how the report was read

At #239's review the owner found one rule, "a trigger on its subject's
departure names the existence that left", spelled three times, and one fact,
"the damage source as it deals damage", split across two files.
`similar_functions.py` had seen neither: each copy was under its 60-token
floor and written differently. A6l widens the instrument (its docstring says
how, and what it still cannot see) and reads its first report over the whole
engine.

The report over `main` has five sections and 140 entries:

| section | what it keys | entries |
|---|---|---:|
| near-copies, names kept | token 5-grams, functions of 60 tokens or more, cards included | 24 |
| one shape, names differ | the same, identifiers masked | 6 |
| same inputs | parameter types, two or more | 32 |
| same composition | two reads in a row, one a call, in three files or more | 48 |
| same test | an equality test, cut to its last names | 30 |

Every entry has one verdict: **merge**, with the group it lands in (§3);
**keep**, with the reason the copies differ; or **false positive**, the key
shared by chance. The tally is 69 merges, 26 keeps and 45 false positives
(§6 lists every entry). The near-copy keeps are card definitions, which
resemble each other by design; every card-file merge is a helper, not a card.

**How the verdicts were reached.** Two subagents read the three new sections
blind, in a detached worktree at 45a1d0e with `plans/` removed: one the 48
compositions, one the 32 inputs and 30 tests. The 30 token pairs were read
by hand. Every claim that two copies *already disagree* was re-read against the tree
before it was recorded (§3.5); the subagents' verdicts were taken where the
re-reading agreed, and §6 says at the one entry where it did not.

## 1. Calibration, blind

Each positive was found by a subagent in a detached worktree at its commit,
told only to disposition the report and not what to find. The worktrees had
`plans/` removed, and their copy of the script had the docstring's history
and its three examples replaced with neutral ones, since those named the
answers. Each subagent read only its worktree and ran no git command but the
script. For the two pre-review commits the report was the per-PR one, `--against`
the PR's base, which is how a review meets it. Two slips, neither near an
answer: the compositions run wrote its report to a file beside the worktree
and deleted it, and the inputs run read the script's constants past the
docstring.

| positive | commit | report | where it lands | the blind verdict |
|---|---|---|---|---|
| one rule spelled three times: `departed_source_frame` (resolve), `as_it_left` (legality), `subject_left_as`'s derivation (stack) | 25df5d4, #239 before review | `--against 50cd3b3`, 20 entries | composition `departure_frame() + subject` (`departed_source_frame`, `resolve_top_of_stack`, `objects_referred_to`, `bound_characteristics`); test `Some(_) == subject` (`departed_source_frame`, legality's `as_it_left`) | merge: "an object a stack entry names is read as it left", all three spellings and two more |
| one fact split across two files: `dealer` (replacement/gather), `damage_source_characteristics` (oracle) | 25df5d4 | the same | same inputs `GameState, ObjectId, Option<DepartedFrame>` | merge: "the identity half and the characteristics half of one rule", five more places |
| `mana_supply`'s private copy of `permanents_controlled_by` | 2be5a93^ | `--against a1815e3`, #225's base, 23 entries | same inputs `GameState, PlayerId -> Vec<ObjectId>`; composition `battlefield_ids_ordered() + controls()` | merge: "`permanents_of` computes the same thing as `permanents_controlled_by`" |
| the same list built inline at `costs.rs`, `resolve.rs`, `turns.rs` and `ui/why.rs` | 45a1d0e, `main` | the full report | composition 3, `battlefield_ids_ordered() + controls()`, all five | merge, `permanents_controlled_by` the helper |
| `object_matches_filter` and `_for_instance`, which must stay found | 50cd3b3 | the full report | near-copy 2 | not blind: the token scan's algorithm is unchanged, and its two sections read the same 30 pairs on `main` as before |

In the full report at each commit, not blind, the positives rank composition
5 of 49 and test 5 of 30 (the rule), inputs 29 of 32 (the damage source),
inputs 27 of 30 (`permanents_of`) and composition 3 of 48 (the loops). No
section cuts below its rules, so a rank is a reading order, not a threshold.

The two old commits' blind runs found more than the positives. At 2be5a93^,
seven more merges in #225's `mana_supply.rs`, which predicts what the
engine's mana and trigger paths will do by restating them; at 25df5d4, ten
more. Those still on `main` are §3's.

## 2. What it misses — the second opinion

A model pass, not shown the report, read the trigger, stack, resolution,
zone and legality code (18 files, about 14,000 lines) for one fact computed
in more than one place: 24 findings and 6 minor ones. Judged by hand (a
listing counts when the entry's key *is* the fact, not when two of its
functions share some other key):

| | listed by default | with `--all` | not listed |
|---|---:|---:|---:|
| the 30 findings | 11, two more in part | 18, one more in part | 11 |

The misses are of three kinds the docstring names: a copy inside one function
(the `EachOf` players, twice in `resolve_primitive`); a fact that is one
expression (CR 603.7a's `existed_before`); and one fact reached through
different fields (the frame a record carries, read three ways). The new
sections key every function however short, so what limits them is the
regex's reach, not a copy's length. `--all` lists compositions in two files
as well as three, at 143 entries against 48; the per-PR report uses it, since
a branch's own copies are few.

| # | the fact | listed | lands in |
|---:|---|---|---|
| 1 | which permanents a resolution's effect applies to (CR 611.2c) | no | M3 |
| 2 | the row a resolution's continuous effect writes | `--all` | M3 |
| 3 | a triggered ability's arm, limit and "if", in that order | yes | T1 |
| 4 | CR 603.4's intervening "if" | yes | T1 |
| 5 | queueing a pending trigger | yes | T2 |
| 6 | CR 603.2c's "one or more" fold | no | T3 |
| 7 | when a registry entry may read a record (CR 603.7a) | no | T4 |
| 8 | what a resolution put into a zone since a mark | `--all` | L6 |
| 9 | CR 603.10a's handoff before an object leaves | `--all` | L5 |
| 10 | which permanents a decided `PlayerLoses` takes | yes | L7 |
| 11 | who an opponent is (CR 102.1) | yes | M1, M4 |
| 12 | a spell on the stack, counting the resolving one | in part | M1 |
| 13 | the card a departed frame's candidate is built from | no | T6 |
| 14 | an ability leaving the stack ceases to exist | yes | M3 |
| 15 | CR 603.2h's (identity, controller) key | no | T5 |
| 16 | this reference is still the object (CR 400.7) | yes | L2 |
| 17 | the object an entry's text calls "this" | no | M3 |
| 18 | whose control a returning object comes back under | `--all` | M3 |
| 19 | no entry under a departed player's control (CR 800.4b, d) | yes | M1 |
| 20 | the players still in the game, in APNAP order | yes | M1 |
| 21 | the players an `EachOf` slice names | no | M3 |
| 22 | a permanent's host (CR 303.4m) | in part; `--all` | M1 |
| 23 | the CR 603.10a frame a record carries | no | L3 |
| 24 | life gained or lost from a `LifeChanged` record | no | T9 |
| C1 | "the same ability, whichever instance" (CR 603.7h) | `--all` | T9 |
| C2 | which record kind each arm reads | yes | T7 |
| C3 | the blocker's half of CR 509.1a | yes | M1 |
| C4 | the top N cards of a library | `--all` | M3 |
| C5 | the resolving object for a context | no | M3 |
| C6 | whether an ability is triggered | no | T9 |

**Whether to repeat it is the owner's call.** The brief recommended the model
pass once, as a check on the inventory, and not as the recurring instrument,
since it is not repeatable. On its surface it found 11 facts the report does
not list at all, three of them copies that already disagree (§3.5's affected
set, "existed before" and the departed card). Running it on the subsystem a spine phase closes, at that close, is the
option if the measure is wanted again.

## 3. The merges, by where they land

A letter and number names a fact: L for the last-known-information and
zone-change surface (items 176 and 177's), T for the trigger code (A6e's),
M for the merges outside both, a PR each, and C0 for the card-authoring
constructors. Where a place is named by function, the function holds the copy.

### 3.1 Inputs to items 176 and 177's design (`codebase-state.md`, A6d)

The design settles the zone-change record's contract and, with it, what an
object "as it left" is. Seven facts on that surface are read in more than one
place today, so the design names each one's single home.

- **L1. The damage source as it deals damage (CR 608.2h, 609.7a): an object's
  last known information when it left, else the object as it is.** Five
  spellings: `damage_source_existence` and `damage_source_characteristics`
  (`oracle/characteristics.rs`, the pair #239's review put side by side),
  inline in `pattern_watches`' filter (`replacement/gather.rs`, the frame or
  the live object), `retarget_destination`'s `ToDamageSourceController`
  (`replacement/pipeline.rs`, the frame's controller or the live one), and
  the display's `as_it_left` and `format_event` (`ui/display.rs`, the label).
  One value with four projections: identity, characteristics, controller, a
  filter's answer.
- **L2. This reference is still the object (CR 400.7).** `RememberedObject::found`
  and `is` (`types/triggers.rs`), `TargetRef::still_exists`
  (`engine/targeting.rs`), `bound_object` (`triggers/binding.rs`),
  `this_object` (`engine/resolve.rs`), `find_matches` (`triggers/dispatch.rs`),
  `matches_without_shortcuts` (`triggers/audit.rs`), `returns.rs:177`,
  `ui/ask.rs:1314` and `scenario/write.rs`'s `write_this_turn_counts`: an
  epoch comparison or `object_ref(id) == Some(r)`, ten places. Item 177 puts
  the epoch a move stamped on the record, which is the same identity.
- **L3. The CR 603.10a frame a record carries** (a `ZoneChange`'s or a
  `LeftTheGame`'s `lki`): `frame_of` and `find_matches`' second leg
  (`triggers/dispatch.rs`) and `departure_frame` (`triggers/binding.rs`,
  which adds the subject check). A method on the record.
- **L4. Taking a mover's frame:** `take_departure_frame` opens with
  `take_named_frame`'s body (`triggers/dispatch.rs`), and `capture_frame`
  repeats the lookup.
- **L5. The handoff before an object leaves (CR 603.10a):** take its frame if
  it leaves the battlefield, `hand_over_departed_frame`, then
  `note_delayed_source_moving` — in `perform_zone_change`
  (`engine/actions.rs`) and `owned_objects_leave` (`engine/leaving.rs`), the
  same three steps.
- **L6. What a resolution put into a zone since a mark:** `exiled_since`
  (`engine/returns.rs`, CR 610.3's returns) and `remembered_since`
  (`engine/resolve.rs`, CR 603.7c's "that card"), each filtering the
  resolution's records, checking the object is still there and removing
  duplicates. `remembered_since` also counts a token created there, which
  "that card" may name and a return never does.
- **L7. Which permanents a decided `PlayerLoses` takes off the battlefield:**
  `departs_an_ability_list_source` asks of the sources the player owns,
  `capture_departure_frames` frames every permanent, since CR 800.4a's last
  clause exiles the ones they control too (`triggers/dispatch.rs`). The
  nested exile's own capture covers part of the gap; the design reads whether
  a controlled source the player does not own is looked back at.

And **the rename (§4)**, which the design's text would otherwise be written
in the vocabulary it replaces.

### 3.2 The trigger code — slotted to A6e by name

Item 6's close audit (`roadmap-v2.md` A6e) takes these with its hygiene pass,
`codebase-state.md` item 247.

- **T1. A triggered ability's tail: the first matching arm, CR 603.2h's
  limits, then CR 603.4's intervening "if".** `match_def`
  (`triggers/dispatch.rs`) and `match_delayed` (`triggers/delayed.rs`) end
  the same way; the "if" alone is asked again in `when_not_done`
  (`delayed.rs`), in `resolve_taken` (`engine/stack.rs`, CR 603.4's second
  check, at resolution), and in `mana_supply::mana_trigger_adds` (M2).
- **T2. Queueing a pending trigger:** `queue_matches` (`dispatch.rs`) and
  `queue_pending` (`delayed.rs`) each apply "triggers only once each turn",
  number the trigger and push it; the `AbilityTriggered` emit loop and the
  match-to-pending conversion (`pending_of`) are written twice too. They
  differ on `caused_by` for an empty match, which no match is.
- **T3. CR 603.2c's "one or more" fold:** `match_candidates` (`dispatch.rs`)
  and `add_occurrences` (`delayed.rs`); `when_not_done` builds a
  `MatchedTrigger` by hand.
- **T4. When a registry entry may read a record (CR 603.7a):**
  `existed_before` (`delayed.rs`) excludes the batch the entry was made in,
  `is_due` (`delayed.rs`, CR 610.3's returns) does not, and `UntilReturn`
  has no `created_in`. §3.5.
- **T5. CR 603.2h's (identity, controller) key:** `within_once_per_turn_limit`
  (`dispatch.rs`), `resolve_taken`'s action gate (`stack.rs`) and the
  placement's elision (`triggers/placement.rs`).
- **T6. The `TriggerCandidate` for an object:** `find_matches` builds it, and
  `matches_without_shortcuts` (`triggers/audit.rs`) writes the literal again,
  twice. For an object that left the game the audit's card is its name only,
  where the dispatcher's (`frame_card`) carries the frame's abilities; the
  audit's `describe` compares no card, so no verdict differs. The audit is
  the dispatcher's matcher run over every object (TR-1b), so it should
  differ only in which objects it visits. The `.unwrap_or(object.owner)` after
  `controller_or_owner` never fires (here and at `replacement/gather.rs:362`).
- **T7. Which arms carry a multiplicity, and which record kind each arm
  reads.** `CountableEvent::once_per_event` (`cards/authoring/triggers.rs`)
  lists the first in `TriggerEvent::multiplicity`'s groups and its doc says
  a new arm joins both; the second is written five times (`record_kinds`,
  `subject_of`, `player_of`, `amount_of`, `occurrences_matching_arm`),
  exhaustive on the arm side since item 231 and not on the record's.
- **T8. Whether a delayed trigger's extra turn is still coming:**
  `drop_delayed_triggers_of_gone_turns` (`delayed.rs`) and the waiting panel's
  `waiting_trigger` (`ui/waiting.rs`), which also asks that the turn's player
  is still in the game. The panel should read the engine's answer.
- **T9. Small:** life gained or lost from a `LifeChanged` record, three times
  (`dispatch.rs`, `TriggerEvent::amount_of`, `triggers/history.rs`); "the same
  ability, whichever instance" (CR 603.7h), three times (`history.rs` twice,
  `dispatch.rs`); whether an ability is triggered, read off its kind in
  `register_static_effects` and off its effect's shape in `frame_card` and
  the audit.

### 3.3 Outside the trigger code — a PR each, `roadmap-v2.md` A6m

- **M1. The board's queries** (item 242). The permanents a player controls,
  inline at `sacrifice_candidates`, `choose_as_it_applies`,
  `process_untap_step` and `block_refusals` beside `permanents_controlled_by`;
  the blocker's half of CR 509.1a in `can_block` and `legal_blockers`; the
  players still in the game, built from `0..num_players()` in about twelve
  places, two of them in APNAP order (`players_in`,
  `place_pending_triggers`), three more reading `player_lost` itself; "this
  player left a multiplayer game" (CR 800.4), four times, once per token in
  `create_tokens`; who an opponent is, three ways (the pipeline's
  `opponents_of`, `resolve_player_ref`, and `SetController`'s seat count, which
  is CR 102.2's two-player answer on purpose), and whether a `PlayerRef` names
  a player, seven times; the candidates for a `SelectionFilter`, built by
  `has_legal_choices` arm for arm as `enumerate_legal_selections_upto` builds
  them, whose own doc says its bound exists so they can be one pass; a spell
  on the stack, which `is_spell_on_stack` answers without the resolving one
  and `damage_sources` adds back; "in this player's hand", the CR 305.1 and
  601.3 permission seam, in `can_play_land` and `can_begin_to_cast`; a
  permanent's host, about eleven inline reads and `ObjectSet::Host` resolved
  three times; lethal damage (CR 702.19b, 702.2c), inline in
  `assign_trample_damage`, again in `lethal_damage_for`, which only tests call,
  and in `ui/decision.rs`'s two defaults, which nothing calls.
- **M2. Mana** (item 243). A cost's demand, the pips of each type and the
  generic count, five times: `ManaPool::can_pay`, `can_pay_with_context`
  (only tests call it), `ManaPool::pay`, `remaining_cost_after_pool` and
  `ui/ask.rs`'s `pips_owed`, with `ManaCost::colored_count` a sixth that only
  tests call and that misses the `Colorless` spelling. And
  `oracle/mana_supply.rs` (MA-1) restating the paths it predicts: a mana
  ability's resolution context (`resolve_mana_effect`,
  `resolve_mana_trigger_effect`, `mana_production_of`; `trigger_mana` builds
  it without the ability's source); the `ProduceMana` proposal in three
  performers; the host's mana in `resolve_mana_trigger_effect` and
  `trigger_mana`; the `ManaAdded` arm and its subject, which
  `mana_trigger_adds` rewrites beside `subject_matches`; a static replacement
  ability that exists here, which `scan_mana_production_watchers` asks beside
  `push_static_ability_replacements`; a cost list's mana component, in
  `split_mana_component` and `can_activate_as_its_controller`; whether {T} or
  {Q} can be paid, in `check_cost_resource` and `pay_single_cost`; and the
  payable mana abilities, in `available_mana_sources` and `ManaSupply::read`.
- **M3. Resolution and the engine's records** (item 244). The permanents a
  resolution's effect applies to (CR 611.2c): `affected_permanents`, inlined
  at four primitives and skipped by seven, and by the Layer 6 register,
  which `GrantKeywordFlag` alone feeds through it (§3.5); the row a
  resolution's continuous effect writes, hand-built at seven primitives
  beside `register_resolution_ability_effect`; a static ability lowered to
  rows, four times (`register_static_effects`,
  `register_granted_static_effects`, `register_copied_static_effects`,
  `would_be_rows`, which lacks the assertion), with `create_in_zone`
  repeating `arrive_in_zone`'s CR 113.6 registration; an ability leaving the
  stack ceases to exist, seven sites beside `rollback_ability_activation`;
  "is this body a restriction", seen through "as long as" by
  `register_static_effects` and `prohibition` and not by `Summary::of`'s
  granted and copied legs (§3.5); and smaller ones: the object an entry's
  text calls "this", an effective ability by id (three), the printed
  definition an id is an instance of (three), the rows a source's statics
  generate (three), an entry counter's (kind, putter) key (three), the
  players an `EachOf` slice names (twice in one function), the top N cards of
  a library (four), whose control a return comes back under (two), the
  resolving object for a context (two), CR 109.5's "you" in
  `FilterPlayers::for_source` and `you_for`, and a trace record's batch
  field (four builders).
- **M4. The two `ObjectFilter` leaf tables** (item 245):
  `compute::object_matches_filter`, inside the layer walk, and
  `targeting::object_matches_filter_with`, for a selection or an
  `ObjectSet`. They differ where one refuses a leaf the other answers:
  `NotSource` with no source, `PowerLE` with no power, `OtherThanInstance`
  outside an announcement, and `PlayerRef::Owner`. Whether they should agree
  is the merge's design question, which is why it is its own PR.
  `targeting.rs`'s doc cites `codebase-state.md` item 14 for the pair, an
  archived item about something else.
- **M5. The dev GUI and the scenario format** (item 246), the dev GUI's turn
  in the engine-then-window cadence. "The first draw still to skip" in the
  loader and the writer; a card reference's match in the loader and the
  editor; a permanent's seat words in the writer and the editor; whether an
  allocation answers, in `ui/ask.rs` and the CLI; a permanent's keywords as
  words, in the display and the window; `card_name`, which is
  `get_effective_name` with a fallback; and the take-it-or-leave-it prompt,
  three identical bodies in `ui/ask.rs`.

### 3.4 Card authoring — C0

The static-ability constructors private to three card files
(`static_replacement` three times, `static_conditional_replacement`,
`static_restriction`), four token builders that are one vanilla-token
constructor, and `intrinsic_mana_ability`, documented as mirroring
`CardDataBuilder::mana_ability_single`. All are `codebase-state.md` "Before
card breadth" item 10's, decided at CV-2a's review for C0, the
`cards/authoring` constructors.

### 3.5 Copies that already disagree

Every one is unreachable today: no registered card, path or caller reaches
the place where the copies part. Each is in its merge's item, and each fix
that changes an answer is shown to fail first.

| copies | where they part | why no registered card reaches it | owner |
|---|---|---|---|
| the affected set at resolution (M3) | `SetPowerToughness`, `SwitchPowerToughness`, `ChangeColor`, `ChangeType`, `RemoveFromCombat`, `RemoveAllDamage`, `Restrict` and the Layer 6 register read only the targets, so "each creature you control" affects nothing | every registered filtered use of those primitives is a static ability, lowered by `register_static_effects`; the resolutions that write a filtered recipient (`ModifyPowerToughness`, `GrantKeywordFlag`, `Untap`, `DealDamage`, `CreateReplacement`) go through it | item 244 |
| a restriction through "as long as" (M3) | `Summary::of`'s granted and copied legs miss a conditional "can't", so `is_prohibited`'s gate skips a board whose only restriction is one; the gate's own comment says a route without a leg is dead | no registered ability is a conditional restriction | item 244 |
| lethal damage (M1) | `lethal_damage_for` clamps to at least 1, and `default_trample_assignment` gives a damaged deathtouch blocker 0; the engine's one path, inline in `assign_trample_damage`, gives the remainder | the first is called only by tests, the other two by nothing | item 242 |
| a spell on the stack (M1) | `is_spell_on_stack` leaves out the resolving spell; `damage_sources` adds it; the audit's `objects_that_may_trigger` admits a resolving ability, which its doc excludes | the Spell filter's one registered user is Counterspell's target, which cannot be the resolving spell (CR 115.5) | item 242 |
| a cost's pips of one type (M2) | `colored_count` misses `ManaSymbol::Colorless` | only tests call it | item 243 |
| the `ManaAdded` arm (M2) | `mana_trigger_adds` matches "this" by id, skips the trigger's limits and asks no CR 113.6 zone | item 213 already records the limits as over-offered; the producer is on the battlefield when asked, so its id is its identity | item 243 |
| CR 603.7a's "existed before" (T4) | `is_due` reads records of the batch its return was made in | `wait_to_return` makes a return after its exile batch closes, and no rider makes one | item 247 |
| an extra turn still coming (T8) | the waiting panel also asks that the turn's player is in the game | only the panel reads its copy | item 247 |
| the departed candidate's card (T6) | the audit's has no abilities | `describe` compares no card | item 247 |

## 4. The rename: "departed frame" to LKI

**Recommended: its own mechanical PR, right after this one and before items
176 and 177's design.** The design writes the record's contract in this
vocabulary, so renaming first means it is written once. And
`plans/glossary.md` defines **departed** as a player who has left the game
(CR 104.5), which is what `engine/leaving.rs`, `fuzz_games`' "Departed-owned
permanents" and the multiplayer tests mean by it, so `DepartedFrame` is a
second sense of a glossary word. CR 608.2h's term is "last known
information"; #239's names already say LKI (`carried_lki`, `lki_of`).

Counted on `main`: `DepartedFrame` at 50 sites in `src`; `departure_frame`
at 23, and 2 in tests; `departure_frames` 9, `take_departure_frame` 7,
`hand_over_departed_frame` 4, `capture_departure_frames` 3, `departed_frame`
4, and 1 in tests; and the `departed` fields of `PendingTrigger`,
`StackEntry` and `ResolvingObject` with their readers. About 140 code sites,
and about 27 mentions in the live docs, 19 of them in
`triggers-architecture.md`. The rename keeps the player's sense, and each new
name is chosen by the call-site rule (`engineering-practices.md` §2b).

## 5. Side notes

1. **What the old scan could not see.** It cut each file at its first
   `#[cfg(test)]`, which in `engine/costs.rs` and `engine/layers/explain.rs`
   is an attribute on one item, so it read none of their 15 functions, one of
   the four inline loops among them; and it stripped comments before strings.
   Fixed in this PR; its two sections read the same 30 pairs as before.
2. **A battle's attacker reads the wrong player in the scenario loader.**
   `scenario/build.rs`'s `attacked_player` takes a battle's controller, where
   CR 506.2 and 508.5 make its protector the defending player. Unreachable
   while the engine refuses an
   attack on a battle (`combat/validation.rs`); `backlog.md` §2.23 carries it.
3. **Two stale comments**, fixed with their merges:
   `validate_damage_source`'s doc says it loses the resolving spell, which
   `damage_sources` now adds (M1); `has_legal_choices`' "the sibling arm
   below asks it too" (M1).

## 6. Every entry

One table per section, in the report's order; **where** names §3's group.

### near-copies, names kept: 24

| # | key | members | verdict | where | why |
|---:|---|---|---|---|---|
| 1 | 1.00 | `cards/phase_rd_cards::static_replacement`, `cards/phase_re_cards::static_replacement` | merge | C0 | one static-replacement constructor, private in three card files |
| 2 | 0.82 | `cards/phase_tr1_cards::saproling_token`, `cards/phase_tr2a_cards::elf_warrior_token` | merge | C0 | token builders; one vanilla-token constructor |
| 3 | 0.79 | `cards/phase_rd_cards::static_replacement`, `cards/phase_re9_cards::static_replacement` | merge | C0 | as 1 |
| 4 | 0.79 | `cards/phase_re9_cards::static_replacement`, `cards/phase_re_cards::static_replacement` | merge | C0 | as 1 |
| 5 | 0.78 | `cards/phase_ld_cards::blood_moon`, `cards/phase_ld_cards::self_stripping_land` | keep |  | card definitions, alike by design |
| 6 | 0.75 | `cards/phase_rd_cards::static_replacement`, `cards/phase_re_cards::static_conditional_replacement` | merge | C0 | the conditional variant of 1 |
| 7 | 0.75 | `cards/phase_re_cards::static_replacement`, `cards/phase_re_cards::static_conditional_replacement` | merge | C0 | as 6 |
| 8 | 0.70 | `types/mana::can_pay_with_context`, `types/mana::can_pay` | merge | M2 | one cost-demand tally; `can_pay_with_context` has only test callers |
| 9 | 0.68 | `cards/phase_rd_cards::static_replacement`, `cards/phase_re_cards::static_restriction` | merge | C0 | the restriction variant of 1 |
| 10 | 0.68 | `cards/phase_re_cards::static_replacement`, `cards/phase_re_cards::static_restriction` | merge | C0 | as 9 |
| 11 | 0.68 | `cards/phase5_pre_cards::zhalfirin_shapecraft`, `cards/phase5_pre_cards::inside_out` | keep |  | card definitions, alike by design |
| 12 | 0.66 | `cards/phase_cm_cards::trinisphere`, `cards/phase_cm_cards::locked_sphere` | keep |  | card definitions, alike by design |
| 13 | 0.65 | `cards/phase_rb_cards::zombie_token`, `cards/phase_tr1_cards::saproling_token` | merge | C0 | as 2 |
| 14 | 0.65 | `cards/phase_ld_cards::call_to_serve_spell`, `cards/phase_ld_cards::on_serras_wings_spell` | keep |  | card definitions, alike by design |
| 15 | 0.64 | `oracle/legality::playable_lands`, `oracle/mana_helpers::castable_spells_with` | keep |  | two rules over the hand (CR 305.1, 601.3); their shared "in this player's hand" is M1 |
| 16 | 0.63 | `ui/ask::ask_order_cost_reductions`, `ui/ask::ask_order_triggers` | keep |  | one prompt per CR decision; the shared body is the prompt protocol, `validate_ordering` already one |
| 17 | 0.62 | `cards/authoring/triggers::once_per_event`, `types/triggers::multiplicity` | merge | T7 | which arms carry a multiplicity, listed twice; its doc says a new arm joins both |
| 18 | 0.62 | `cards/authoring/triggers::triggered_ability`, `cards/phase_re9_cards::static_replacement` | merge | C0 | the triggered sibling of 1, already in `cards/authoring` |
| 19 | 0.62 | `cards/phase_re_cards::parallel_lives`, `cards/phase_re_cards::doubling_season` | keep |  | card definitions, alike by design |
| 20 | 0.61 | `engine/layers/land_types::intrinsic_mana_ability`, `objects/card_data::mana_ability_single` | merge | C0 | documented as mirroring `mana_ability_single`: one {T}: Add constructor |
| 21 | 0.61 | `cards/phase_re_cards::static_conditional_replacement`, `cards/phase_re_cards::static_restriction` | merge | C0 | as 6 and 9 |
| 22 | 0.61 | `cards/phase_rb_cards::zombie_token`, `cards/phase_tr2a_cards::elf_warrior_token` | merge | C0 | as 2 |
| 23 | 0.60 | `cards/phase_rb_cards::zombie_token`, `cards/phase_re_cards::vanilla_token` | merge | C0 | as 2 |
| 24 | 0.60 | `cards/phase_re_cards::vanilla_token`, `cards/phase_tr1_cards::saproling_token` | merge | C0 | as 2 |

### one shape, names differ: 6

| # | key | members | verdict | where | why |
|---:|---|---|---|---|---|
| 1 | 1.00 | `ui/ask::ask_commander_to_command_zone`, `ui/ask::ask_apply_optional_effect` | merge | M5 | take it or leave it, with `ask_apply_optional_replacement` a third identical body |
| 2 | 0.96 | `types/triggers::subject_of`, `types/triggers::player_of` | merge | T7 | which record kind each arm reads, in five projections; exhaustive on the arm side only |
| 3 | 0.90 | `cards/keyword_creatures::thornweald_archer`, `cards/keyword_creatures::elvish_archers` | keep |  | card definitions, alike by design |
| 4 | 0.87 | `cards/phase_ld_cards::lands_have_flying`, `cards/phase_ld_cards::land_creatures_have_flying` | keep |  | card definitions, alike by design |
| 5 | 0.87 | `cards/alpha::counterspell`, `cards/alpha::burst_of_energy` | keep |  | card definitions, alike by design |
| 6 | 0.86 | `cards/alpha::counterspell`, `cards/phase_lc_cards::moonlace` | keep |  | card definitions, alike by design |

### same inputs: 32

| # | key | members | verdict | where | why |
|---:|---|---|---|---|---|
| 1 | `AbilityDef, GameState, ObjectId, PlayerId` | `oracle/mana_helpers::can_activate`, `oracle/mana_supply::offered_in_the_window` | false positive |  | CR 602.2's legality against what the mana window counts |
| 2 | `AbilityId, GameState, ObjectId` | `ui/display::ability_text`, `ui/why::static_ability_words` | merge | M3 | an effective ability's words by id; also at `run_priority_round` and `activate_mana_ability` |
| 3 | `ActionContext, GameState, PlayerId` | `engine/leaving::player_left_the_game`, `engine/zones::draw_card` | false positive |  | unrelated performers |
| 4 | `AttackTarget, GameState` | `scenario/build::attacked_player`, `ui/display::attack_target_name` | false positive |  | a player resolved against a target named (side note 2) |
| 5 | `Board<>, ContinuousEffect, GameState, usize` | `engine/layers/board::static_ability_still_exists`, `engine/layers/compute::for_row` | false positive |  | an existence check against a resolver's constructor |
| 6 | `Board<>, GameState, ObjectId, Option<PlayerId>, usize` | `engine/layers/compute::for_source`, `engine/layers/condition::you_for` | merge | M3 | `for_source` computes CR 109.5's "you" with `you_for`'s body |
| 7 | `CopiableValues, EffectiveCharacteristics` | `engine/layers/copy::apply_to`, `ui/display::type_line_against` | false positive |  | the Layer 1 write against a display diff |
| 8 | `DecisionProvider, GameState -> Result<(),String>` | `engine/priority::perform_sba_and_triggers`, `engine/sba::check_state_based_actions_loop` | false positive |  | the first calls the second |
| 9 | `DelayedTrigger, GameState` | `engine/triggers/delayed::delayed_referents`, `ui/waiting::waiting_trigger` | false positive |  | referents against a display row (T8 is the row's other half) |
| 10 | `EffectiveCharacteristics, GameState, ObjectId, TypeLines` | `ui/display::of`, `snapshot::card_with` | false positive |  | caller and callee |
| 11 | `FilterIdentity<>, GameState, Option<ObjectId>, PlayerId, SelectionFilter, usize` | `engine/targeting::has_legal_choices`, `oracle/legality::enumerate_legal_selections_upto` | merge | M1 | the same candidate set per `SelectionFilter`: `has_legal_choices(n)` is `upto(n).len() >= n` |
| 12 | `GameObject, GameState` | `engine/zones::create_in_zone`, `state/game_state::add_object` | false positive |  | `create_in_zone` calls `add_object` (side note 3) |
| 13 | `GameState, ObjectFilter, ObjectId, PlayerId` | `engine/costs::sacrifice_candidates`, `engine/targeting::object_matches_filter` | false positive |  | a candidate list built on the matcher |
| 14 | `GameState, ObjectId, ObjectId` | `state/game_state::attach`, `ui/why::copy_refusal` | false positive |  | unrelated |
| 15 | `Into<String>, usize` | `scenario/text::syntax_error`, `state/decision_log/text::at` | false positive |  | two formats' error constructors |
| 16 | `ManaCost, ManaType` | `types/mana::colored_count`, `ui/ask::pips_owed` | merge | M2 | a cost's pips of one type; `colored_count` has only test callers and misses the `Colorless` spelling |
| 17 | `Option<[u64]>, [u64], u64` | `scenario/setup::generic_split`, `ui/ask::forced_allocation` | keep |  | a standard split against detecting a forced answer |
| 18 | `str, str` | `scenario/text::player_number_after`, `state/decision_log/text::parsed` | false positive |  | unrelated parsers |
| 19 | `ChoiceOption, GameState` | `ui/choice_types::as_logged`, `ui/display::option_label`, `prompt::option_view` | keep |  | the log's fixed printed name, read without the layers, against the live label |
| 20 | `GameState, ObjectId -> String` | `oracle/characteristics::get_effective_name`, `ui/display::card_name`, `ui/display::named` | merge | M5 | `card_name` is `get_effective_name` with a fallback; `named` is a label |
| 21 | `GameState, PlayerId -> Vec<ObjectId>` | `oracle/board::permanents_controlled_by`, `oracle/legality::legal_attackers`, `oracle/legality::legal_blockers` | merge | M1 | `legal_blockers` restates `can_block`'s blocker half |
| 22 | `GameState, PlayerId, u32` | `engine/triggers/history::begin_turn_history`, `engine/turns::expire_until_your_next_turn`, `state/game_state::begin_turn` | keep |  | the history needs a lagging copy of the turn, which `begin_turn` overwrites |
| 23 | `GameState, [EventSeq]` | `engine/triggers/audit::describe_window`, `engine/triggers/delayed::take_returns_due`, `engine/triggers/history::advance_history` | false positive |  | three readers of one window |
| 24 | `ManaCost, ManaPool` | `oracle/mana_helpers::remaining_cost_after_pool`, `types/mana::can_pay`, `types/mana::pay_specific_only` | merge | M2 | the cost-demand tally of 8 and 16 |
| 25 | `DecisionProvider, GameState, ObjectId, PlayerId` | `engine/put_on_stack::cast_spell`, `engine/returns::choose_what_it_enchants`, `ui/ask::ask_commander_to_command_zone`, `ui/ask::ask_apply_optional_effect` | merge | M5 | the two `ask_*` share 1's body; `cast_spell` and `choose_what_it_enchants` are unrelated |
| 26 | `EnterMods, GameState, ObjectId, PlayerId` | `engine/layers/lookahead::new`, `engine/layers/lookahead::compute_as_entering`, `engine/replacement/lookahead::for_entering`, `state/game_state::place_on_battlefield` | keep |  | one chain modeling the performer, both through `PermanentState::entering` |
| 27 | `FieldValue, str` | `state/trace::get`, `ui/what_happened::text`, `ui/what_happened::number`, `ui/what_happened::id` | false positive |  | typed accessors over one `get` |
| 28 | `GameAction, GameState` | `engine/actions::is_a_departed_tokens_move`, `engine/replacement/gather::chooser_for`, `engine/replacement/gather::commander_zone_replacement`, `engine/replacement/lookahead::new` | false positive |  | different questions about one proposal |
| 29 | `GameState, ObjectId, Option<DepartedFrame>` | `engine/triggers/dispatch::hand_over_departed_frame`, `oracle/characteristics::damage_source_existence`, `oracle/characteristics::damage_source_characteristics`, `ui/display::as_it_left` | merge | L1 | the damage source as it deals damage: these two are its identity and characteristics, and three more spellings are inline (the blind read said keep, the pair being side by side on purpose; the other three are not in the entry) |
| 30 | `ActionContext, GameState -> Result<(),String>` | `engine/leaving::exile_objects_no_player_in_game_controls`, `engine/turns::start_first_turn`, `engine/turns::on_turn_begin`, `engine/turns::process_untap_step`, `engine/turns::process_draw_step` | false positive |  | turn hooks |
| 31 | `GameState, ObjectId -> bool` | `engine/cost_determination/gather::prints_cost_ability`, `engine/triggers/dispatch::visible_to_all`, `oracle/characteristics::is_instant_or_has_flash`, `oracle/characteristics::is_creature`, `oracle/characteristics::has_summoning_sickness`, `oracle/characteristics::has_permanent_type` | false positive |  | distinct predicates |
| 32 | `GameState, ObjectId, Zone -> Result<(),String>` | `engine/actions::announce_token_created`, `engine/zones::move_object`, `engine/zones::put_token_into`, `engine/zones::arrive_in_zone`, `engine/zones::remove_from_zone_collection`, `engine/zones::add_to_zone_collection` | false positive |  | stages of one move, performer and emitter apart |

### same composition: 48

| # | key | members | verdict | where | why |
|---:|---|---|---|---|---|
| 1 | `ability_type + get_effective_abilities()` | `engine/mana::activate_mana_ability`, `engine/priority::run_priority_round`, `engine/restriction/predicate::prohibition`, `oracle/mana_supply::read`, `oracle/mana_supply::available_mana_sources`, `oracle/mana_supply::read`, `scenario/build::activated_ability`, `ui/display::ability_texts`, `ui/why::priority_refusals`, `ui/why::window_refusals` | merge | M2, M3 | the payable mana abilities twice in `mana_supply` (M2); an effective ability by id three times (M3) |
| 2 | `in_game() + num_players()` | `engine/combat/steps::process_declare_attackers`, `engine/priority::run_priority_round`, `engine/resolve::players_in`, `engine/triggers/placement::place_pending_triggers`, `oracle/legality::enumerate_legal_selections_upto` | merge | M1 | the players still in the game, built inline in about twelve places |
| 3 | `battlefield_ids_ordered() + controls()` | `engine/costs::sacrifice_candidates`, `engine/resolve::choose_as_it_applies`, `engine/turns::process_untap_step`, `oracle/board::permanents_controlled_by`, `ui/why::block_refusals` | merge | M1 | the permanents a player controls: `permanents_controlled_by` and four inline copies |
| 4 | `damage_marked + get_effective_toughness()` | `engine/combat/keywords::lethal_damage_for`, `engine/combat/keywords::assign_trample_damage`, `engine/sba::check_state_based_actions`, `ui/decision::default_damage_assignment`, `ui/decision::default_trample_assignment`, `ui/display::format_permanent` | merge | M1 | lethal damage: the engine's one use is inline, the helper only tests call, the `ui/decision.rs` pair has no caller |
| 5 | `evaluate_amount() + mana` | `engine/mana::resolve_mana_effect`, `engine/resolve::resolve_primitive`, `engine/triggers/dispatch::resolve_mana_trigger_effect`, `oracle/mana_supply::add_mana_produced`, `oracle/mana_supply::trigger_mana` | merge | M2 | the `ProduceMana` proposal in three performers; the host's mana in two places |
| 6 | `has_summoning_sickness() + tapped` | `engine/costs::check_cost_resource`, `engine/costs::pay_single_cost`, `oracle/legality::can_attack`, `ui/display::format_permanent`, `snapshot::permanent` | merge | M2 | whether {T} or {Q} can be paid, in `check_cost_resource` and `pay_single_cost` |
| 7 | `in_game() + is_multiplayer()` | `engine/actions::entry_proposal`, `engine/combat/resolution::assign_combat_damage`, `engine/layers/compute::resolve_set_controller`, `engine/resolve::resolve_primitive` | merge | M1 | CR 800.4's "left a multiplayer game", four times |
| 8 | `ability_source + object_ref()` | `engine/mana::resolve_mana_effect`, `engine/resolve::resolve_primitive`, `engine/resolve::this_object`, `engine/triggers/dispatch::resolve_mana_trigger_effect`, `oracle/mana_supply::mana_production_of` | merge | M2 | a mana ability's resolution context, three times |
| 9 | `remove_object() + stack` | `engine/put_on_stack::rollback_ability_activation`, `engine/resolve::resolve_primitive`, `engine/stack::resolve_taken`, `engine/stack::handle_fizzle`, `engine/triggers/placement::place_one` | merge | M3 | an ability leaving the stack ceases to exist, seven sites |
| 10 | `abilities + make_mut()` | `engine/layers/compute::apply_resolved`, `engine/layers/copy::except`, `engine/layers/copy::modify`, `engine/layers/land_types::apply_set_subtypes`, `engine/layers/land_types::apply_add_subtype`, `engine/resolve::apply_copy` | false positive |  | the `Arc` copy-on-write idiom |
| 11 | `saturating_sub() + to_vec()` | `scenario/setup::generic_split`, `ui/ask::forced_allocation`, `ui/random::allocate`, `view_model::allocate` | keep |  | four allocation policies |
| 12 | `object + object_ref()` | `engine/resolve::remembered_since`, `engine/triggers/dispatch::capture_frame`, `engine/triggers/dispatch::take_named_frame`, `engine/triggers/dispatch::take_departure_frame`, `engine/triggers/dispatch::find_matches`, `oracle/characteristics::damage_source_existence`, `types/triggers::is`, `types/triggers::found` | merge | L2, L4 | "this reference is still the object" (L2); taking a mover's frame (L4) |
| 13 | `battlefield + is_creature()` | `engine/combat/validation::can_block`, `engine/sba::check_state_based_actions`, `oracle/legality::can_attack`, `ui/display::format_permanent` | false positive |  | each asks "a creature?" for its own rule |
| 14 | `named() + player_name()` | `ui/display::attack_target_name`, `ui/display::damage_target_name`, `ui/what_happened::iteration_lines`, `ui/why::ranges_over`, `snapshot::target_name` | keep |  | one formatter per target enum over shared leaves |
| 15 | `compute_characteristics() + types` | `engine/actions::for_object`, `engine/triggers/history::history_update`, `oracle/characteristics::is_instant_or_has_flash`, `oracle/characteristics::is_creature`, `oracle/characteristics::has_summoning_sickness`, `oracle/characteristics::get_effective_types`, `oracle/characteristics::has_type`, `oracle/characteristics::has_permanent_type`, `snapshot::permanent` | keep |  | the wrappers themselves (`is_creature` is `has_type`'s, word for word) |
| 16 | `controller + settled_holds()` | `engine/resolve::resolution_condition_holds`, `engine/stack::resolve_taken`, `engine/triggers/delayed::when_not_done`, `engine/triggers/delayed::match_delayed`, `engine/triggers/dispatch::match_def` | merge | T1 | CR 603.4's intervening "if" asked four times; `resolution_condition_holds` is CR 608.2c |
| 17 | `battlefield + controls()` | `engine/combat/validation::can_block`, `oracle/legality::can_attack`, `ui/random::pick_number`, `ui/why::block_refusals` | false positive |  | different rules |
| 18 | `battlefield_ids_ordered() + stack` | `engine/leaving::owned_objects_leave`, `engine/targeting::has_legal_choices`, `oracle/legality::enumerate_legal_selections_upto`, `oracle/legality::damage_sources`, `state/game_state::zone_ids_ordered` | merge | M1 | `has_legal_choices` rebuilds `enumerate_legal_selections_upto`'s candidates; a zone order twice |
| 19 | `static_ability_atoms() + static_object_set(); static_ability_atoms() + static_primitive_rows(); static_object_set() + static_primitive_rows()` | `engine/layers/lookahead::would_be_rows`, `engine/resolve::register_granted_static_effects`, `state/game_state::register_static_effects`, `state/game_state::register_copied_static_effects` | merge | M3 | a static ability lowered to rows, four times; `would_be_rows` lacks the assertion |
| 20 | `matched() + object_matches_filter_of_source()` | `engine/replacement/gather::set_affects`, `engine/triggers/dispatch::subject_matches`, `oracle/mana_supply::mana_trigger_adds` | merge | M2 | `mana_trigger_adds` restates `subject_matches`; `set_affects` is replacement's |
| 21 | `event + format_event(); format_event() + names` | `engine/triggers/audit::describe_window`, `ui/display::format_event_log`, `snapshot::build` | false positive |  | three callers of one formatter |
| 22 | `end() + key().begin_array()` | `engine/replacement/pipeline::record`, `engine/trace_records::batch`, `state/trace::field_strs`, `state/trace::field_u64s`, `state/trace::field_usizes` | keep |  | the JSON array idiom, typed overloads of one writer |
| 23 | `ability_source + untargeted(); object_ref() + untargeted()` | `engine/mana::resolve_mana_effect`, `engine/triggers/dispatch::resolve_mana_trigger_effect`, `oracle/mana_supply::mana_production_of` | merge | M2 | as 8 |
| 24 | `as_resolved_targets() + chosen_targets` | `engine/stack::resolve_taken`, `scenario/setup::confirm_played`, `snapshot::stack_item` | false positive |  | a projection of the targets |
| 25 | `definition() + id.definition()` | `engine/layers/compute::apply_resolved`, `scenario/write::write_this_turn_counts`, `ui/why::static_ability_words` | false positive |  | each through the one `definition()` |
| 26 | `execute_action() + resolving()` | `engine/mana::resolve_mana_effect`, `engine/resolve::resolve_primitive`, `engine/triggers/dispatch::resolve_mana_trigger_effect` | merge | M2 | as 5 |
| 27 | `position_word() + step` | `scenario/build::new`, `scenario/text::parse_header`, `scenario/text::fmt`, `editor::view` | false positive |  | writer, parser, error text and window call one helper |
| 28 | `str() + u64()` | `state/trace::write`, `ui/what_happened::what_happened`, `ui/what_happened::record_lines`, `ui/what_happened::iteration_lines`, `ui/what_happened::placement`, `ui/why::applied_already`, `ui/why::asked_about_these_events` | keep |  | readers of the trace schema |
| 29 | `controls() + is_creature()` | `engine/combat/validation::can_block`, `oracle/legality::can_attack`, `oracle/legality::legal_blockers`, `ui/why::block_refusals` | merge | M1 | as same inputs 21 |
| 30 | `field_opt_u64() + field_u64()` | `engine/replacement/pipeline::record`, `engine/trace_records::batch`, `engine/trace_records::batch_end`, `state/trace::trace_game`, `state/trace::emit_record` | merge | M3 | a record's batch field, written by four builders |
| 31 | `split_once() + strip_prefix()` | `scenario/text::parse_card_words`, `scenario/text::parse_setup_action`, `scenario/text::parse_card_word`, `scenario/text::parse_player_line`, `state/decision_log/text::read`, `save::read` | false positive |  | each its own line grammar |
| 32 | `field_str() + field_u64()` | `engine/replacement/pipeline::record`, `engine/trace_records::batch`, `engine/trace_records::layer_walk`, `engine/trace_records::decision`, `engine/trace_records::priority_rejected`, `state/trace::trace_game`, `state/trace::trace_objects` | false positive |  | each record kind's own fields |
| 33 | `stack + take_stack_entry()` | `engine/put_on_stack::rollback_ability_activation`, `engine/resolve::resolve_primitive`, `engine/stack::resolve_top_of_stack` | merge | M3 | as 9 |
| 34 | `ability_type + functions_in()` | `engine/replacement/gather::push_static_ability_replacements`, `oracle/mana_supply::scan_mana_production_watchers`, `state/game_state::register_static_effects` | keep |  | CR 113.6 is already one helper; the kind filter is each reader's |
| 35 | `subtypes + word()` | `types/effects::effective_name`, `ui/display::type_line_against`, `app::type_line` | false positive |  | a token's name (CR 111.4) is another rule |
| 36 | `functions_in() + types` | `engine/replacement/gather::push_static_ability_replacements`, `engine/triggers/dispatch::match_candidates`, `oracle/mana_supply::scan_mana_production_watchers` | keep |  | as 34 |
| 37 | `event + events.record()` | `engine/triggers/delayed::occurrences_among`, `engine/triggers/dispatch::dispatch_inner`, `engine/triggers/dispatch::find_matches`, `engine/triggers/dispatch::occurrences_matching_arm`, `engine/triggers/history::history_update` | false positive |  | the record accessor; the arm match is already one helper |
| 38 | `controls() + tapped` | `engine/combat/validation::can_block`, `oracle/legality::can_attack`, `oracle/legality::legal_blockers`, `ui/random::pick_number` | merge | M1 | as same inputs 21 |
| 39 | `effect + effect.as_cost_modification()` | `engine/zone_function::functioning_zones`, `state/continuous_effects::of`, `state/game_state::register_static_effects` | merge | M3 | "is this body a restriction", seen through "as long as" in two places and not in two |
| 40 | `checked_sub() + saturating_add()` | `scenario/build::history_counts_by_turn`, `scenario/write::since_your_last_turn_differs`, `editor::permanent_rows` | keep |  | the scenario writer predicts the loader on purpose |
| 41 | `battlefield + get_effective_toughness()` | `engine/combat/keywords::lethal_damage_for`, `engine/combat/keywords::assign_trample_damage`, `engine/sba::check_state_based_actions`, `ui/decision::default_damage_assignment`, `ui/decision::default_trample_assignment` | merge | M1 | as 4 |
| 42 | `objects + pass_membership()` | `engine/layers/board::frame_of`, `engine/layers/board::frame_at_ceiling`, `engine/layers/compute::compute_characteristics`, `engine/layers/compute::walk_uncached`, `engine/layers/explain::explain` | keep |  | memo, live, reference, ceiling and recorded walks dispatch on one helper |
| 43 | `battlefield.contains_key() + execute_action()` | `engine/actions::perform_action`, `engine/combat/steps::process_declare_attackers`, `engine/resolve::resolve_primitive` | false positive |  | each caller's "still there?" |
| 44 | `get_effective_controller() + stack` | `engine/leaving::uncarded_stack_objects_cease`, `engine/leaving::exile_objects_no_player_in_game_controls`, `engine/resolve::resolve_primitive`, `engine/stack::resolve_top_of_stack` | keep |  | two CR 800.4a sentences and a resolution reading its controller |
| 45 | `compute_characteristics() + keyword_flags` | `oracle/characteristics::has_keyword`, `oracle/characteristics::is_instant_or_has_flash`, `ui/display::collect_keywords`, `snapshot::permanent` | merge | M5 | a permanent's keywords as words, in the display and the window |
| 46 | `in_game() + players` | `engine/sba::check_state_based_actions`, `engine/targeting::validate_player_target`, `engine/targeting::validate_any_target`, `engine/targeting::has_legal_choices`, `state/game_state::next_player_in_game`, `state/game_state::settle_game_result` | merge | M1 | as 2; `validate_any_target` restates `validate_player_target` |
| 47 | `controller_or_owner() + owner` | `engine/replacement/gather::gather`, `engine/triggers/audit::matches_without_shortcuts`, `engine/triggers/dispatch::find_matches` | merge | T6 | the dispatcher's `TriggerCandidate` literal, written again in the audit |
| 48 | `effect + get_effective_abilities()` | `engine/put_on_stack::activate_ability`, `engine/restriction/predicate::prohibition`, `oracle/mana_supply::mana_trigger_adds` | false positive |  | each a different ability's effect |

### same test: 30

| # | key | members | verdict | where | why |
|---:|---|---|---|---|---|
| 1 | `definition() == definition()` | `engine/layers/compute::apply_resolved`, `engine/resolve::with_this_ability`, `engine/triggers/dispatch::occurrences_matching_arm`, `scenario/write::write_this_turn_counts`, `ui/why::window_refusals`, `ui/why::static_ability_words` | merge | M3 | the printed definition an ability id is an instance of, three times |
| 2 | `object == object` | `engine/resolve::remembered_since`, `engine/triggers/binding::carried_lki`, `engine/triggers/delayed::note_delayed_source_left`, `engine/triggers/dispatch::capture_frame`, `engine/triggers/dispatch::take_named_frame`, `engine/triggers/dispatch::take_departure_frame` | merge | L4 | `take_departure_frame` opens with `take_named_frame`'s body |
| 3 | `zone_change_epoch == zone_change_epoch` | `engine/targeting::still_exists`, `engine/triggers/audit::matches_without_shortcuts`, `engine/triggers/binding::bound_object`, `scenario/write::write_this_turn_counts` | merge | L2 | CR 400.7's "still this object", four here and seven more `object_ref(id) == Some(r)` |
| 4 | `Some(_) == limit` | `engine/stack::resolve_taken`, `engine/triggers/delayed::queue_pending`, `engine/triggers/dispatch::queue_matches` | merge | T2 | "triggers only once each turn" and the queue step, twice |
| 5 | `controller == pid` | `engine/layers/compute::object_matches_filter`, `engine/resolve::resolve_player_ref`, `engine/targeting::object_matches_filter_with` | merge | M4, M1 | the two `ObjectFilter` leaf tables (M4); the `PlayerRef` predicate (M1) |
| 6 | `controller == player` | `engine/triggers/dispatch::player_ref_is`, `engine/triggers/placement::place_pending_triggers`, `types/effects::contains` | merge | M1 | the `PlayerRef` predicate, Opponent as `!= you` |
| 7 | `owner == player` | `engine/leaving::owned_objects_leave`, `engine/triggers/dispatch::departs_an_ability_list_source`, `engine/triggers/dispatch::capture_departure_frames`, `engine/triggers/dispatch::player_ref_is`, `scenario/build::player_leaves` | keep |  | CR 800.4a's owned set read by the performer and a batch check; L7 asks the design |
| 8 | `source == source` | `engine/keywords::add_lifelink_gain`, `engine/triggers/dispatch::occurrences_matching_arm`, `state/continuous_effects::remove_static_by_source`, `state/continuous_effects::retime_static_rows`, `state/continuous_effects::remove_rows_ending_with` | merge | M3 | the rows a source's static abilities generate, three times |
| 9 | `step == step` | `scenario/text::fmt`, `ui/replay::next`, `editor::view` | false positive |  | unrelated |
| 10 | `Some(_) == current` | `engine/triggers/delayed::drop_delayed_triggers_of_gone_turns`, `editor::card_edit` | false positive |  | the engine side is T8's |
| 11 | `Some(_) == extra_turn` | `engine/triggers/delayed::in_turn`, `ui/waiting::waiting_trigger` | merge | T8 | whether an extra turn is still coming, with two definitions |
| 12 | `Some(_) == host` | `engine/triggers/dispatch::subject_matches`, `oracle/mana_supply::mana_trigger_adds` | merge | M2 | as composition 20 |
| 13 | `Some(_) == step` | `scenario/build::new`, `scenario/build::begin_turns`, `scenario/write::write_game` | merge | M5 | "the first draw still to skip" in the loader and the writer |
| 14 | `Some(_) == subject` | `engine/triggers/binding::carried_lki`, `view_model::item` | false positive |  | `carried_lki` is the rule's one place since #239 |
| 15 | `active_player == player` | `scenario/write::since_your_last_turn_differs`, `ui/auto_yield::holds` | false positive |  | unrelated |
| 16 | `controller == owner` | `scenario/write::permanent_line`, `editor::set_seats` | merge | M5 | a permanent's seat words in the writer and the editor |
| 17 | `counter == counter` | `engine/replacement/pipeline::apply_rewrite`, `types/replacement::merge`, `types/replacement::take_back` | merge | M3 | an entry counter's (kind, putter) key, three times |
| 18 | `max == min` | `ui/ask::only_pick`, `view_model::pick_rule` | false positive |  | a label against forced-answer detection |
| 19 | `mode == mode` | `app::draw`, `session::show` | false positive |  | window state |
| 20 | `name == name()` | `state/trace::named`, `types/effects::named` | false positive |  | one idiom over two enums |
| 21 | `object == source` | `engine/triggers/delayed::note_delayed_source_left`, `state/continuous_effects::retire_earlier_copies_of` | false positive |  | the static-row half is 8's |
| 22 | `owner == pid` | `engine/layers/compute::object_matches_filter`, `engine/targeting::object_matches_filter_with` | merge | M4 | as 5 |
| 23 | `owner == player_id` | `oracle/legality::can_play_land`, `oracle/mana_helpers::can_begin_to_cast` | merge | M1 | "in this player's hand", the CR 305.1/601.3 permission seam |
| 24 | `player == player` | `ui/replay::next`, `ui/why::refusals` | false positive |  | unrelated |
| 25 | `player == seat` | `scenario/setup::priority`, `editor::apply`, `editor::seat_edit` | false positive |  | unrelated |
| 26 | `sum == total` | `ui/ask::allocation_fits`, `ui/cli::allocate` | merge | M5 | whether an allocation answers, in `ui/ask.rs` and the CLI |
| 27 | `tag == tag` | `scenario/build::one_named`, `editor::names` | merge | M5 | card reference matching in the loader and the editor |
| 28 | `timestamp == timestamp` | `engine/layers/board::applications_in_layer`, `state/continuous_effects::retime_static_rows` | false positive |  | an assertion of the writer's invariant |
| 29 | `turn == turn_number` | `ui/auto_yield::holds`, `ui/replay::next` | false positive |  | unrelated |
| 30 | `words == words` | `objects/card_data::build`, `scenario/write::push_line` | false positive |  | unrelated |
