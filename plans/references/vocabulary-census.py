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
`cr-coverage-audit.md#4d-words-and-their-spellings`; `check_type_surface.py`'s
third check holds them true, and this reads the same walk to show its parts.
"""

import argparse
import sys
from pathlib import Path

PLANS = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(PLANS))
from check_type_surface import AUDIT, read_register, read_type_declarations, read_vocabulary, vocabulary_scope  # noqa: E402

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")


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
    classes, words = vocabulary
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
