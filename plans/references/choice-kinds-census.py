"""The choices printed cards ask a player to make, by family and by shape.

    python plans/references/choice-kinds-census.py                    # the counts
    python plans/references/choice-kinds-census.py --names FAMILY     # one family's cards and the clause that put them there
    python plans/references/choice-kinds-census.py --deck NAMES.txt   # a second column: how many of a deck list's cards
    python plans/references/choice-kinds-census.py --names FAMILY --sample 7   # twelve of a family's cards, seeded

Behind `modes-plus-choice-kinds`' design note
(`plans/handoffs/modes-plus-choice-kinds.md`): the families each decision's
table counts.

WHAT IT COUNTS
--------------
A card is in a family when one of its clauses asks for that choice, read off
the clause and never off a word anywhere in the text: reminder text is
stripped first, a mode is a bulleted list behind a "choose" instruction (CR
700.2), and an "unless" counts only when a player's action follows it (CR
118.12a), never a condition ("can't block unless you control", "enters tapped
unless"). Where the clause sits (a spell, after an activation cost's colon,
behind a trigger word) is read off the line, so the "where" rows are
approximate. In review (2026-11-02) every family was read against the row's
words: every hit of each family under 40, two samples of each larger one
(`--sample 7` and `--sample 8`), and each hit's matching clause across the
whole family; the predicates below carry what that reading corrected.

The corpus is paper. Alchemy's digital-only cards are in scope and were
counted on Scryfall on 2026-11-01 (`game:arena -game:paper`): 84 bulleted,
63 with "unless", 12 "divided as you choose", 61 "may pay", 16 choosing a
value, and 65 drafting from a spellbook, a pick among card definitions.

INPUT
-----
`.census-cast.json` in this directory, `cast-census.py`'s cache of every
Commander-legal paper card (`legal:commander -t:token game:paper -is:funny`).
If it is missing, run `python plans/references/cast-census.py` first (about
seven minutes). Its fetch date is printed with the counts. A deck list is one
card name a line: `tdc-names.txt` beside this script is Scryfall's
`e:tdc -t:basic` (unique cards, 373, fetched 2026-11-01), the five decks v0.5
plays.
"""
import json
import os
import random
import re
import sys
from collections import OrderedDict

HERE = os.path.dirname(os.path.abspath(__file__))
CORPUS = os.path.join(HERE, ".census-cast.json")

REMINDER = re.compile(r"\([^()]*\)")
NUMBER = r"(?:one|two|three|four|five|six|x)"

# --- CR 700.2: modes -------------------------------------------------------
# Spree's and the pawprint's lists have their own heads.
SPREE_HEAD = re.compile(r"^\+ ", re.M)
PAW_HEAD = re.compile(r"choose up to [^.\n]*\{P\}", re.I)
TRIGGER_WORD = re.compile(r"^(?:when|whenever|at the beginning|at end of)\b", re.I)
# An ability word, a Saga's chapter numbers or an Attraction's "Visit" ahead of
# the clause. A chapter ability and a visit are triggered (CR 714.2b, 717.3).
LINE_PREFIX = re.compile(r"^(?:([IVX]+(?:, [IVX]+)*|Visit) — |[A-Z][\w' ,-]* — )")


def triggered(line):
    m = LINE_PREFIX.match(line)
    return bool(TRIGGER_WORD.match(line[m.end():] if m else line)) or bool(m and m.group(1))


MODE_COUNT = re.compile(
    r"\bchoose[s]? (one or both|one or more|" + NUMBER + r" or more|any number|up to " + NUMBER + r"|" + NUMBER + r")\b",
    re.I,
)


def modal_heads(text):
    """Each mode instruction as (its line, its count word): a line naming a
    count of modes with a bulleted list under it. The instruction ends in a
    dash or, on the commander-conditional cycles, a sentence ("Choose one. If
    you control a commander ..., you may choose both instead.")."""
    lines = text.split("\n")
    out = []
    for i, line in enumerate(lines[:-1]):
        if not lines[i + 1].lstrip().startswith("•"):
            continue
        m = MODE_COUNT.search(line)
        if m:
            out.append((line.strip(), m.group(1).lower()))
    return out


