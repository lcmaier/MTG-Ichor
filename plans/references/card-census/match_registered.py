"""Map registered card names to Forge cardsfolder paths; write sparse-checkout patterns.

usage: match_registered.py forge_paths.txt registered.txt letter out_patterns.txt out_map.tsv
"""
import re
import sys

paths_file, reg_file, letter, out_patterns, out_map = sys.argv[1:6]


def norm(s):
    s = s.lower().replace("'", "").replace(",", "")
    s = re.sub(r"[^a-z0-9]+", "_", s)
    return s.strip("_")


index = {}
for p in open(paths_file, encoding="utf8").read().split():
    if not p.endswith(".txt") or "/" not in p:
        continue
    folder, base = p.split("/", 1)
    if len(folder) != 1:
        continue  # skip rebalanced/, upcoming/
    index.setdefault(norm(base[:-4]), p)

names = [n for n in open(reg_file, encoding="utf8").read().split("\n") if n]
found, missing = {}, []
for n in names:
    p = index.get(norm(n))
    if p:
        found[n] = p
    else:
        missing.append(n)

with open(out_map, "w", encoding="utf8") as f:
    for n, p in sorted(found.items()):
        f.write(f"{n}\t{p}\n")
with open(out_patterns, "w", encoding="utf8") as f:
    f.write(f"/forge-gui/res/cardsfolder/{letter}/\n")
    for p in sorted(set(found.values())):
        f.write(f"/forge-gui/res/cardsfolder/{p}\n")
print(f"registered {len(names)}, matched {len(found)}, unmatched {len(missing)}")
print("unmatched:", "; ".join(missing))
