#!/usr/bin/env python3
"""
check_claude_md - CLAUDE.md's line budget, as a gate rather than a wish.

CLAUDE.md loads into every session, so a stale claim there is worse than no
claim, and every line it carries is a line that competes with the task. "Keep
it durable" was the rule for two years and the file crossed 300 lines twice,
because a rule that can fail silently is not a mechanism.

This is the mechanism. It fails the way a warning fails.

    python plans/check_claude_md.py          # exit 1 if over budget
    python plans/check_claude_md.py --budget 220   # a raise you have to type

The per-section table is the actionable half: when the total is over, it names
which section to cut. The rule for cutting is in
`plans/engineering-practices.md` - state the invariant in at most three lines
and point at the doc that carries the reasoning.
"""

import argparse
import sys
from pathlib import Path

# CLAUDE.md is full of em-dashes; the Windows console defaults to cp1252 and
# would mangle the section names this prints. Same guard as `specdb.py`.
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

BUDGET = 200
ROOT = Path(__file__).resolve().parent.parent


def sections(lines):
    """Split on `## ` headings. Everything before the first one is the preamble."""
    out = [("(preamble)", 0)]
    for line in lines:
        if line.startswith("## "):
            out.append((line[3:].strip(), 1))
        else:
            out[-1] = (out[-1][0], out[-1][1] + 1)
    return out


def main():
    ap = argparse.ArgumentParser(description="Check CLAUDE.md against its line budget.")
    ap.add_argument("--budget", type=int, default=BUDGET)
    ap.add_argument("--file", default=str(ROOT / "CLAUDE.md"))
    args = ap.parse_args()

    path = Path(args.file)
    if not path.exists():
        print(f"not found: {path}")
        return 1

    lines = path.read_text(encoding="utf-8").splitlines()
    total = len(lines)

    width = max(len(name) for name, _ in sections(lines))
    for name, count in sorted(sections(lines), key=lambda s: -s[1]):
        print(f"  {name:<{width}}  {count:>4}")
    print()

    if total > args.budget:
        over = total - args.budget
        print(f"{path.name}: {total}/{args.budget} lines - OVER BUDGET by {over}.")
        print("Cut a section, or move its reasoning into plans/ and leave a pointer.")
        print("Adding a section requires removing one; see plans/engineering-practices.md.")
        return 1

    print(f"{path.name}: {total}/{args.budget} lines - {args.budget - total} to spare.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