def mode_lines(text):
    """Each mode's own line: a bullet, spree's "+ {cost} —" or a pawprint's
    "{P} —", its quoted text (a token's or a granted ability's) dropped,
    since a target there is that ability's, not the mode's."""
    return [re.sub(r'"[^"]*"', "", line) for line in text.split("\n")
            if line.lstrip().startswith(("•", "+ ", "{P}"))]


def keyword_modal(c, t):
    """A modal spell whose head is a keyword's: spree is "choose one or more"
    (CR 702.172a), tiered "choose one" (702.183a), and a pawprint list counts
    its symbols (700.2i)."""
    return bool(SPREE_HEAD.search(t) or PAW_HEAD.search(t) or "Tiered" in c["kw"])


# --- CR 118.12a: "unless [a player] [does something]" ------------------------
UNLESS_ACT = re.compile(
    r"\bunless (?:(?:its|their|that|the|target|any|an?|each|this|that spell's|that ability's|"
    r"enchanted|equipped) )?(?:[\w']+ ){0,3}?"
    r"(pays?|sacrifices?|discards?|exiles?|returns?|puts?|reveals?|taps?|removes?|draws?|"
    r"loses?|mills?|gives?|shuffles?|untaps?|casts?|has [\w ]*? pay)\b",
    re.I,
)


def sentence_before(t, at):
    return t[max(t.rfind(". ", 0, at), t.rfind("\n", 0, at)) + 1:at]


def unquoted(text):
    """`text` without its quoted abilities: a "can't" inside a granted
    ability's quotes is that ability's, not the sentence's."""
    return re.sub(r'"[^"]*"', "", text)


def unless_actions(t):
    """Each "unless" a player's action follows, never a condition on what has
    happened ("unless you've cast", "unless this spell was cast")."""
    return [m for m in UNLESS_ACT.finditer(t)
            if not re.search(r"(?:'ve|was|were|has|have|had) cast\b|\bcast (?:a|an|another)\b[^.]*\b(?:this|last) turn\b",
                             t[m.start():t.find(".", m.start()) if "." in t[m.start():] else len(t)], re.I)]


def unless_acts(t):
    """CR 118.12a's "unless": a player's action after it, in a sentence that
    is no "can't": "can't attack you unless their controller pays" is a cost
    to attack (CR 508.1h, 509.1d), not a choice at resolution."""
    return [m for m in unless_actions(t) if "can't" not in unquoted(sentence_before(t, m.start())).lower()]


def unless_costs_to_attack(t):
    return [m for m in unless_actions(t) if "can't" in unquoted(sentence_before(t, m.start())).lower()]


UNLESS_WHO = re.compile(r"\bunless (any player|you|its controller|that player|they|their controller|its owner|"
                        r"that spell's controller|that ability's controller|target player|an opponent|"
                        r"each opponent|he or she|enchanted creature's controller|equipped creature's controller)\b", re.I)
WARD = re.compile(r"\bward\b", re.I)
WARD_NONMANA = re.compile(r"\bward\s*—", re.I)
WARD_GRANT = re.compile(r"\b(?:has|have|gains?|gets? [^.]*and) \"?ward\b", re.I)
# Ward on a line of the card's own keywords ("Flying, ward {2}", "Ward—Pay 3 life").
WARD_PRINTED = re.compile(r"^(?:[\w ]+, )*ward\b", re.I | re.M)

# --- CR 118.12 / 107.3f: a cost paid at resolution ---------------------------
MAY_PAY = re.compile(r"\b(you|that player|they|any player|an opponent|each opponent|its controller)"
                     r" may pay ([^.]*?)[.,] (?:if|when)\b", re.I)
