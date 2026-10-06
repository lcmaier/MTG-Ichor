#!/usr/bin/env python3
"""The feedback-loop census -- every table in `plans/references/feedback-loops.md`.

    python plans/references/feedback-loops.py               # print the tables
    python plans/references/feedback-loops.py --write       # splice them into feedback-loops.md between its markers
    python plans/references/feedback-loops.py --names ID    # the first page of names behind one class (ID, or ID:producers)
    python plans/references/feedback-loops.py --names Q     # ... or behind a raw query, which BASE is prefixed to
    python plans/references/feedback-loops.py --calibrate   # the registered cards in each class, and every contradiction
    python plans/references/feedback-loops.py --check       # each count beside the same regex over the cast census's corpus
    python plans/references/feedback-loops.py --no-fetch    # caches only; a query the cache lacks is an error
    python plans/references/feedback-loops.py --refresh     # drop the caches first

WHY THIS EXISTS
---------------
A loop, here, is a write that can change a decision the same pass is still
making: CR 616.1f's re-gather, CR 613.8c's re-sort, CR 704.3's re-check, and
the dispatch after every batch that CR 603.2 and 603.3b describe. Kaito, Bane
of Nightmares beside Spark Double closed the first one, and a judges' thread
found it, not the engine (`copy-effects-architecture.md` §4.1a). This census
lists, per loop, the card classes that close it, so the next such card is a
query result rather than a thread.

WHAT A CLASS IS
---------------
A family of printed cards that supplies the write, the decision that reads
it, or both, of one feedback the loop's re-run has to see. A class is one
row: its rules, the Scryfall terms it adds to `BASE`, its `total_cards`, the
registered cards that fall in it, and one disposition. Loop 4's rows count
both halves, the triggers that watch an event and the triggers whose
resolution produces it. `SEARCHED` holds the shapes the CR allows that print
nothing, each tried in two phrasings; `RESIDUAL` holds what the class queries
found that closes no loop.

Each class's terms are a list of Scryfall terms ANDed after `BASE`, and only
three forms are used, `o:/re/`, `-o:/re/` and `t:/re/`, under `or` groups,
so `--calibrate` and `--check` read the same strings as regexes over oracle
text with reminder text removed. `[^.]` stands for "within the sentence":
Scryfall's negated brackets stop at a paragraph's end, so the local reading
adds the newline, and with it `--check` matched every count of the run the
doc records. Scryfall refuses a long regex as "too complex" and says so only
in a warning, which `count` turns into an error; a long shape is several
short regexes under one `or`.

THE DISPOSITIONS
----------------
  test      a test that closes the loop on a registered card or a cheap
            fixture, cited by name (`here` marks one this census wrote);
  register  a row of `engineering-practices.md` §3.4b, where a ruling and the
            CR disagree;
  backlog   a `backlog.md` entry, where the engine has no surface yet and
            nothing owns one;
  owned     the surface is missing and a slotted phase already owns it, so a
            backlog entry would be a second owner (`backlog.md` §0).
The script refuses a cited test that does not exist and a register row that
is not in the register.

WHAT THE NUMBERS ARE NOT
------------------------
Not a work estimate, and not the size of a risk: a class of a thousand cards
the engine closes by construction costs nothing, and a class of one can be
the Kaito board. The counts say how often a loop's input is printed.

WHEN TO DELETE THIS FILE
------------------------
When every class is a cited test or a register row and the tables have no
customer -- or when the counts are stale enough that a reader would trust a
fresh look over them, which a comment cannot prevent.

Results are cached in this directory (gitignored, `.census-*.json`). Scryfall
asks for a courteous request rate and bans a burst; the delay is deliberate.
"""
import argparse
import datetime as _dt
import json
import os
import re
import subprocess
import sys
import time
import urllib.parse

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
DOC = os.path.join(HERE, "feedback-loops.md")
COUNTS = os.path.join(HERE, ".census-loops-counts.json")
REGISTERED_CACHE = os.path.join(HERE, ".census-loops-registered.json")
CAST_CORPUS = os.path.join(HERE, ".census-cast.json")
CAST_REGISTERED = os.path.join(HERE, ".census-cast-registered.json")
CR = os.path.join(ROOT, "MTG-Rules", "versions", "tmnt.txt")
CRATE = os.path.join(ROOT, "mtgsim")
REGISTRY = os.path.join(CRATE, "src", "cards", "registry.rs")
PRACTICES = os.path.join(ROOT, "plans", "engineering-practices.md")
BACKLOG = os.path.join(ROOT, "plans", "backlog.md")

UA = "mtgsim-research/1.0 (contact: maiercluke@gmail.com)"
BASE = "legal:commander -t:token game:paper -is:funny"
DELAY = 0.35

_no_fetch = False

# ==========================================================================
# The loops
# ==========================================================================

LOOPS = {
    1: ("An entry's writes and CR 616.1's re-gather", "616.1f"),
    2: ("A layer effect and CR 613.8's dependency", "613.8c"),
    3: ("A state-based action's result and the next check", "704.3"),
    4: ("An event a triggered ability produces, and what else triggers", "603.3b"),
}

# The sentence a pronoun or a name may open: "this creature", "it", "Kaito".
SELF = r"[^,.]+"
# A triggered ability's opening, up to its comma.
TRIG = r"(when|whenever|at the beginning of)[^.]*, "
# A static on its own counters or tapped status that changes its own
# characteristics, either word order, and CR 711's leveler lines. Four
# regexes under one `or`, because Scryfall refuses the one regex as "too
# complex" and says so only in a warning.
SELF_CLOSER = ("or", [
    r"o:/as long as " + SELF + r" has [^.]*counters? on (it|him|her), (it|he|she|this [a-z]+)('s| is| gets| has| loses)/",
    r"o:/as long as " + SELF + r" is (tapped|untapped), (it|he|she|this [a-z]+)('s| is| gets| has| loses)/",
    r"o:/(this [a-z]+|^[^,.]+) (has|gets|is) [^.]* as long as (it|he|she) (has [^.]*counters? on|is (tapped|untapped))/",
    r"o:/^level \d/",
])
# "For as long as that creature has a shadow counter on it": a resolution's
# duration on another object, which the self-closer's first regex also reads
# (the residual's `duration`).
NOT_A_DURATION = r"-o:/for as long as (that|those) [a-z]+ (has|have)/"

