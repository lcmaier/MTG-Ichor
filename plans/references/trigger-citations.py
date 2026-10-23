"""Pass 2's citation list: every `triggers-architecture.md` § citation in tracked files.

A citation is the doc's file name closed by a backtick, then `§` and a section
token (`triggers-architecture.md` §4.2). Follow-on tokens in the same phrase
(`§4.2 and §4.3`, `§3–§6`) are counted apart, as are the doc's own bare
self-citations. Each lands on a heading; each heading is classed against the
item 6 audit's eviction table (plans/handoffs/item-6-audit.md, pass 2).

    python plans/references/trigger-citations.py ROOT [--list]
"""
import re
import subprocess
import sys
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(sys.argv[1])
LIST = "--list" in sys.argv
DOC = "plans/triggers-architecture.md"

STRICT = re.compile(r"triggers-architecture\.md`\s*§\s*([0-9]+(?:\.[0-9]+)*[a-z]?)")
FOLLOW = re.compile(r"^(?:[\s,;/&]|and|or|to|–|—|-)+§\s*([0-9]+(?:\.[0-9]+)*[a-z]?)")
BARE = re.compile(r"(?<![A-Za-z0-9`.])§\s*([0-9]+(?:\.[0-9]+)*[a-z]?)")
OTHER_DOC = re.compile(r"[A-Za-z0-9_\-]+\.(?:md|rs|py|txt)`?\s*$")

# The eviction table: whole sections that move, and sections part of which moves.
MOVES = {"0", "1", "2.1", "8", "14", "17"}
PARTLY = {"11.1": "its reading", "12": "the two re-counts and the TR-4/TR-5 roll-ups",
          "4.9": "the design input for `loop-detection` (~260 of its lines)",
          "15": "the findings pass 1 closes"}


def headings():
    """Section token -> (line, heading text) for the doc's numbered headings."""
    out = {}
    for i, line in enumerate((ROOT / DOC).read_text(encoding="utf-8").splitlines(), 1):
        m = re.match(r"^(#{2,4})\s+([0-9]+(?:\.[0-9]+)*[a-z]?)\.?\s+(.*)", line)
        if m:
            out[m.group(2)] = (i, line.strip())
    return out


def classify(token):
    parts = token.split(".")
    for k in range(len(parts), 0, -1):
        prefix = ".".join(parts[:k])
        if prefix in MOVES:
            return "moves"
        if prefix in PARTLY:
            return "partly"
    return "stays"


def land(token, heads):
    if token in heads:
        return token, heads[token]
    parts = token.split(".")
    for k in range(len(parts) - 1, 0, -1):
        prefix = ".".join(parts[:k])
        if prefix in heads:
            return prefix, (heads[prefix][0], heads[prefix][1] + f"  [no heading {token}]")
    return None, (0, f"[no heading {token}]")


def tracked():
    out = subprocess.run(["git", "ls-files", "plans", "mtgsim/src", "mtgsim/tests", "devgui"],
                         cwd=ROOT, capture_output=True, text=True, encoding="utf-8").stdout
    return [p for p in out.splitlines() if p.endswith((".md", ".rs", ".py", ".txt", ".toml", ".scenario"))]


def area(path):
    if path.startswith("plans/archive/"):
        return "plans/archive"
    if path.startswith("plans/"):
        return "plans"
    if path.startswith("mtgsim/src/"):
        return "src"
    if path.startswith("mtgsim/tests/"):
        return "tests"
    return "devgui"