PAY_X_RESOLUTION = re.compile(r"\bpays? (?:\{x\}|x life|any amount of)", re.I)
# A payment that is made at another time than a resolution: as a permanent
# enters (a replacement, CR 614.12), as an additional cost, at any time a
# player could act (a special action), or as part of an activation's cost.
AS_ENTERS_PAYMENT = re.compile(r"\bas [^.]*?\benters\b", re.I)
NOT_AT_RESOLUTION = re.compile(r"\badditional cost\b|\bany time you could\b|\brather than\b", re.I)


def payments_at_resolution(t, pattern):
    """Each match of `pattern` that pays as an effect resolves."""
    out = []
    for m in pattern.finditer(t):
        before = sentence_before(t, m.start())
        sentence = before + t[m.start():t.find(".", m.end()) if "." in t[m.end():] else len(t)]
        line_rest = t[m.end():t.find("\n", m.end()) if "\n" in t[m.end():] else len(t)]
        if AS_ENTERS_PAYMENT.search(before) or NOT_AT_RESOLUTION.search(sentence) or "can't" in unquoted(before).lower():
            continue
        if ":" in line_rest.split(".")[0]:
            continue  # an activation's cost, ahead of its colon
        out.append(m)
    return out


def payments_as_it_enters(t):
    return [m for pattern in (MAY_PAY, PAY_X_RESOLUTION) for m in pattern.finditer(t)
            if AS_ENTERS_PAYMENT.search(sentence_before(t, m.start()))]

# --- CR 601.2d / 608.2d: division --------------------------------------------
DIVIDE = re.compile(r"\b(?:divided (?:as you choose )?among|distribute[sd]? [^.]*? among)\s+([^.]*)", re.I)
# The objects divided among are themselves the targets ("one, two, or three
# targets", "any number of target creatures"), not things a target controls.
AMONG_TARGETS = re.compile(r"(?:(?:one|two|three|four|five|six|any number of|up to \w+),? (?:or )?)*"
                           r"(?:other )?(?:\w+ )?targets?\b", re.I)
# "Divided among them instead" restates a division among targets.
AMONG_THOSE = re.compile(r"(?:those|them)\b", re.I)


def divisions(t):
    """Each division as (it is among targets, the match), combat damage
    assignment left out (CR 510.1)."""
    out = []
    for m in DIVIDE.finditer(t):
        if "assign" in sentence_before(t, m.start()).lower():
            continue
        out.append((bool(AMONG_TARGETS.match(m.group(1))), m))
    if any(targeted for targeted, _ in out):
        out = [(targeted or bool(AMONG_THOSE.match(m.group(1))), m) for targeted, m in out]
    return out

# --- orders a card or a rule hands a player ---------------------------------
ANY_ORDER = re.compile(r"\bin any order\b", re.I)
ANY_ORDER_LIBRARY = re.compile(r"(?:top|bottom) of (?:your|their|its owner's|that player's|his or her) library in any order", re.I)
MASS_ENTRY = re.compile(r"\b(?:return|put)s? (?:all|each|any number of)\b[^.]*?\bonto the battlefield", re.I)
SEVERAL_PLAYERS = re.compile(r"\beach (?:player|opponent)\b|\ball players\b|\bits owner's\b|\btheir owners'", re.I)


def entries_at_once(t):
    """Each clause putting several objects onto the battlefield at once, as
    (whether several players' objects enter, the match)."""
    return [(bool(SEVERAL_PLAYERS.search(sentence_before(t, m.start()) + m.group(0))), m)
            for m in MASS_ENTRY.finditer(t)
            if not re.search(r"\bthen\b|\bweren't\b|\binto (?:their|your|its|his)", m.group(0), re.I)]

