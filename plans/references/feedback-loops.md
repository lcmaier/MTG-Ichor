# Where a write feeds a decision in the same pass — the feedback-loop census

> **Status:** run 2026-10-06 against #228's merge, for `roadmap-v2.md` row
> A6i, which this closes (#229). **Authority:** every count is
> `plans/references/feedback-loops.py`'s, spliced in between this file's
> markers by `--write`. The classes, their queries and their dispositions live
> in the script's tables and change only there. On what exists,
> `codebase-state.md` wins. **Re-run:** the counts take about two minutes
> from Scryfall and are cached; `--check` reads every class's regexes over
> the cast census's cached corpus in seconds, and `--calibrate` lists the
> registered cards in each class.

## 0. The question, and how to read the tables

Kaito, Bane of Nightmares beside Spark Double was found by a judges' thread,
not by the engine (`copy-effects-architecture.md` §4.1a, "The Kaito board"):
a permanent whose type hangs on its own counters, entering where the counters
it gets decide which effects apply to its entry. This census asks where else
the rules let a write change a decision the same pass is still making, and
which printed cards make it happen, so that the next such card is a query
result.

A **loop** here is a write that can change a decision the same pass is still
making. The CR re-runs four such passes:

| Loop | The write | The decision it can change | The re-run | Where the engine runs it |
|---|---|---|---|---|
| 1 | a replacement effect's rewrite of an event, an entry's counters, status, copy and edits among them | which replacement effects still apply, and what each writes | CR 616.1f's re-gather, and 616.2 | `apply_replacements`, one gather per iteration (`engine-map.md` §1.2's steps 5–9, seam S6) |
| 2 | a continuous effect applied in a layer | which of that layer's effects exist, what each applies to and what it does | CR 613.8c's re-sort | `next_ready` and `depends_on`, in every layer of every walk |
| 3 | a state-based action's result | which state-based actions apply at the next check | CR 704.3's repeated check | `check_state_based_actions_loop` |
| 4 | the events a triggered ability's resolution performs | what else triggers | the dispatch at every batch's close, then CR 603.3b's placement | `dispatch_batch`, then `perform_sba_and_triggers` |

Loops that never stop are not counted: CR 104.4b's draw and CR 731's
shortcuts are `backlog.md` §2.28's and TR-6's (state triggers, the loop, and
the rule-owned arm).

A **class** is a family of printed cards that supplies the write, the
decision that reads it, or both. Each row gives the Scryfall terms the class
adds to the corpus query, its `total_cards`, the registered cards in it, and
one disposition:

- **test**: a test on a registered card or a cheap fixture closes the loop
  today, cited by name. "Written here" marks the seven this census added, in
  `mtgsim/tests/feedback_loop_census_test.rs`;
- **register row**: a row of `engineering-practices.md` §3.4b's register,
  where a ruling and the CR disagree;
- **backlog**: a `backlog.md` entry, where the engine has no surface yet and
  no document owns one;
- **owned**: the surface is missing and a phase on the route already owns it.
  A backlog entry there would be a second owner (`backlog.md` §0: an entry
  graduates when a document takes it), so the row names the phase.

The corpus is the cast census's:

<!-- feedback-loops: begin CORPUS -->
`legal:commander -t:token game:paper -is:funny`: 32,115 cards (`total_cards`).
<!-- feedback-loops: end CORPUS -->

**Every number sits beside its query.** A row's terms are added to the corpus
query; `--names ID` prints the first page of cards behind any row. Loop 4's
rows count two halves: the triggers that watch an event, and the triggers
whose resolution produces it. Oracle text is read with reminder text removed,
as Scryfall's `o:` reads it.

<!-- feedback-loops: begin DATE -->
Counted 2026-10-06: the counts cache's newest entry 2026-10-06, the registered cards fetched 2026-10-05.
<!-- feedback-loops: end DATE -->

## 1. The seams, read for a fifth loop

The engine map names fourteen seams, each one subsystem reading another's
output (`engine-map.md` §1.3). Read for a write that feeds a decision in the
same pass, they hold the four loops and no fifth:

- S6 is loop 1. S2–S5 and S7, the entering object's frame, the gather, the
  "can't", the copy and the amount, are its inputs, each read again on every
  iteration.
- Loop 2 runs inside every walk, so inside S2's frame and S13's match too.
  That is one loop nested in another, not a new one.
- S8–S12 run once per batch. The look-back frames, registration and the
  dispatch read what phase 2 performed, and nothing they decide is decided
  again in that batch.
- S14 is loop 4.

The reading found three widenings instead of a fifth loop:

1. **Loop 1 is every event's, not only an entry's.** CR 616.1f re-gathers
   after any rewrite: a death that becomes an exile, damage that becomes
   damage to another object, an amount that doubles. A rider's event goes
   through the same gather with its lineage's applied set (CR 614.5), which
   is what stops it. The feeds table is the entry half's premise, and
   `classify`'s cells are the rest (`replacement-architecture.md` §3.5, §4.1).