# ==========================================================================
# The classes. `(loop, id, class, rules, terms, disposition)`. Loop 4's
# terms are the watchers', and two more fields follow: the producers' terms,
# and which side the disposition says the engine cannot express yet; a side a
# row does not count is `None`. A disposition is `(kind, text)`; backticked
# snake_case names in a `test` text must be test functions.
# ==========================================================================

CLASSES = [
    # ---- loop 1 -----------------------------------------------------------
    (1, "self", "A permanent whose own characteristics hang on its counters or its tapped status (the Kaito shape)",
     ("614.12", "616.1f", "711.2"),
     [SELF_CLOSER, NOT_A_DURATION],
     ("register", "`lookahead-entry-counters`, whose third board is Kaito's: "
                  "`kaito_cast_beside_oath_of_gideon_is_its_controllers_order`, "
                  "`kaito_enters_tapped_under_an_opponents_creatures_enter_tapped`. The feeds table's last row "
                  "(`Condition::reads_entry_state`) is what makes the order a question")),
    (1, "status", "A status every entering object of a kind gets: enters tapped, or untapped",
     ("614.1c", "110.5b"),
     [r"o:/(creatures|artifacts|lands|permanents|planeswalkers|enchantments)[^.]* enter (the battlefield )?(tapped|untapped)/"],
     ("test", "`two_archelos_in_opposite_states_ask_the_entering_permanents_controller`: the feeds table's (c), "
              "two statuses that set opposite values are an order")),
    (1, "entry-counters", "Counters every entering object of a kind gets (\"enters with an additional counter\")",
     ("614.1c", "122.6"),
     [r"o:/(you control|other creature|each creature|each other|planeswalkers?|creatures?)[^.]* enters? (the battlefield )?with [^.]*additional [^.]*counters?/",
      r"-o:/as (a |an )?cop(y|ies) of/"],
     ("test", "`kaito_cast_beside_oath_of_gideon_is_its_controllers_order`: an Oath of Gideon-shaped fixture, "
              "whose planeswalker filter CR 306.5b's counters turn off")),
    (1, "multiplier", "A counter doubler or plus, which applies only once the entry carries counters",
     ("616.2", "122.6"),
     [r"o:/counters? would be (put|placed) on|would put [^.]*counters? on/"],
     ("test", "`shimmerer_under_biomancer_and_doubling_season_reaches_every_order`: the feeds table's (d), "
              "7, 8 or 10 by which writer goes first")),
    (1, "edit", "A type an entry adds (a Mutant, a copy's \"in addition\"), beside a filter that excludes it",
     ("614.1c", "707.9b"),
     [r"o:/enters? (the battlefield )?[^.]*(as an? |is an? |except it's an? |except [^.]* is an? )[^.]*in addition to its other types/"],
     ("test", "`master_biomancer_beside_a_non_mutant_filter_is_an_order`")),
    (1, "exit", "An exit or a \"can't\" that reads the entering object's frame",
     ("614.12", "614.17d"),
     [r"o:/if (a|an|another|one or more) [^.]*would enter|can't enter/"],
     ("test", "`test_containment_priest_reads_the_look_ahead_frame`")),
    (1, "copy-extra", "A copy that adds counters as it applies, its two exceptions reading each other",
     ("616.1c", "707.9e", "707.9f"),
     [r"o:/enters? (the battlefield )?as (a |an )?cop(y|ies)[^.]*except[^.]*(enters? with|counters?|tapped)|enter as cop(y|ies)[^.]*except[^.]*(with|counters?)/"],
     ("register", "`copy-exception-conditions`: "
                  "`spark_double_copying_kaito_gets_one_loyalty_counter_and_a_plus_one`")),
    (1, "kind", "A rewrite that changes what happens (a death, a discard or a draw becomes something else), "
                "so another effect stops or starts applying",
     ("616.1f", "616.2"),
     [r"o:/would (die|be destroyed|be put into (a|your|an opponent's|its owner's) graveyard|be exiled|leave the battlefield|draw)[^.]*instead/"],
     ("test", "`leyline_first_still_offers_academy_because_the_card_was_still_discarded`")),
    (1, "subject", "A rewrite that changes whom the event affects: damage dealt to another object or player instead",
     ("616.1f", "614.9"),
     [r"o:/(is|are) dealt to [^.]* instead|deals? that damage to [^.]* instead|that damage is dealt to [^.]* instead/"],
     ("test", "here: `palisade_giant_beside_guardian_seraph_is_an_order_the_redirect_can_end`")),
    (1, "amount", "Two amount rewrites on one event, each reading what the other left",
     ("616.1e", "616.1f"),
     [r"o:/(twice that many|that many plus|half that many|twice that much|double that|that much damage plus|that much life plus|deals? (double|twice) that)/"],
     ("test", "`mending_hands_beside_furnace_prevents_then_doubles_or_doubles_then_prevents`")),
    # ---- loop 2 -----------------------------------------------------------
    (2, "types", "A card-type change over a set chosen by type, beside another card-type change (layer 4)",
     ("613.8a", "613.1d"),
     [("or", [r"o:/(is|are) [^.]*(artifact|creature|enchantment|land)s? in addition to their other types/",
              r"o:/are [^.]*creatures that are still lands/",
              r"o:/each non(creature|land|artifact) [a-z]+ (is|becomes) an? [^.]*creature/"])],
     ("test", "`test_urborg_never_applies_beside_blood_moon_ashaya_and_opalescence`: Opalescence's creatures "
              "are what Ashaya makes lands, a CR 613.8b loop with Blood Moon")),
    (2, "land-type", "A land-type change, which strips the land's own abilities (CR 305.7), beside a land whose "
                     "static is itself a type change",
     ("305.7", "613.8a"),
     [("or", [r"o:/lands?[^.]* (are|is) (an? )?(plains|islands?|swamps?|mountains?|forests?)\b/",
              r"o:/lands?[^.]* (are|is) every basic land type/"])],
     ("test", "`test_urborg_and_blood_moon_in_both_orders`")),
    (2, "lose-all", "An ability removal beside a static whose existence it ends (CR 604.2)",
     ("613.8a", "604.2"),
     [r"o:/lose(s)? all (other )?abilities/"],
     ("test", "`test_yixlid_jailer_turns_wonder_off_through_the_existence_check`")),
    (2, "ability-filter", "A grant over a set chosen by an ability (\"creatures you control with first strike have "
                          "double strike\"), beside another grant or removal (layer 6)",
     ("613.8a",),
     [("or", [r"o:/(creatures?|permanents?)( you control)? with (flying|first strike|vigilance|trample|deathtouch|lifelink|reach|haste|menace|defender|hexproof) (have|has) /",
              r"o:/controls? an? creature with (flying|first strike|trample|vigilance)[^.]*, this creature has/"])],
     ("backlog", "§2.43: `ObjectFilter` has no leaf that reads an ability or a keyword")),
    (2, "text", "A text change (layer 3), whose rewrite another effect's text, set or work can hang on",
     ("613.8a", "612.1"),
     [r"o:/change the text|text of [^.]* by replacing|exchange the text boxes|has the full text/"],
     ("owned", "`layers-architecture.md`: Layer 3 is unbuilt (`backlog.md` §3's row 612), and `Channels` "
               "models no text, so its build owes the channel")),
    # ---- loop 3 -----------------------------------------------------------
    (3, "anthem", "A static on a creature or planeswalker that holds other creatures' toughness up",
     ("704.3", "704.5f", "704.5g"),
     ["t:/creature|planeswalker/",
      r"o:/\b(creatures|[a-z]+s)( you control)? get \+\d+\/\+\d+(?![^.]*until)/"],
     ("test", "here: `an_animated_glorious_anthem_dies_and_the_next_check_takes_what_it_held_up`")),
    (3, "count", "A power or toughness that counts creatures or permanents, which falls when others die",
     ("704.3", "604.3"),
     ["t:/creature/",
      r"o:/(each equal to|power is equal to|toughness is equal to|\+1\/\+1 for each) [^.]*(you control|on the battlefield)/"],
     ("test", "here: `keldon_warlord_dies_on_the_check_after_pyroclasms_other_victims`")),
    (3, "token", "A creature token a state-based action puts into a graveyard, which CR 704.5d then removes",
     ("117.5", "704.5d"),
     [r"o:/create[^.]* creature tokens?\b/"],
     ("test", "here: `a_soldier_token_dies_and_ceases_to_exist_in_one_cycle`, ATOM-117.5-001's board on Raise the "
              "Alarm's token. The engine's 704.5d sweep reads after the check's batch, so it lands one check early "
              "(`codebase-state.md` item 6)")),
    (3, "aura", "An Aura whose object a state-based action removes or changes",
     ("704.5m", "704.5p"),
     ["t:/aura/"],
     ("test", "`test_an_aura_that_becomes_a_creature_unattaches_then_dies`: 704.5p, then 704.5m on the next pass")),
    (3, "cant-lose", "A static that refuses a loss, which a state-based action can remove",
     ("704.3", "704.5a"),
     [r"o:/can't lose the game/"],
     ("test", "here: `platinum_angel_dying_in_the_check_that_refused_the_loss_loses_on_the_next`")),
    (3, "counter-ability", "An ability a state-based action reads that hangs on +1/+1 counters, which CR 704.5q removes",
     ("704.5q",),
     [("or", [r"o:/with an? \+1\/\+1 counters? on (it|them)[^.]* (have|has) [^.]*indestructible/",
              r"o:/as long as [^.]*\+1\/\+1 counter[^.]*indestructible/",
              r"o:/indestructible as long as [^.]*\+1\/\+1 counter/"])],
     ("test", "the one card is Voice of the Blessed (indestructible at ten +1/+1 counters), which is not registered, "
              "and no test builds its board. The next check's read is pinned by "
              "`keldon_warlord_dies_on_the_check_after_pyroclasms_other_victims` (here), and the annihilation by "
              "`test_battlegrowth_and_chainbreaker_reach_counter_annihilation`")),
    (3, "departure", "A player's loss, whose objects leave with them (CR 800.4a) before the next check: "
                     "a static that reaches every player's creatures, an Aura on another's permanent",
     ("704.3", "800.4a"),
     [r"o:/^((all|other|each other|each) )?(non)?([a-z-]+ )?(creatures?|[a-z]+s) get [+-]\d+\/[+-]\d+(?![^.]*until)/"],
     ("test", "`a_creature_leaves_the_game_with_its_owner_even_while_someone_else_controls_it`: 704.5a's "
              "loss, CR 800.4a's departure, then 704.5m on the next check")),
    (3, "saga", "A Saga, whose sacrifice waits on its own chapter ability (CR 704.5s): the check reads the "
                "trigger queue",
     ("704.5s", "714.4"),
     ["t:/saga/"],
     ("backlog", "§2.44: Sagas and dungeons, the state-based actions that wait on a trigger")),
    (3, "battle", "A battle, whose state-based action waits on its own trigger (CR 704.5v)",
     ("704.5v", "310.4c"),
     ["t:/battle/"],
     ("backlog", "§2.23, which gains 704.5v's wait")),
    # ---- loop 4: (watchers, producers) ------------------------------------
    (4, "gain", "Life gained", ("603.2", "119.9"),
     [r"o:/whenever (you|a player|an opponent|another player) gains? life/"],
     ("test", "here: `soul_wardens_gain_triggers_nykthos_paragon_at_the_next_placement`"),
     ["o:/" + TRIG + r"[^.]*\b(you|its controller|that player|target player) gains? (\d+|x|that much|life equal)/"]),
    (4, "loss", "Life lost, by damage or by an instruction", ("603.2", "119.3"),
     [r"o:/whenever (you|a player|an opponent|each opponent|another player) loses? life/"],
     ("test", "here: `blood_artists_drain_triggers_the_opponents_vengeful_warchief`"),
     [("or", ["o:/" + TRIG + r"[^.]*\bloses? (\d+|x|that much) life/",
              "o:/" + TRIG + r"[^.]*deals? (\d+|x) damage to (each opponent|target player|any target|target opponent|that player)/"])]),
    (4, "enter", "A permanent entering, a token created", ("603.6a", "111.1"),
     [r"o:/whenever (a|an|another|one or more) [^.]*\benters?\b/"],
     ("test", "`warmaster_makes_one_token_however_many_elves_enter`: its own Elf token enters its condition, and "
              "\"only once each turn\" refuses it"),
     ["o:/" + TRIG + r"[^.]*\bcreate/"]),
    (4, "dies", "A creature dying", ("603.6c", "700.4"),
     [r"o:/whenever [^.]*\b(dies|die)\b|whenever [^.]*is put into a graveyard from the battlefield/"],
     ("test", "the path, `soul_wardens_gain_triggers_nykthos_paragon_at_the_next_placement` (here), and the "
              "arm, `blood_artist_dying_beside_two_creatures_triggers_three_times`; no registered trigger sacrifices "
              "or destroys"),
     ["o:/" + TRIG + r"[^.]*\b(sacrifices?|destroy)\b/"]),
    (4, "draw", "A card drawn", ("603.2", "121.1"),
     [r"o:/whenever (you|a player|an opponent|each opponent) draws? (a|your|their|one or more)/"],
     ("test", "the path (here, as `dies`) and the arm, `psychosis_crawler_cast_from_hand_drains_once_per_card_drawn`; "
              "no registered trigger draws"),
     ["o:/" + TRIG + r"[^.]*\bdraws? (a|two|three|that many|x|cards?)\b/"]),
    (4, "tap", "Becoming tapped or untapped", ("603.2e",),
     [r"o:/whenever [^.]*becomes (tapped|untapped)/"],
     ("test", "the path (here, as `dies`) and the arm, `entering_tapped_is_not_becoming_tapped`; no registered "
              "card on either side"),
     ["o:/" + TRIG + r"[^.]*\b(tap|untap) (target|another|up to|all|each)/"]),
    (4, "counters-put", "Counters put on a permanent or a player", ("603.2", "122.6"),
     [r"o:/whenever (one or more )?[^.]*counters? (is|are) put on/"],
     ("owned", "TR-5b, combat's shapes, targeting, counters, prevention, the multiplier's second half "
               "(`triggers-architecture.md` §12): `CountersPutOn` is unbuilt"),
     ["o:/" + TRIG + r"[^.]*\bput (a|an|one|two|three|x|that many) [^.]*counters? on/"], "watchers"),
    (4, "cast", "A spell cast while something resolves: cascade, discover, \"you may cast\"", ("603.2", "702.85a", "701.57a"),
     [r"o:/whenever (you|a player|an opponent) casts?\b/"],
     ("owned", "`permission-architecture.md`, its \"cast while something resolves\" family"),
     [("or", ["o:/" + TRIG + r"[^.]*\byou may cast (it|that card|that spell|the copy|a copy|copies|them|those cards|the exiled card)/",
              r"o:/\bcascade\b|discover \d/"])], "producers"),
    (4, "tier-2", "An ability triggering: the trigger condition is another ability triggering", ("603.3b",),
     [r"o:/whenever [^.]*abilit(y|ies)[^.]* triggers?\b(?! (an additional|only once))|whenever [^.]*causes a triggered ability [^.]*to trigger/"],
     ("test", "`a_trigger_on_a_trigger_is_placed_in_the_second_tier`, on a Strict Proctor-shaped fixture"),
     None),
    (4, "target", "Becoming the target of an ability as it goes on the stack (ward): CR 603.3b's last sentence, "
                  "where loop 4 meets loop 3",
     ("603.3b", "702.21a"),
     [r"o:/becomes the target of [^.]*abilit|^ward/"],
     ("owned", "TR-5a, combat's shapes, targeting, counters, prevention, the multiplier's first half: "
               "`BecomesTarget` is unbuilt"),
     None, "watchers"),
    (4, "state", "A state trigger, which triggers again once it leaves the stack (CR 603.8)", ("603.8",),
     [r"o:/^when(ever)? (you|a player|an opponent) controls? no |^when [^.]* has no [^.]*counters on it/"],
     ("owned", "TR-6, state triggers, the loop, and the rule-owned arm"),
     None, "watchers"),
    (4, "delayed", "A delayed or reflexive trigger a resolution makes", ("603.7", "603.12"),
     None,
     ("owned", "TR-3a and TR-3b, delayed, reflexive, and \"until\": the producers that come after CV-1b"),
     [r"o:/at the beginning of (the|your|that player's) next|when you do\b|when they do\b/"], "producers"),
]