# --- CR 608.2d: a value chosen as an effect applies ---------------------------
AS_ENTERS = re.compile(r"\bas (?:this|~|[\w ,'-]+?) (?:enters|is turned face up)[^.]*?,\s*([^.]*?choose[^.]*)", re.I)
# The words with a rules meaning, so that "choose X or Y" between them picks a
# type, a keyword or a color, not a word CR 607.2f names: the card types and
# colors here, every word of every type line and every keyword added at load.
RULES_WORDS = {"land", "creature", "artifact", "enchantment", "instant", "sorcery", "planeswalker",
               "battle", "kindred", "white", "blue", "black", "red", "green", "colorless"}


class PrintedOptions:
    """"Choose X or Y" (or "X, Y, or Z"): with `rules_meaning` false, words
    with no rules meaning (CR 607.2f: odd or even, left or right); with it
    true, a type, a keyword or a color out of the few the card prints."""
    PICK = re.compile(r"\b(?:secretly )?chooses? (?!one or|both or)((?:[a-z-]+, )*[a-z-]+),? or ([a-z-]+)\b"
                      r"(?! (?:more|both|creature|land|artifact|enchantment|card))", re.I)

    def __init__(self, rules_meaning):
        self.rules_meaning = rules_meaning

    def search(self, text):
        for m in self.PICK.finditer(text):
            words = [w.lower() for w in m.group(1).split(", ")] + [m.group(2).lower()]
            if all(w.removeprefix("non").lstrip("-") in RULES_WORDS | {""} for w in words) == self.rules_meaning:
                return m
        return None


VALUES = OrderedDict([
    ("a color", re.compile(r"\bchooses? (?:a|one|two|any) colors?\b|\ba color of (?:your|their) choice\b", re.I)),
    ("a creature type", re.compile(r"\bchooses? (?:[\w ]+? and )?a creature type\b|\bcreature type of (?:your|their) choice\b", re.I)),
    ("a card type", re.compile(r"\bchooses? a (?:card|permanent) type\b|\bcard type of (?:your|their) choice\b", re.I)),
    ("a basic land type", re.compile(r"\bchooses? a basic land type\b|\bbasic land type of (?:your|their) choice\b", re.I)),
    ("a card name", re.compile(r"\bchooses? (?:[\w ]+? and )?an? (?:[\w-]+ )?card name\b|\bname a card\b|\bchoose a [\w -]*? name\b", re.I)),
    ("a number", re.compile(r"\bchooses? (?:[\w ,-]+? and )?a number(?! of)\b", re.I)),
    ("a player, not targeted", re.compile(r"\bchooses? (?:[\w ]+? and )?(?:a|an|another) (?:player|opponent)\b|\b(?:player|opponent) of (?:your|their) choice\b", re.I)),
    ("a word with no rules meaning", PrintedOptions(rules_meaning=False)),
    ("a type, keyword or color out of a printed few", PrintedOptions(rules_meaning=True)),
    ("a pile (CR 700.3)", re.compile(r"\binto two piles\b|\bseparates? [^.]*piles\b", re.I)),
    ("a vote (CR 701.38)", re.compile(r"(?<!n't )\bvotes? for\b|\bwill of the council\b|\bcouncil's dilemma\b", re.I)),
])

AS_CAST_COST = re.compile(r"As an additional cost to cast this spell, ([^.]*)\.", re.I)
ENTERS_UNDER_CHOICE = re.compile(r"\benters under the control of an? (?:opponent|player) of your choice\b", re.I)
# A value the targets' own words read, so it is chosen while casting, before
# them (CR 601.2c; Aphetto Dredging's ruling).
CHOSEN_FOR_TARGETS = re.compile(r"\btargets? [^.]*?\bof (?:the|a) [\w ]+? of your choice\b", re.I)
# A time no resolution sets: before the game, while drafting, as a permanent
# becomes attached or transforms, or each time a static replacement or
# prevention applies (CR 614, 615).
ANOTHER_TIME = re.compile(r"\bbefore the game begins\b|\bas you draft\b|\bduring the draft\b|\bduring (?:their|your) untap step\b"
                          r"|\bas [^.]*? (?:becomes attached|transforms)\b"
                          r"|^(?:[A-Z][\w' ,-]* — )?if\b[^.]*\b(?:instead|prevent)\b|\bif you tap [^.]*\bfor mana\b", re.I)
