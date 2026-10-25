#!/usr/bin/env python3
"""The comment census: `engineering-practices.md#the-comment-sweep`'s instruments, in code.

    python plans/references/comment-census.py                  # the readings, on the working tree
    python plans/references/comment-census.py --rev 0ac5c98    # the same readings at a commit, read from git
    python plans/references/comment-census.py --list tight     # the tight tier's hits, undated first
    python plans/references/comment-census.py --list blocks    # the blocks over four lines, history ones marked
    python plans/references/comment-census.py --history F...   # the whole-file history grep, before a PR opens

WHY THIS EXISTS
---------------
§2.1's records wrote their tiers in prose, and two of the three runs after
the first had to re-derive them before they could compare: the tight tier
"was never written as a regex", and the history count the item 6 audit's
brief carried stated no definition at all. Every definition a reading rests
on is a constant below, so a reading at an old commit and one today come
from the same instrument.

THE DEFINITIONS
---------------
  * A COMMENT LINE is a tracked `.rs` line whose first non-space is `//`.
    Its kind is `//!` (module), `///` but not `////` (doc), or inline.
  * A BLOCK is a maximal run of consecutive comment lines of one kind. "Over
    four lines" is five or more.
  * A hit is DATED when its block carries a date, a PR number or a phase
    code (`DATED`). The records say "paragraph"; the block is what lands
    nearest the third record's readings
    (`archive/engineering-practices-records.md#21-the-comment-audit--why-the-guard-is-a-sweep-and-not-a-budget`).
  * The WIDE TIER is the sweep's grep as its first record wrote it, over comment
    lines in `.rs` files and over every line of live `plans/` markdown
    (`plans/archive/` and the corpus out). It is the count to record.
  * The TIGHT TIER is the list to read: about, roughly or one in before a
    quantity; counted or measured as words of their own; any N% or N of M.
  * A HISTORY WORD is what §2.1's second record cut: a phase code, "used
    to", "until <date>", "this PR", "the old", "no longer", a PR number. A
    block over four lines holding one NARRATES HISTORY.

The areas: non-card `src/` with `bin/` for the tiers, as every record read
them, and without `bin/` for the line counts and the block census, as the
third record's table did; `cards/` and `tests/` beside them.

Read at the records' own trees (`--rev`), these give the wide tier, the
comment lines and the blocks over four lines exactly, and the tight tier and
the history blocks within a few; §2.1's fourth record has the table.
"""

import argparse
import re
import subprocess
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent

WIDE = re.compile(r"(roughly|about|one in|[0-9]+%|[0-9]+ of [0-9]+|measured|counted)")

QUANTITY = (
    r"(?:~?\d[\d,.]*|a (?:dozen|hundred|thousand|third|quarter|half|tenth|fifth)|half|"
    r"one|two|three|four|five|six|seven|eight|nine|ten|eleven|twelve|fifteen|twenty|thirty|"
    r"forty|fifty|sixty|seventy|eighty|ninety|a few)\b"
)
TIGHT = re.compile(
    rf"\b(?:about|roughly|one in)\s+{QUANTITY}"
    r"|\b(?:counted|measured)\b"
    r"|\d+(?:\.\d+)?%"
    r"|\b\d+ of \d+\b",
    re.IGNORECASE,
)

PHASE_CODE = (
    r"\b(?:[A-Z]{2}-\d+[a-z]?|A\d+[a-z]+|Phase [0-9A-Z]+[a-z]?|R[A-H]|L[A-Z]-?\d*|"
    r"[LTD]\d+[a-z]?|CL)\b"
    r"|`[a-z0-9]+(?:-[a-z0-9]+)+`"  # a phase still to come, by name (engineering-practices.md#naming-a-phase)
)
DATE = r"\b20\d\d-\d\d(?:-\d\d)?\b"
PR_NUMBER = r"(?:\bPR\s*#?\d+|#\d{2,})\b"
DATED = re.compile(rf"{DATE}|{PR_NUMBER}|{PHASE_CODE}")
HISTORY = re.compile(
    rf"{PHASE_CODE}|\bused to\b|\buntil {DATE}|\bthis PR\b|\bthe old\b|\bno longer\b|{PR_NUMBER}"
)


def area_of(path: str) -> str | None:
    if path.startswith("mtgsim/src/cards/"):
        return "cards"
    if path.startswith("mtgsim/src/bin/"):
        return "bin"
    if path.startswith("mtgsim/src/"):
        return "src"
    if path.startswith("mtgsim/tests/"):
        return "tests"
    return None


def is_live_plan(path: str) -> bool:
    return (
        path.startswith("plans/")
        and path.endswith(".md")
        and not path.startswith(("plans/archive/", "plans/atomic-tests/"))
    )


def tracked_files(rev: str | None) -> dict[str, str]:
    """Every `.rs` file under `mtgsim/src` and `mtgsim/tests` and every live plan, by path."""
    roots = ["mtgsim/src", "mtgsim/tests", "plans"]
    if rev is None:
        names = subprocess.run(
            ["git", "ls-files", "--", *roots], cwd=ROOT, capture_output=True, text=True, check=True
        ).stdout.split("\n")
    else:
        names = subprocess.run(
            ["git", "ls-tree", "-r", "--name-only", rev, "--", *roots],
            cwd=ROOT, capture_output=True, text=True, check=True,
        ).stdout.split("\n")
    names = [n for n in names if (n.endswith(".rs") and area_of(n)) or is_live_plan(n)]
    if rev is None:
        return {n: (ROOT / n).read_text(encoding="utf-8") for n in names if (ROOT / n).exists()}
    batch = subprocess.run(
        ["git", "cat-file", "--batch"], cwd=ROOT, check=True, capture_output=True,
        input="".join(f"{rev}:{n}\n" for n in names).encode(),
    ).stdout
    texts, at = {}, 0
    for name in names:
        header_end = batch.index(b"\n", at)
        size = int(batch[at:header_end].split()[2])
        texts[name] = batch[header_end + 1 : header_end + 1 + size].decode("utf-8")
        at = header_end + 1 + size + 1
    return texts