# Shapes the CR allows that print nothing. `(loop, id, shape, rules, phrasing 1, phrasing 2, why)`.
SEARCHED = [
    (1, "copy-by-counters", "A copy that becomes applicable once the entry carries counters (CR 616.2), "
                            "which would put a multiplier between two copies (`codebase-state.md` item 189)",
     ("616.2", "707.9e"),
     [r"o:/enters? (the battlefield )?as (a )?cop(y|ies)[^.]*(counter|power|toughness)/"],
     [r"o:/cop(y|ies) of [^.]*(if|as long as|with) [^.]*counters? on (it|them)/"],
     "every hit's counters or power belong to the donor (Deceptive Frostkite, The Master, Volrath) or to the "
     "copy's own exception (Spark Double, Moritte, Altered Ego); nothing reads the entering object's counters"),
    (2, "pt-filter", "A power or toughness bonus over a set chosen by power or toughness, as a static (layer 7c)",
     ("613.8a",),
     [r"o:/creatures?( you control)? with (base )?(power|toughness)[^.]* (get|gets) [+-]\d+\/[+-]\d+(?![^.]*until)/"],
     [r"o:/(each|all|other) creatures? with (power|toughness) \d+ or (less|greater)[^.]* (get|gets|have|has)(?![^.]*until)/"],
     "what prints the filter is activated or triggered, a resolution's locked set (CR 611.2c); `filter_reads` "
     "puts `PowerLE` on the power channel, so a first static goes to the exact test"),
    (2, "pt-amount", "A power or toughness that reads another creature's power or toughness, as a static",
     ("613.8a", "604.3"),
     [r"o:/(gets \+x\/\+x|equal to x)[^.]*where x is the (total|greatest) (power|toughness) (of|among) [^.]*creatures you control(?![^.]*until)/"],
     [r"o:/(power|toughness)[^.]* (is|are)( each)? equal to the (total|greatest) (power|toughness) (of|among) [^.]*creatures you control/"],
     "printed only as a resolution's or an entry's amount (Towering Titan's counters); the leaf would be "
     "`cost-architecture.md` §3.9's, and `amount_reads` would need its arm by hand (item 217)"),
    (2, "colors", "A color change over a set chosen by color (layer 5)", ("613.8a", "105.3"),
     [r"o:/(white|blue|black|red|green|colorless|multicolored|non(white|blue|black|red|green)) (creatures|permanents)[^.]* (is|are) (white|blue|black|red|green|colorless)\b/"],
     [r"o:/(each|all) (white|blue|black|red|green|non(white|blue|black|red|green)|colorless|multicolored) [^.]*(becomes?|is|are) [^.]*(white|blue|black|red|green|colorless)\b/"],
     "every printed color setter names its set by type or controller"),
    (2, "control", "A control change over a set chosen by who controls it (layer 2)", ("613.8a", "613.1b"),
     [r"o:/you control (all|each) [^.]*(your opponents control|opponents control|you don't control)(?![^.]*(until|this turn))/"],
     ["-t:/instant|sorcery/", r"o:/(gain|gains) control of (all|each) (?![^.]*(until|this turn))/"],
     "every printed control change names a fixed object or locks its set as it resolves (CR 611.2c)"),
    (4, "mana", "A triggered mana ability's mana triggering another (CR 605.4a), resolved without the stack",
     ("605.1b", "605.4a"),
     [r"o:/whenever [^.]*causes (you|a player|its controller) to add/"],
     ["t:/land/", r"o:/whenever [^.]*(tapped|tap [^.]*) for mana, [^.]*add/"],
     "the one watcher, Caged Sun, names a land's ability, and no land has a triggered mana ability, so no "
     "chain prints; MA-3 owns what runs inside a mana ability"),
]

