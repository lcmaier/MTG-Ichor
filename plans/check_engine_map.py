#!/usr/bin/env python3
"""
check_engine_map - the engine map names only what exists, and its arm table is the enum.

`plans/engine-map.md` is the page a reader starts from (`engineering-practices.md`
§7.1, tier 3). A map of a moving tree rots without a sound: a function renamed,
a file moved, a `GameAction` variant added. This is the re-reader.

    python plans/check_engine_map.py            # exit 1 when the map has rotted
    python plans/check_engine_map.py --drift    # also list what moved since the pin

**What fails.**

1. The map's arm table (between `<!-- arms:begin -->` and `<!-- arms:end -->`),
   `GameAction`'s variants and `perform_action`'s arms are not one set, or the
   match has a catch-all arm. The variants and the arms are two parses of one
   file: with no catch-all the compiler makes them equal, so if they differ the
   parse broke, and that exits 2.
2. A file, directory, glob or link target the map names does not exist.
3. A name cited beside a location, `name` then `path.rs:N`, does not occur in
   that file's code with comments stripped, or `N` is past the file's end.
4. Any other code name in backticks (one with `_`, `::` or a capital) occurs
   nowhere in the crate's code.
5. A `§N` is not a heading of the doc named before it, or of the map itself.

**What it reports and does not fail on**, listed under `--drift`: a cited name
more than two lines from its line, which is what a refresh moves, and the
`.rs` files under `mtgsim/src` the map does not name. The map is pinned to a
commit, so a line that moved is expected; a name that vanished is not.
"""

import argparse
import re
import sys
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

ROOT = Path(__file__).resolve().parent.parent
MAP = ROOT / "plans" / "engine-map.md"
SRC = ROOT / "mtgsim" / "src"
ACTIONS = SRC / "engine" / "actions.rs"
#: Where a path in the map resolves, in order: `engine/actions.rs` is under src,
#: `tests/...` under the crate, `CLAUDE.md` at the root, a doc beside the map.
BASES = [SRC, SRC.parent, ROOT, MAP.parent]

SPAN = re.compile(r"`([^`\n]+)`")
NAMED = re.compile(r"`([A-Za-z_][\w:]*)` `([\w./-]+\.rs):(\d+)`")
LOCATION = re.compile(r"^([\w./-]+\.rs):(\d+)$")
FILE = re.compile(r"^[\w./*-]+\.(?:rs|md|py|html|toml|yml|jsonl)$")
DIRECTORY = re.compile(r"^[\w./-]+/$")
CODE_NAME = re.compile(r"^[A-Za-z_]\w*(?:::[A-Za-z_]\w*)*$")
DOC_SECTIONS = re.compile(r"`([\w./-]+\.md)`((?:,? §[\w.]*\w)+)")
SECTION = re.compile(r"§([\w.]*\w)")
LINK = re.compile(r"\]\(([^)\s#]+)(?:#[^)]*)?\)")
HEADING = re.compile(r"^#+\s+(\d+[a-z]?(?:\.\d+[a-z]?)*)\.?(?:\s|$)")
IDENT = re.compile(r"[A-Za-z_]\w*")


def read_lines(path: Path) -> list[str]:
    return path.read_text(encoding="utf-8").splitlines()


def idents(path: Path) -> set[str]:
    """Every identifier in the file's code. `//` comments are dropped first, so a
    name that survives only in a comment ("this used to be X") counts as gone."""
    return {i for line in read_lines(path) for i in IDENT.findall(line.split("//", 1)[0])}


def block(lines, opener, end):
    """The lines after the one line matching `opener`, up to the first matching `end`."""
    starts = [i for i, line in enumerate(lines) if opener.search(line)]
    if len(starts) != 1:
        return None
    for j in range(starts[0] + 1, len(lines)):
        if end.match(lines[j]):
            return lines[starts[0] + 1 : j]
    return None


