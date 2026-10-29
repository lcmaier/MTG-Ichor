"""Rank Commander precon products by what the engine still needs to play all of their decks.

usage: precons.py <cards_needs.json> <out.json> [since=2018-01-01]
A product's Scryfall set lists every card printed in its decks, reprints included,
so its card list is the union of its decks: one product is one four-player table.
"""
import collections
import json
import os
import re
import subprocess
import sys
import time
import urllib.parse

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from mapping import status, BREADTH  # noqa: E402

needs_file, out_path = sys.argv[1], sys.argv[2]
since = sys.argv[3] if len(sys.argv) > 3 else "2018-01-01"


def get(url):
    for attempt in range(4):
        raw = subprocess.run(["curl", "-s", "-A", "mtgsim-census/0.1", "-H", "Accept: application/json", url],
                             capture_output=True, text=True, encoding="utf8").stdout
        try:
            return json.loads(raw)
        except json.JSONDecodeError:
            time.sleep(2 + 3 * attempt)
    sys.exit(f"failed: {url}")


def norm(s):
    s = s.split(" // ")[0].lower().replace("'", "").replace(",", "")
    return re.sub(r"[^a-z0-9]+", "_", s).strip("_")


cards = json.load(open(needs_file, encoding="utf8"))
fb = {}
for c in cards:
    if c["name"]:
        fb.setdefault(norm(c["name"]), c)

sets = [s for s in get("https://api.scryfall.com/sets")["data"]
        if s["set_type"] == "commander" and s["released_at"] >= since and 150 <= s["card_count"] <= 700]
products = []
for s in sorted(sets, key=lambda s: s["released_at"]):
    url = "https://api.scryfall.com/cards/search?unique=cards&q=" + urllib.parse.quote(f"e:{s['code']} -t:basic")
    names = []
    while url:
        d = get(url)
        if d.get("object") != "list":
            break
        names += [c["name"] for c in d["data"]]
        url = d.get("next_page") if d.get("has_more") else None
        time.sleep(0.12)
    named, mech, unclear, ok, missing = collections.Counter(), collections.Counter(), 0, 0, 0
    for n in names:
        c = fb.get(norm(n))
        if not c:
            missing += 1
            continue
        st = [status(x) for x in c["needs"]]
        steps = {x[2:] for x in st if x.startswith("M:") and x != "M:" + BREADTH}
        mechs = {x for x, y in zip(c["needs"], st) if y == "M:" + BREADTH}
        for x in steps:
            named[x] += 1
        for x in mechs:
            mech[x] += 1
        unclear += "U" in st
        ok += not steps and not mechs and "U" not in st and "D" not in st
    products.append({"code": s["code"], "name": s["name"], "released": s["released_at"], "cards": len(names),
                     "no_script": missing, "census_ok": ok, "unclear": unclear,
                     "named": dict(named), "mechanics": dict(mech)})
    time.sleep(0.1)
json.dump(products, open(out_path, "w", encoding="utf8"), indent=1)

products.sort(key=lambda p: (len(p["named"]) + len(p["mechanics"]), p["released"]))
print(f"{len(products)} Commander products since {since}, fewest distinct blockers first\n")
print(f"{'code':5s} {'released':10s} {'cards':>5s} {'ok%':>4s} {'steps':>5s} {'mechs':>5s}  name")
for p in products:
    print(f"{p['code']:5s} {p['released']} {p['cards']:5d} {100 * p['census_ok'] / max(p['cards'], 1):4.0f} "
          f"{len(p['named']):5d} {len(p['mechanics']):5d}  {p['name']}")
every = collections.Counter(x for p in products for x in p["named"])
print(f"\nNamed steps, by how many of the {len(products)} products need them:")
for x, k in every.most_common():
    cards_ = sum(p["named"].get(x, 0) for p in products)
    print(f"  {k:3d}/{len(products)}  {x}  ({cards_} cards across all)")
