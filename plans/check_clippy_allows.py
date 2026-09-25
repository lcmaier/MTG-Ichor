#!/usr/bin/env python3
"""
check_clippy_allows - an inline allow of a lint `Cargo.toml` already allows.

`mtgsim/Cargo.toml`'s `[lints.clippy]` table is where this crate allows a
clippy lint, one reason per lint, and CI's clippy step says so: "a new allow
goes there, with its reason, never at the site". An inline
`#[allow(clippy::X)]` for an X the table already allows silences nothing
today. Its cost comes later: the day the table drops X to revisit it (its
`too_many_arguments` entry names Phase 10 as that day), the inline allow keeps
hiding its site from exactly the sweep the removal was meant to start.

    python plans/check_clippy_allows.py      # exit 1 on a redundant inline allow

**The rule: no `allow(clippy::X)` in `mtgsim/` for an X the table allows.**
It reads `#[...]` and `#![...]`, an attribute split over several lines, a list
such as `allow(clippy::a, clippy::b)`, and an `allow` nested in `cfg_attr`.
An inline allow of a lint the table does *not* allow is not this gate's
business; CI's comment is the rule there.
"""

import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CRATE = ROOT / "mtgsim"

ATTRIBUTE = re.compile(r"#!?\[")
ALLOW = re.compile(r"\ballow\s*\(")
CLIPPY_LINT = re.compile(r"clippy::([a-z0-9_]+)")


def table_allows() -> set[str] | None:
    lints = tomllib.loads((CRATE / "Cargo.toml").read_text(encoding="utf-8"))
    table = lints.get("lints", {}).get("clippy")
    if table is None:
        return None
    level = lambda v: v.get("level") if isinstance(v, dict) else v
    return {name.replace("-", "_") for name, v in table.items() if level(v) == "allow"}


def balanced(text: str, start: int, opener: str, closer: str) -> int:
    """The index just past the `closer` matching the `opener` at `start`, or -1."""
    depth = 0
    for i in range(start, len(text)):
        if text[i] == opener:
            depth += 1
        elif text[i] == closer:
            depth -= 1
            if depth == 0:
                return i + 1
    return -1


def inline_allows(path: Path):
    """(line, lint) for every `clippy::` lint inside an `allow(...)` in an attribute.

    Crude but adequate: comment lines are dropped whole, which keeps a doc
    comment that *mentions* an attribute from reading as one, and an
    attribute's own line always starts with `#`.
    """
    lines = path.read_text(encoding="utf-8").splitlines()
    code = "\n".join("" if l.lstrip().startswith("//") else l for l in lines)
    for m in ATTRIBUTE.finditer(code):
        end = balanced(code, m.end() - 1, "[", "]")
        attr = code[m.start() : end if end > 0 else len(code)]
        for a in ALLOW.finditer(attr):
            close = balanced(attr, a.end() - 1, "(", ")")
            args = attr[a.end() : close if close > 0 else len(attr)]
            line = code.count("\n", 0, m.start()) + 1
            for lint in CLIPPY_LINT.findall(args):
                yield line, lint


def main() -> int:
    allowed = table_allows()
    if allowed is None:
        print("clippy allows: mtgsim/Cargo.toml has no [lints.clippy] table; the layout this gate reads has changed.")
        return 2
    files = sorted(p for p in CRATE.rglob("*.rs") if "target" not in p.relative_to(CRATE).parts)
    findings = [
        (p.relative_to(ROOT).as_posix(), line, lint)
        for p in files
        for line, lint in inline_allows(p)
        if lint in allowed
    ]

    print(f"clippy allows: {len(files)} .rs files searched; Cargo.toml allows {', '.join(sorted(allowed))}")
    if not findings:
        print("\nno inline allow repeats the table.")
        return 0

    print(f"\n{len(findings)} redundant inline allow(s):\n")
    for path, line, lint in findings:
        print(f"  {path}:{line}  clippy::{lint}")
    print(
        "\nCargo.toml's [lints.clippy] already allows each of these crate-wide. Delete the\n"
        "attribute: it silences nothing now, and it outlives the table's entry."
    )
    return 1


if __name__ == "__main__":
    sys.exit(main())