2. **Loops 3 and 4 take turns.** CR 117.5, and 603.3b's last sentence:
   state-based actions until none, then placement, then again, and an
   ability that triggers during placement (ward, as an ability targets) goes
   on the stack in the same moment. Three state-based actions read the
   trigger queue themselves (704.5s, 704.5t and 704.5v). The turns are
   `perform_sba_and_triggers`.
3. **A triggered mana ability is loop 4 without the stack** (CR 605.4a): it
   resolves inside the mana ability that triggered it, and its mana is an
   event dispatched at once. No printed chain closes through it (§6), and
   what runs inside a mana ability is MA-3's (any color, riders, the nested
   window).

An earlier layer's write that ends a later layer's effect is a read in layer
order, not a loop: Blood Moon's layer 4 ends The Tabernacle at Pendrell Vale's
layer 6 grant, and Humility's layer 6 ends a creature's own layer 7c anthem.
Within one layer, as with Blood Moon and Urborg, it is CR 613.8a(b)'s
"existence", loop 2.

## 2. Loop 1 — an entry's writes and CR 616.1's re-gather

**Where it runs.** `apply_replacements` gathers the replacement effects that
apply, asks the affected player or controller to choose when two or more are
choosable, applies the one chosen, and gathers again against the rewritten
event. **What it skips, and what holds that.** When every order reaches one
outcome, nobody is asked: `ordering_cannot_change_outcome`, whose entry
premise is the feeds table and whose other premises are `classify`'s cells,
recomputed in debug builds by `check_order_invariance`
(`codebase-state.md` item 185's first piece).

<!-- feedback-loops: begin LOOP1 -->
| Class | CR | Terms | Cards | Registered | Disposition |
|---|---|---|---:|---|---|
| A permanent whose own characteristics hang on its counters or its tapped status (the Kaito shape) (`self`) | 614.12, 616.1f, 711.2 | `(o:/as long as [^,.]+ has [^.]*counters? on (it\|him\|her), (it\|he\|she\|this [a-z]+)('s\| is\| gets\| has\| loses)/ or o:/as long as [^,.]+ is (tapped\|untapped), (it\|he\|she\|this [a-z]+)('s\| is\| gets\| has\| loses)/ or o:/(this [a-z]+\|^[^,.]+) (has\|gets\|is) [^.]* as long as (it\|he\|she) (has [^.]*counters? on\|is (tapped\|untapped))/ or o:/^level \d/) -o:/for as long as (that\|those) [a-z]+ (has\|have)/` | 61 | — | **register row:** `lookahead-entry-counters`, whose third board is Kaito's: `kaito_cast_beside_oath_of_gideon_is_its_controllers_order`, `kaito_enters_tapped_under_an_opponents_creatures_enter_tapped`. The feeds table's last row (`Condition::reads_entry_state`) is what makes the order a question |
| A status every entering object of a kind gets: enters tapped, or untapped (`status`) | 614.1c, 110.5b | `o:/(creatures\|artifacts\|lands\|permanents\|planeswalkers\|enchantments)[^.]* enter (the battlefield )?(tapped\|untapped)/` | 29 | Archelos, Lagoon Mystic, Root Maze | **test:** `two_archelos_in_opposite_states_ask_the_entering_permanents_controller`: the feeds table's (c), two statuses that set opposite values are an order |
| Counters every entering object of a kind gets ("enters with an additional counter") (`entry-counters`) | 614.1c, 122.6 | `o:/(you control\|other creature\|each creature\|each other\|planeswalkers?\|creatures?)[^.]* enters? (the battlefield )?with [^.]*additional [^.]*counters?/ -o:/as (a \|an )?cop(y\|ies) of/` | 57 | Master Biomancer | **test:** `kaito_cast_beside_oath_of_gideon_is_its_controllers_order`: an Oath of Gideon-shaped fixture, whose planeswalker filter CR 306.5b's counters turn off |
| A counter doubler or plus, which applies only once the entry carries counters (`multiplier`) | 616.2, 122.6 | `o:/counters? would be (put\|placed) on\|would put [^.]*counters? on/` | 28 | Doubling Season, Hardened Scales, Primal Vigor, Vorinclex, Monstrous Raider and 1 more | **test:** `shimmerer_under_biomancer_and_doubling_season_reaches_every_order`: the feeds table's (d), 7, 8 or 10 by which writer goes first |
| A type an entry adds (a Mutant, a copy's "in addition"), beside a filter that excludes it (`edit`) | 614.1c, 707.9b | `o:/enters? (the battlefield )?[^.]*(as an? \|is an? \|except it's an? \|except [^.]* is an? )[^.]*in addition to its other types/` | 22 | Master Biomancer | **test:** `master_biomancer_beside_a_non_mutant_filter_is_an_order` |
| An exit or a "can't" that reads the entering object's frame (`exit`) | 614.12, 614.17d | `o:/if (a\|an\|another\|one or more) [^.]*would enter\|can't enter/` | 10 | Containment Priest, Hallowed Moonlight | **test:** `test_containment_priest_reads_the_look_ahead_frame` |
| A copy that adds counters as it applies, its two exceptions reading each other (`copy-extra`) | 616.1c, 707.9e, 707.9f | `o:/enters? (the battlefield )?as (a \|an )?cop(y\|ies)[^.]*except[^.]*(enters? with\|counters?\|tapped)\|enter as cop(y\|ies)[^.]*except[^.]*(with\|counters?)/` | 4 | Spark Double | **register row:** `copy-exception-conditions`: `spark_double_copying_kaito_gets_one_loyalty_counter_and_a_plus_one` |
| A rewrite that changes what happens (a death, a discard or a draw becomes something else), so another effect stops or starts applying (`kind`) | 616.1f, 616.2 | `o:/would (die\|be destroyed\|be put into (a\|your\|an opponent's\|its owner's) graveyard\|be exiled\|leave the battlefield\|draw)[^.]*instead/` | 257 | Alhammarret's Archive, Alms Collector, Bard, King of Dale, Darksteel Colossus and 9 more | **test:** `leyline_first_still_offers_academy_because_the_card_was_still_discarded` |
| A rewrite that changes whom the event affects: damage dealt to another object or player instead (`subject`) | 616.1f, 614.9 | `o:/(is\|are) dealt to [^.]* instead\|deals? that damage to [^.]* instead\|that damage is dealt to [^.]* instead/` | 63 | Palisade Giant, Pariah, Reflect Damage | **test, written here:** `palisade_giant_beside_guardian_seraph_is_an_order_the_redirect_can_end` |
| Two amount rewrites on one event, each reading what the other left (`amount`) | 616.1e, 616.1f | `o:/(twice that many\|that many plus\|half that many\|twice that much\|double that\|that much damage plus\|that much life plus\|deals? (double\|twice) that)/` | 122 | Alhammarret's Archive, Angel of Suffering, Bard, King of Dale, Doubling Season and 9 more | **test:** `mending_hands_beside_furnace_prevents_then_doubles_or_doubles_then_prevents` |
<!-- feedback-loops: end LOOP1 -->

**The Kaito shape** (`self`). Nine of the class change a card type or a
subtype as their counters or status change (§8): Kaito, Arixmethes, Grand
Master of Flowers, Idol of False Gods and Weatherlight Compleated among them.
A planeswalker among them closes the loop alone, since CR 306.5b gives every
planeswalker frame an "enters with" loyalty ability (311 planeswalkers, §8),
and so does a card that prints its own (Arixmethes enters with five slumber
counters); the rest need another writer beside them. The other 52 change
power, toughness or a keyword, which no entry filter reads yet; the feeds
table counts their entry writes as feeding every leaf anyway. All of them
rest on the register's `lookahead-entry-counters`: the look-ahead counts the
counters an entry has already been given, as CR 614.12's text says.

## 3. Loop 2 — a layer effect and CR 613.8's dependency

**Where it runs.** In every layer of every walk, `next_ready` picks the next
effect to apply: one that depends on no effect still waiting, timestamp order
inside a dependency loop (CR 613.8b), and the order read again after each
application (613.8c). **What it skips, and what holds that.** `depends_on`
settles a pair without applying anything when what one reads and what the
other writes share no channel (`Channels`, one bit per characteristic), and
otherwise applies the other under a journal and looks. The channel reads are
exhaustive matches over `ObjectFilter`'s leaves (`filter_reads`) and
`Condition`'s (`condition_reads`), so a new leaf is a compile error there.
`amount_reads` is not: it ends in a wildcard, so a new `AmountExpr` leaf the
walk learns to evaluate would read as independent until its arm is added by
hand (`codebase-state.md` item 217). The channels' grain is a performance
question, `backlog.md` §2.39.

<!-- feedback-loops: begin LOOP2 -->
| Class | CR | Terms | Cards | Registered | Disposition |
|---|---|---|---:|---|---|
| A card-type change over a set chosen by type, beside another card-type change (layer 4) (`types`) | 613.8a, 613.1d | `(o:/(is\|are) [^.]*(artifact\|creature\|enchantment\|land)s? in addition to their other types/ or o:/are [^.]*creatures that are still lands/ or o:/each non(creature\|land\|artifact) [a-z]+ (is\|becomes) an? [^.]*creature/)` | 21 | Ashaya, Soul of the Wild, March of the Machines | **test:** `test_urborg_never_applies_beside_blood_moon_ashaya_and_opalescence`: Opalescence's creatures are what Ashaya makes lands, a CR 613.8b loop with Blood Moon |
| A land-type change, which strips the land's own abilities (CR 305.7), beside a land whose static is itself a type change (`land-type`) | 305.7, 613.8a | `(o:/lands?[^.]* (are\|is) (an? )?(plains\|islands?\|swamps?\|mountains?\|forests?)\b/ or o:/lands?[^.]* (are\|is) every basic land type/)` | 32 | Blood Moon, Everywhere, Urborg, Tomb of Yawgmoth | **test:** `test_urborg_and_blood_moon_in_both_orders` |
| An ability removal beside a static whose existence it ends (CR 604.2) (`lose-all`) | 613.8a, 604.2 | `o:/lose(s)? all (other )?abilities/` | 86 | Humility, Yixlid Jailer | **test:** `test_yixlid_jailer_turns_wonder_off_through_the_existence_check` |
| A grant over a set chosen by an ability ("creatures you control with first strike have double strike"), beside another grant or removal (layer 6) (`ability-filter`) | 613.8a | `(o:/(creatures?\|permanents?)( you control)? with (flying\|first strike\|vigilance\|trample\|deathtouch\|lifelink\|reach\|haste\|menace\|defender\|hexproof) (have\|has) / or o:/controls? an? creature with (flying\|first strike\|trample\|vigilance)[^.]*, this creature has/)` | 7 | — | **backlog:** §2.43: `ObjectFilter` has no leaf that reads an ability or a keyword |
| A text change (layer 3), whose rewrite another effect's text, set or work can hang on (`text`) | 613.8a, 612.1 | `o:/change the text\|text of [^.]* by replacing\|exchange the text boxes\|has the full text/` | 14 | — | **owned:** `layers-architecture.md`: Layer 3 is unbuilt (`backlog.md` §3's row 612), and `Channels` models no text, so its build owes the channel |
<!-- feedback-loops: end LOOP2 -->