def main():
    heads = headings()
    rows = []          # (area, path, line, token, kind)
    for path in tracked():
        try:
            text = (ROOT / path).read_text(encoding="utf-8")
        except (UnicodeDecodeError, FileNotFoundError):
            continue
        for n, line in enumerate(text.splitlines(), 1):
            for m in STRICT.finditer(line):
                rows.append((area(path), path, n, m.group(1), "strict", line[m.end():]))
                rest = line[m.end():]
                while True:
                    f = FOLLOW.match(rest)
                    if not f:
                        break
                    rows.append((area(path), path, n, f.group(1), "follow-on", rest[f.end():]))
                    rest = rest[f.end():]
            if path == DOC:
                for m in BARE.finditer(line):
                    before = line[:m.start()]
                    if OTHER_DOC.search(before) or before.rstrip().endswith("CR"):
                        continue
                    rows.append(("self", path, n, m.group(1), "self", line[m.end():]))

    strict = [r for r in rows if r[4] == "strict"]
    by_area = Counter(r[0] for r in strict)
    print("Strict citations (`triggers-architecture.md` §N):", len(strict))
    for a in ("plans", "plans/archive", "src", "tests", "devgui"):
        print(f"  {a:14} {by_area.get(a, 0)}")
    print("  plans total   ", by_area.get("plans", 0) + by_area.get("plans/archive", 0),
          f"({by_area.get('plans', 0)} outside the archive)")
    print("  src + tests   ", by_area.get("src", 0) + by_area.get("tests", 0))
    print("Follow-on tokens in the same phrase:", sum(1 for r in rows if r[4] == "follow-on"))
    print("The doc's own bare self-citations:", sum(1 for r in rows if r[4] == "self"))

    print("\nBy target section (strict + follow-on, outside the doc itself; self in brackets):")
    per = defaultdict(Counter)
    for r in rows:
        tok, _ = land(r[3], heads)
        per[tok or r[3]][r[4]] += 1
    cls_tot = Counter()
    def order(t):
        return [(int(x), "") if x.isdigit() else (10**6, x) for x in re.sub(r"([a-z])$", r".\1", t).split(".")]
    for tok in sorted(per, key=order):
        c = per[tok]
        k = classify(tok)
        cls_tot[k, "cite"] += c["strict"] + c["follow-on"]
        cls_tot[k, "self"] += c["self"]
        line, head = heads.get(tok, (0, "[no such heading]"))
        note = f"  <- {k}" + (f": {PARTLY.get(tok)}" if tok in PARTLY else "") if k != "stays" else ""
        print(f"  §{tok:6} {c['strict']:4} +{c['follow-on']:<3} [{c['self']:3}]  L{line:<5} {head[:70]}{note}")
    print("\nBy eviction class:")
    for k in ("moves", "partly", "stays"):
        print(f"  {k:7} citations {cls_tot[k, 'cite']:4}   self {cls_tot[k, 'self']:4}")

    unresolved = [r for r in rows if r[3] not in heads]
    if unresolved:
        print("\nTokens with no heading of their own:", Counter(r[3] for r in unresolved).most_common())

    print("\nMoved-target citations, outside the doc, by what the words after the token name:")
    verdicts = Counter()
    listing = defaultdict(list)
    for r in rows:
        if r[4] == "self" or classify(r[3]) == "stays":
            continue
        v = part_named(r[3], r[5])
        verdicts[r[3], v] += 1
        listing[r[3], v].append(f"{r[1]}:{r[2]}")
    for (tok, v), n in sorted(verdicts.items()):
        print(f"  §{tok:5} {v:44} {n}")
    if LIST:
        for (tok, v), where in sorted(listing.items()):
            print(f"\n  §{tok} — {v}:")
            byfile = defaultdict(list)
            for w in where:
                f, l = w.rsplit(":", 1)
                byfile[f].append(l)
            for f, ls in byfile.items():
                print(f"    {f}: {', '.join(ls)}")


#: Phase stubs stay; §12's two re-counts and its TR-4/TR-5 roll-ups move.
STUB = re.compile(r"^\W{0,6}(?:'s\s+)?(?:\(|,\s*)?\s*(?:the\s+)?TR-[0-9][a-z]\b")
ROLLUP = re.compile(r"^\W{0,6}(?:'s\s+)?(?:\(|,\s*)?\s*(?:the\s+)?TR-[45]\b(?![a-z])|re-?count|roll-?up", re.I)


def part_named(token, after):
    """Which part of a partly-moving section a citation names, from its next words."""
    head = after[:90]
    if token == "12":
        if ROLLUP.search(head):
            return "a re-count or roll-up (moves)"
        if STUB.search(head) or re.search(r"TR-[0-9][a-z]?\b", head[:40]):
            return "a phase stub (stays)"
        return "the section (doubtful: a reader decides)"
    if token == "4.9":
        if re.search(r"B14|loop-detection|design input|detect", head, re.I):
            return "`loop-detection`'s design input (moves)"
        if re.search(r"draw", head, re.I):
            return "the draw (stays)"
        return "the section (doubtful: a reader decides)"
    if token == "11.1":
        return "its reading (moves)" if re.search(r"read|measur", head, re.I) else "the section (doubtful: a reader decides)"
    if token == "15":
        m = re.search(r"(?:item|finding)s?\s+([0-9]+)", head)
        return f"finding {m.group(1)} (moves if pass 1 closes it)" if m else "the section (doubtful: a reader decides)"
    return "whole section (moves)"


if __name__ == "__main__":
    main()