# What the class queries found that closes no loop. `(id, what, terms, why)`.
RESIDUAL = [
    ("rule-or-others", "A condition on its own counters or tapped status whose effect is not on its own "
                       "characteristics: a rule it changes, or other objects",
     [r"o:/as long as [^,.]+ (has [^.]*counters? on (it|him|her)|is (tapped|untapped))/",
      ("nor", SELF_CLOSER[1])],
     "no layer atom on the entering object, so no entry write changes what the CR 614.12 frame gathers; a "
     "rule-changing static reads its counters when the rule is asked, later"),
    ("duration", "An effect that lasts for as long as a counter stays (CR 611.2b)",
     [r"o:/for as long as (it|that creature|that land|that permanent|each of those [a-z]+|they) (has|have) an? [a-z]+ counters? on (it|them)/"],
     "the counter is removed by an activation or a trigger in a later pass; the walk only reads whether it "
     "is still there"),
]

# Detail counts a section's text cites. `(id, what, terms)`.
DETAILS = [
    ("self-type", "of the Kaito shape, those whose hang changes a card type or a subtype",
     [("or", [r"o:/as long as [^,.]+ has [^.]*counters? on (it|him|her), [^.]*in addition to its other types/",
              r"o:/as long as [^,.]+ has [^.]*counters? on (it|him|her), (it|he|she)('s| is) an? [^.]*(creature|land)/",
              r"o:/as long as [^,.]+ is (tapped|untapped), (it|he|she)('s| is) an? [^.]*(creature|land)/"]),
      NOT_A_DURATION]),
    ("planeswalkers", "every planeswalker, each of which CR 306.5b gives an \"enters with\" loyalty ability",
     ["t:/planeswalker/"]),
]

