#!/usr/bin/env python3
"""
panic_surface - the crate's panic surface, split into engine and test by the
separator the post-RE audit's pass 3 built (2026-09-15).

    python plans/panic_surface.py            # the table
    python plans/panic_surface.py --list engine unwrap   # every engine `.unwrap()` site
    python plans/panic_surface.py --by-dir engine unwrap  # per-directory counts

Why a path glob is not enough: `mtgsim/src` holds 59 `#[cfg(test)] mod tests`
modules at the bottom of engine files (about 16,000 lines of the 60,000), so
"panics in src" overstates the engine by the tests' share. The separator is
the column-0 `#[cfg(test)]` marker: everything from it to the end of the
file is unit-test code. An indented `#[cfg(test)]` (one site, `costs.rs`) is a
test-only item inside an impl and is counted as test up to its closing brace.

Categories:
  engine      mtgsim/src, minus src/bin, minus src/test_support.rs, minus the
              cfg(test) tails -- the code a training batch runs
  harness     src/bin (cli_play, fuzz_games), minus its cfg(test) tails
  unit-tests  the cfg(test) tails of src files
  tests       mtgsim/tests
  support     src/test_support.rs (feature-gated, never in a release build)

Comment lines and trailing `//` comments are stripped before matching, so a
mention of `.unwrap()` in prose is not a site. `debug_assert*` is reported but
is not a release panic; `assert*` is.
"""

import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(ROOT, "mtgsim", "src")
TESTS = os.path.join(ROOT, "mtgsim", "tests")

PATTERNS = [
    ("panic!", re.compile(r"\bpanic!\s*[(\[]")),
    ("unreachable!", re.compile(r"\bunreachable!\s*[(\[]")),
    ("todo!/unimplemented!", re.compile(r"\b(todo|unimplemented)!\s*[(\[]")),
    ("unwrap()", re.compile(r"\.unwrap\(\)")),
    ("expect(", re.compile(r"\.expect\(")),
    ("assert!/assert_eq!/assert_ne!", re.compile(r"(?<![a-z_])assert(_eq|_ne)?!\s*[(\[]")),
    ("debug_assert* (not in release)", re.compile(r"\bdebug_assert(_eq|_ne)?!\s*[(\[]")),
]

MARKER = re.compile(r"^([ \t]*)#\[cfg\(test\)\]\s*$")


def strip_comment(line):
    """Drop a `//` comment unless the `//` sits inside a string literal (rough)."""
    s = line.lstrip()
    if s.startswith("//"):
        return ""
    out = []
    in_str = False
    i = 0
    while i < len(line):
        c = line[i]
        if c == '"' and (i == 0 or line[i - 1] != "\\"):
            in_str = not in_str
        if not in_str and line.startswith("//", i):
            break
        out.append(c)
        i += 1
    return "".join(out)


def split_file(path):
    """(engine, engine_line_numbers, test, test_line_numbers) for one src file,
    by the cfg(test) separator."""
    lines = open(path, encoding="utf-8").read().split("\n")
    engine, eng_nums, test, test_nums = [], [], [], []
    i = 0
    n = len(lines)
    while i < n:
        m = MARKER.match(lines[i])
        if m and m.group(1) == "":
            # column-0 marker: the rest of the file is the test module
            test.extend(lines[i:])
            test_nums.extend(range(i + 1, n + 1))
            break
        if m:
            # indented marker: the next item, to the closing brace at its indent
            indent = m.group(1)
            j = i + 1
            while j < n and not (lines[j].startswith(indent + "}") and len(lines[j].rstrip()) == len(indent) + 1):
                j += 1
            test.extend(lines[i:j + 1])
            test_nums.extend(range(i + 1, j + 2))
            i = j + 1
            continue
        engine.append(lines[i])
        eng_nums.append(i + 1)
        i += 1
    return engine, eng_nums, test, test_nums


def count(lines, pat):
    return sum(len(pat.findall(strip_comment(l))) for l in lines)


def sites(lines, pat, rel, offset_lines=None):
    out = []
    for k, l in enumerate(lines):
        if pat.search(strip_comment(l)):
            out.append(f"{rel}:{(offset_lines or [None] * len(lines))[k] or '?'}: {l.strip()}")
    return out


def gather():
    cats = {"engine": [], "harness": [], "unit-tests": [], "tests": [], "support": []}
    # (category, relpath, lines, line-numbers)
    for dp, _, fns in os.walk(SRC):
        for fn in sorted(fns):
            if not fn.endswith(".rs"):
                continue
            p = os.path.join(dp, fn)
            rel = os.path.relpath(p, ROOT).replace("\\", "/")
            all_lines = open(p, encoding="utf-8").read().split("\n")
            if fn == "test_support.rs":
                cats["support"].append((rel, all_lines, list(range(1, len(all_lines) + 1))))
                continue
            engine, eng_nums, test, test_nums = split_file(p)
            main_cat = "harness" if "/src/bin/" in rel else "engine"
            cats[main_cat].append((rel, engine, eng_nums))
            if test:
                cats["unit-tests"].append((rel, test, test_nums))
    for dp, _, fns in os.walk(TESTS):
        for fn in sorted(fns):
            if fn.endswith(".rs"):
                p = os.path.join(dp, fn)
                rel = os.path.relpath(p, ROOT).replace("\\", "/")
                all_lines = open(p, encoding="utf-8").read().split("\n")
                cats["tests"].append((rel, all_lines, list(range(1, len(all_lines) + 1))))
    return cats


def main():
    cats = gather()
    if len(sys.argv) >= 4 and sys.argv[1] in ("--list", "--by-dir"):
        cat, which = sys.argv[2], sys.argv[3]
        pat = next(p for name, p in PATTERNS if name.startswith(which))
        if sys.argv[1] == "--list":
            for rel, lines, nums in cats[cat]:
                for s in sites(lines, pat, rel, nums):
                    print(s)
        else:
            by = {}
            for rel, lines, nums in cats[cat]:
                d = rel.rsplit("/", 1)[0]
                by[d] = by.get(d, 0) + count(lines, pat)
            for d, c in sorted(by.items(), key=lambda kv: -kv[1]):
                print(f"{c:6d}  {d}")
            print("--- top files ---")
            per = sorted(((count(lines, pat), rel) for rel, lines, nums in cats[cat]), reverse=True)
            for c, rel in per[:15]:
                if c:
                    print(f"{c:6d}  {rel}")
        return
    print("| pattern | engine | harness | unit-tests | tests | support |")
    print("|---|---:|---:|---:|---:|---:|")
    for name, pat in PATTERNS:
        row = [sum(count(lines, pat) for _, lines, _ in cats[c]) for c in ("engine", "harness", "unit-tests", "tests", "support")]
        print(f"| `{name}` | " + " | ".join(str(x) for x in row) + " |")
    print()
    for c in ("engine", "harness", "unit-tests", "tests", "support"):
        total = sum(len(lines) for _, lines, _ in cats[c])
        files = len(cats[c])
        print(f"{c}: {files} files, {total} lines")


if __name__ == "__main__":
    main()