# A mana ability (CR 605.1a): a cost, a colon, and an effect that adds mana
# and targets nothing. Its choice is the mana model's, made as it resolves
# off the stack (605.3b).
MANA_ABILITY = re.compile(r"^(?:[A-Z][\w' ,-]* — )?[^.:\"]+:(?![^\"]*\btarget)[^\"]*\badds?\b", re.I)


def value_clauses(t):
    """Each stretch of the text that can choose a value, with when it is
    chosen: "resolution", "enters", "cost", "targets", "mana ability" or
    "another time"."""
    for line in t.split("\n"):
        for m in AS_ENTERS.finditer(line):
            yield "enters", m.group(1)
        for m in AS_CAST_COST.finditer(line):
            yield "cost", m.group(1)
        rest = AS_CAST_COST.sub("", AS_ENTERS.sub("", line))
        for m in ENTERS_UNDER_CHOICE.finditer(rest):
            yield "enters", m.group(0)
        rest = ENTERS_UNDER_CHOICE.sub("", rest)
        if ANOTHER_TIME.search(rest):
            yield "another time", rest
        elif MANA_ABILITY.search(rest):
            yield "mana ability", rest
        else:
            for m in CHOSEN_FOR_TARGETS.finditer(rest):
                yield "targets", m.group(0)
            yield "resolution", CHOSEN_FOR_TARGETS.sub("", rest)

FAMILIES = OrderedDict()


def family(name):
    def wrap(f):
        FAMILIES[name] = f
        return f
    return wrap


def is_spell(card):
    return bool(re.search(r"\b(?:Instant|Sorcery)\b", card["type"]))


@family("modal: any")
def _(c, t):
    return modal_heads(t) or keyword_modal(c, t)


def activated(line):
    return ":" in line.lower().split("choose")[0] and not triggered(line)


@family("modal: a spell's modes")
def _(c, t):
    return is_spell(c) and (keyword_modal(c, t)
                            or any(not triggered(line) and not activated(line) for line, _ in modal_heads(t)))


@family("modal: an activated ability's modes")
def _(c, t):
    return any(activated(line) for line, _ in modal_heads(t))


@family("modal: a triggered ability's modes")
def _(c, t):
    return any(triggered(line) for line, _ in modal_heads(t))


@family("modal: a permanent spell's own modes, not in an ability")
def _(c, t):
    return not is_spell(c) and any(not triggered(line) and not activated(line) for line, _ in modal_heads(t))


def optional_count(line):
    return re.search(r"\byou may choose (?:one|two|three|up to)\b", line, re.I) and "instead" not in line


@family("count: exactly one")
def _(c, t):
    return "Tiered" in c["kw"] or any(n == "one" for _, n in modal_heads(t))


@family("count: exactly two or more")
def _(c, t):
    return any(n in ("two", "three", "four", "five") and not optional_count(line) for line, n in modal_heads(t))


@family("count: one or both")
def _(c, t):
    return any(n == "one or both" for _, n in modal_heads(t))


@family("count: one or more")
def _(c, t):
    return "Spree" in c["kw"] or any(n in ("one or more", "two or more") for _, n in modal_heads(t))


@family("count: any number")
def _(c, t):
    return any(n == "any number" for _, n in modal_heads(t))


@family("count: up to N")
def _(c, t):
    return any(n.startswith("up to") for _, n in modal_heads(t))


@family("count: X, or a number the text computes")
def _(c, t):
    return (any(n in ("x", "up to x") for _, n in modal_heads(t))
            or re.search(r"choose (?:that many|a number of)[^.\n]*—", t, re.I))