def comment_kind(line: str) -> str | None:
    s = line.lstrip()
    if not s.startswith("//"):
        return None
    if s.startswith("//!"):
        return "module"
    if s.startswith("///") and not s.startswith("////"):
        return "doc"
    return "inline"


def comment_body(line: str) -> str:
    return line.lstrip().lstrip("/!").strip()


def blocks(lines: list[str]):
    """(first line number, kind, the block's lines) for every block, one-based."""
    start, kind = None, None
    for i, line in enumerate(lines + [""]):
        k = comment_kind(line)
        if k != kind:
            if kind is not None:
                yield start + 1, kind, lines[start:i]
            start, kind = (i, k) if k else (None, None)


def tight_hits(text: str):
    """(line number, dated, the line) for each tight-tier hit in a comment."""
    for first, _, block in blocks(text.splitlines()):
        dated = any(DATED.search(l) for l in block)
        for j, line in enumerate(block):
            if TIGHT.search(comment_body(line)):
                yield first + j, dated, line.strip()


def readings(texts: dict[str, str]) -> dict:
    r = Counter()
    for path, text in texts.items():
        if is_live_plan(path):
            r["wide plans"] += sum(1 for l in text.splitlines() if WIDE.search(l))
            continue
        area = area_of(path)
        tier_area = "src" if area == "bin" else area
        lines = text.splitlines()
        comments = [l for l in lines if comment_kind(l)]
        r[f"wide {tier_area}"] += sum(1 for l in comments if WIDE.search(l))
        if tier_area == "src":
            for _, dated, _ in tight_hits(text):
                r["tight"] += 1
                r["tight undated"] += not dated
        if area == "bin":
            continue
        r[f"lines {area}"] += len(lines)
        r[f"comments {area}"] += len(comments)
        if area != "src":
            continue
        for _, kind, block in blocks(lines):
            if len(block) > 4:
                r[f"long {kind}"] += len(block)
                if any(HISTORY.search(comment_body(l)) for l in block):
                    r["history blocks"] += 1
                    r["history lines"] += len(block)
    return r


def print_readings(r: dict, label: str) -> None:
    src_share = r["comments src"] / r["lines src"] if r["lines src"] else 0
    print(f"comment census at {label}")
    print(f"  wide tier, non-card src (bin in) / cards / tests / live plans: "
          f"{r['wide src']:,} / {r['wide cards']:,} / {r['wide tests']:,} / {r['wide plans']:,}")
    print(f"  tight tier, non-card src (bin in): {r['tight']:,} ({r['tight undated']:,} undated)")
    print(f"  comment lines, non-card src (bin out): {r['comments src']:,} of {r['lines src']:,} "
          f"lines ({src_share:.1%}); cards {r['comments cards']:,}; tests {r['comments tests']:,}")
    print(f"  in blocks over four lines, doc / inline / module: "
          f"{r['long doc']:,} / {r['long inline']:,} / {r['long module']:,}")
    print(f"  blocks over four lines narrating history: {r['history blocks']:,} "
          f"({r['history lines']:,} lines)")


def list_tight(texts: dict[str, str]) -> None:
    hits = [
        (dated, path, n, line)
        for path, text in sorted(texts.items())
        if area_of(path) in ("src", "bin")
        for n, dated, line in tight_hits(text)
    ]
    for dated, path, n, line in sorted(hits, key=lambda h: (h[0], h[1], h[2])):
        print(f"{'dated  ' if dated else 'UNDATED'} {path}:{n}  {line[:110]}")


def list_blocks(texts: dict[str, str]) -> None:
    for path, text in sorted(texts.items()):
        if area_of(path) != "src":
            continue
        for first, kind, block in blocks(text.splitlines()):
            if len(block) > 4:
                words = sorted({m.group(0) for l in block for m in HISTORY.finditer(comment_body(l))})
                mark = f"  HISTORY: {', '.join(words)}" if words else ""
                print(f"{path}:{first}  {kind} {len(block)}{mark}")


def history_grep(paths: list[str]) -> int:
    """Every comment line in the named files holding a history word; the count found."""
    found = 0
    for p in paths:
        text = (ROOT / p).read_text(encoding="utf-8") if (ROOT / p).exists() else ""
        for n, line in enumerate(text.splitlines(), 1):
            if comment_kind(line) and HISTORY.search(comment_body(line)):
                found += 1
                print(f"{p}:{n}  {line.strip()[:110]}")
    return found


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--rev", help="read the tree at this commit instead of the working tree")
    ap.add_argument("--list", choices=["tight", "blocks"])
    ap.add_argument("--history", nargs="+", metavar="FILE", help="repo-relative paths")
    args = ap.parse_args()
    if args.history:
        found = history_grep(args.history)
        print(f"\n{found} comment line(s) with a history word in {len(args.history)} file(s)")
        return 0
    texts = tracked_files(args.rev)
    if args.list == "tight":
        list_tight(texts)
    elif args.list == "blocks":
        list_blocks(texts)
    else:
        print_readings(readings(texts), args.rev or "the working tree")
    return 0


if __name__ == "__main__":
    sys.exit(main())
