"""How tangled is each part of the CR? Size, and how many other rules cite it.

A unit is a three-digit section (613), except 701 and 702, whose numbered
entries are each a keyword action or keyword (702.37 morph).

usage: cr-coupling.py MTG-Rules/versions/tmnt.txt [unit,unit,...]
Prints the units cited by the most others, the largest by words, each named
system's citers, and the keyword families that cite a base rule (708 is cited
by Morph, Manifest, Cloak and Disguise). Written for `route-review`, 2026-10-27.
"""
import re
import sys
from collections import defaultdict

path = sys.argv[1]
lines = open(path, encoding="utf-8").read().split("\n")

RULE = re.compile(r"^(\d{3})\.(\d+)([a-z]*)\.?\s")
REF_HEAD = re.compile(r"\b(?:rules?|section)\s+(\d{3}(?:\.\d+[a-z]?)?(?:(?:,\s*(?:and\s+|or\s+)?|\s+and\s+|\s+or\s+|[–-])\d{3}(?:\.\d+[a-z]?)?)*)")
NUM = re.compile(r"\d{3}(?:\.\d+[a-z]?)?")
TITLE = re.compile(r"^(\d{3})\.\s+(.+)$")
SUBTITLE = re.compile(r"^(70[12])\.(\d+)\.\s+(.+)$")
#: A 701/702 entry's title ("702.37. Morph") is a few words; a longer line
#: under the same numbering is a rule's text, not a keyword's name.
KEYWORD_TITLE_MAX = 60

def unit_of(num):
    sec = num[:3]
    if sec in ("701", "702") and "." in num:
        return sec + "." + re.match(r"\d{3}\.(\d+)", num).group(1)
    return sec

titles = {}
rules = []  # (rule_id, unit, text)
started = False
for l in lines:
    if l.strip() == "Glossary" and started:
        break
    m = TITLE.match(l)
    if m and not RULE.match(l):
        titles[m.group(1)] = m.group(2).strip()
    m2 = SUBTITLE.match(l)
    if m2 and not m2.group(3)[:1].isdigit() and len(m2.group(3)) < KEYWORD_TITLE_MAX:
        titles[m2.group(1) + "." + m2.group(2)] = m2.group(3).strip()
    m = RULE.match(l)
    if m:
        started = True
        rid = f"{m.group(1)}.{m.group(2)}{m.group(3)}"
        rules.append([rid, unit_of(rid), l])
    elif started and rules and l.startswith("Example"):
        rules[-1][2] += " " + l

size = defaultdict(int)
words = defaultdict(int)
fan_in_rules = defaultdict(set)
fan_in_units = defaultdict(set)
fan_out_units = defaultdict(set)
for rid, unit, text in rules:
    size[unit] += 1
    words[unit] += len(text.split())
    for head in REF_HEAD.finditer(text):
        for n in NUM.findall(head.group(1)):
            target = unit_of(n)
            if target == unit:
                continue
            fan_in_rules[target].add(rid)
            fan_in_units[target].add(unit)
            fan_out_units[unit].add(target)

units = set(size) | set(fan_in_rules)
rows = []
for u in units:
    rows.append((u, titles.get(u, ""), size[u], words[u], len(fan_in_rules[u]),
                 len(fan_in_units[u]), len(fan_out_units[u])))

def show(key, n, label):
    print(f"\n== top {n} by {label}")
    print(f"{'unit':<8}{'title':<44}{'rules':>6}{'words':>7}{'in_r':>6}{'in_u':>6}{'out_u':>6}")
    for r in sorted(rows, key=key, reverse=True)[:n]:
        print(f"{r[0]:<8}{r[1][:43]:<44}{r[2]:>6}{r[3]:>7}{r[4]:>6}{r[5]:>6}{r[6]:>6}")

show(lambda r: r[5], 40, "distinct units citing it (fan-in)")
show(lambda r: r[3], 25, "words")
if len(sys.argv) > 2:
    for u in sys.argv[2].split(","):
        r = [x for x in rows if x[0] == u]
        print(u, r, sorted(fan_in_units[u])[:60])

SYSTEMS = {
    "BUILT layers (611, 613)": ["611", "613"],
    "BUILT replacement/prevention (614-616)": ["614", "615", "616"],
    "BUILT triggers (603)": ["603"],
    "BUILT copy (707)": ["707"],
    "BUILT casting + activating (601, 602)": ["601", "602"],
    "BUILT SBA (704)": ["704"],
    "BUILT costs (118)": ["118"],
    "BUILT mana (106, 605)": ["106", "605"],
    "combat (506-511)": ["506", "507", "508", "509", "510", "511"],
    "multi-face cards (709-712, 715, 718-721)": ["709", "710", "712", "715", "718", "719", "720", "721"],
    "face-down (708)": ["708"],
    "linked abilities (607)": ["607"],
    "multiplayer (800-811)": [f"8{n:02d}" for n in range(0, 12)],
    "Commander (903)": ["903"],
    "zones + hidden info (400-408)": [f"40{n}" for n in range(0, 9)],
    "starting the game, mulligans (103)": ["103"],
    "timing, priority, special actions (116, 117)": ["116", "117"],
    "numbers and symbols, X, hybrid (107)": ["107"],
    "resolving, effects (608, 609)": ["608", "609"],
    "controlling another player (722)": ["722"],
    "merging (729)": ["729"],
    "subgames, restarting (726, 728)": ["726", "728"],
    "illegal actions, shortcuts (731, 732)": ["731", "732"],
    "text-changing (612)": ["612"],
}
print("\n== systems: words, rules, distinct outside units citing them")
out = []
for name, us in SYSTEMS.items():
    s = set(us)
    w = sum(words[u] for u in us); n = sum(size[u] for u in us)
    citers = set().union(*[fan_in_units[u] for u in us]) - s
    out.append((len(citers), w, n, name))
for c, w, n, name in sorted(out, reverse=True):
    print(f"{name:<48}{w:>7}{n:>6}{c:>6}")

print("\n== keyword families: 701/702 entries citing a base rule")
for base in ["708", "712", "709", "707", "729", "607", "903", "722", "613", "614", "603", "802"]:
    fam = sorted(u for u in fan_in_units[base] if u.startswith(("701.", "702.")))
    print(base, titles.get(base, ""), "->", ", ".join(f"{u} {titles.get(u,'')}" for u in fam))