## 4. Loop 3 — a state-based action's result and the next check

**Where it runs.** `check_state_based_actions_loop` repeats the check while
it performs anything. A check decides CR 704.5a–c, f–j and m, and 704.6d,
against one board and performs them as one batch (CR 704.3), and past a cap a
check that keeps performing is CR 104.4b's draw. **What it does inside one
check.** Four sweeps read the board after the batch has performed: 704.5n's
and 704.5p's detachments, 704.5q's annihilation and 704.5d's removal. Each
sees what CR 704.3 shows only to the next check, one check early. CR 117.5
places nothing between two checks, so the result is the CR's, except where a
sweep's own condition reads what the batch changed. 704.5n and 704.5p ask
whether a host or an attachment is a creature, and a creature that animates
an attachment, dying in the batch, changes the answer: the CR detaches the
attachment at this check and puts an Aura into the graveyard at the next
(704.5m), and the engine leaves it attached. No registered pair reaches it,
and `codebase-state.md` item 6, which already owns routing these sweeps
through the batch, records it.

<!-- feedback-loops: begin LOOP3 -->
| Class | CR | Terms | Cards | Registered | Disposition |
|---|---|---|---:|---|---|
| A static on a creature or planeswalker that holds other creatures' toughness up (`anthem`) | 704.3, 704.5f, 704.5g | `t:/creature\|planeswalker/ o:/\b(creatures\|[a-z]+s)( you control)? get \+\d+\/\+\d+(?![^.]*until)/` | 303 | — | **test, written here:** `an_animated_glorious_anthem_dies_and_the_next_check_takes_what_it_held_up` |
| A power or toughness that counts creatures or permanents, which falls when others die (`count`) | 704.3, 604.3 | `t:/creature/ o:/(each equal to\|power is equal to\|toughness is equal to\|\+1\/\+1 for each) [^.]*(you control\|on the battlefield)/` | 168 | Ashaya, Soul of the Wild, Keldon Warlord | **test, written here:** `keldon_warlord_dies_on_the_check_after_pyroclasms_other_victims` |
| A creature token a state-based action puts into a graveyard, which CR 704.5d then removes (`token`) | 117.5, 704.5d | `o:/create[^.]* creature tokens?\b/` | 2,474 | Divine Visitation, Elvish Warmaster, Hordeling Outburst, Kalitas, Traitor of Ghet and 2 more | **test, written here:** `a_soldier_token_dies_and_ceases_to_exist_in_one_cycle`, ATOM-117.5-001's board on Raise the Alarm's token. The engine's 704.5d sweep reads after the check's batch, so it lands one check early (`codebase-state.md` item 6) |
| An Aura whose object a state-based action removes or changes (`aura`) | 704.5m, 704.5p | `t:/aura/` | 1,241 | Holy Strength, Pariah, Wild Growth | **test:** `test_an_aura_that_becomes_a_creature_unattaches_then_dies`: 704.5p, then 704.5m on the next pass |
| A static that refuses a loss, which a state-based action can remove (`cant-lose`) | 704.3, 704.5a | `o:/can't lose the game/` | 11 | Platinum Angel | **test, written here:** `platinum_angel_dying_in_the_check_that_refused_the_loss_loses_on_the_next` |
| An ability a state-based action reads that hangs on +1/+1 counters, which CR 704.5q removes (`counter-ability`) | 704.5q | `(o:/with an? \+1\/\+1 counters? on (it\|them)[^.]* (have\|has) [^.]*indestructible/ or o:/as long as [^.]*\+1\/\+1 counter[^.]*indestructible/ or o:/indestructible as long as [^.]*\+1\/\+1 counter/)` | 1 | — | **test:** the path, `keldon_warlord_dies_on_the_check_after_pyroclasms_other_victims` (here), and the rule, `test_battlegrowth_and_chainbreaker_reach_counter_annihilation`; no registered card carries it |
| A player's loss, whose objects leave with them (CR 800.4a) before the next check: a static that reaches every player's creatures, an Aura on another's permanent (`departure`) | 704.3, 800.4a | `o:/^((all\|other\|each other\|each) )?(non)?([a-z-]+ )?(creatures?\|[a-z]+s) get [+-]\d+\/[+-]\d+(?![^.]*until)/` | 52 | — | **test:** `a_creature_leaves_the_game_with_its_owner_even_while_someone_else_controls_it`: 704.5a's loss, CR 800.4a's departure, then 704.5m on the next check |
| A Saga, whose sacrifice waits on its own chapter ability (CR 704.5s): the check reads the trigger queue (`saga`) | 704.5s, 714.4 | `t:/saga/` | 223 | — | **backlog:** §2.44: Sagas and dungeons, the state-based actions that wait on a trigger |
| A battle, whose state-based action waits on its own trigger (CR 704.5v) (`battle`) | 704.5v, 310.4c | `t:/battle/` | 36 | — | **backlog:** §2.23, which gains 704.5v's wait |
<!-- feedback-loops: end LOOP3 -->