RAISED_COUNT = re.compile(r"\b(you may )?choose (?:two|three|both|all|up to|any number|one or more)\b[^.]*\binstead\b", re.I)


@family("count: raised by a condition, the player may (\"you may choose both instead\")")
def _(c, t):
    return modal_heads(t) and any(m.group(1) for m in RAISED_COUNT.finditer(t))


@family("count: raised by a condition, the count becomes (\"choose both instead\")")
def _(c, t):
    return modal_heads(t) and any(not m.group(1) for m in RAISED_COUNT.finditer(t))


@family("a mode chosen at random")
def _(c, t):
    return any("at random" in line.lower() for line, _ in modal_heads(t))


@family("count: optional (\"you may choose two\")")
def _(c, t):
    return any(optional_count(line) for line, _ in modal_heads(t))


@family("CR 700.2d: the same mode more than once")
def _(c, t):
    return re.search(r"same mode more than once", t, re.I)


@family("CR 700.2e: another player chooses the mode")
def _(c, t):
    return any(re.search(r"\b(?:opponent|player|they) chooses? ", line, re.I) for line, _ in modal_heads(t))


@family("CR 700.2c/f: a mode with its own target")
def _(c, t):
    return (modal_heads(t) or keyword_modal(c, t)) and any("target" in line.lower() for line in mode_lines(t))


@family("item 154: each mode must target a different player")
def _(c, t):
    return re.search(r"each mode must target a different", t, re.I)


@family("a mode not chosen for this ability this turn")
def _(c, t):
    return re.search(r"\bchoose one that hasn't been chosen this turn\b", t, re.I)


@family("a mode not chosen for this ability while its object exists (Demonic Pact)")
def _(c, t):
    return re.search(r"\bchoose one that hasn't been chosen\b(?! this turn)", t, re.I)


@family("a value not chosen for this ability while its object exists (Garth One-Eye)")
def _(c, t):
    return re.search(r"\bchoose (?!one\b)[^.]*? that hasn't been chosen\b(?! this)", t, re.I)


@family("an object chosen earlier in this resolution, excluded (Fortunate Few)")
def _(c, t):
    return re.search(r"\bchooses [^.]*? that hasn't been chosen\b", t, re.I)


@family("CR 700.2h: spree, a cost per mode")
def _(c, t):
    return "Spree" in c["kw"] or SPREE_HEAD.search(t)


@family("CR 700.2i: pawprint modes")
def _(c, t):
    return PAW_HEAD.search(t)


@family("CR 702.42: entwine")
def _(c, t):
    return "Entwine" in c["kw"]


@family("CR 702.120: escalate")
def _(c, t):
    return "Escalate" in c["kw"]


@family("tiered, a cost per mode by tier")
def _(c, t):
    return "Tiered" in c["kw"]


@family("unless: any")
def _(c, t):
    return unless_acts(t)


@family("unless: pays")
def _(c, t):
    return any(m.group(1).lower().startswith("pay") for m in unless_acts(t))


@family("unless: another action (sacrifices, discards, ...)")
def _(c, t):
    return any(not m.group(1).lower().startswith("pay") for m in unless_acts(t))


@family("not CR 118.12a: a can't with a price (CR 508.1h, 509.1d)")
def _(c, t):
    return unless_costs_to_attack(t)


@family("unless: any player may (asked of each in turn)")
def _(c, t):
    return (re.search(r"\bunless any (?:player|one player)\b|\bany player may (?:pay|sacrifice|discard)\b", t, re.I)
            and not re.search(r"\bto ignore this effect\b", t, re.I))


@family("unless: the payer is named by the text (\"its controller\", \"target player\", \"they\"), not \"you\"")
def _(c, t):
    acts = {m.start() for m in unless_acts(t)}
    return any(m.group(1).lower() != "you" for m in UNLESS_WHO.finditer(t) if m.start() in acts)


@family("ward: printed")
def _(c, t):
    return "Ward" in c["kw"] and WARD_PRINTED.search(t)


