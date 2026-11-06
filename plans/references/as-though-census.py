"""What printed "as though" effects re-answer, by the question each one changes.

    python plans/references/as-though-census.py            # the counts
    python plans/references/as-though-census.py --names    # each family's cards and clauses

Behind `as-though-effects` (`backlog.md` 2.24, `cant-effects-architecture.md`
3.7). A card is in a family by the clause that does it: the sentence holding
"as though" (CR 609.4), or a wording the CR defines as the same fiction. Reminder
text is stripped first, so a keyword counts by its keyword.

A word search is not enough, which is what the 2026-09-09 count of 287 was: it
split the shapes by rough query, and it could not see the cards that apply the
same fiction without the words. So this also counts CR 118.14's "mana of any type
can be spent", and "any time you could cast an instant", which is flash's timing
stated outright.

The second axis is where the effect comes from, read off the clause's shape, so
it is approximate:
  self    the card's own spell or ability ("this spell")
  linked  one object or one cast another effect named ("that spell", "the exiled card")
  turn    a one-shot that lasts this turn
  static  a standing effect over a class of things

The classifier counts; it does not judge. A clause it cannot place lands in
UNPLACED for a person to read. Every family under 40 was read card by card.

INPUT
-----
`.census-cast.json` in this directory, `cast-census.py`'s cache of every
Commander-legal paper card. If it is missing, run `python
plans/references/cast-census.py` first (about seven minutes). Banned cards and
digital-only cards are not in it.
"""
import json
import os
import re
import sys
from collections import Counter, defaultdict

HERE = os.path.dirname(os.path.abspath(__file__))
CORPUS = os.path.join(HERE, ".census-cast.json")

# Checked in order, so a clause lands in the first family that matches.
FAMILIES = [
    ("payment", r"as though (it|they) (were|was) (\w+ )?(mana|colorless|red|white|blue|black|green)|mana of any type can be spent"),
    ("timing: flash, on a choice made casting", r"had flash (if you (pay|behold)|if (it|x|that spell)\b|by tapping)"),
    ("timing: flash", r"as though (it|they|that spell|those spells) had flash"),
    ("timing: stated outright", r"any time you could cast an instant"),
    ("activation: haste", r"activate .*as though .*had haste"),
    ("activation: a use limit", r"as though .*(haven't been activated|hadn't been activated|loyalty abilities (had|have) been activated)"),
    ("attack: defender", r"as though (it|they) didn't have defender"),
    ("attack: haste", r"as though (it|they|those creatures) had haste"),
    ("block: untapped", r"as though (it|they) (were|was) untapped"),
    ("block: evasion", r"as though (it|they|that creature|those creatures) (had|didn't have) (\w+walk|shadow|reach|\w+ abilities|landwalk)|didn't have .*walk"),
    ("target: hexproof or shroud", r"as though .*didn't have (hexproof|shroud)"),
    ("damage: unblocked", r"as though (it|he|she|they) (weren't|wasn't) blocked"),
    ("damage: its source", r"as though (its|the|that) source had"),
    ("value: power", r"as though (its|his|her|their) power were"),
    ("value: counters", r"as though (it|they) had no"),
    ("zone", r"as though (those|the|that|it|they)( cards?)? (were|was) in"),
]


def strip_reminder(text):
    return re.sub(r"\([^)]*\)", "", text)


def clauses(text):
    for line in strip_reminder(text).split("\n"):
        for sentence in re.split(r'(?<=[.!?"])\s+', line):
            if re.search(r"\bas though\b|mana of any type can be spent|any time you could cast an instant", sentence, re.I):
                yield sentence.strip()


def family(clause):
    for name, pattern in FAMILIES:
        if re.search(pattern, clause, re.I):
            return name
    return "UNPLACED"


def source(clause):
    lowered = clause.lower()
    if re.search(r"\bthis (spell|card|creature|ability)\b|^this ", lowered) and "as though" in lowered:
        if re.search(r"\bthis spell\b|\bthis card\b|cast this", lowered):
            return "self"
    if re.search(r"\bthat (spell|card|creature|permanent)\b|\bthose (spells|cards)\b|the exiled|exiled with|\bthem\b", lowered):
        return "linked"
    if "this turn" in lowered or "until end of turn" in lowered or "until your next turn" in lowered:
        return "turn"
    return "static"


def main():
    with open(CORPUS, encoding="utf8") as handle:
        corpus = json.load(handle)
    by_family = defaultdict(list)
    for card in corpus["cards"]:
        for clause in clauses(card["text"]):
            by_family[family(clause)].append((card["name"], clause, source(clause)))
    names_only = {name for rows in by_family.values() for name, _, _ in rows}
    print(f"corpus fetched {corpus['fetched']}: {len(corpus['cards'])} cards, {len(names_only)} with a clause")
    print(f"{'family':32} {'cards':>5}  self linked turn static")
    for name in sorted(by_family, key=lambda family_name: -len({row[0] for row in by_family[family_name]})):
        rows = by_family[name]
        sources = Counter(row[2] for row in rows)
        cards = len({row[0] for row in rows})
        print(f"{name:32} {cards:5}  {sources['self']:4} {sources['linked']:6} {sources['turn']:4} {sources['static']:6}")
    if "--names" in sys.argv:
        for name, rows in sorted(by_family.items()):
            print(f"\n=== {name}")
            for card, clause, where in sorted(rows):
                print(f"  [{where}] {card}: {clause}")


if __name__ == "__main__":
    main()