## 5. Loop 4 — an event a triggered ability produces, and what else triggers

**Where it runs.** A resolution proposes through the same chokepoint as
everything else (S14), so the window its batch closes is dispatched like any
other (S12, S13): `dispatch_batch` matches the window's events against the
trigger sources' effective abilities and queues what matches, and
`perform_sba_and_triggers` places it at the next CR 117.5 moment. The path is
one whatever the event, so a row whose two halves the engine expresses is
closed by a test of the path,
`soul_wardens_gain_triggers_nykthos_paragon_at_the_next_placement`, and a test
of its arm. **What it skips, and what holds that.** The dispatcher asks only
the sources its gate admits, and TR-1b's audit runs the same match with the
gate off across audited fuzz games (`triggers-architecture.md` §4.10). CR
603.3b's ordering prompt is skipped when no order can change the game (§5.2).

<!-- feedback-loops: begin LOOP4 -->
| Class | CR | Watchers: terms | Watchers | Producers: terms | Producers | Registered | Disposition |
|---|---|---|---:|---|---:|---|---|
| Life gained (`gain`) | 603.2, 119.9 | `o:/whenever (you\|a player\|an opponent\|another player) gains? life/` | 96 | `o:/(when\|whenever\|at the beginning of)[^.]*, [^.]*\b(you\|its controller\|that player\|target player) gains? (\d+\|x\|that much\|life equal)/` | 707 | Blood Artist, Nykthos Paragon, Paladin of Atonement, Soul Warden | **test, written here:** `soul_wardens_gain_triggers_nykthos_paragon_at_the_next_placement` |
| Life lost, by damage or by an instruction (`loss`) | 603.2, 119.3 | `o:/whenever (you\|a player\|an opponent\|each opponent\|another player) loses? life/` | 16 | `(o:/(when\|whenever\|at the beginning of)[^.]*, [^.]*\bloses? (\d+\|x\|that much) life/ or o:/(when\|whenever\|at the beginning of)[^.]*, [^.]*deals? (\d+\|x) damage to (each opponent\|target player\|any target\|target opponent\|that player)/)` | 961 | Blood Artist, Psychosis Crawler, Vengeful Warchief | **test, written here:** `blood_artists_drain_triggers_the_opponents_vengeful_warchief` |
| A permanent entering, a token created (`enter`) | 603.6a, 111.1 | `o:/whenever (a\|an\|another\|one or more) [^.]*\benters?\b/` | 751 | `o:/(when\|whenever\|at the beginning of)[^.]*, [^.]*\bcreate/` | 1,699 | Elvish Warmaster, Soul Warden, Verdant Force | **test:** `warmaster_makes_one_token_however_many_elves_enter`: its own Elf token enters its condition, and "only once each turn" refuses it |
| A creature dying (`dies`) | 603.6c, 700.4 | `o:/whenever [^.]*\b(dies\|die)\b\|whenever [^.]*is put into a graveyard from the battlefield/` | 516 | `o:/(when\|whenever\|at the beginning of)[^.]*, [^.]*\b(sacrifices?\|destroy)\b/` | 1,175 | Blood Artist | **test:** the path, `soul_wardens_gain_triggers_nykthos_paragon_at_the_next_placement` (here), and the arm, `blood_artist_dying_beside_two_creatures_triggers_three_times`; no registered trigger sacrifices or destroys |
| A card drawn (`draw`) | 603.2, 121.1 | `o:/whenever (you\|a player\|an opponent\|each opponent) draws? (a\|your\|their\|one or more)/` | 131 | `o:/(when\|whenever\|at the beginning of)[^.]*, [^.]*\bdraws? (a\|two\|three\|that many\|x\|cards?)\b/` | 1,264 | Psychosis Crawler | **test:** the path (here, as `dies`) and the arm, `psychosis_crawler_cast_from_hand_drains_once_per_card_drawn`; no registered trigger draws |
| Becoming tapped or untapped (`tap`) | 603.2e | `o:/whenever [^.]*becomes (tapped\|untapped)/` | 130 | `o:/(when\|whenever\|at the beginning of)[^.]*, [^.]*\b(tap\|untap) (target\|another\|up to\|all\|each)/` | 308 | — | **test:** the path (here, as `dies`) and the arm, `entering_tapped_is_not_becoming_tapped`; no registered card on either side |
| Counters put on a permanent or a player (`counters-put`) | 603.2, 122.6 | `o:/whenever (one or more )?[^.]*counters? (is\|are) put on/` | 36 | `o:/(when\|whenever\|at the beginning of)[^.]*, [^.]*\bput (a\|an\|one\|two\|three\|x\|that many) [^.]*counters? on/` | 1,567 | Cosi's Trickster, Nykthos Paragon, Paladin of Atonement, Vengeful Warchief | **owned:** TR-5b, combat's shapes, targeting, counters, prevention, the multiplier's second half (`triggers-architecture.md` §12): `CountersPutOn` is unbuilt |
| A spell cast while something resolves: cascade, discover, "you may cast" (`cast`) | 603.2, 702.85a, 701.57a | `o:/whenever (you\|a player\|an opponent) casts?\b/` | 1,187 | `(o:/(when\|whenever\|at the beginning of)[^.]*, [^.]*\byou may cast (it\|that card\|that spell\|the copy\|a copy\|copies\|them\|those cards\|the exiled card)/ or o:/\bcascade\b\|discover \d/)` | 84 | — | **owned:** `permission-architecture.md`, its "cast while something resolves" family |
| An ability triggering: the trigger condition is another ability triggering (`tier-2`) | 603.3b | `o:/whenever [^.]*abilit(y\|ies)[^.]* triggers?\b(?! (an additional\|only once))\|whenever [^.]*causes a triggered ability [^.]*to trigger/` | 4 | — | — | — | **test:** `a_trigger_on_a_trigger_is_placed_in_the_second_tier`, on a Strict Proctor-shaped fixture |
| Becoming the target of an ability as it goes on the stack (ward): CR 603.3b's last sentence, where loop 4 meets loop 3 (`target`) | 603.3b, 702.21a | `o:/becomes the target of [^.]*abilit\|^ward/` | 229 | — | — | — | **owned:** TR-5a, combat's shapes, targeting, counters, prevention, the multiplier's first half: `BecomesTarget` is unbuilt |
| A state trigger, which triggers again once it leaves the stack (CR 603.8) (`state`) | 603.8 | `o:/^when(ever)? (you\|a player\|an opponent) controls? no \|^when [^.]* has no [^.]*counters on it/` | 26 | — | — | — | **owned:** TR-6, state triggers, the loop, and the rule-owned arm |
| A delayed or reflexive trigger a resolution makes (`delayed`) | 603.7, 603.12 | — | — | `o:/at the beginning of (the\|your\|that player's) next\|when you do\b\|when they do\b/` | 693 | — | **owned:** TR-3a and TR-3b, delayed, reflexive, and "until": the producers that come after CV-1b |
<!-- feedback-loops: end LOOP4 -->

