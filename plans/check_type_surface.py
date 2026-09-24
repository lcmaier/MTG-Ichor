#!/usr/bin/env python3
"""
check_type_surface - every field `GameState` holds is named in the audit.

`plans/cr-coverage-audit.md` §4 promised that a new type joins the type-surface
sweep the day it joins `GameState`. Nothing held the promise, and it lapsed:
between `3c322e5` and 2026-09-24 `GameState` went from 28 fields to 52, and the
sweep read none of the new ones until the re-sweep caught up (§4a).

    python plans/check_type_surface.py      # exit 1 if a field goes unnamed

**The rule: each field of `pub struct GameState` appears in backticks in
`cr-coverage-audit.md`.** §4b is where each one is named, beside the pass that
read it or the reason it holds no fact about the game. A pull request that adds
a field asks §2's question of it and adds the name there.

**Two parses, and they must agree.** The declaration and `GameState::new`'s
struct literal each list every field, since Rust refuses a literal that leaves
one out. So the gate reads both, and it fails if they differ. A regex that
drops a field and still reports a confident answer is the defect §1 records
twice over.

**What it does not see is a field added to a type `GameState` already holds**
(`GameObject.timestamp`, `PlayerState.counters`). That was the owner's call on
2026-09-24: the swept record structs are the most-edited in the tree, and a
gate on their fields would fail every rename there.
"""

import re
import sys
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

ROOT = Path(__file__).resolve().parent.parent
STATE = ROOT / "mtgsim" / "src" / "state" / "game_state.rs"
AUDIT = ROOT / "plans" / "cr-coverage-audit.md"

DECLARATION = re.compile(r"^pub struct GameState \{$")
#: A field at the struct's own indentation. A doc comment starts with `/` and
#: an attribute with `#`, so neither is mistaken for one.
DECLARED = re.compile(r"^    (?:pub(?:\([^)]*\))?\s+)?([a-z_][a-z0-9_]*)\s*:")

LITERAL = re.compile(r"^        GameState \{$")
#: A field one level inside the literal: `resolving: None,` or the shorthand
#: `starting_life,`. A nested block's own lines sit deeper and are skipped.
INITIALIZED = re.compile(r"^            ([a-z_][a-z0-9_]*)\s*[:,]")


def fields(lines, opener, field):
    """The fields inside the one block whose first line matches `opener`.

    `None` unless exactly one line matches, so a second `GameState {` literal
    stops the gate rather than being read in place of the first.
    """
    starts = [i for i, line in enumerate(lines) if opener.match(line)]
    if len(starts) != 1:
        return None
    start = starts[0]
    indent = lines[start][: len(lines[start]) - len(lines[start].lstrip())]
    for end in range(start + 1, len(lines)):
        if lines[end].startswith(indent + "}"):
            body = lines[start + 1 : end]
            return [m.group(1) for line in body if (m := field.match(line))]
    return None


def main() -> int:
    lines = STATE.read_text(encoding="utf-8").splitlines()
    declared = fields(lines, DECLARATION, DECLARED)
    initialized = fields(lines, LITERAL, INITIALIZED)

    if not declared or not initialized:
        print(
            f"type surface: could not find one `pub struct GameState` and one "
            f"`GameState {{` literal in {STATE.relative_to(ROOT)}.\nThe layout "
            "this gate reads has changed; update its two openers."
        )
        return 2
    if set(declared) != set(initialized):
        print("type surface: the declaration and `GameState::new` disagree.")
        for name in sorted(set(declared) - set(initialized)):
            print(f"  declared, not initialized: {name}")
        for name in sorted(set(initialized) - set(declared)):
            print(f"  initialized, not declared: {name}")
        print("\nOne of the two parses dropped a field; fix the gate's patterns.")
        return 2

    audit = AUDIT.read_text(encoding="utf-8")
    missing = [name for name in declared if f"`{name}`" not in audit]

    print(f"type surface: {len(declared)} GameState fields, read twice and in agreement")
    if not missing:
        print(f"\nevery field is named in {AUDIT.relative_to(ROOT)}.")
        return 0

    print(f"\n{len(missing)} field(s) {AUDIT.name} does not name:\n")
    for name in missing:
        print(f"  {name}")
    print(
        "\nAsk the type-surface question of each (cr-coverage-audit.md §2): what "
        "can the CR\nrequire here that this cannot represent? Then name it in "
        "§4b, beside the answer."
    )
    return 1


if __name__ == "__main__":
    sys.exit(main())
