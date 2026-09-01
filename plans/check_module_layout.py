#!/usr/bin/env python3
"""
check_module_layout - a `mod.rs` declares and re-exports; it does not define.

Every `mod.rs` in this crate was a pure re-exporter for the first two years,
by convention and without anyone writing the convention down. Then two phases
in a row put a few hundred lines of working code in one, because "the module
is small enough to be one file" is a locally reasonable thought that produces
a globally inconsistent tree: `engine/replacement/` splits into `gather.rs`,
`instance.rs` and `pipeline.rs`, so a reader looking for the equivalent of
`gather.rs` under `engine/restriction/` should find a file, not a `mod.rs`.

The cost of the drift is not aesthetic. `mod.rs` is the file a reader opens to
find out *what a module contains*; once it also contains the implementation,
that question needs a second read, and every subsequent file added to the
module has to argue about where it goes.

This is the mechanism, and it fails the way a warning fails.

    python plans/check_module_layout.py       # exit 1 if a mod.rs defines items

**What is allowed in a `mod.rs`:** module docs, `mod`/`pub mod` declarations,
`use`/`pub use` re-exports, and attributes. **What is not:** `fn`, `struct`,
`enum`, `trait`, `impl`, `type`, `const`, `static`, and `macro_rules!`.

`src/lib.rs` and `src/main.rs` are exempt: they are crate roots rather than
module roots, and `lib.rs` in particular is where a crate-level re-export lives.
"""

import re
import sys
from pathlib import Path

CRATE_SRC = Path(__file__).resolve().parent.parent / "mtgsim" / "src"

# Item keywords, anchored at the start of a line so a mention inside a doc
# comment or a string does not count. `pub`, `pub(crate)` and friends may
# precede; `unsafe`/`async`/`default` may too.
ITEM = re.compile(
    r"^\s*(?:pub(?:\s*\([^)]*\))?\s+)?"
    r"(?:default\s+|unsafe\s+|async\s+|extern\s+\"[^\"]*\"\s+)*"
    r"(fn|struct|enum|union|trait|impl|type|const|static|macro_rules!)\b"
)

# `pub use`/`use` and `mod`/`pub mod` are the whole allowed vocabulary, and
# neither can be confused with an item by the regex above.


def strip_block_comments(text: str) -> str:
    """Blank out /* ... */ so an item named inside one does not count.

    Line comments are left alone: `ITEM` is anchored to the start of a line and
    a `//` prefix already fails that anchor.
    """
    out, depth, i = [], 0, 0
    while i < len(text):
        if text.startswith("/*", i):
            depth += 1
            i += 2
        elif text.startswith("*/", i) and depth:
            depth -= 1
            i += 2
        else:
            out.append(" " if (depth and text[i] != "\n") else text[i])
            i += 1
    return "".join(out)


def offenders(root: Path):
    """Every `mod.rs` under `root` that defines an item, with the lines."""
    for path in sorted(root.rglob("mod.rs")):
        source = strip_block_comments(path.read_text(encoding="utf-8"))
        hits = [
            (n, line.rstrip())
            for n, line in enumerate(source.splitlines(), 1)
            if ITEM.match(line)
        ]
        if hits:
            yield path, hits


def main() -> int:
    if not CRATE_SRC.is_dir():
        print(f"no crate source at {CRATE_SRC}", file=sys.stderr)
        return 2

    found = list(offenders(CRATE_SRC))
    checked = sum(1 for _ in CRATE_SRC.rglob("mod.rs"))

    if not found:
        print(f"module layout: {checked} mod.rs files, all pure re-exporters.")
        return 0

    print("module layout: a `mod.rs` defines items. Move them to a named file.\n")
    for path, hits in found:
        rel = path.relative_to(CRATE_SRC.parent.parent)
        print(f"  {rel}")
        for n, line in hits[:8]:
            print(f"    {n:>5}  {line[:88]}")
        if len(hits) > 8:
            print(f"          ... and {len(hits) - 8} more")
        print()
    print(
        "A `mod.rs` declares and re-exports; the code goes in a sibling named for\n"
        "what it does — `engine/replacement/` is the pattern to copy. See\n"
        "`plans/engineering-practices.md` §6."
    )
    return 1


if __name__ == "__main__":
    sys.exit(main())