CR 603.2d's multipliers (Panharmonicon's "triggers an additional time")
change how many times an ability triggers, reading the event that caused it.
They write nothing a trigger reads, so they are TR-5b's multiplier and not a
class here.

## 6. Searched, and nothing printed

The CR allows each shape below, and no Commander-legal card prints it. Each
was searched in two phrasings, and the last column says what the few hits
are instead; most are a resolution's or a trigger's effect, whose set is
fixed as it resolves (CR 611.2c).

<!-- feedback-loops: begin SEARCHED -->
| Loop | Shape | CR | First phrasing | Cards | Second phrasing | Cards | Why nothing is owed |
|---:|---|---|---|---:|---|---:|---|
| 1 | A copy that becomes applicable once the entry carries counters (CR 616.2), which would put a multiplier between two copies (`codebase-state.md` item 189) (`copy-by-counters`) | 616.2, 707.9e | `o:/enters? (the battlefield )?as (a )?cop(y\|ies)[^.]*(counter\|power\|toughness)/` | 9 | `o:/cop(y\|ies) of [^.]*(if\|as long as\|with) [^.]*counters? on (it\|them)/` | 9 | every hit's counters or power belong to the donor (Deceptive Frostkite, The Master, Volrath) or to the copy's own exception (Spark Double, Moritte, Altered Ego); nothing reads the entering object's counters |
| 2 | A power or toughness bonus over a set chosen by power or toughness, as a static (layer 7c) (`pt-filter`) | 613.8a | `o:/creatures?( you control)? with (base )?(power\|toughness)[^.]* (get\|gets) [+-]\d+\/[+-]\d+(?![^.]*until)/` | 1 | `o:/(each\|all\|other) creatures? with (power\|toughness) \d+ or (less\|greater)[^.]* (get\|gets\|have\|has)(?![^.]*until)/` | 0 | what prints the filter is activated or triggered, a resolution's locked set (CR 611.2c); `filter_reads` puts `PowerLE` on the power channel, so a first static goes to the exact test |
| 2 | A power or toughness that reads another creature's power or toughness, as a static (`pt-amount`) | 613.8a, 604.3 | `o:/(gets \+x\/\+x\|equal to x)[^.]*where x is the (total\|greatest) (power\|toughness) (of\|among) [^.]*creatures you control(?![^.]*until)/` | 1 | `o:/(power\|toughness)[^.]* (is\|are)( each)? equal to the (total\|greatest) (power\|toughness) (of\|among) [^.]*creatures you control/` | 0 | printed only as a resolution's or an entry's amount (Towering Titan's counters); the leaf would be `cost-architecture.md` §3.9's, and `amount_reads` would need its arm by hand (item 217) |
| 2 | A color change over a set chosen by color (layer 5) (`colors`) | 613.8a, 105.3 | `o:/(white\|blue\|black\|red\|green\|colorless\|multicolored\|non(white\|blue\|black\|red\|green)) (creatures\|permanents)[^.]* (is\|are) (white\|blue\|black\|red\|green\|colorless)\b/` | 1 | `o:/(each\|all) (white\|blue\|black\|red\|green\|non(white\|blue\|black\|red\|green)\|colorless\|multicolored) [^.]*(becomes?\|is\|are) [^.]*(white\|blue\|black\|red\|green\|colorless)\b/` | 0 | every printed color setter names its set by type or controller |
| 2 | A control change over a set chosen by who controls it (layer 2) (`control`) | 613.8a, 613.1b | `o:/you control (all\|each) [^.]*(your opponents control\|opponents control\|you don't control)(?![^.]*(until\|this turn))/` | 2 | `-t:/instant\|sorcery/ o:/(gain\|gains) control of (all\|each) (?![^.]*(until\|this turn))/` | 21 | every printed control change names a fixed object or locks its set as it resolves (CR 611.2c) |
| 4 | A triggered mana ability's mana triggering another (CR 605.4a), resolved without the stack (`mana`) | 605.1b, 605.4a | `o:/whenever [^.]*causes (you\|a player\|its controller) to add/` | 1 | `t:/land/ o:/whenever [^.]*(tapped\|tap [^.]*) for mana, [^.]*add/` | 0 | the one watcher, Caged Sun, names a land's ability, and no land has a triggered mana ability, so no chain prints; MA-3 owns what runs inside a mana ability |
<!-- feedback-loops: end SEARCHED -->

