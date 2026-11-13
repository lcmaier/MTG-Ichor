"""What printed cards ask of a unit of mana, by the clause that asks it.

    python plans/references/mana-provenance-census.py            # the counts
    python plans/references/mana-provenance-census.py --names    # each family's cards and the clause that put them there
    python plans/references/mana-provenance-census.py --names "source"   # only the families whose name holds the word

Behind `mana-architecture.md`'s provenance design (`codebase-state.md` item
33): the per-unit record, what a spending restriction reads, what reads the
mana a payment spent, and mana kept past a step's end.

WHAT IT COUNTS
--------------
A card is in a family when one of its sentences *does* the thing: restricts
"this mana", gives it a rider, keeps it, asks {S} in a cost, or reads what was
spent. Reminder text is stripped first, so a keyword counts by its keyword
(`kw`), never by the words of its reminder. A card with two such sentences is
in two families, and the totals at the foot of each section are distinct cards.

A restriction is classified by everything its predicate reads, so "Spend this
mana only to cast a creature spell of the chosen type or a legendary creature
spell" is in three rows: a spell by its type line, a spell by a choice the
source made, and a spell by another characteristic.

The classifier counts; it does not judge. A sentence that matches a section's
gate and none of its rows is printed as "unbucketed" for a human to read.

INPUT
-----
`.census-cast.json` in this directory, `cast-census.py`'s cache of every
Commander-legal paper card (`legal:commander -t:token game:paper -is:funny`).
If it is missing, run `python plans/references/cast-census.py` first (about
seven minutes). Its fetch date is printed with the counts.
"""
import json
import os
import re
import sys
from collections import defaultdict

HERE = os.path.dirname(os.path.abspath(__file__))
CORPUS = os.path.join(HERE, ".census-cast.json")

RESTRICTION = re.compile(r"spend this mana only|this mana can't be spent|can be spent only", re.I)

# What a restriction's predicate reads. Every row that matches counts.
RESTRICTION_READS = [
    ("the purpose is casting a spell", r"\bcast\b"),
    ("the purpose is activating an ability", r"activate|equip abilit|power-up abilit|gain a class level"),
    ("the purpose is another cost or action", r"cumulative upkeep|costs? that contains? \{|face up|morph cost|unlock|foretell a card|foretell cards|disturb cost"),
    ("one part of a cost (generic)", r"generic mana costs"),
    ("a spell's type line (card type, subtype, supertype)",
     r"cast (?:an? |the )?(?:[A-Z][\w'-]+|creature|artifact|instant|sorcery|enchantment|planeswalker|noncreature|nonartifact|legendary|outlaw|colorless [A-Z])"),
    ("a spell or source of a type its source chose", r"chosen type"),
    ("a spell's color, mana value, cost symbols, keywords or abilities",
     r"multicolored|monocolored|colorless|exactly three colors|one or more colors|mana value|\{X\} in (?:its|their) mana cost|with devoid|no abilities|that has freerunning|that have foretell|with flashback"),
    ("how the spell is cast (zone, kicked, face down, owner, a named card)",
     r"from (?:your |a )?(?:graveyard|hand|exile)|anywhere other than|kicked|face-down|you don't own|your commander|last card exiled"),
    ("an ability's source, by its type line or color", r"abilit(?:y|ies) of"),
    ("an ability by its kind (equip, power-up, Class level)", r"equip abilit|power-up abilit|gain a class level"),
    ("stated as what it can't pay", r"can't be spent"),
]

RIDER = re.compile(
    r"if (?:any of )?(?:that|this) mana is spent|if you spend this mana|when that mana is spent|when you spend this mana"
    r"|spend this mana only[^.]*, and that spell", re.I)
RIDER_KINDS = [
    ("a delayed trigger when the mana is spent (CR 106.6, 603.7a)", r"^when "),
    ("the spell it paid for can't be countered", r"can't be countered"),
    ("the permanent the spell becomes gains an ability", r"\bgains\b"),
    ("the permanent the spell becomes enters with counters", r"enters with"),
]

KEEPS = [
    ("this mana is kept, until end of turn or combat (on the unit)", r"don't lose this mana"),
    ("unspent mana is kept, by a static ability", r"don't lose unspent|doesn't empty|don't empty|doesn't lose unspent"),
    ("losing unspent mana is replaced (it changes type)", r"would lose unspent mana"),
    ("losing unspent mana has a result (life lost, moved to another player)", r"losing unspent mana|mana lost this way"),
]

SNOW_COST = [
    ("{S} in a mana cost", None),
    ("{S} in an activation cost", r"\{S\}[^:.]*:"),
    ("{S} in another cost (cumulative upkeep, kicker, an effect's payment)", r"(?:upkeep|kicker|pays?|pay) [^.]*\{S\}|\{S\}[^.]*rather than"),
    ("reads {S} spent", r"\{S\}[^.]*(?:was|were) spent|for each \{S\} spent|\{S\} spent"),
]

