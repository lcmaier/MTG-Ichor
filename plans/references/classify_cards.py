#!/usr/bin/env python3
"""
classify_cards - which of this crate's card definitions are real printings?

    python plans/references/classify_cards.py

Three sets come out, and `codebase-state.md` "Before card breadth" item 10
turns on the difference between them:

- **printings** — a registered name Scryfall knows. The card list.
- **fixtures** — a card function that is either registered under a name
  Scryfall does not know (`Loyalty Probe`) or not registered at all
  (`flight_clause`, `generic_reducer`, the `*_spell` stand-ins).
- **the playable pool** — everything `registry.rs` registers, drawn from both.
  `fuzz_games` and `cli_play` can only build a deck from this, which is why it
  will always be able to contain a fixture.

The card-list reorganization (roadmap-v2 §C's C0) is a move driven by that
classification, so the classification has to be re-runnable rather than a
number someone wrote down once. It also prints the printing-set histogram,
which is what ruled *out* filing the list by set: 62 sets for 97 cards, 47
of them holding exactly one.

**The set column is a printing, not necessarily the first.** Scryfall's
collection endpoint answers a name lookup with one printing of its choosing;
finding the earliest would be one `prints_search_uri` fetch per card. The
histogram is used only to show that set-based filing fragments at this scale,
and it fragments either way.

Scryfall 403s a bare urllib request, so this shells out to curl with a UA
header, the way `plans/references/*-census.py` and `CLAUDE.md` both say to.
"""

import collections
import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path

CARDS = Path(__file__).resolve().parent.parent.parent / "mtgsim" / "src" / "cards"
CHUNK = 75  # Scryfall's documented maximum per /cards/collection request


def registered() -> list[tuple[str, str, str]]:
    """(card name, module, function) for every `registry.register` call."""
    src = (CARDS / "registry.rs").read_text(encoding="utf-8")
    return re.findall(r'registry\.register\(\s*"([^"]+)",\s*([a-z_0-9]+)::([a-z_0-9]+)', src)


def defined() -> list[tuple[str, str]]:
    """(module, function) for every `pub fn` in a card file."""
    out = []
    for path in sorted(CARDS.glob("*.rs")):
        if path.stem in ("registry", "mod"):
            continue
        for fn in re.findall(r"^pub fn ([a-z_0-9]+)", path.read_text(encoding="utf-8"), re.M):
            out.append((path.stem, fn))
    return out


def scryfall(names: list[str]) -> tuple[list[dict], list[str]]:
    found, missing = [], []
    for i in range(0, len(names), CHUNK):
        body = json.dumps({"identifiers": [{"name": n} for n in names[i : i + CHUNK]]})
        with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False, encoding="utf-8") as f:
            f.write(body)
            payload = f.name
        proc = subprocess.run(
            [
                "curl", "-s",
                "-H", "User-Agent: mtgsim-dev/1.0",
                "-H", "Content-Type: application/json",
                "--data-binary", f"@{payload}",
                "https://api.scryfall.com/cards/collection",
            ],
            capture_output=True, check=True,
        )
        Path(payload).unlink(missing_ok=True)
        # Bytes, decoded as UTF-8 explicitly: Scryfall returns accented card
        # names and Windows would otherwise decode the pipe as cp1252 and die.
        data = json.loads(proc.stdout.decode("utf-8"))
        found.extend(data.get("data", []))
        missing.extend(nf.get("name") for nf in data.get("not_found", []))
    return found, missing


def main() -> int:
    reg = registered()
    reg_fns = {(m, f) for _, m, f in reg}
    names = sorted({n for n, _, _ in reg})

    found, missing = scryfall(names)

    unregistered = [(m, f) for m, f in defined() if (m, f) not in reg_fns]

    print(f"registered names        {len(names)}")
    print(f"  printings             {len(found)}")
    print(f"  registered fixtures   {len(missing)}")
    for m in sorted(missing):
        print(f"      {m}")
    print(f"unregistered fixtures   {len(unregistered)}")
    for m, f in unregistered:
        print(f"      {m}::{f}")
    print(f"\nfixtures, total         {len(missing) + len(unregistered)}")

    sets = collections.Counter(c.get("set", "?") for c in found)
    print(f"\nprinting sets among the {len(found)} printings: {len(sets)}")
    print("  largest buckets: " + ", ".join(f"{s}={n}" for s, n in sets.most_common(5)))
    singles = sum(1 for n in sets.values() if n == 1)
    print(f"  sets holding exactly one card: {singles}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