# ==========================================================================
# Scryfall
# ==========================================================================


def _get(url):
    """curl with a UA header: Scryfall 403s a bare request."""
    for attempt in range(6):
        out = subprocess.run(["curl", "-s", "-A", UA, url], capture_output=True,
                             text=True, encoding="utf-8").stdout
        try:
            d = json.loads(out)
        except json.JSONDecodeError:
            time.sleep(5 * (attempt + 1))
            continue
        if d.get("object") == "error" and d.get("status") == 429:
            time.sleep(8 * (attempt + 1))
            continue
        return d
    raise RuntimeError("rate limited out: " + url)


def _post(url, body):
    for attempt in range(6):
        out = subprocess.run(["curl", "-s", "-A", UA, "-H", "Content-Type: application/json",
                              "-d", json.dumps(body), url], capture_output=True, text=True,
                             encoding="utf-8").stdout
        try:
            d = json.loads(out)
        except json.JSONDecodeError:
            time.sleep(5 * (attempt + 1))
            continue
        if d.get("object") == "error" and d.get("status") == 429:
            time.sleep(8 * (attempt + 1))
            continue
        return d
    raise RuntimeError("rate limited out: " + url)


def render_term(term):
    """A term as Scryfall reads it: a string, or an `or` group, or a negated
    one (`nor`)."""
    if isinstance(term, tuple):
        kind, parts = term
        group = "(%s)" % " or ".join(parts)
        return group if kind == "or" else "-" + group
    return term


def query(terms):
    return " ".join([BASE] + [render_term(t) for t in terms])


_counts = None


