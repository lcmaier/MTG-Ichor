"""The unclear residue, ranked by Commander-legal cards: what to map next.

usage: residue.py <cards_needs.json> <scryfall.json> [N] [--examples]
"""
import collections
import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from mapping import status  # noqa: E402

cards = json.load(open(sys.argv[1], encoding="utf8"))
scry = json.load(open(sys.argv[2], encoding="utf8"))
top_n = int(sys.argv[3]) if len(sys.argv) > 3 and sys.argv[3].isdigit() else 80
examples = "--examples" in sys.argv


def norm(s):
    s = s.split(" // ")[0].lower().replace("'", "").replace(",", "")
    return re.sub(r"[^a-z0-9]+", "_", s).strip("_")


fb = {}
for c in cards:
    if c["name"]:
        fb.setdefault(norm(c["name"]), c)
pop = [fb[norm(s["name"])] for s in scry if s["commander"] == "legal" and norm(s["name"]) in fb]

touch, alone, ex = collections.Counter(), collections.Counter(), collections.defaultdict(list)
for c in pop:
    st = {n: status(n) for n in c["needs"]}
    u = [n for n, s in st.items() if s == "U"]
    blocked = any(s.startswith("M:") for s in st.values())
    for n in u:
        touch[n] += 1
        if len(ex[n]) < 2:
            ex[n].append(c["name"])
    if len(u) == 1 and not blocked:
        alone[u[0]] += 1
unclear_cards = sum(1 for c in pop if any(status(n) == "U" for n in c["needs"]) and not any(status(n).startswith("M:") for n in c["needs"]))
print(f"{len(touch)} distinct unclear needs; {unclear_cards} cards blocked only by unclear needs ({100 * unclear_cards / len(pop):.1f}%)")
for n, k in touch.most_common(top_n):
    tail = f"  e.g. {'; '.join(ex[n])}" if examples else ""
    print(f"{k:5d} {alone[n]:5d}  {n}{tail}")