The hits: Wargling (a trigger behind an ability word), Tuya Bearclaw (a
trigger, "until end of turn"), Ghostly Flame (a rule about damage sources,
not a color change), Friendly Rivalry and Graceful Takedown (spells), the
second control phrasing's 21 (triggers, activated and loyalty abilities, and
one Saga's chapter), and Caged Sun (§1's third widening).

Layer 1 is absent on purpose: a copy is a snapshot (CR 707.2b), so no copy
effect depends on another (`copy-effects-architecture.md` §5.2). Merged
permanents are a live relationship, and CV-7 (merging and meld) asks the
question again when it builds them.

## 7. The residual — what the class queries found that closes no loop

<!-- feedback-loops: begin RESIDUAL -->
| Found | Terms | Cards | Why it closes no loop |
|---|---|---:|---|
| A condition on its own counters or tapped status whose effect is not on its own characteristics: a rule it changes, or other objects (`rule-or-others`) | `o:/as long as [^,.]+ (has [^.]*counters? on (it\|him\|her)\|is (tapped\|untapped))/ -(o:/as long as [^,.]+ has [^.]*counters? on (it\|him\|her), (it\|he\|she\|this [a-z]+)('s\| is\| gets\| has\| loses)/ or o:/as long as [^,.]+ is (tapped\|untapped), (it\|he\|she\|this [a-z]+)('s\| is\| gets\| has\| loses)/ or o:/(this [a-z]+\|^[^,.]+) (has\|gets\|is) [^.]* as long as (it\|he\|she) (has [^.]*counters? on\|is (tapped\|untapped))/ or o:/^level \d/)` | 42 | no layer atom on the entering object, so no entry write changes what the CR 614.12 frame gathers; a rule-changing static reads its counters when the rule is asked, later |
| An effect that lasts for as long as a counter stays (CR 611.2b) (`duration`) | `o:/for as long as (it\|that creature\|that land\|that permanent\|each of those [a-z]+\|they) (has\|have) an? [a-z]+ counters? on (it\|them)/` | 17 | the counter is removed by an activation or a trigger in a later pass; the walk only reads whether it is still there |
<!-- feedback-loops: end RESIDUAL -->

