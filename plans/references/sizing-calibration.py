#!/usr/bin/env python3
"""How far estimates ran from what landed, grouped by the estimate's kind, and
what share of merged PRs was the route. Writes `sizing-calibration.md`, which
`roadmap-v2.md`'s Sizing section reads.

    python plans/references/sizing-calibration.py

Measured from git, so a PR body's own figure never enters it: code plus tests
(`mtgsim/src`, `mtgsim/tests`), lines added across the merge's first parent,
review rounds included, generated fingerprints left out. That is the band's
measure. The estimates are keyed by hand from each PR's Size section or the
doc that set them, since a loose number regex reads a date as a range.
"""

import contextlib
import datetime
import io
import re
import statistics
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
OUT = Path(__file__).with_suffix(".md")
#: Where the window closes: #313's merge, which the tables were read at.
END = "71e0919e79bc460e3af029478874aacdc997770a"
LAST_DAY = datetime.date(2026, 11, 6)
FORTNIGHTS = 4

HEADER = """# Sizing calibration: how far estimates ran from what landed

**Generated** by `sizing-calibration.py`, read at #313's merge (2026-11-06); a
dated record, not a live count. Measured from git: code plus tests, lines added
across the merge, review rounds included. A PR body's own figure can differ,
and #307's and #308's are added plus removed: #307's "2.2×" is 1.70× by lines
added.

**How `roadmap-v2.md`'s Sizing section uses each kind:**

- **Set at a design stop:** take the top as the expected size, and 0.7–1.6 of
  it as the range.
- **A plan's sum carried at code ×1.5–2 plus tests:** about 1.15× its top.
- **A plain sum of `**Sized:**` lines:** about 2× its top, 1.4–2.7, and only
  for the items it lists, so add the feature itself.
- **A route's PR count before any design:** about 3× its top, 3.5–4× for a whole
  subsystem and 1.5–2.5× for a smaller row.
- **A PR count set where the row was designed:** it holds. TR-5 is the one that
  doubled, and TR-3 the one that grew by half.
"""

# (PR, phase, estimate's low, its top, where it was set)
DESIGN_STOP = [
    (244, "RH-2", 845, 845, "`replacement-architecture.md` §2b.10"),
    (251, "TR-4a", 2120, 2440, "A6p's re-count"),
    (252, "TR-4b", 1150, 1850, "A6p's re-count"),
    (253, "TR-4c", 1560, 1750, "A6p's re-count"),
    (254, "TR-4d", 1175, 2040, "A6p's re-count"),
    (255, "TR-5a", 1380, 1600, "its design"),
    (256, "TR-5e", 1465, 1680, "its design"),
    (257, "TR-5f", 1770, 1995, "its re-count"),
    (258, "TR-5b", 1630, 1850, "its design"),
    (259, "MA-2", 1320, 1360, "its design"),
    (261, "TR-5c", 2045, 2370, "its design"),
    (262, "TR-5h", 1630, 1920, "its design"),
    (263, "TR-5d", 880, 1000, "TR-5h's three designs"),
    (264, "TR-5g", 1330, 1510, "TR-5h's three designs"),
    (267, "CL-1", 1350, 1750, "#266's design"),
    (268, "TR-6", 1305, 1720, "its design"),
    (269, "TR-7", 975, 1300, "its design"),
    (308, "`divided-among-targets`", 1200, 1650, "`choices-architecture.md`, re-sized at #307's ratio"),
]
# A plan's sum of `**Sized:**` lines with the code at 1.5-2x plus the tests:
# (PRs, row, low, top, where it was set).
CARRIED = [
    ((284,), "`history-and-frame-records`", 1700, 2000, "#283's plan"),
    ((288,), "`resolution-context-rider`", 130, 160, "#283's plan"),
    ((289, 290), "`trigger-matcher-merges`", 1350, 1800, "#283's plan"),
    ((291,), "`one-fact-one-type`", 1700, 2100, "#283's plan"),
    ((293, 294), "`batches-durations-and-control`", 1750, 2100, "#283's plan"),
]
# A plain sum of `**Sized:**` lines or a brief's figure, with no ratio on it.
# Only part of a PR's measure can be compared, so `measured` is the figure the
# PR body itself gave for that part, and None takes the PR's whole measure.
PLAIN_SUM = [
    (307, "#306's decision 4", 440, 760, None, "#306; `options-named-by-what-they-are` is that decision"),
    (310, "the hunt's six fixes", 360, 360, None, "item sizes of about 155 and 205"),
    (305, "`scripted-answers-by-identity`", 700, 900, None, "its brief"),
    (313, "#313's ladder", 100, 150, 400, "item 377; the PR body's part"),
    (313, "#313's item 378", 30, 50, 120, "item 378; the PR body's part"),
]
# A route's PR count, written before any design: (row, low, top, landed, what landed).
ROUTE_COUNT = [
    ("Item 5's remainder", 3, 5, 20, "RC 6, RD 4, RE 10"),
    ("Item 6, triggers", 4, 6, 21, "TR-1 to TR-7"),
    ("`card-layer-cleanup`", 1, 2, 5, "#301–#305"),
    ("Item 7, CR 613.8", 1, 2, 3, "LI-1 to LI-3"),
]
# A PR count set where the row was designed: (row, count, landed, what landed).
DESIGN_COUNT = [
    ("RD", 4, 4, "RD-1 to RD-4"),
    ("RE", 9, 10, "RE-1 to RE-10"),
    ("CM", 5, 5, "CM-0 to CM-4"),
    ("RH", 3, 3, "RH-1 to RH-3"),
    ("TR-3", 2, 3, "TR-3a to TR-3c"),
    ("TR-4", 4, 4, "TR-4a to TR-4d"),
    ("TR-5", 4, 8, "TR-5a to TR-5h"),
    ("`readiness`", 2, 2, "#299, #300"),
    ("`modes-plus-choice-kinds`", 5, 6, "named so far; two landed"),
]

