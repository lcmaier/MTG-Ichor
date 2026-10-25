#!/usr/bin/env python3
"""
replace_exact - apply a list of exact-text replacements, all or none.

For an edit the Edit tool is the wrong size for: a counted sweep across many
files, or one change made at dozens of sites. A sweep is checked whole before
any file is written, so a miss leaves the tree as it was, and the edits live
in a plain-text file, so nothing in them is escaped:

    === mtgsim/src/engine/sba.rs
    <<<<<<< old
    fn still_arrived(
    =======
    fn arrivals_still_here(
    >>>>>>> new
    <<<<<<< old x3
    still_arrived(game)
    =======
    arrivals_still_here(game)
    >>>>>>> new

`=== <path>` names the file the blocks under it edit, relative to the
repository root, once per edit file. Each block's old text must occur exactly once, or exactly N
times with `old xN`, in the file as the earlier blocks left it. Every block of
every file is checked before anything is written; then each file is written
whole, to a temporary file renamed over the original. A file stored with CRLF
has the blocks matched and written with CRLF.

    python plans/replace_exact.py EDITS.txt            # apply, exit 1 on any miss
    python plans/replace_exact.py EDITS.txt --dry-run  # check only
"""

import argparse
import os
import re
import sys
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

ROOT = Path(__file__).resolve().parent.parent
FILE_LINE = re.compile(r"^=== (.+?)\s*$")
OLD_LINE = re.compile(r"^<<<<<<< old(?: x(\d+))?\s*$")
MID_LINE = "======="
NEW_LINE = ">>>>>>> new"


def parse(text):
    """The edit file as [(path, [(old, new, count)])], in file order."""
    files, block, part = [], None, None
    for number, line in enumerate(text.split("\n"), 1):
        if block is None:
            if m := FILE_LINE.match(line):
                if any(path == m.group(1) for path, _ in files):
                    sys.exit(f"line {number}: {m.group(1)} is named twice; put all its blocks under one `===` line")
                files.append((m.group(1), []))
            elif m := OLD_LINE.match(line):
                if not files:
                    sys.exit(f"line {number}: a block before any `=== <path>` line")
                block, part = {"old": [], "new": [], "count": int(m.group(1) or 1)}, "old"
            elif line.strip():
                sys.exit(f"line {number}: expected `=== <path>` or `<<<<<<< old`, got {line!r}")
        elif part == "old" and line == MID_LINE:
            part = "new"
        elif part == "new" and line == NEW_LINE:
            files[-1][1].append(("\n".join(block["old"]), "\n".join(block["new"]), block["count"]))
            block = None
        else:
            block[part].append(line)
    if block is not None:
        sys.exit("the last block has no `>>>>>>> new` line")
    return files


def main():
    ap = argparse.ArgumentParser(description="Apply exact-text replacements, all or none.")
    ap.add_argument("edits")
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()

    planned, failures = [], []
    for path, blocks in parse(Path(args.edits).read_text(encoding="utf-8")):
        target = ROOT / path
        if not target.is_file():
            failures.append(f"{path}: no such file")
            continue
        with open(target, encoding="utf-8", newline="") as f:
            text = f.read()
        eol = "\r\n" if "\r\n" in text else "\n"
        for index, (old, new, count) in enumerate(blocks, 1):
            old, new = old.replace("\n", eol), new.replace("\n", eol)
            found = text.count(old)
            if found != count:
                failures.append(f"{path}, block {index}: old text found {found} times, expected {count}")
                continue
            text = text.replace(old, new)
        planned.append((target, path, text, len(blocks)))

    if failures:
        print("Nothing written:")
        for failure in failures:
            print(f"  {failure}")
        return 1
    for target, path, text, count in planned:
        if not args.dry_run:
            temporary = target.with_name(target.name + ".replace_exact")
            with open(temporary, "w", encoding="utf-8", newline="") as f:
                f.write(text)
            os.replace(temporary, target)
        print(f"{'checked' if args.dry_run else 'wrote'} {path}: {count} block(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