def enum_and_arms():
    """`GameAction`'s variants, `perform_action`'s arms and any catch-all arm."""
    lines = read_lines(ACTIONS)
    enum = block(lines, re.compile(r"^pub enum GameAction \{$"), re.compile(r"^\}"))
    body = block(lines, re.compile(r"^    fn perform_action\("), re.compile(r"^    \}$"))
    if enum is None or body is None:
        return None
    variants = [m.group(1) for line in enum if (m := re.match(r"^    ([A-Z]\w*)\b", line))]
    # The match's arms sit at twelve spaces; a nested proposal inside an arm sits deeper.
    arms = [m.group(1) for line in body if (m := re.match(r"^ {12}(?:\| )?GameAction::(\w+)", line))]
    catch_all = [line.strip() for line in body if re.match(r"^ {12}[a-z_]\w*\s*(?:if .*)?=>", line)]
    return variants, arms, catch_all


def resolve(token: str, context: list[str]) -> list[Path]:
    """Every existing path `token` names. A bare file name resolves against the
    directories its own line names (`pipeline.rs` beside `engine/replacement/`),
    and with none on the line it may sit anywhere in the crate (`mod.rs`)."""
    prefixes = context if "/" not in token else []
    for base in BASES:
        found = [p for prefix in prefixes + [""] for p in base.glob(prefix + token.rstrip("/"))]
        if found:
            return found
    if "/" not in token and not prefixes:
        return list(SRC.rglob(token))
    return []


def headings(path: Path) -> set[str]:
    return {m.group(1) for line in read_lines(path) if (m := HEADING.match(line))}


class Reader:
    """One pass over the map, collecting what fails, what drifted and what it named."""

    def __init__(self):
        self.problems, self.drifted, self.named = [], [], set()
        self.counts = dict.fromkeys(["names", "paths", "links", "sections"], 0)
        sources = list(SRC.rglob("*.rs")) + list((SRC.parent / "tests").rglob("*.rs"))
        self.crate = set().union(*(idents(p) for p in sources))
        self.own = headings(MAP)
        self.file_idents = {}

    def cited(self, where, name, file, at):
        """`name` cited at `file:at`: it must occur in that file's code."""
        path = next((p for p in resolve(file, []) if p.is_file()), None)
        self.counts["names"] += 1
        if path is None:
            self.problems.append(f"{where}: `{name}` is cited in {file}, which does not exist")
            return
        self.named.add(path)
        if path not in self.file_idents:
            self.file_idents[path] = idents(path)
        source = read_lines(path)
        if any(seg not in self.file_idents[path] for seg in name.split("::")):
            self.problems.append(f"{where}: `{name}` does not occur in {file}'s code")
        elif int(at) > len(source):
            self.problems.append(f"{where}: {file}:{at} is past the file's {len(source)} lines")
        else:
            last = re.compile(rf"\b{re.escape(name.split('::')[-1])}\b")
            if not any(last.search(s) for s in source[max(0, int(at) - 3) : int(at) + 2]):
                self.drifted.append(f"{where}: `{name}` is no longer at {file}:{at}")

    def token(self, where, token, context):
        """A backticked token that is not half of a cited pair."""
        if m := LOCATION.match(token):
            path = next(iter(resolve(m.group(1), [])), None)
            self.counts["paths"] += 1
            if path is None:
                self.problems.append(f"{where}: {token} names a file that does not exist")
            elif int(m.group(2)) > len(read_lines(path)):
                self.problems.append(f"{where}: {token} is past the file's end")
            else:
                self.named.add(path)
        elif FILE.match(token) or DIRECTORY.match(token):
            self.counts["paths"] += 1
            found = resolve(token, context)
            self.named |= {p for p in found if p.suffix == ".rs"}
            if not found:
                self.problems.append(f"{where}: `{token}` does not exist")
        elif CODE_NAME.match(token) and ("_" in token or ":" in token or token[0].isupper()):
            self.counts["names"] += 1
            if any(seg not in self.crate for seg in token.split("::")):
                self.problems.append(f"{where}: `{token}` occurs nowhere in the crate's code")

    def sections(self, where, line):
        """`doc.md` §N against the doc's headings; any other §N against the map's own."""
        rest = line
        for doc, refs in DOC_SECTIONS.findall(line):
            rest = rest.replace(refs, "", 1)
            path = next(iter(resolve(doc, [])), None)
            if path is None:
                self.problems.append(f"{where}: `{doc}` does not exist")
                continue
            have = headings(path)
            for section in SECTION.findall(refs):
                self.counts["sections"] += 1
                if section not in have:
                    self.problems.append(f"{where}: {doc} has no §{section}")
        for section in SECTION.findall(rest):
            self.counts["sections"] += 1
            if section not in self.own:
                self.problems.append(f"{where}: the map has no §{section}")

    def line(self, n, line):
        where = f"engine-map.md:{n}"
        tokens = SPAN.findall(line)
        context = [t for t in tokens if DIRECTORY.match(t)]
        context += [t.rsplit("/", 1)[0] + "/" for t in tokens if FILE.match(t) and "/" in t]
        paired = set()
        for name, file, at in NAMED.findall(line):
            paired |= {name, f"{file}:{at}"}
            self.cited(where, name, file, at)
        for token in tokens:
            if token not in paired:
                self.token(where, token, context)
        for target in LINK.findall(line):
            if not target.startswith("http"):
                self.counts["links"] += 1
                if not (MAP.parent / target).exists():
                    self.problems.append(f"{where}: the link {target} goes nowhere")
        self.sections(where, line)