## 8. Detail counts the sections cite

<!-- feedback-loops: begin DETAILS -->
| Detail | Terms | Cards |
|---|---|---:|
| of the Kaito shape, those whose hang changes a card type or a subtype (`self-type`) | `(o:/as long as [^,.]+ has [^.]*counters? on (it\|him\|her), [^.]*in addition to its other types/ or o:/as long as [^,.]+ has [^.]*counters? on (it\|him\|her), (it\|he\|she)('s\| is) an? [^.]*(creature\|land)/ or o:/as long as [^,.]+ is (tapped\|untapped), (it\|he\|she)('s\| is) an? [^.]*(creature\|land)/) -o:/for as long as (that\|those) [a-z]+ (has\|have)/` | 9 |
| every planeswalker, each of which CR 306.5b gives an "enters with" loyalty ability (`planeswalkers`) | `t:/planeswalker/` | 311 |
<!-- feedback-loops: end DETAILS -->

## 9. Calibration — the registered cards

The same terms read as regexes over the registered cards' Oracle text. A
registered card in a class the census calls inexpressible would be a
contradiction: a backlog or owned row whose surface the engine already plays.
For loop 4 only the half the disposition says is missing counts.

<!-- feedback-loops: begin CALIBRATION -->
181 names registered in `mtgsim/src/cards/registry.rs`; Scryfall knows 180 of them, and the rest are fixtures: Loyalty Probe.
56 of them fall in at least one class.

