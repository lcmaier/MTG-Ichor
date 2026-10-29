#!/usr/bin/env python3
"""The vocabulary census: every arm of the types a card's text is written in.

    python plans/references/vocabulary-census.py              # each reached enum's arms, with payloads
    python plans/references/vocabulary-census.py --counts     # enums and arms per class, the scope's size
    python plans/references/vocabulary-census.py --docs       # each arm with its doc's first sentence
    python plans/references/vocabulary-census.py --words      # only the `words` class, the arms the register lines

WHY THIS EXISTS
---------------
`concept-census.py` asks of every field the game holds which CR concept it
holds, and stops at the types a card's text is written in, the register's
**Stops at:** list (`cr-coverage-audit.md#4c-concepts-and-their-owners`).
Nobody had asked those types the same question, and they spelled the same few
words once per type: "this object" as `EffectRecipient::ThisObject`,
`Antecedent::ThisObject`, `ObjectSet::SourceOnly`, `TriggerSubject::ThisObject`
and three more. This lists every arm in that scope, unit arms included, so a
person can group them by the word each one spells.

The classes, and each word arm's verdict, are the register's,
`cr-coverage-audit.md#4d-words-and-their-spellings`, a dated record since
`route-review` (2026-10-29); this reads its walk to show the parts.
"""

import argparse
import re
import sys
from pathlib import Path

PLANS = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(PLANS))
from check_type_surface import AUDIT, SRC, ROOT, read_register  # noqa: E402
from rust_source import type_declarations  # noqa: E402

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

VOCABULARY = "## 4d. Words and their spellings"
#: A class's bullet: "- **values** — reason: `A`, `B`", wrapping.
CLASS = re.compile(r"^- \*\*([a-z]+)\*\* —")
WORD_ENTRY = re.compile(r"^#### `([A-Z][A-Za-z0-9_]*)`$")
#: An arm's line: "- `Arm` — **verdict** ...".
ARM = re.compile(r"^- `([A-Z][A-Za-z0-9_]*)` — (.*)$")
FIELDS_HEADING = "### The fields"
#: A field's line: "- `Owner.field` — ..." or "- `Owner::Variant.0` — ...".
FIELD = re.compile(r"^- `([A-Z][A-Za-z0-9_]*(?:::[A-Z][A-Za-z0-9_]*)?\.[A-Za-z0-9_]+)` — (.*)$")
#: Where the walk stops: the game's state, which `check_type_surface.py`'s
#: register reads, and the keys and containers, which spell no word.
NOT_VOCABULARY = {
    "GameState", "GameObject", "PermanentState", "PlayerState", "ObjectId", "ObjectRef", "PlayerId",
    "AbilityId", "ZoneChangeEpoch", "EventId", "BatchId", "String", "Vec", "Option", "Box", "Arc", "Rc",
}


def read_vocabulary(text: str):
    """§4d's set-aside classes, as {class: enums}, its word entries, as
    {enum: {arm: line}}, and its field lines, as {field: line}; `None`
    without the section."""
    lines = text.splitlines()
    starts = [i for i, line in enumerate(lines) if line.strip() == VOCABULARY]
    if len(starts) != 1:
        return None
    bullet_lines: dict[str, list[str]] = {}
    words: dict[str, dict[str, str]] = {}
    fields: dict[str, str] = {}
    current_class, current_enum, in_fields = None, None, False
    for line in lines[starts[0] + 1 :]:
        if line.startswith("## ") or line.startswith("---"):
            break
        if line.startswith("### "):
            in_fields = line.strip() == FIELDS_HEADING
        if in_fields:
            if m := FIELD.match(line):
                fields[m.group(1)] = m.group(2)
            continue
        if m := CLASS.match(line):
            current_class, current_enum = m.group(1), None
            bullet_lines[current_class] = [line]
        elif m := WORD_ENTRY.match(line):
            current_class, current_enum = None, m.group(1)
            words[current_enum] = {}
        elif current_enum and (m := ARM.match(line)):
            words[current_enum][m.group(1)] = m.group(2)
        elif not line.strip() or line.startswith("#") or line.startswith("- "):
            current_class = None
        elif current_class:
            bullet_lines[current_class].append(line)
    # A class's reason may name types too; its members are the names after
    # the colon that ends the reason.
    classes = {
        name: set(re.findall(r"`([A-Z][A-Za-z0-9_]*)`", re.split(r":\s+(?=`)", " ".join(lines), maxsplit=1)[-1]))
        for name, lines in bullet_lines.items()
    }
    return classes, words, fields


def read_type_declarations():
    """Every struct and enum outside test code, by name; a name declared
    twice keeps its first, the vocabulary being read and not its doubles."""
    found = {}
    for path in sorted(SRC.rglob("*.rs")):
        for decl in type_declarations(path.relative_to(ROOT).as_posix(), path.read_text(encoding="utf-8")):
            found.setdefault(decl.name, decl)
    return found


def vocabulary_scope(declared, roots) -> list[str]:
    """The structs and enums `roots` hold, transitively, stopping at the
    game's state and the keys."""
    seen, todo = set(), sorted(roots)
    while todo:
        name = todo.pop()
        if name in seen or name in NOT_VOCABULARY or name not in declared:
            continue
        seen.add(name)
        todo.extend(sorted(declared[name].mentions() - seen))
    return sorted(seen)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--counts", action="store_true")
    parser.add_argument("--docs", action="store_true")
    parser.add_argument("--words", action="store_true")
    args = parser.parse_args()

    text = AUDIT.read_text(encoding="utf-8")
    register, vocabulary = read_register(text), read_vocabulary(text)
    if register is None or vocabulary is None:
        print("no §4c or §4d section in cr-coverage-audit.md")
        return 2
    stops, _ = register
    classes, words, _ = vocabulary
    classes = dict(classes, words=set(words))
    declared = read_type_declarations()
    enums = [declared[n] for n in vocabulary_scope(declared, stops) if declared[n].kind == "enum"]

    if args.counts:
        print(f"scope: {len(enums)} enums with {sum(len(e.members) for e in enums)} arms")
        for name, members in classes.items():
            reached = [e for e in enums if e.name in members]
            print(f"  {name:9} {len(reached):3} enums, {sum(len(e.members) for e in reached):4} arms")
        return 0

    shown = [e for e in enums if not args.words or e.name in words]
    for item in shown:
        print(f"\n== enum {item.name}  [{item.path}:{item.line}]")
        for member, payload, doc in item.members:
            print(f"  {member}{(' ' + payload) if payload else ''}" + (f"\n      {doc}" if args.docs and doc else ""))
    return 0


if __name__ == "__main__":
    sys.exit(main())