def main() -> int:
    ap = argparse.ArgumentParser(description="Check plans/engine-map.md against the tree.")
    ap.add_argument("--drift", action="store_true", help="list moved lines and unnamed files")
    args = ap.parse_args()

    parsed = enum_and_arms()
    if parsed is None:
        print(f"engine map: no one `pub enum GameAction {{` and `fn perform_action(` in "
              f"{ACTIONS.relative_to(ROOT)}; the layout this gate reads has changed.")
        return 2
    variants, arms, catch_all = parsed
    # Without a catch-all the compiler makes the two lists one set, so a
    # difference means a pattern here broke. With one, the difference is real.
    if not catch_all and set(variants) != set(arms):
        print("engine map: the two parses of actions.rs disagree, so a pattern broke: "
              f"enum only {sorted(set(variants) - set(arms))}, arms only {sorted(set(arms) - set(variants))}")
        return 2

    lines = read_lines(MAP)
    table = block(lines, re.compile(r"<!-- arms:begin -->"), re.compile(r"<!-- arms:end -->")) or []
    mapped = [m.group(1) for line in table if (m := re.match(r"^\| `(\w+)` \|", line))]

    reader = Reader()
    if catch_all:
        unarmed = sorted(set(variants) - set(arms))
        reader.problems.append(f"perform_action has a catch-all arm {catch_all}, "
                               f"so no arm answers for {unarmed}")
    for name in sorted(set(variants) - set(mapped)):
        reader.problems.append(f"GameAction::{name} has no row in the map's arm table")
    for name in sorted(set(mapped) - set(variants)):
        reader.problems.append(f"the map's arm table lists {name}, which GameAction no longer has")

    fenced = False
    for n, line in enumerate(lines, 1):
        if line.startswith("```"):
            fenced = not fenced
        elif not fenced:
            reader.line(n, line)

    unnamed = sorted(p.relative_to(SRC).as_posix() for p in SRC.rglob("*.rs") if p not in reader.named)
    c = reader.counts
    print(f"engine map: {len(variants)} GameAction variants, {len(arms)} perform_action arms, "
          f"{len(mapped)} rows in the map's arm table")
    print(f"engine map: {c['names']} names, {c['paths']} paths, {c['links']} links and "
          f"{c['sections']} section references checked")
    if args.drift:
        for d in reader.drifted:
            print(f"  drift: {d}")
        for u in unnamed:
            print(f"  unnamed: {u}")
    elif reader.drifted or unnamed:
        print(f"engine map: {len(reader.drifted)} cited lines moved since the pin and "
              f"{len(unnamed)} files go unnamed; --drift lists them")
    if not reader.problems:
        print("\nevery name in plans/engine-map.md still exists, and its arm table is the enum.")
        return 0
    print(f"\n{len(reader.problems)} problem(s):\n")
    for p in reader.problems:
        print(f"  {p}")
    print("\nRefresh the map (plans/engine-map.md §3): fix each name, then re-pin its header.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