@family("ward: a cost that is not mana alone")
def _(c, t):
    return WARD_NONMANA.search(t)


@family("ward: granted")
def _(c, t):
    return WARD_GRANT.search(t)


@family("echo and cumulative upkeep")
def _(c, t):
    return "Echo" in c["kw"] or "Cumulative upkeep" in c["kw"]


@family("a cost paid at resolution: \"you may pay ... if/when\"")
def _(c, t):
    return any(m.group(1).lower() == "you" for m in payments_at_resolution(t, MAY_PAY))


@family("a cost paid at resolution by a player the text names (\"its controller may pay\")")
def _(c, t):
    return any(m.group(1).lower() != "you" for m in payments_at_resolution(t, MAY_PAY))


# An X the text fixes, so nothing is chosen (CR 107.3f), unless the text
# defines it by the payment itself ("where X is the total amount of mana paid
# this way") or only bounds it.
X_FIXED = re.compile(r"\bwhere X is (?!the (?:total )?amount [^.]*paid|less than|at most)"
                     r"|\bX is (?!the (?:total )?amount [^.]*paid)(?:the|equal)\b", re.I)


def x_fixed_by_its_activation(t, m):
    line_start = t.rfind("\n", 0, m.start()) + 1
    cost = t[line_start:m.start()].split(":")
    return len(cost) > 1 and "X" in cost[0]


@family("an amount chosen as the effect pays it: {X}, X life, any amount")
def _(c, t):
    return ("{X}" not in (c.get("mana") or "") and not X_FIXED.search(t)
            and any(not x_fixed_by_its_activation(t, m) for m in payments_at_resolution(t, PAY_X_RESOLUTION)))


@family("not at resolution: a payment as a permanent enters (a replacement, CR 614.12)")
def _(c, t):
    return payments_as_it_enters(t)


@family("divided: among targets (CR 601.2d)")
def _(c, t):
    return any(targeted for targeted, _ in divisions(t))


@family("divided: among objects not targeted (CR 608.2d)")
def _(c, t):
    return any(not targeted for targeted, _ in divisions(t))


@family("divided: prevention (CR 615.7, item 92)")
def _(c, t):
    return re.search(r"prevent the next [^.]* divided as you choose", t, re.I)


def among_targets(t):
    """What each division among targets is among, its first clause's words."""
    return [m.group(1).lower() for targeted, m in divisions(t) if targeted]


@family("divided, its count: one, two, or three")
def _(c, t):
    return any(re.match(r"(?:one|two)(?:,| or)", among) for among in among_targets(t))


@family("divided, its count: up to N")
def _(c, t):
    return any(re.match(r"up to (?:one|two|three|four|five|six)\b", among) for among in among_targets(t))


@family("divided, its count: any number")
def _(c, t):
    return any(among.startswith("any number") for among in among_targets(t))


@family("divided, its count: X or up to X")
def _(c, t):
    return any(re.match(r"(?:up to )?x\b", among) for among in among_targets(t))


@family("divided, its verb: damage")
def _(c, t):
    return any(targeted and m.group(0).lower().startswith("divided") for targeted, m in divisions(t))


@family("divided, its verb: counters")
def _(c, t):
    return any(targeted and m.group(0).lower().startswith("distribute") for targeted, m in divisions(t))


@family("divided evenly, no player's split (not CR 601.2d)")
def _(c, t):
    return re.search(r"\bdivided evenly\b", t, re.I)


@family("divided: a later clause reads a target's share")
def _(c, t):
    return bool(divisions(t)) and re.search(r"\bfor each [^.]*counter you put on [^.]*this way\b", t, re.I)


@family("order: a library order, \"in any order\"")
def _(c, t):
    return ANY_ORDER_LIBRARY.search(t)


@family("order: a library order, other wording (\"put them back in any order\")")
def _(c, t):
    return ANY_ORDER.search(t) and not ANY_ORDER_LIBRARY.search(t)