def count(terms):
    """`total_cards` for `BASE` and `terms`, cached by the query string."""
    global _counts
    q = query(terms)
    if _counts is None:
        _counts = json.load(open(COUNTS, encoding="utf-8")) if os.path.exists(COUNTS) else {}
    if q in _counts:
        return _counts[q]["total"]
    if _no_fetch:
        raise SystemExit("--no-fetch, and the count cache lacks: " + q)
    d = _get("https://api.scryfall.com/cards/search?q=%s&unique=cards" % urllib.parse.quote(q))
    if d.get("object") == "error":
        if d.get("code") != "not_found":
            raise RuntimeError("%s -> %s" % (q, d.get("details")))
        n = 0
    else:
        n = d["total_cards"]
    if d.get("warnings"):
        raise SystemExit("Scryfall ignored part of %s: %s" % (q, d["warnings"]))
    _counts[q] = {"total": n, "fetched": _dt.date.today().isoformat()}
    json.dump(_counts, open(COUNTS, "w", encoding="utf-8"), indent=1, sort_keys=True)
    time.sleep(DELAY)
    return n


def names(q, limit=60):
    d = _get("https://api.scryfall.com/cards/search?q=%s&unique=cards" % urllib.parse.quote(q))
    if d.get("object") == "error":
        print(d.get("details"))
        return
    print("query:", q)
    print("total:", d.get("total_cards"))
    for c in d.get("data", [])[:limit]:
        t = c.get("oracle_text") or " | ".join(f.get("oracle_text", "") for f in c.get("card_faces", []))
        print("  - %s :: %s" % (c["name"], t.replace("\n", " | ")[:220]))


# ==========================================================================
# The same terms as regexes, for the registered cards and the cast corpus
# ==========================================================================


def _card(c):
    faces = c.get("card_faces") or []
    text = c.get("oracle_text")
    if text is None:
        text = "\n".join(f.get("oracle_text", "") for f in faces)
    return {"name": c["name"], "text": text, "type": c.get("type_line", "")}


def oracle(card):
    """Oracle text as `o:` reads it: reminder text removed."""
    return re.sub(r"\([^)]*\)", "", card.get("text") or "")


TERM = re.compile(r"^(-?)(o|t):/(.*)/$", re.S)


def term_holds(term, card):
    if isinstance(term, tuple):
        kind, parts = term
        hit = any(term_holds(t, card) for t in parts)
        return hit if kind == "or" else not hit
    m = TERM.match(term)
    if not m:
        raise SystemExit("a term the regex reading cannot read: " + term)
    neg, field, pat = m.groups()
    # Scryfall's regexes are newline-sensitive in a negated bracket too:
    # `[^.]` stops at the end of a paragraph there and not in Python.
    pat = pat.replace("[^.]", "[^.\n]").replace("[^,.]", "[^,.\n]")
    text = oracle(card) if field == "o" else card.get("type", "")
    hit = re.search(pat, text, re.I | re.M) is not None
    return hit != bool(neg)


def holds(terms, card):
    return all(term_holds(t, card) for t in terms)


def registered():
    src = open(REGISTRY, encoding="utf-8").read()
    return re.findall(r'registry\.register\(\s*"([^"]+)"', src)


def registered_cards():
    """The registered printings' oracle text, by `/cards/collection`."""
    want = registered()
    if os.path.exists(REGISTERED_CACHE):
        cached = json.load(open(REGISTERED_CACHE, encoding="utf-8"))
        if sorted(cached.get("asked", [])) == sorted(want):
            return cached
    # The cast census fetched the same printings the same way; reuse them
    # while they cover the registry, rather than spend requests on them.
    if os.path.exists(CAST_REGISTERED):
        cast = json.load(open(CAST_REGISTERED, encoding="utf-8"))
        known = {c["name"] for c in cast["cards"]} | set(cast.get("missing", []))
        if set(want) <= known:
            return {"fetched": cast["fetched"], "asked": want, "missing": cast.get("missing", []),
                    "cards": [c for c in cast["cards"] if c["name"] in set(want)]}
    if _no_fetch:
        raise SystemExit("--no-fetch, and the registered-cards cache is missing or stale")
    found, missing = [], []
    for i in range(0, len(want), 75):
        d = _post("https://api.scryfall.com/cards/collection",
                  {"identifiers": [{"name": n} for n in want[i:i + 75]]})
        found += [_card(c) for c in d.get("data", [])]
        missing += [x.get("name") for x in d.get("not_found", [])]
        time.sleep(DELAY)
    out = {"fetched": _dt.date.today().isoformat(), "asked": want, "cards": found, "missing": missing}
    json.dump(out, open(REGISTERED_CACHE, "w", encoding="utf-8"))
    return out


def members(terms, cards):
    return sorted(c["name"] for c in cards if holds(terms, c))


# ==========================================================================
# The audits: rule numbers against the CR, and dispositions against the tree
# ==========================================================================


def every_cr():
    out = [r for row in CLASSES for r in row[3]] + [r for row in SEARCHED for r in row[3]]
    out += [LOOPS[k][1] for k in LOOPS]
    return sorted(set(out), key=lambda r: [int(x) if x.isdigit() else x for x in re.split(r"(\d+)", r)])


def audit_labels():
    if not os.path.exists(CR):
        return [], ["the CR is not at %s; labels unaudited" % CR]
    text = open(CR, encoding="utf-8", errors="replace").read()
    found, bad = [], []
    for cr in every_cr():
        m = re.search(r"^%s[. ].*$" % re.escape(cr), text, re.M)
        (found if m else bad).append((cr, m.group(0)[:100]) if m else cr)
    return found, bad


def _test_functions():
    names_ = set()
    for top in ("src", "tests"):
        for dirpath, _, files in os.walk(os.path.join(CRATE, top)):
            for f in files:
                if f.endswith(".rs"):
                    src = open(os.path.join(dirpath, f), encoding="utf-8").read()
                    names_ |= set(re.findall(r"\bfn ([a-z_0-9]+)\s*\(", src))
    return names_


