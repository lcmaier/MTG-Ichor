"""The census by first printing, in release order: what set-by-set authoring meets, and when.

usage: by_set.py <cards_needs.json> <first_prints.json>
"""
import collections
import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from mapping import status, BREADTH  # noqa: E402

cards = json.load(open(sys.argv[1], encoding="utf8"))
firsts = json.load(open(sys.argv[2], encoding="utf8"))


def norm(s):
    s = s.split(" // ")[0].lower().replace("'", "").replace(",", "")
    return re.sub(r"[^a-z0-9]+", "_", s).strip("_")


fb = {}
for c in cards:
    if c["name"]:
        fb.setdefault(norm(c["name"]), c)

rows = []  # (first print, needs -> status)
for f in firsts:
    c = fb.get(norm(f["name"]))
    if c:
        rows.append((f, {n: status(n) for n in c["needs"]}))
rows.sort(key=lambda r: (r[0]["released_at"], r[0]["set"]))
print(f"{len(rows)} first printings joined to a Forge script (of {len(firsts)})")

BR = "M:" + BREADTH


def owners(st):
    """Named steps and breadth mechanics a card waits on (unclear counted as its own)."""
    named = {s[2:] for s in st.values() if s.startswith("M:") and s != BR}
    mech = {n for n, s in st.items() if s == BR}
    return named, mech, "U" in st.values(), "D" in st.values()


# ---- each set, in release order ----------------------------------------------
sets = collections.OrderedDict()
for f, st in rows:
    key = (f["released_at"], f["set"])
    sets.setdefault(key, []).append((f, st))

first_need, sets_needing, cards_needing = {}, collections.defaultdict(set), collections.Counter()
for (date, code), members in sets.items():
    for f, st in members:
        named, mech, _, _ = owners(st)
        for o in named | {f"mech:{m}" for m in mech}:
            first_need.setdefault(o, (date, code, members[0][0]["set_name"]))
            sets_needing[o].add(code)
            cards_needing[o] += 1

print("\n== The first sets, in release order ==")
print(f"{'released':10s} {'set':5s} {'type':12s} {'cards':>5s} {'census-ok':>9s} {'top1000':>7s}  named steps it needs")
for (date, code), members in list(sets.items())[:14]:
    ok = sum(1 for _, st in members if not any(s.startswith("M:") or s in ("U", "D") for s in st.values()))
    top = sum(1 for f, _ in members if f["edhrec_rank"] and f["edhrec_rank"] <= 1000)
    need = collections.Counter(o for _, st in members for o in owners(st)[0])
    print(f"{date} {code:5s} {members[0][0]['set_type']:12s} {len(members):5d} {ok:9d} {top:7d}  "
          + ", ".join(f"{o} {k}" for o, k in need.most_common()))

# ---- when each named step is first needed ------------------------------------
print("\n== Each named step: the first set that needs it, and how widely ==")
named_steps = sorted((o for o in first_need if not o.startswith("mech:")), key=lambda o: first_need[o])
for o in named_steps:
    d, code, name = first_need[o]
    print(f"  {o:42s} first {d} {code:5s} {name[:28]:28s} sets {len(sets_needing[o]):4d}  cards {cards_needing[o]:5d}")

# ---- breadth mechanics: how many sets each touches --------------------------
mechs = [o for o in first_need if o.startswith("mech:")]
spread = collections.Counter()
for m in mechs:
    k = len(sets_needing[m])
    spread["1 set" if k == 1 else "2-4 sets" if k <= 4 else "5-9 sets" if k <= 9 else "10+ sets"] += 1
print(f"\n== {len(mechs)} breadth mechanics, by how many sets need them ==")
for b in ("1 set", "2-4 sets", "5-9 sets", "10+ sets"):
    in_b = [m for m in mechs if (lambda k: ("1 set" if k == 1 else "2-4 sets" if k <= 4 else "5-9 sets" if k <= 9 else "10+ sets"))(len(sets_needing[m])) == b]
    print(f"  {b:9s} {spread[b]:4d} mechanics, {sum(cards_needing[m] for m in in_b):5d} cards")
print("  the 10+ set ones:", ", ".join(f"{m[5:]} ({len(sets_needing[m])} sets, {cards_needing[m]} cards, first {first_need[m][1]})"
                                       for m in sorted(mechs, key=lambda m: -len(sets_needing[m])) if len(sets_needing[m]) >= 10))

# ---- where the most-played cards were first printed ---------------------------
print("\n== Where EDHREC's top 1,000 were first printed ==")
tops = [(f["released_at"][:4], f["set_type"]) for f, _ in rows if f["edhrec_rank"] and f["edhrec_rank"] <= 1000]
by_era = collections.Counter("1993-2002" if y <= "2002" else "2003-2010" if y <= "2010" else "2011-2017" if y <= "2017" else "2018-2026" for y, _ in tops)
all_era = collections.Counter("1993-2002" if f["released_at"][:4] <= "2002" else "2003-2010" if f["released_at"][:4] <= "2010" else "2011-2017" if f["released_at"][:4] <= "2017" else "2018-2026" for f, _ in rows)
for e in ("1993-2002", "2003-2010", "2011-2017", "2018-2026"):
    print(f"  {e}: {by_era[e]:4d} of the top 1,000; {all_era[e]:5d} cards in all")
print("  by set type:", dict(collections.Counter(t for _, t in tops).most_common(8)))
