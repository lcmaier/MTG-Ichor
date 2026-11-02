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
approximate; the family rows were read card by card for every family under 40
and twelve of each larger one (`--sample 7`).

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
    return [line for line in text.split("\n") if line.lstrip().startswith("•")]


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


def unless_acts(t):
    """CR 118.12a's "unless": a player's action after it, in a sentence that
    is no "can't": "can't attack you unless their controller pays" is a cost
    to attack (CR 508.1h, 509.1d), not a choice at resolution."""
    return [m for m in UNLESS_ACT.finditer(t) if "can't" not in sentence_before(t, m.start()).lower()]


def unless_costs_to_attack(t):
    return [m for m in UNLESS_ACT.finditer(t) if "can't" in sentence_before(t, m.start()).lower()]


UNLESS_WHO = re.compile(r"\bunless (any player|you|its controller|that player|they|their controller|its owner|"
                        r"that spell's controller|that ability's controller|target player|an opponent|"
                        r"each opponent|he or she|enchanted creature's controller|equipped creature's controller)\b", re.I)
WARD = re.compile(r"\bward\b", re.I)
WARD_NONMANA = re.compile(r"\bward\s*—", re.I)
WARD_GRANT = re.compile(r"\b(?:has|have|gains?|gets? [^.]*and) ward\b", re.I)

# --- CR 118.12 / 107.3f: a cost paid at resolution ---------------------------
MAY_PAY = re.compile(r"\b(?:you|that player|they|any player|an opponent|each opponent|its controller)"
                     r" may pay ([^.]*?)[.,] (?:if|when)\b", re.I)
PAY_X_RESOLUTION = re.compile(r"\bpays? (?:\{x\}|x life|any amount of)", re.I)

# --- CR 601.2d / 608.2d: division --------------------------------------------
DIVIDE = re.compile(r"\b(?:divided (?:as you choose )?among|distribute[sd]? [^.]*? among)\s+([^.]*)", re.I)

# --- orders a card or a rule hands a player ---------------------------------
ANY_ORDER = re.compile(r"\bin any order\b", re.I)
ANY_ORDER_LIBRARY = re.compile(r"(?:top|bottom) of (?:your|their|its owner's|that player's|his or her) library in any order", re.I)
MASS_ENTRY = re.compile(r"\b(?:return|put)s? (?:all|each|any number of)\b[^.]*?\bonto the battlefield", re.I)

# --- CR 608.2d: a value chosen as an effect applies ---------------------------
AS_ENTERS = re.compile(r"\bas (?:this|~|[\w ,'-]+?) (?:enters|is turned face up)[^.]*?,\s*([^.]*?choose[^.]*)", re.I)
VALUES = OrderedDict([
    ("a color", re.compile(r"\bchooses? (?:a|one|two|any) colors?\b|\ba color of (?:your|their) choice\b", re.I)),
    ("a creature type", re.compile(r"\bchooses? a creature type\b|\bcreature type of (?:your|their) choice\b", re.I)),
    ("a card type", re.compile(r"\bchooses? a (?:card|permanent) type\b|\bcard type of (?:your|their) choice\b", re.I)),
    ("a basic land type", re.compile(r"\bchooses? a basic land type\b|\bbasic land type of (?:your|their) choice\b", re.I)),
    ("a card name", re.compile(r"\bchooses? a (?:[\w-]+ )?card name\b|\bname a card\b|\bchoose a [\w -]*? name\b", re.I)),
    ("a number", re.compile(r"\bchooses? a number\b|\bsecretly choose\b", re.I)),
    ("a player, not targeted", re.compile(r"\bchooses? (?:a|an|another) (?:player|opponent)\b|\b(?:player|opponent) of (?:your|their) choice\b", re.I)),
    ("a word with no rules meaning", re.compile(r"\bchooses? (?!one or)([a-z]+) or ([a-z]+)\b(?! (?:more|both|creature|land|artifact|enchantment|card))", re.I)),
    ("a pile (CR 700.3)", re.compile(r"\binto two piles\b|\bseparates? [^.]*piles\b", re.I)),
    ("a vote (CR 701.38)", re.compile(r"\bvotes? for\b|\bwill of the council\b|\bcouncil's dilemma\b", re.I)),
])

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
    return modal_heads(t) or SPREE_HEAD.search(t) or PAW_HEAD.search(t)


def activated(line):
    return ":" in line.lower().split("choose")[0] and not triggered(line)


@family("modal: a spell's modes")
def _(c, t):
    return is_spell(c) and any(not triggered(line) and not activated(line) for line, _ in modal_heads(t))


@family("modal: an activated ability's modes")
def _(c, t):
    return any(activated(line) for line, _ in modal_heads(t))


@family("modal: a triggered ability's modes")
def _(c, t):
    return any(triggered(line) for line, _ in modal_heads(t))


@family("modal: a permanent spell's own modes, not in an ability")
def _(c, t):
    return not is_spell(c) and any(not triggered(line) and not activated(line) for line, _ in modal_heads(t))


@family("count: exactly one")
def _(c, t):
    return any(n == "one" for _, n in modal_heads(t))


@family("count: exactly two or more")
def _(c, t):
    return any(n in ("two", "three", "four", "five") for _, n in modal_heads(t))