def audit_dispositions():
    """Every cited test is a function in the crate, every register row is in
    §3.4b's table, every backlog section is a heading of `backlog.md`."""
    fns = _test_functions()
    register = open(PRACTICES, encoding="utf-8").read()
    backlog = open(BACKLOG, encoding="utf-8").read()
    bad = []
    for row in CLASSES:
        kind, text = row[5]
        if kind == "test":
            for name in re.findall(r"`([a-z][a-z0-9]*(?:_[a-z0-9]+){2,})`", text):
                if name not in fns:
                    bad.append("%s: no test function `%s`" % (row[1], name))
        if kind == "register":
            for rid in re.findall(r"`([a-z]+(?:-[a-z]+)+)`", text):
                if not re.search(r"^\| `%s` \|" % re.escape(rid), register, re.M):
                    bad.append("%s: no register row `%s`" % (row[1], rid))
        if kind == "backlog":
            for sec in re.findall(r"§(2\.\d+)", text):
                if not re.search(r"^### %s " % re.escape(sec), backlog, re.M):
                    bad.append("%s: no backlog section %s" % (row[1], sec))
    return bad


# ==========================================================================
# Rendering
# ==========================================================================


def fmt(n):
    return "{:,}".format(n) if isinstance(n, int) else str(n)


def esc(s):
    return s.replace("|", "\\|")


def code(terms):
    return "`%s`" % esc(" ".join(render_term(t) for t in terms))


def shown(names_, k=4):
    if not names_:
        return "—"
    return esc(", ".join(names_[:k]) + (" and %d more" % (len(names_) - k) if len(names_) > k else ""))


KIND_WORD = {"test": "test", "register": "register row", "backlog": "backlog", "owned": "owned"}


def disposition_cell(disp):
    kind, text = disp
    word = KIND_WORD[kind]
    if kind == "test" and text.startswith("here: "):
        return "**test, written here:** %s" % esc(text[len("here: "):])
    return "**%s:** %s" % (word, esc(text))


def sides(row):
    """A row's watchers' terms, producers' terms, and the side its
    disposition says is missing; loops 1 to 3 count one side, the first."""
    if row[0] != 4:
        return row[4], None, None
    return row[4], row[6], (row[7] if len(row) > 7 else None)


def render_loop(loop, reg_cards):
    rows = [r for r in CLASSES if r[0] == loop]
    L = []
    if loop == 4:
        L.append("| Class | CR | Watchers: terms | Watchers | Producers: terms | Producers | Registered | Disposition |")
        L.append("|---|---|---|---:|---|---:|---|---|")
        for r in rows:
            watch, prod, _ = sides(r)
            reg = [n for terms in (watch, prod) if terms for n in members(terms, reg_cards)]
            L.append("| %s (`%s`) | %s | %s | %s | %s | %s | %s | %s |" % (
                esc(r[2]), r[1], ", ".join(r[3]),
                code(watch) if watch else "—", fmt(count(watch)) if watch else "—",
                code(prod) if prod else "—", fmt(count(prod)) if prod else "—",
                shown(sorted(set(reg))), disposition_cell(r[5])))
        return "\n".join(L)
    L.append("| Class | CR | Terms | Cards | Registered | Disposition |")
    L.append("|---|---|---|---:|---|---|")
    for r in rows:
        L.append("| %s (`%s`) | %s | %s | %s | %s | %s |" % (
            esc(r[2]), r[1], ", ".join(r[3]), code(r[4]), fmt(count(r[4])),
            shown(members(r[4], reg_cards)), disposition_cell(r[5])))
    return "\n".join(L)


def render_searched():
    L = ["| Loop | Shape | CR | First phrasing | Cards | Second phrasing | Cards | Why nothing is owed |",
         "|---:|---|---|---|---:|---|---:|---|"]
    for loop, sid, shape, rules, p1, p2, why in SEARCHED:
        L.append("| %d | %s (`%s`) | %s | %s | %s | %s | %s | %s |" % (
            loop, esc(shape), sid, ", ".join(rules), code(p1), fmt(count(p1)), code(p2), fmt(count(p2)), esc(why)))
    return "\n".join(L)


def render_residual():
    L = ["| Found | Terms | Cards | Why it closes no loop |", "|---|---|---:|---|"]
    for rid, what, terms, why in RESIDUAL:
        L.append("| %s (`%s`) | %s | %s | %s |" % (esc(what), rid, code(terms), fmt(count(terms)), esc(why)))
    return "\n".join(L)


def render_details():
    L = ["| Detail | Terms | Cards |", "|---|---|---:|"]
    for did, what, terms in DETAILS:
        L.append("| %s (`%s`) | %s | %s |" % (esc(what), did, code(terms), fmt(count(terms))))
    return "\n".join(L)


def render_summary():
    kinds = ["test", "register", "backlog", "owned"]
    L = ["| Loop | Classes | Test, cited | Test, written here | Register row | Backlog | Owned | Searched, none printed |",
         "|---|---:|---:|---:|---:|---:|---:|---:|"]
    tot = [0] * 7
    for loop in LOOPS:
        rows = [r for r in CLASSES if r[0] == loop]
        cited = sum(1 for r in rows if r[5][0] == "test" and not r[5][1].startswith("here: "))
        here = sum(1 for r in rows if r[5][0] == "test" and r[5][1].startswith("here: "))
        rest = [sum(1 for r in rows if r[5][0] == k) for k in kinds[1:]]
        none = sum(1 for s in SEARCHED if s[0] == loop)
        cells = [len(rows), cited, here] + rest + [none]
        tot = [a + b for a, b in zip(tot, cells)]
        L.append("| %d. %s | %s |" % (loop, LOOPS[loop][0], " | ".join(str(c) for c in cells)))
    L.append("| **all** | %s |" % " | ".join("**%d**" % c for c in tot))
    return "\n".join(L)