SPENT = re.compile(r"(?:was|were|been) spent|mana spent|spent to (?:cast|activate)|\bexpend", re.I)
SPENT_READS = [
    ("which source made it, and its type with it ({S} of a color)", r"\{s\} of [^.]*spent"),
    ("which source made it (a Treasure, a snow source)", r"mana from [^.]*spent|\{s\}[^.]*spent|produced by [^.]*spent|produce is spent"),
    ("its colors or types", r"colors? of mana spent|\{[wubrgc]\}[^.]*(?:was|were|wasn't) spent|\{[wubrgc]\} spent|(?:white|blue|black|red|green|colorless|colored) mana (?:was|were) spent|(?:white|blue|black|red|green|colorless) mana spent|color with the mana spent|type of mana spent|mana of the same color"),
    ("how much, of any kind", r"amount of mana spent|\bmana (?:was|were) spent|mana spent to|total mana|much mana"),
    ("how much this turn (expend, CR 700.14)", r"\bexpend"),
]
SPENT_WHERE = [
    ("read of an ability's activation", r"spent to activate|activation cost"),
    ("read by a trigger or effect while the spell is on the stack", r"^(?:[a-z ]+ — )?(?:whenever|when) (?:you|a player|an opponent|one or more)[^,]*(?:casts?|activates?)\b|that spell|cast them\b|cast that|\bexpend|spells? you cast|other creatures"),
    ("read as the permanent it became enters, or after", r"enters|this creature|this permanent|this artifact|^~|\bit\b"),
    ("read by the spell as it resolves", r"this spell"),
]

COST_NAMES_ITS_MANA = [
    ("a cost that takes only mana of a color", r"spend only (?:white|blue|black|red|green|colorless|colored) mana"),
    ("a cost that takes only mana from a kind of source", r"spend only mana produced by"),
    ("a spell no mana may pay for", r"can't spend mana to cast"),
]

AS_THOUGH = re.compile(r"spend mana as though|as though it were mana|mana of any type can be spent|can spend mana of any type", re.I)


def strip_reminder(text):
    return re.sub(r"\([^()]*\)", "", text or "")


def sentences(text):
    for line in strip_reminder(text).split("\n"):
        for sentence in re.split(r"(?<=[.!])\s+", line):
            if sentence.strip():
                yield sentence.strip().strip('"')


def main():
    names = "--names" in sys.argv
    only = next((a.lower() for a in sys.argv[1:] if not a.startswith("--")), None)
    with open(CORPUS, encoding="utf-8") as corpus:
        data = json.load(corpus)

    sections = defaultdict(lambda: defaultdict(dict))  # section -> family -> name -> clause

    def put(section, family, card, clause):
        sections[section][family].setdefault(card["name"], clause)

    for card in data["cards"]:
        short = card["name"].split(",")[0].split(" //")[0]
        if "{S}" in (card.get("mana") or ""):
            put("3. {S} in a cost, and what makes snow mana", SNOW_COST[0][0], card, card["mana"])
        if "Snow" in card["type"] and card.get("produced"):
            put("3. {S} in a cost, and what makes snow mana", "a snow permanent that makes mana (a snow source)", card, card["type"])
        for keyword in card.get("kw") or []:
            if keyword in ("Converge", "Sunburst"):
                put("4. What reads the mana a payment spent", "its colors or types", card, f"kw: {keyword}")
                put("4b. Where the spent mana is read", "read as the permanent it became enters, or after", card, f"kw: {keyword}")
            if keyword == "Firebending":
                put("2. Mana kept past a step's end, and mana lost", KEEPS[0][0], card, "kw: Firebending")
        for raw in sentences(card["text"]):
            clause = raw.replace(short, "~")
            low = clause.lower()
            if RESTRICTION.search(clause):
                matched = False
                for family, pattern in RESTRICTION_READS:
                    if re.search(pattern, clause if "[A-Z]" in pattern else low, re.I if "[A-Z]" not in pattern else 0):
                        put("1. A restriction on what a unit may pay (CR 106.6)", family, card, clause)
                        matched = True
                if not matched:
                    put("1. A restriction on what a unit may pay (CR 106.6)", "unbucketed", card, clause)
            if RIDER.search(clause):
                kind = next((family for family, pattern in RIDER_KINDS if re.search(pattern, low)), "unbucketed")
                put("1b. A rider on what the unit is spent on (CR 106.6)", kind, card, clause)
            for family, pattern in KEEPS:
                if re.search(pattern, low):
                    put("2. Mana kept past a step's end, and mana lost", family, card, clause)
            if "{S}" in clause:
                kind = next((family for family, pattern in SNOW_COST[1:] if re.search(pattern, clause, re.I)), "unbucketed")
                put("3. {S} in a cost, and what makes snow mana", kind, card, clause)
            if SPENT.search(clause) and not (RIDER.search(clause) or RESTRICTION.search(clause) or AS_THOUGH.search(clause)):
                what = next((family for family, pattern in SPENT_READS if re.search(pattern, low)), "unbucketed")
                put("4. What reads the mana a payment spent", what, card, clause)
                where = next((family for family, pattern in SPENT_WHERE if re.search(pattern, low)), "unbucketed")
                put("4b. Where the spent mana is read", where, card, clause)
            for family, pattern in COST_NAMES_ITS_MANA:
                if re.search(pattern, low):
                    put("5. A cost that names the mana that may pay it", family, card, clause)
            if AS_THOUGH.search(clause):
                put("6. Mana spent as though another type (CR 609.4b), counted only", "any", card, clause)

    print(f"corpus: {data['query']}, fetched {data['fetched']}, {len(data['cards'])} cards\n")
    for section in sorted(sections):
        families = sections[section]
        distinct = {name for cards in families.values() for name in cards}
        print(f"{section}: {len(distinct)} cards")
        for family, cards in sorted(families.items(), key=lambda item: -len(item[1])):
            print(f"  {len(cards):4}  {family}")
            if names and (only is None or only in family.lower() or only in section.lower()):
                for name, clause in sorted(cards.items()):
                    print(f"          {name}: {clause[:220]}")
        print()


if __name__ == "__main__":
    main()