No class that the census calls inexpressible has a registered card in it.
<!-- feedback-loops: end CALIBRATION -->

## 10. What the census settles, and what it leaves

<!-- feedback-loops: begin SUMMARY -->
| Loop | Classes | Test, cited | Test, written here | Register row | Backlog | Owned | Searched, none printed |
|---|---:|---:|---:|---:|---:|---:|---:|
| 1. An entry's writes and CR 616.1's re-gather | 10 | 7 | 1 | 2 | 0 | 0 | 1 |
| 2. A layer effect and CR 613.8's dependency | 5 | 3 | 0 | 0 | 1 | 1 | 4 |
| 3. A state-based action's result and the next check | 9 | 3 | 4 | 0 | 2 | 0 | 0 |
| 4. An event a triggered ability produces, and what else triggers | 12 | 5 | 2 | 0 | 0 | 5 | 1 |
| **all** | **36** | **18** | **7** | **2** | **3** | **6** | **6** |
<!-- feedback-loops: end SUMMARY -->

- **Seven tests written here**, on registered cards: the redirect that ends
  a prevention's applicability (Palisade Giant beside Guardian Seraph), four
  state-based boards (an animated Glorious Anthem, Keldon Warlord after
  Pyroclasm, Platinum Angel, and a Soldier token on ATOM-117.5-001's board),
  and two chains (Soul Warden into Nykthos Paragon, Blood Artist into an
  opponent's Vengeful Warchief).
- **The register is unchanged.** Its two rows are loop 1's, and no other
  class found a ruling against the CR.
- **Backlog:** §2.43, a filter that reads an ability or a keyword (loop 2),
  and §2.44, Sagas and dungeons (loops 3 and 4), are new; §2.23 gains
  704.5v's wait.
- **Owned:** TR-3a and TR-3b, TR-5a, TR-5b, TR-6, `permission-architecture.md`
  and Layer 3 each own a class whose surface is unbuilt, and each loop's
  architecture document points here.
- **Engine findings, recorded rather than fixed:** `amount_reads`' wildcard
  (item 217) and the post-batch sweeps (item 6).

## 11. What the census cannot see

- Its queries read Oracle text. A feedback in the rules and not the words,
  such as CR 306.5b's loyalty on every planeswalker, is a detail count, not a
  class query.
- A new card that prints a feedback in new words is found only if a phrasing
  here catches it; the residual is the first place to look.
- The counts say how often a loop's input prints, not how often a game
  reaches it, nor what it would cost.

## 12. The rule numbers, audited

<!-- feedback-loops: begin LABELS -->
Every rule number the tables cite, resolved in `MTG-Rules/versions/tmnt.txt`: 105.3, 110.5b, 111.1, 117.5, 119.3, 119.9, 121.1, 122.6, 305.7, 310.4c, 603.2, 603.2e, 603.3b, 603.6a, 603.6c, 603.7, 603.8, 603.12, 604.2, 604.3, 605.1b, 605.4a, 612.1, 613.1b, 613.1d, 613.8a, 613.8c, 614.1c, 614.9, 614.12, 614.17d, 616.1c, 616.1e, 616.1f, 616.2, 700.4, 701.57a, 702.21a, 702.85a, 704.3, 704.5a, 704.5d, 704.5f, 704.5g, 704.5m, 704.5p, 704.5q, 704.5s, 704.5v, 707.9b, 707.9e, 707.9f, 711.2, 714.4, 800.4a.
<!-- feedback-loops: end LABELS -->
