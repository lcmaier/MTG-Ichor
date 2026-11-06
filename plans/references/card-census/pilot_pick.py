"""The card pilot's draw: four cards a deck, seeded, spread across verdicts.

    python plans/references/card-census/pilot_pick.py <deck_cards.json>

`deck_cards.json` is a name-to-Scryfall-card map of the five lists. The
census's per-card verdicts are not kept (only its counts are, and re-running
it needs a Forge clone), so a card's verdict here is a read of its Oracle text
for the route rows the census names, first match wins, and `plain` for a card
that names none. That is what `plain` approximates: the census's "authorable",
which README.md says cannot be trusted on its own.
"""

import glob
import json
import random
import re
import sys
from pathlib import Path

SEED = 20261106
ROOT = Path(__file__).resolve().parent.parent.parent.parent
BASICS = {"Plains", "Island", "Swamp", "Mountain", "Forest", "Wastes"}
VERDICTS = [
    ("casting-from-other-zones", r"flashback|escape|encore|eternalize|suspend|you may cast|from your graveyard.*cast|cast .* from"),
    ("any-color", r"mana of any|any color|any type"),
    ("modal", r"choose (one|two|three|up to)|tempting offer|council's dilemma"),
    ("remaining-costs", r"^(discard|remove|exile)[^:\n]*:|, discard[^:\n]*:|, remove[^:\n]*:|, exile[^:\n]*:|as an additional cost"),
    ("information-model", r"reveal|look at|top of (your|their) library|search your library"),
    ("copies", r"copy"),
    ("combat", r"can't (attack|block)|attacks? each|goad|must be blocked|menace"),
]


def oracle(card):
    faces = card.get("card_faces") or [card]
    return "\n".join(f.get("oracle_text", "") for f in faces).lower()


def verdict(card):
    text = oracle(card)
    for name, pattern in VERDICTS:
        if re.search(pattern, text, re.M):
            return name
    return "plain"


def main():
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    cards = json.load(open(sys.argv[1], encoding="utf-8"))
    registry = (ROOT / "mtgsim/src/cards/registry.rs").read_text(encoding="utf-8")
    registered = set(re.findall(r'registry\.register\(\s*"([^"]+)"', registry))
    rng = random.Random(SEED)
    taken, counts = set(), {}
    for card in cards.values():
        counts[verdict(card)] = counts.get(verdict(card), 0) + 1
    print(f"seed {SEED}; verdicts over the {len(cards)} cards: {sorted(counts.items(), key=lambda kv: -kv[1])}\n")
    used = {}
    for path in sorted(glob.glob(str(ROOT / "mtgsim/decks/tdc-*.txt"))):
        names = []
        for line in open(path, encoding="utf-8"):
            m = re.match(r"\d+ (.+)$", line.strip())
            if m and m.group(1) not in BASICS and m.group(1) not in registered and m.group(1) not in taken:
                names.append(m.group(1))
        rng.shuffle(names)
        picked = []
        # The verdict used least so far goes first, so twenty cards span them.
        while len(picked) < 4:
            names.sort(key=lambda n: used.get(verdict(cards[n]), 0))
            name = names.pop(0)
            picked.append(name)
            taken.add(name)
            used[verdict(cards[name])] = used.get(verdict(cards[name]), 0) + 1
        print(Path(path).stem)
        for name in picked:
            print(f"  {verdict(cards[name]):26} {name}")


if __name__ == "__main__":
    main()
