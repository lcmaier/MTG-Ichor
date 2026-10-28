"""Fetch every paper, non-joke card from Scryfall's search, keeping the census's fields.

usage: fetch_scryfall.py <out.json>
Scryfall 403s a bare urllib request, so this shells out to curl with a UA header,
and sleeps between pages to stay under its rate limit.
"""
import json
import subprocess
import sys
import time
import urllib.parse

out_path = sys.argv[1]
q = "game:paper -is:funny"
url = "https://api.scryfall.com/cards/search?unique=cards&order=name&q=" + urllib.parse.quote(q)
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
        cards.append({
            "name": c.get("name"),
            "layout": c.get("layout"),
            "edhrec_rank": c.get("edhrec_rank"),
            "type_line": c.get("type_line"),
            "set_type": c.get("set_type"),
            "commander": c.get("legalities", {}).get("commander"),
            "modern": c.get("legalities", {}).get("modern"),
            "standard": c.get("legalities", {}).get("standard"),
        })
    pages += 1
    url = d.get("next_page") if d.get("has_more") else None
    time.sleep(0.15)
json.dump(cards, open(out_path, "w", encoding="utf8"))
print(f"{len(cards)} cards over {pages} pages; total_cards said {d.get('total_cards')}")