@family("count: one or both")
def _(c, t):
    return any(n == "one or both" for _, n in modal_heads(t))


@family("count: one or more")
def _(c, t):
    return any(n in ("one or more", "two or more") for _, n in modal_heads(t))


@family("count: any number")
def _(c, t):
    return any(n == "any number" for _, n in modal_heads(t))


@family("count: up to N")
def _(c, t):
    return any(n.startswith("up to") for _, n in modal_heads(t))


@family("count: X, or a number the text computes")
def _(c, t):
    return any(n == "x" for _, n in modal_heads(t)) or re.search(r"choose (?:that many|a number of)[^.\n]*—", t, re.I)


@family("count: raised by a condition (\"you may choose two instead\")")
def _(c, t):
    return modal_heads(t) and re.search(r"\byou may choose (?:two|three|both|all|up to|any number|one or more)[^.]*instead\b", t, re.I)


@family("a mode chosen at random")
def _(c, t):
    return any("at random" in line.lower() for line, _ in modal_heads(t))


@family("count: optional (\"you may choose two\")")
def _(c, t):
    return any(re.search(r"\byou may choose (?:one|two|three|up to)\b", line, re.I) and "instead" not in line for line, _ in modal_heads(t))


@family("CR 700.2d: the same mode more than once")
def _(c, t):
    return re.search(r"same mode more than once", t, re.I)


@family("CR 700.2e: another player chooses the mode")
def _(c, t):
    return any(re.search(r"\b(?:opponent|player|they) chooses? ", line, re.I) for line, _ in modal_heads(t))


@family("CR 700.2c/f: a mode with its own target")
def _(c, t):
    return modal_heads(t) and any("target" in line.lower() for line in mode_lines(t))


@family("item 154: each mode must target a different player")
def _(c, t):
    return re.search(r"each mode must target a different", t, re.I)


@family("a mode not chosen before (\"that hasn't been chosen\")")
def _(c, t):
    return re.search(r"(?:that|which) (?:hasn't|haven't|has not) been chosen", t, re.I)


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
    return re.search(r"\bunless any (?:player|one player)\b|\bany player may (?:pay|sacrifice|discard)\b", t, re.I)


@family("unless: the payer is not the controller")
def _(c, t):
    acts = {m.start() for m in unless_acts(t)}
    return any(m.group(1).lower() != "you" for m in UNLESS_WHO.finditer(t) if m.start() in acts)


@family("ward: printed")
def _(c, t):
    return "Ward" in c["kw"] and not WARD_GRANT.search(t)


@family("ward: a cost that is not mana")
def _(c, t):
    return WARD_NONMANA.search(t)


@family("ward: granted")
def _(c, t):
    return WARD_GRANT.search(t)


@family("echo and cumulative upkeep")
def _(c, t):
    return "Echo" in c["kw"] or "Cumulative upkeep" in c["kw"]


@family("a cost paid at resolution: \"may pay ... if/when\"")
def _(c, t):
    return MAY_PAY.search(t)


@family("an amount chosen as the effect pays it: {X}, X life, any amount")
def _(c, t):
    return "{X}" not in (c.get("mana") or "") and PAY_X_RESOLUTION.search(t)


@family("divided: among targets (CR 601.2d)")
def _(c, t):
    return any("target" in m.group(1).lower() for m in DIVIDE.finditer(t))


@family("divided: among objects not targeted (CR 608.2d)")
def _(c, t):
    return any("target" not in m.group(1).lower() for m in DIVIDE.finditer(t))


@family("divided: prevention (CR 615.7, item 92)")
def _(c, t):
    return re.search(r"prevent the next [^.]* divided as you choose", t, re.I)


@family("order: a library order, \"in any order\"")
def _(c, t):
    return ANY_ORDER_LIBRARY.search(t)


@family("order: \"in any order\", anything else")
def _(c, t):
    return ANY_ORDER.search(t) and not ANY_ORDER_LIBRARY.search(t)


@family("order: objects entering at once under one player (CR 613.7m)")
def _(c, t):
    return MASS_ENTRY.search(t)


@family("order: a creature whose power or toughness reads the pool or the life total (CR 601.2h)")
def _(c, t):
    return "Creature" in c["type"] and re.search(
        r"(?:power|toughness|gets \+1/\+1)[^.]*(?:equal to|for each)[^.]*(?:unspent|mana in your mana pool|your life total)",
        t, re.I)


for _value, _rx in VALUES.items():
    def _make(rx):
        def at_resolution(c, t):
            return rx.search(AS_ENTERS.sub("", t))
        def as_entering(c, t):
            return any(rx.search(m.group(1)) for m in AS_ENTERS.finditer(t))
        return at_resolution, as_entering
    _res, _ent = _make(_rx)
    FAMILIES["value at resolution: " + _value] = _res
    FAMILIES["value as it enters: " + _value] = _ent


@family("value at resolution: a card name narrowed by what the card is")
def _(c, t):
    return re.search(r"\bchooses? an? (?:nonland|creature|land|noncreature|artifact|instant or sorcery|nonbasic land)[\w -]*? card name\b", t, re.I)


def load():
    with open(CORPUS, encoding="utf-8") as f:
        data = json.load(f)
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
