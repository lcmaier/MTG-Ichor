"""The pilot census: coverage, blockers and the unlock curve for one letter.

usage: census.py <cards_needs.json> <scryfall.json> <registered_map.tsv> <letter>
"""
import collections
import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from mapping import status  # noqa: E402

needs_file, scry_file, reg_file, letter = sys.argv[1:5]
TOP_RANK = 5000


def norm(s):
    s = s.split(" // ")[0].lower().replace("'", "").replace(",", "")
    return re.sub(r"[^a-z0-9]+", "_", s).strip("_")


forge = json.load(open(needs_file, encoding="utf8"))
by_path = {c["path"]: c for c in forge}
forge_by_name = {}
for c in forge:
    if c["name"]:
        forge_by_name.setdefault(norm(c["name"]), c)


def verdicts(card):
    return {n: status(n) for n in card["needs"]}


def bin_of(v):
    s = set(v.values())
    if "D" in s:
        return "digital"
    if any(x.startswith("M:") for x in s):
        return "blocked"
    if "U" in s:
        return "unclear"
    return "authorable"


# ---- calibration: the registered cards -------------------------------------
print("== Calibration: registered cards ==")
reg = [line.split("\t") for line in open(reg_file, encoding="utf8").read().splitlines() if line]
cal = collections.Counter()
misses = []
for name, path in reg:
    v = verdicts(by_path[path])
    b = bin_of(v)
    cal[b] += 1
    if b != "authorable":
        misses.append((name, b, sorted(n for n, s in v.items() if s not in ("B", "F"))))
print(dict(cal))
for name, b, why in misses:
    print(f"  {b:10s} {name}: {', '.join(why)}")

# ---- the letter --------------------------------------------------------------
scry = json.load(open(scry_file, encoding="utf8"))
pop, no_script = [], []
for s in scry:
    if s["commander"] != "legal":
        continue
    c = forge_by_name.get(norm(s["name"]))
    if not c or (letter != "all" and not c["path"].startswith(letter + "/")):
        no_script.append(s["name"])
        continue
    pop.append((s, c, verdicts(c)))
print(f"\n== Letter {letter}: {len(pop)} Commander-legal cards with a Forge script; {len(no_script)} without ==")
if no_script:
    print("  without a script (sample):", "; ".join(no_script[:12]))


def report(rows, label):
    bins = collections.Counter(bin_of(v) for _, _, v in rows)
    n = len(rows)
    print(f"\n-- {label}: {n} cards --")
    for k in ("authorable", "unclear", "blocked", "digital"):
        print(f"  {k:10s} {bins[k]:5d}  {100 * bins[k] / max(n, 1):5.1f}%")
    any_block, alone = collections.Counter(), collections.Counter()
    for _, _, v in rows:
        owners = {s[2:] for s in v.values() if s.startswith("M:")}
        has_u = "U" in v.values()
        for o in owners:
            any_block[o] += 1
        if len(owners) == 1 and not has_u:
            alone[next(iter(owners))] += 1
    print("  blocker                                      blocks  alone")
    for o, k in any_block.most_common():
        print(f"  {o:44s} {k:6d} {alone[o]:6d}")
    return any_block


def curve(rows, label, unclear_counts_as_built):
    owners_of = []
    for _, _, v in rows:
        owners = {s[2:] for s in v.values() if s.startswith("M:")}
        if "U" in v.values() and not unclear_counts_as_built:
            owners.add("(unclear needs)")
        if "D" in v.values():
            continue
        owners_of.append(owners)
    done = set()
    total = len(owners_of)
    ready = sum(1 for o in owners_of if not o)
    print(f"\n-- Unlock curve, {label} ({'unclear = built' if unclear_counts_as_built else 'unclear = blocking'}) --")
    print(f"  {'today':44s} {ready:5d}  {100 * ready / total:5.1f}%")
    while True:
        gains = collections.Counter()
        for o in owners_of:
            rest = o - done
            if len(rest) == 1:
                gains[next(iter(rest))] += 1
        if not gains:
            break
        best, g = gains.most_common(1)[0]
        if best == "(unclear needs)" and len(gains) > 1:
            best, g = [x for x in gains.most_common() if x[0] != "(unclear needs)"][0]
        done.add(best)
        ready += g
        print(f"  + {best:42s} {ready:5d}  {100 * ready / total:5.1f}%")
    left = total - ready
    if left:
        print(f"  {'(left: two or more owners at once)':44s} {left:5d}")


report(pop, f"letter {letter}, Commander-legal")
top = [r for r in pop if r[0]["edhrec_rank"] and r[0]["edhrec_rank"] <= TOP_RANK]
report(top, f"letter {letter}, EDHREC rank <= {TOP_RANK}")
curve(pop, f"letter {letter}, Commander-legal", False)
curve(pop, f"letter {letter}, Commander-legal", True)
curve(top, f"letter {letter}, EDHREC rank <= {TOP_RANK}", False)

# ---- the residue: unclear needs, by cards they touch -------------------------
u = collections.Counter(n for _, _, v in pop for n, s in v.items() if s == "U")
print(f"\n== Unclear needs: {len(u)} distinct; the most common ==")
for n, k in u.most_common(40):
    print(f"  {k:4d} {n}")
