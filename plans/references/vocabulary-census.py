#!/usr/bin/env python3
"""The vocabulary census: every arm of the types a card's text is written in.

    python plans/references/vocabulary-census.py              # each type's arms, with payloads
    python plans/references/vocabulary-census.py --counts     # arms per type, the scope's size
    python plans/references/vocabulary-census.py --docs       # each arm with its doc's first sentence

WHY THIS EXISTS
---------------
`concept-census.py` asks of every field the game holds which CR concept it
holds, and stops at the types a card's text is written in, the register's
**Stops at:** list (`cr-coverage-audit.md#4c-concepts-and-their-owners`).
Nobody had asked those types the same question, and they spell the same few
words once per type: "this object" as `EffectRecipient::ThisObject`,
`Antecedent::ThisObject`, `ObjectSet::SourceOnly` and
`TriggerSubject::ThisObject`. This lists every arm in that scope, unit arms
included, so a person can group them by the word each one spells.

THE SCOPE
---------
The register's **Stops at:** types, and every struct or enum they hold,
transitively, through payloads and fields. A type the game holds as state is
the concept census's, so the walk stops at `GameState` and at the ids.
"""

import argparse
import re
import sys
from collections import defaultdict
from pathlib import Path

PLANS = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(PLANS))
from check_type_surface import AUDIT, ROOT, SRC, read_register  # noqa: E402
from rust_source import (  # noqa: E402
    ATTRIBUTE, ITEM, TYPE_NAME, _body_start, _closing, _doc_above, _split_top, code_only, test_items,
)

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

#: Where the walk stops: the game's state, which the concept census reads, and
#: the keys, which name a thing rather than spell a word for it.
NOT_VOCABULARY = {
    "GameState", "GameObject", "PermanentState", "PlayerState", "ObjectId", "ObjectRef", "PlayerId",
    "AbilityId", "ZoneChangeEpoch", "EventId", "BatchId", "String", "Vec", "Option", "Box", "Arc", "Rc",
}


class Item:
    """One struct or enum: its kind, file, line, and for an enum its arms as
    (name, payload, doc), for a struct its fields as (name, type, doc)."""

    def __init__(self, kind, name, path, line):
        self.kind, self.name, self.path, self.line = kind, name, path, line
        self.members: list[tuple[str, str, str]] = []

    def mentions(self) -> set[str]:
        return {t for _, payload, _ in self.members for t in TYPE_NAME.findall(payload)}


def read_items() -> dict[str, Item]:
    items: dict[str, Item] = {}
    for path in sorted(SRC.rglob("*.rs")):
        rel = path.relative_to(ROOT).as_posix()
        text = path.read_text(encoding="utf-8")
        code, lines = code_only(text), text.splitlines()
        tests = test_items(code)
        line_of = lambda offset: code.count("\n", 0, offset) + 1
        for m in ITEM.finditer(code):
            if any(a <= m.start() < b for a, b in tests):
                continue
            kind, name = m.groups()
            k = _body_start(code, m.end())
            if k is None or code[k] == ";":
                continue
            item = Item(kind, name, rel, line_of(m.start()))
            body = code[k + 1 : _closing(code, k)]
            blanked = ATTRIBUTE.sub(lambda a: " " * len(a.group(0)), body)
            for offset, part in _split_top(blanked):
                doc = _doc_above(lines, line_of(k + 1 + offset))
                first = doc.split(". ")[0][:160]
                if kind == "enum":
                    arm = re.match(r"([A-Z][A-Za-z0-9_]*)\s*(.*)", part, re.S)
                    if arm:
                        item.members.append((arm.group(1), " ".join(arm.group(2).split()), first))
                elif code[k] == "{" and ":" in part:
                    field, ty = part.split(":", 1)
                    field = re.sub(r"^\s*pub(?:\([^)]*\))?\s+", "", field).strip()
                    item.members.append((field, " ".join(ty.split()), first))
            # A name declared twice (a test double, a module-private helper)
            # keeps its first; the census reads the vocabulary, not the doubles.
            items.setdefault(name, item)
    return items


def scope(items: dict[str, Item], roots: set[str]) -> list[str]:
    seen, todo = set(), sorted(roots)
    while todo:
        name = todo.pop()
        if name in seen or name in NOT_VOCABULARY or name not in items:
            continue
        seen.add(name)
        todo.extend(sorted(items[name].mentions() - seen))
    return sorted(seen)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--counts", action="store_true")
    parser.add_argument("--docs", action="store_true")
    args = parser.parse_args()

    register = read_register(AUDIT.read_text(encoding="utf-8"))
    if register is None:
        print("no register section in cr-coverage-audit.md")
        return 2
    stops, _ = register
    items = read_items()
    names = scope(items, stops)
    enums = [items[n] for n in names if items[n].kind == "enum"]
    structs = [items[n] for n in names if items[n].kind == "struct"]

    if args.counts:
        arms = sum(len(e.members) for e in enums)
        print(f"scope: {len(enums)} enums with {arms} arms, {len(structs)} structs")
        by_file = defaultdict(int)
        for e in enums:
            by_file[e.path] += len(e.members)
        for path, n in sorted(by_file.items(), key=lambda kv: -kv[1]):
            print(f"  {n:4}  {path}")
        return 0

    for item in enums + structs:
        print(f"\n== {item.kind} {item.name}  [{item.path}:{item.line}]")
        for member, payload, doc in item.members:
            shown = f"  {member}{(' ' + payload) if payload and item.kind == 'enum' else (': ' + payload if payload else '')}"
            print(shown + (f"\n      {doc}" if args.docs and doc else ""))
    return 0


if __name__ == "__main__":
    sys.exit(main())