ROUTE_PREFIXES = {"triggers", "replacement", "mana", "copy", "layers", "choices", "cards", "targeting", "state"}
GUI_PREFIXES = {"devgui", "ui", "setup"}
PROCESS_PREFIXES = {"process", "docs", "tooling", "dependabot"}


def git(*args):
    done = subprocess.run(["git", "-C", str(ROOT), *args], capture_output=True, text=True,
                          encoding="utf-8", errors="replace", check=True)
    return done.stdout


def merges():
    """`{pr: (merge hash, merge day, branch)}`, every PR merged on the first parent."""
    out = {}
    for line in git("log", "--merges", "--first-parent", END, "--format=%H %cs %s").splitlines():
        m = re.match(r"(\w+) (\S+) Merge pull request #(\d+) from [^/]+/(\S+)", line)
        if m:
            out[int(m.group(3))] = (m.group(1), m.group(2), m.group(4))
    return out


def added(merge):
    total = 0
    for row in git("diff", "--numstat", f"{merge}^1", merge, "--", "mtgsim/src", "mtgsim/tests").splitlines():
        lines, _, path = row.split("\t")
        if lines != "-" and not path.endswith("card_fingerprints.txt"):
            total += int(lines)
    return total


def spread(ratios):
    return (f"{len(ratios)} rows: mean {statistics.mean(ratios):.2f}, median {statistics.median(ratios):.2f}, "
            f"range {min(ratios):.2f}–{max(ratios):.2f}")


def rng(low, top):
    return f"{low:,}" if low == top else f"{low:,}–{top:,}"


def prs(numbers):
    return ", ".join(f"#{n}" for n in numbers)


