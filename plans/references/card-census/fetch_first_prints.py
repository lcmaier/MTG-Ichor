"""Each card's first printing: its set, the set's type and release date.

usage: fetch_first_prints.py <out.json>
`not:reprint` keeps only original printings; `order=released` asc puts the
earliest first, so the printing `unique=cards` keeps is the card's first.
"""
import json
import subprocess
import sys
import time
import urllib.parse

out_path = sys.argv[1]
q = "game:paper -is:funny not:reprint"
url = ("https://api.scryfall.com/cards/search?unique=cards&order=released&dir=asc&q="
       + urllib.parse.quote(q))
cards, pages = [], 0
while url:
    for attempt in range(4):
        raw = subprocess.run(
            ["curl", "-s", "-A", "mtgsim-census/0.1", "-H", "Accept: application/json", url],
            capture_output=True, text=True, encoding="utf8",
        ).stdout
        try:
            d = json.loads(raw)
            if d.get("object") == "list":
                break
        except json.JSONDecodeError:
            pass
        time.sleep(2 + 3 * attempt)
    else:
        sys.exit(f"page {pages + 1} failed: {raw[:200]}")
    for c in d["data"]:
        cards.append({k: c.get(k) for k in ("name", "set", "set_name", "set_type", "released_at", "edhrec_rank")}
                     | {"commander": c.get("legalities", {}).get("commander")})
    pages += 1
    url = d.get("next_page") if d.get("has_more") else None
    time.sleep(0.15)
json.dump(cards, open(out_path, "w", encoding="utf8"))
print(f"{len(cards)} first printings over {pages} pages; total_cards said {d.get('total_cards')}")