def calibration(reg):
    """Each registered card's classes, and every class whose registered
    members contradict its disposition: a registered card in a class the
    census says the engine cannot express."""
    cards = reg["cards"]
    by_card = {}
    contradictions = []
    for r in CLASSES:
        watch, prod, missing = sides(r)
        ms = [n for terms in (watch, prod) if terms for n in members(terms, cards)]
        for n in ms:
            by_card.setdefault(n, []).append("%d:%s" % (r[0], r[1]))
        if r[5][0] in ("backlog", "owned"):
            # Loop 4 lacks one side; a registered card on the other is expected.
            short = {"watchers": watch, "producers": prod}.get(missing, watch)
            hit = members(short, cards) if short else []
            if hit:
                contradictions.append((r[1], hit))
    L = ["%d names registered in `mtgsim/src/cards/registry.rs`; Scryfall knows %d of them%s." % (
        len(reg["asked"]), len(cards),
        ", and the rest are fixtures: " + ", ".join(reg["missing"]) if reg["missing"] else "")]
    L.append("%d of them fall in at least one class." % len(by_card))
    L.append("")
    if contradictions:
        L.append("| Class | Disposition | Registered cards in it |")
        L.append("|---|---|---|")
        for cid, ms in contradictions:
            disp = next(r[5][0] for r in CLASSES if r[1] == cid)
            L.append("| `%s` | %s | %s |" % (cid, disp, esc(", ".join(ms))))
    else:
        L.append("No class that the census calls inexpressible has a registered card in it.")
    return "\n".join(L)


def check_corpus():
    """Each class's terms over the cast census's cached corpus, beside
    Scryfall's count: where the two dialects of a regex read differently."""
    if not os.path.exists(CAST_CORPUS):
        print("no cast census corpus at %s; run plans/references/cast-census.py first" % CAST_CORPUS)
        return
    corpus = json.load(open(CAST_CORPUS, encoding="utf-8"))
    cards = corpus["cards"]
    print("cast corpus: %d cards, fetched %s, query `%s`" % (len(cards), corpus.get("fetched"), corpus.get("query")))
    rows = [(r[1], sides(r)[0]) for r in CLASSES if sides(r)[0]]
    rows += [(r[1] + ":producers", sides(r)[1]) for r in CLASSES if sides(r)[1]]
    rows += [(s[1] + ":1", s[4]) for s in SEARCHED] + [(s[1] + ":2", s[5]) for s in SEARCHED]
    rows += [(r[0], r[2]) for r in RESIDUAL] + [(d[0], d[2]) for d in DETAILS]
    for rid, terms in rows:
        local = sum(1 for c in cards if holds(terms, c))
        print("  %-22s scryfall %6s   local %6d" % (rid, fmt(count(terms)), local))


def splice(doc_text, key, body):
    begin, end = "<!-- feedback-loops: begin %s -->" % key, "<!-- feedback-loops: end %s -->" % key
    i, j = doc_text.index(begin) + len(begin), doc_text.index(end)
    return doc_text[:i] + "\n" + body + "\n" + doc_text[j:]


def find_terms(cid):
    want_producers = cid.endswith(":producers")
    key = cid[: -len(":producers")] if want_producers else cid
    for r in CLASSES:
        if r[1] == key:
            return sides(r)[1] if want_producers else sides(r)[0]
    for s in SEARCHED:
        if s[1] + ":1" == cid:
            return s[4]
        if s[1] + ":2" == cid:
            return s[5]
    for r in RESIDUAL:
        if r[0] == cid:
            return r[2]
    for d in DETAILS:
        if d[0] == cid:
            return d[2]
    return None


def main():
    global _no_fetch
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0],
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--write", action="store_true", help="splice the tables into feedback-loops.md")
    ap.add_argument("--names", metavar="ID_OR_Q", help="the first page of names behind a class or a query")
    ap.add_argument("--calibrate", action="store_true", help="the registered cards in each class")
    ap.add_argument("--check", action="store_true", help="each count beside the cast corpus's")
    ap.add_argument("--refresh", action="store_true", help="drop the caches first")
    ap.add_argument("--no-fetch", action="store_true", help="never hit the network")
    args = ap.parse_args()
    _no_fetch = args.no_fetch
    if args.refresh:
        for p in (COUNTS, REGISTERED_CACHE):
            if os.path.exists(p):
                os.remove(p)
    if args.names:
        terms = find_terms(args.names)
        names(query(terms) if terms else query([args.names]))
        return

    found, bad = audit_labels()
    if bad:
        raise SystemExit("a label cites a rule that is not in the CR: %s" % bad)
    if args.check:
        check_corpus()
        return
    wrong = audit_dispositions()
    if wrong:
        raise SystemExit("a disposition names what the tree does not have:\n  " + "\n  ".join(wrong))

    reg = registered_cards()
    if args.calibrate:
        print(calibration(reg))
        return

    blocks = {
        "CORPUS": "`%s`: %s cards (`total_cards`)." % (BASE, fmt(count([]))),
        "SUMMARY": render_summary(),
        "LOOP1": render_loop(1, reg["cards"]),
        "LOOP2": render_loop(2, reg["cards"]),
        "LOOP3": render_loop(3, reg["cards"]),
        "LOOP4": render_loop(4, reg["cards"]),
        "SEARCHED": render_searched(),
        "RESIDUAL": render_residual(),
        "DETAILS": render_details(),
        "CALIBRATION": calibration(reg),
        "LABELS": "Every rule number the tables cite, resolved in `MTG-Rules/versions/tmnt.txt`: %s." % ", ".join(
            cr for cr, _ in found),
    }
    newest = max((v["fetched"] for v in (_counts or {}).values()), default="--")
    blocks["DATE"] = "Counted %s: the counts cache's newest entry %s, the registered cards fetched %s." % (
        _dt.date.today().isoformat(), newest, reg["fetched"])
    if args.write:
        raw = open(DOC, "rb").read()
        eol = "\r\n" if raw.count(b"\r\n") > raw.count(b"\n") // 2 else "\n"
        text = raw.decode("utf-8").replace("\r\n", "\n")
        for k, v in blocks.items():
            text = splice(text, k, v)
        open(DOC, "wb").write(text.replace("\n", eol).encode("utf-8"))
        print("wrote", os.path.relpath(DOC, ROOT))
    else:
        for k, v in blocks.items():
            print("## %s\n\n%s\n" % (k, v))


if __name__ == "__main__":
    main()
