#!/usr/bin/env python3
"""
check_string_literals - a run of spaces in the middle of a message is a scar.

Rust lets a string literal span lines with a trailing backslash, which eats the
newline *and* the next line's indentation:

    "a sentence that is too long \\
     to fit on one line"          ->  "a sentence that is too long to fit on one line"

Every long message in this tree is written that way. The failure mode is that
the backslash goes missing — a tool that rewrites the file, a copy through a
shell heredoc, a hand edit that reflows the line — and what is left is the
indentation, baked into the string:

    "a sentence that is too long              to fit on one line"

The message still compiles, still prints, and still reads as a sentence to
whoever wrote it. It is only wrong on the one occasion anybody sees it: a
panic, an `Err` a test prints, a log line a person is reading because something
already went wrong.

**It had landed four times before anyone noticed**, in `phase_rb_cards.rs`
(twice), `put_on_stack.rs` and `resolve.rs`, over about three weeks. A note in
somebody's memory is not a mechanism; this is.

    python plans/check_string_literals.py      # exit 1 on a scarred literal

**The rule: no run of 10 or more spaces inside a string literal.** Ten rather
than three because column-aligned output is a real thing and this tree has a lot
of it — `fuzz_games`' report is nothing but label-and-value columns, and its
widest gap is seven. A lost continuation leaves the source indentation, which in
this tree is never under ten. Measured when the gate was written: 33 lines carry
a run of ten, two of them legitimate.

**The one exemption is that alignment**, recognized by shape rather than by
path: a run that sits between a `:` and a `{` is a label followed by a format
placeholder, which is what a column is. Nothing else is exempt, and a file is
never exempt.

**If you need a long message**, use the backslash continuation like everything
else — and if a tool keeps eating it, write the message as one long line and let
`rustfmt` leave it alone. A literal is not the place to be tidy.
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CRATE = ROOT / "mtgsim"

#: Shorter than this and column alignment is indistinguishable from a scar.
MIN_RUN = 10

RUN = re.compile(r"\S {%d,}\S" % MIN_RUN)
#: `"Errors:          {}"` — a label, a gap, a placeholder. That is a column.
ALIGNED = re.compile(r":\s{%d,}\{" % MIN_RUN)


def literals(line: str):
    """Yield the string literals on one line, crudely but adequately.

    Good enough because the thing being looked for is a run of spaces, and
    neither a char literal nor a lifetime can contain one.
    """
    out, i, n = [], 0, len(line)
    while i < n:
        if line[i] == '"':
            j, buf = i + 1, ['"']
            while j < n:
                if line[j] == "\\":
                    buf.append(line[j : j + 2])
                    j += 2
                    continue
                buf.append(line[j])
                if line[j] == '"':
                    j += 1
                    break
                j += 1
            out.append("".join(buf))
            i = j
        else:
            i += 1
    return out


def scarred(path: Path):
    bad = []
    for n, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        body = line.lstrip()
        if body.startswith("//"):
            continue
        for lit in literals(line):
            if RUN.search(lit) and not ALIGNED.search(lit):
                bad.append((n, lit.strip()))
    return bad


def main() -> int:
    files = sorted(
        list((CRATE / "src").rglob("*.rs")) + list((CRATE / "tests").rglob("*.rs"))
    )
    findings = []
    for p in files:
        for n, lit in scarred(p):
            findings.append((p.relative_to(ROOT), n, lit))

    print(f"string literals: {len(files)} .rs files searched")
    if not findings:
        print(f"\nno literal carries a run of {MIN_RUN}+ spaces outside a column.")
        return 0

    print(f"\n{len(findings)} scarred literal(s):\n")
    for path, n, lit in findings:
        shown = lit if len(lit) <= 100 else lit[:97] + '..."'
        print(f"  {path}:{n}\n      {shown}")
    print(
        "\nA run of spaces in the middle of a message is a lost `\\` line "
        "continuation.\nJoin the line, or re-add the backslash. See this "
        "file's docstring for why\nthis is a gate and not a note."
    )
    return 1


if __name__ == "__main__":
    sys.exit(main())