def line_tables(all_merges):
    print("### Set at a design stop\n")
    print("| PR | Phase | Estimate | Measured | × its top | Set by |\n|---|---|---:|---:|---:|---|")
    ratios = []
    for pr, name, low, top, source in DESIGN_STOP:
        measured = added(all_merges[pr][0])
        ratios.append(measured / top)
        print(f"| #{pr} | {name} | {rng(low, top)} | {measured:,} | {measured / top:.2f} | {source} |")
    print(f"\n{spread(ratios)}.\n")

    print("### A plan's sum, with the ratio already on it\n")
    print("| PRs | Row | Carried | Measured | × its top | Set by |\n|---|---|---:|---:|---:|---|")
    ratios = []
    for numbers, name, low, top, source in CARRIED:
        measured = sum(added(all_merges[n][0]) for n in numbers)
        ratios.append(measured / top)
        print(f"| {prs(numbers)} | {name} | {rng(low, top)} | {measured:,} | {measured / top:.2f} | {source} |")
    print(f"\n{spread(ratios)}.\n")

    print("### A plain sum of `**Sized:**` lines\n")
    print("| PR | Part | Estimate | Measured | × its top | Set by |\n|---|---|---:|---:|---:|---|")
    ratios = []
    for pr, name, low, top, part, source in PLAIN_SUM:
        measured = part if part is not None else added(all_merges[pr][0])
        ratios.append(measured / top)
        print(f"| #{pr} | {name} | {rng(low, top)} | {measured:,} | {measured / top:.2f} | {source} |")
    print(f"\n{spread(ratios)}.\n")


def count_tables():
    print("### A route's PR count, before any design\n")
    print("| Row | Guessed | Landed | × its top | What landed |\n|---|---:|---:|---:|---|")
    ratios = []
    for name, low, top, landed, what in ROUTE_COUNT:
        ratios.append(landed / top)
        print(f"| {name} | {low}–{top} | {landed} | {landed / top:.1f} | {what} |")
    print(f"\n{spread(ratios)}.\n")

    print("### A PR count set where the row was designed\n")
    print("| Row | Set | Landed | × | What landed |\n|---|---:|---:|---:|---|")
    ratios = []
    for name, count, landed, what in DESIGN_COUNT:
        ratios.append(landed / count)
        print(f"| {name} | {count} | {landed} | {landed / count:.2f} | {what} |")
    print(f"\n{spread(ratios)}.\n")


def kind(branch):
    prefix = branch.split("/")[0]
    if branch.startswith("tooling/a6g") or prefix in GUI_PREFIXES:
        return "dev GUI"
    if prefix in ROUTE_PREFIXES:
        return "route"
    if prefix == "audit":
        return "audit"
    if prefix in PROCESS_PREFIXES:
        return "process and docs"
    return "detour"


def overhead(all_merges):
    first = LAST_DAY - datetime.timedelta(days=14 * FORTNIGHTS - 1)
    window = [(day, kind(branch)) for _, day, branch in all_merges.values() if day >= first.isoformat()]
    kinds = ("route", "dev GUI", "audit", "process and docs", "detour")
    print("### What was merged, by branch prefix\n")
    print("| Fortnight from | " + " | ".join(kinds) + " | All |\n|---|" + "---:|" * (len(kinds) + 1))
    for n in range(FORTNIGHTS):
        begin = (first + datetime.timedelta(days=14 * n)).isoformat()
        end = (first + datetime.timedelta(days=14 * (n + 1))).isoformat()
        here = [k for day, k in window if begin <= day < end]
        print(f"| {begin} | " + " | ".join(str(here.count(k)) for k in kinds) + f" | {len(here)} |")
    every = [k for _, k in window]
    print("| **All** | " + " | ".join(f"**{every.count(k)}**" for k in kinds) + f" | **{len(every)}** |")
    route = every.count("route")
    print(f"\n{route} of {len(every)} merge commits on the route: {len(every) / route:.1f} merged PRs for each "
          f"route PR, and {route / (2 * FORTNIGHTS):.1f} route PRs a week. A PR rebased in has no merge commit "
          f"and is not counted. `setup/` is the dev GUI's own track; `state/` is the clone's.")


def main():
    all_merges = merges()
    body = io.StringIO()
    with contextlib.redirect_stdout(body):
        print(HEADER)
        print("## Lines\n")
        line_tables(all_merges)
        print("## PR counts\n")
        count_tables()
        print("## Overhead, the eight weeks to #313\n")
        overhead(all_merges)
        sizes = [added(all_merges[pr][0]) for pr, *_ in DESIGN_STOP]
        print(f"\nA phase PR's size over the design-stop rows: median {statistics.median(sizes):,.0f}, "
              f"mean {statistics.mean(sizes):,.0f}.")
    OUT.write_text(body.getvalue(), encoding="utf-8", newline="\n")
    print(f"wrote {OUT.name}")


if __name__ == "__main__":
    main()
