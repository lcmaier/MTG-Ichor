"""The five lists' cards off Scryfall, for `pilot_pick.py`.

    python -I plans/references/card-census/pilot_fetch.py <out.json>

Fetched by name, since `e:tdc` misses the 12 list cards printed in `tdm`.
"""

import json
import subprocess
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent.parent))
from check_state_of_play import deck_cards  # noqa: E402


def main():
    names = sorted(deck_cards())
    cards = {}
    for start in range(0, len(names), 70):
        ids = [{"name": n.split(" // ")[0]} for n in names[start:start + 70]]
        raw = subprocess.run(
            ["curl", "-s", "-A", "mtgsim-census/0.1", "-H", "Content-Type: application/json", "-H", "Accept: application/json",
             "-d", json.dumps({"identifiers": ids}), "https://api.scryfall.com/cards/collection"],
            capture_output=True, text=True, encoding="utf-8").stdout
        for card in json.loads(raw)["data"]:
            cards[card["name"]] = card
        time.sleep(0.2)
    missing = [n for n in names if n not in cards]
    json.dump(cards, open(sys.argv[1], "w", encoding="utf-8"))
    print(f"{len(cards)} of {len(names)} fetched{'; missing ' + ', '.join(missing) if missing else ''}")


if __name__ == "__main__":
    main()