@family("order: objects entering at once (CR 613.7m)")
def _(c, t):
    return entries_at_once(t)


@family("order: objects entering at once under several players, each ordering their own (CR 613.7m, 101.4)")
def _(c, t):
    return any(several for several, _ in entries_at_once(t))


@family("order: a creature whose power or toughness reads the pool or the life total (CR 601.2h)")
def _(c, t):
    return "Creature" in c["type"] and re.search(
        r"(?:power|toughness|gets [+-]\w/[+-]\w)[^.]*(?:equal to|for each|where X is)[^.]*(?:unspent|mana in your mana pool|your life total)",
        t, re.I)


def chosen_when(rx, timings):
    return lambda c, t: any(rx.search(text) for when, text in value_clauses(t) if when in timings)


for _value, _rx in VALUES.items():
    FAMILIES["value at resolution: " + _value] = chosen_when(_rx, ("resolution",))
    FAMILIES["value as it enters: " + _value] = chosen_when(_rx, ("enters",))
    FAMILIES["value as an additional cost to cast it: " + _value] = chosen_when(_rx, ("cost",))

for _timing, _name in [("targets", "a value chosen while casting, which its targets read (CR 601.2c)"),
                       ("mana ability", "a value chosen in a mana ability (CR 605.3b), the mana model's"),
                       ("another time", "a value chosen before the game, while drafting, as it attaches or transforms, or as a replacement applies")]:
    FAMILIES[_name] = (lambda timing: lambda c, t: any(
        rx.search(text) for when, text in value_clauses(t) if when == timing for rx in VALUES.values()))(_timing)


@family("an object chosen as an additional cost to cast it, not targeted")
def _(c, t):
    return any(re.search(r"\bchoose (?:a|an|another) [\w ,-]*?(?:creature|permanent|artifact|card)s? (?:you control|you own|in your)",
                         m.group(1), re.I)
               for m in AS_CAST_COST.finditer(t))


NARROWED_NAME = re.compile(r"\bchooses? an? (?:nonland|creature|land|noncreature|artifact|instant or sorcery|nonbasic land)"
                           r"[\w -]*? card name\b|\bcard name other than\b", re.I)
FAMILIES["a card name narrowed by what the card is, at resolution or as it enters"] = chosen_when(NARROWED_NAME, ("resolution", "enters"))


def load():
    with open(CORPUS, encoding="utf-8") as f:
        data = json.load(f)
    for card in data["cards"]:
        RULES_WORDS.update(word.lower() for word in re.findall(r"[A-Za-z]+", card["type"]))
        RULES_WORDS.update(kw.lower() for kw in card["kw"])
    return data["fetched"], data["cards"]


def main(argv):
    fetched, cards = load()
    deck = None
    if "--deck" in argv:
        with open(argv[argv.index("--deck") + 1], encoding="utf-8") as f:
            deck = {line.strip() for line in f if line.strip()}
    texts = [(c, REMINDER.sub("", c.get("text") or "")) for c in cards]
    if "--names" in argv:
        want = argv[argv.index("--names") + 1]
        hits = [(c, t) for c, t in texts if FAMILIES[want](c, t) and (deck is None or c["name"] in deck)]
        if "--sample" in argv:
            hits = random.Random(int(argv[argv.index("--sample") + 1])).sample(hits, min(12, len(hits)))
        for c, t in hits:
            print(c["name"], "|", t.replace("\n", " / ")[:240])
        return
    print(f"corpus fetched {fetched}, {len(cards)} cards")
    for name, f in FAMILIES.items():
        hits = [c["name"] for c, t in texts if f(c, t)]
        in_deck = sum(1 for n in hits if deck and n in deck)
        print(f"{len(hits):6d}" + (f" {in_deck:4d}" if deck else "") + f"  {name}")


if __name__ == "__main__":
    main(sys.argv[1:])
