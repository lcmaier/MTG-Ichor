"""The held-out draw: seeded cards from outside the five lists.

    python plans/references/card-census/fetch_scryfall.py <cards.json>
    python plans/references/card-census/held_out_draw.py <cards.json> --seed <YYYYMMDD>

`cards.json` is `fetch_scryfall.py`'s output, every paper card with its
Commander legality. The population is the Commander-legal ones less the five
lists and the basic lands, by name and sorted, so one seed draws the same cards
from the same fetch. The fetch is not kept, so the draw is: its cards and the
population's size go to `results/held-out-draw-<seed>.txt`. Each card drawn is
then read against the route for a need no row and no gap holds; README.md's
"The held-out draws" keeps each draw's count.
"""

import argparse
import json
import random
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent.parent))
from check_state_of_play import BASIC_LANDS, deck_cards  # noqa: E402

RESULTS = Path(__file__).resolve().parent / "results"

def population(cards, in_the_lists):
    """The names a draw is made from, sorted."""
    legal = {card["name"] for card in cards if card.get("commander") == "legal"}
    return sorted(legal - in_the_lists - BASIC_LANDS)


def main():
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    parser = argparse.ArgumentParser()
    parser.add_argument("cards", help="fetch_scryfall.py's output")
    parser.add_argument("--seed", type=int, required=True, help="the day of the draw, YYYYMMDD")
    parser.add_argument("--count", type=int, default=30)
    parser.add_argument("--results", type=Path, default=RESULTS, help="where the draw is kept")
    args = parser.parse_args()
    names = population(json.loads(Path(args.cards).read_text(encoding="utf-8")), deck_cards())
    drawn = [f"{args.count} of {len(names)} cards outside the lists, seed {args.seed}:"]
    drawn += [f"  {name}" for name in random.Random(args.seed).sample(names, args.count)]
    kept = args.results / f"held-out-draw-{args.seed}.txt"
    kept.write_text("\n".join(drawn) + "\n", encoding="utf-8", newline="\n")
    print("\n".join(drawn))
    print(f"kept in {kept}")


if __name__ == "__main__":
    main()
