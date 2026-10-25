#!/usr/bin/env python3
"""
check_practices_page - the rules page stays short, and every citation of a doc's
section says where it leads.

`plans/engineering-practices.md` grew to 2,571 lines because nothing counted
it, and its sections were cited by number, which says nothing about where a
citation leads (the owner, at #286's review). Three checks:

1. **The page's budget.** The page is over `BUDGET` lines and fails, the way
   `CLAUDE.md`'s budget does; a raise is typed (`--budget N`).
2. **No citation of the page by section number.** `engineering-practices.md`
   followed by a section sign, "section" or a dotted number fails in any
   tracked file but the records in `NOT_READ`, which keep the old numbers on
   purpose. An old number is read in
   `plans/archive/engineering-practices-records.md`'s table.
3. **Every anchor resolves.** A `<doc>.md#<anchor>` citation in a tracked
   file, and a `(#<anchor>)` link inside a doc, must name a heading of that
   doc, by GitHub's anchor for it: the heading's text lowercased, punctuation
   dropped, spaces as hyphens, and a repeat suffixed `-1`, `-2`. A doc is
   found by its path's tail, so `fuzz-record.md#…` and
   `plans/fuzz-record.md#…` name one file.

    python plans/check_practices_page.py               # exit 1 on any failure
    python plans/check_practices_page.py --budget 650  # a raise you have to type
"""

import argparse
import re
import subprocess
import sys
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

ROOT = Path(__file__).resolve().parent.parent
PAGE = "plans/engineering-practices.md"
BUDGET = 625
TEXT_SUFFIXES = (".md", ".rs", ".py", ".yml", ".yaml", ".toml", ".sh", ".html", ".txt", ".json")
# Records, not read for citations: the archive and the docs `CLAUDE.md` calls
# historical, accurate as they stood; the pinned trace pages, never edited; and
# the corpus's authoring trail, which nothing reads.
NOT_READ = ("plans/archive/", "plans/traces/", "design_doc.md", "plans/roadmap.md",
            "plans/workflow-prompts.md", "plans/atomic-tests/summaries/")

# The page's name, then what may sit between it and a section number: a closing
# backtick, a possessive, and the punctuation and comment markers of a line break.
NUMBERED_CITATION = re.compile(
    r"engineering-practices(?:\.md)?`?(?:'s)?[\s/!#*>,(]*(?:§|section\s+\d|\d+\.\d)")
ANCHOR_CITATION = re.compile(r"(?<![\w./-])((?:[\w-]+/)*[\w.-]+\.md)#([\w-]+)")
LOCAL_LINK = re.compile(r"\]\(#([\w-]+)\)")
HEADING = re.compile(r"^#{1,6} (.+?)\s*#*\s*$")


def tracked():
    out = subprocess.run(["git", "-C", str(ROOT), "ls-files"], capture_output=True, text=True).stdout
    return [p for p in out.split("\n") if p and p.endswith(TEXT_SUFFIXES)]


def read(path):
    try:
        return (ROOT / path).read_text(encoding="utf-8")
    except (UnicodeDecodeError, FileNotFoundError):
        return None


def anchor_of(heading):
    """GitHub's anchor for a heading's text."""
    text = re.sub(r"\[([^\]]*)\]\([^)]*\)", r"\1", heading)
    text = text.replace("`", "").replace("*", "")
    return re.sub(r"[^\w\- ]", "", text.lower()).replace(" ", "-")


def anchors(text):
    seen, out, fenced = {}, set(), False
    for line in text.splitlines():
        if line.lstrip().startswith("```"):
            fenced = not fenced
            continue
        m = None if fenced else HEADING.match(line)
        if m:
            a = anchor_of(m.group(1))
            n = seen.get(a, 0)
            out.add(a if n == 0 else f"{a}-{n}")
            seen[a] = n + 1
    return out


def line_of(text, pos):
    return text.count("\n", 0, pos) + 1


def main():
    ap = argparse.ArgumentParser(description="Check the rules page's budget and every section citation.")
    ap.add_argument("--budget", type=int, default=BUDGET)
    args = ap.parse_args()

    files = tracked()
    docs = [p for p in files if p.endswith(".md")]
    cache = {}

    def doc_anchors(path):
        if path not in cache:
            cache[path] = anchors(read(path) or "")
        return cache[path]

    def resolve(cited):
        hits = [d for d in docs if d == cited or d.endswith("/" + cited)]
        return hits[0] if len(hits) == 1 else None

    failures = []
    page_lines = len((read(PAGE) or "").splitlines())
    if page_lines > args.budget:
        failures.append(f"{PAGE}: {page_lines}/{args.budget} lines, over budget by {page_lines - args.budget}"
                        " - state a rule in a line or two and point at its reasoning")

    resolved = 0
    for path in files:
        text = read(path)
        if text is None:
            continue
        if path.startswith(NOT_READ):
            continue
        for m in NUMBERED_CITATION.finditer(text):
            failures.append(f"{path}:{line_of(text, m.start())}: cites {PAGE} by section number;"
                            " cite its heading's anchor (the archived record's table maps each number)")
        for m in ANCHOR_CITATION.finditer(text):
            cited, anchor = m.group(1), m.group(2)
            target = resolve(cited)
            if target is None:
                failures.append(f"{path}:{line_of(text, m.start())}: `{cited}` names no single tracked doc")
            elif anchor not in doc_anchors(target):
                failures.append(f"{path}:{line_of(text, m.start())}: `{cited}#{anchor}` is no heading of {target}")
            else:
                resolved += 1
        if path.endswith(".md"):
            for m in LOCAL_LINK.finditer(text):
                if m.group(1) not in doc_anchors(path):
                    failures.append(f"{path}:{line_of(text, m.start())}: `#{m.group(1)}` is no heading here")
                else:
                    resolved += 1

    print(f"practices page: {page_lines}/{args.budget} lines; {resolved} anchor citations resolved"
          f" in {len(files)} tracked files")
    if failures:
        for f in failures:
            print("  " + f)
        print(f"{len(failures)} failures.")
        return 1
    print("the page is within its budget, nothing cites it by number, and every anchor names a heading.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
