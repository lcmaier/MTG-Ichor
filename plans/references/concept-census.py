#!/usr/bin/env python3
"""The concept census: `cr-coverage-audit.md` §4c's instrument, in code.

    python plans/references/concept-census.py                  # the scope's counts, by class
    python plans/references/concept-census.py --read           # the holders a person reads, by type
    python plans/references/concept-census.py --narrow-keys    # the ObjectId holders, for the narrow-key check
    python plans/references/concept-census.py --holders NAME   # one register concept's holders, by type
    python plans/references/concept-census.py --unclaimed      # --read, less every holder a concept claims

WHY THIS EXISTS
---------------
The census asks of each field the game holds which CR concept it holds, the
key the CR gives that concept, and which one holder owns it. Read by hand,
that is about 850 fields; a pilot on 191 of them found that about three
quarters are mechanical from the type alone. This script classes those, so
the reading is the rest, and lists what no register entry claims yet.

THE CLASSES
-----------
Every holder in the register's scope (`check_type_surface.py`'s `scope`) is
in exactly one, the first that applies:

  * KEY: its type names only keys (an id, an existence, an event or batch, an
    ability's identity), with containers around them. A key holds no value,
    but an `ObjectId` key gets `--narrow-keys`'s check.
  * ALLOCATOR: a counter ids come from (`next_*`).
  * OBSERVER: a handle to a trace, a log writer, a diagnostic tally or an
    audit's record, which nothing in a game branches on.
  * READ: everything else, which a person reads.
"""

import argparse
import re
import sys
from collections import Counter
from pathlib import Path

PLANS = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(PLANS))
from check_type_surface import AUDIT, Tree, holdings, read_register  # noqa: E402

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

KEYS = {
    "ObjectId", "ObjectRef", "PlayerId", "EventId", "BatchId", "AbilityId", "AbilityIdentity",
    "ZoneChangeEpoch", "DelayedTriggerId", "ExtraTurnId", "ReplacementInstanceId", "UntilReturnId",
    "TriggerId", "TargetRef", "ObjectOrPlayer",
}
CONTAINERS = {
    "Option", "Vec", "Box", "Rc", "Arc", "IdSet", "IdMap", "HashSet", "HashMap", "BTreeMap", "BTreeSet",
    "SmallVec", "VecDeque", "ObjectTable", "SharedList", "Range", "RangeInclusive", "NonZeroU32",
}
OBSERVERS = {
    "TraceHandle", "TraceSink", "RecorderHandle", "EventRecorder", "DecisionLogHandle", "Diagnostics",
    "DispatchAudit", "AuditScope", "LayerMemo",
}


def classify(holder) -> str:
    types = holder.types() - CONTAINERS
    if holder.field.startswith("next_"):
        return "allocator"
    if types & OBSERVERS:
        return "observer"
    if types and types <= KEYS:
        return "key"
    return "read"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--read", action="store_true")
    parser.add_argument("--narrow-keys", action="store_true")
    parser.add_argument("--holders", metavar="NAME")
    parser.add_argument("--unclaimed", action="store_true")
    args = parser.parse_args()

    register = read_register(AUDIT.read_text(encoding="utf-8"))
    if register is None:
        print("no register section in cr-coverage-audit.md")
        return 2
    stops, concepts = register
    tree = Tree()
    by_type = tree.by_type
    in_scope = sorted(tree.scope(stops))
    held = [holder for name in in_scope for holder in by_type[name]]

    if args.holders:
        matching = [c for c in concepts if c.name.lower().startswith(args.holders.lower())]
        for concept in matching:
            print(f"== {concept.name}  ({'; '.join(t + (' named ' + w if w else '') for t, w in concept.queries)})")
            for name, n in holdings(concept, by_type, set(in_scope)).items():
                listed = concept.listed.get(name, 0)
                print(f"  {name} x{n}" + ("" if listed == n else f"   (listed x{listed})"))
                for holder in by_type[name]:
                    if concept.claims(holder):
                        print(f"      {holder.name()} : {holder.ty}")
        return 0 if matching else 1

    if args.narrow_keys:
        for holder in held:
            if "ObjectId" in holder.types() and classify(holder) == "key":
                print(f"{holder.name()} : {holder.ty}  [{holder.path}:{holder.line}]")
        return 0

    if args.read or args.unclaimed:
        claimed = {id(h) for c in concepts for h in held if c.claims(h)}
        current = None
        for holder in held:
            if classify(holder) != "read" or (args.unclaimed and id(holder) in claimed):
                continue
            if holder.owner != current:
                current = holder.owner
                print(f"\n== {current}  [{holder.path}]")
            doc = holder.doc[:220] + ("..." if len(holder.doc) > 220 else "")
            print(f"  {holder.name()} : {holder.ty}" + (f"\n      {doc}" if doc else ""))
        return 0

    classes = Counter(classify(holder) for holder in held)
    claimed = sum(1 for h in held if any(c.claims(h) for c in concepts))
    print(f"scope: {len(in_scope)} types, {len(held)} holders")
    for name, n in classes.most_common():
        print(f"  {n:4}  {name}")
    print(f"claimed by the register's {len(concepts)} concepts: {claimed}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
