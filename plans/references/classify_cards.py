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
number someone wrote down once.

    python plans/references/classify_cards.py --first-printings

**The set a card is filed under is its *first* printing**, which is the only
key that never changes — C0 files the list by it. That costs one request per
card and is behind the flag; without it the run is two requests and answers
only the printing/fixture split, which is what most re-runs want.

Scryfall 403s a bare urllib request, so this shells out to curl with a UA
header, the way `plans/references/*-census.py` and `CLAUDE.md` both say to.
**Its rate limit is ~10 requests/second and it means it** — the first draft of
the `--first-printings` pass slept 80ms and got a 60-second ban that came back
as 76 cards "not found", which is a wrong answer rather than an error. Hence
`SLEEP` and the retry below.
"""

import collections
import json
import re
import subprocess
import sys
import tempfile
import time
import urllib.parse
from pathlib import Path

CARDS = Path(__file__).resolve().parent.parent.parent / "mtgsim" / "src" / "cards"
CHUNK = 75  # Scryfall's documented maximum per /cards/collection request
SLEEP = 0.15  # Scryfall asks for <10 req/s; 80ms earned a 60-second ban.


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


def curl(args: list[str]) -> dict:
    """One Scryfall call, retrying once through a rate-limit answer.

    A 429 comes back as a JSON `error` object rather than a non-zero exit, so
    a caller that only checked the exit code would read "rate limited" as
    "no such card" — which is exactly how this script first reported 76 of
    its 97 cards as fixtures.
    """
    for attempt in range(2):
        proc = subprocess.run(
            ["curl", "-s", "-H", "User-Agent: mtgsim-dev/1.0", *args],
            capture_output=True, check=True,
        )
        data = json.loads(proc.stdout.decode("utf-8"))
        if data.get("object") == "error" and "rate-limit" in data.get("details", ""):
            if attempt == 0:
                time.sleep(65)
                continue
        return data
    return data


def first_printings(names: list[str]) -> dict[str, tuple[str, str]]:
    """name -> (set code, year) of its earliest printing.

    One request per card: Scryfall's collection endpoint answers a name lookup
    with a printing of its own choosing, and there is no bulk way to ask for
    the earliest. `unique=prints` ordered ascending by release makes the first
    result the answer.
    """
    out = {}
    for i, n in enumerate(names, 1):
        q = urllib.parse.quote(f'!"{n}"')
        data = curl([
            f"https://api.scryfall.com/cards/search"
            f"?q={q}&unique=prints&order=released&dir=asc"
        ])
        if data.get("object") == "error" or not data.get("data"):
            out[n] = ("?", "?")
        else:
            c = data["data"][0]
            out[n] = (c.get("set", "?"), c.get("released_at", "????")[:4])
        if i % 20 == 0:
            print(f"    ... {i}/{len(names)}", file=sys.stderr)
        time.sleep(SLEEP)
    return out


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

    if "--first-printings" not in sys.argv:
        print("\n(pass --first-printings for the set histogram C0 files by)")
        return 0

    real = sorted({c["name"] for c in found})
    firsts = first_printings(real)
    sets = collections.Counter(s for s, _ in firsts.values())
    years = {s: y for s, y in firsts.values()}
    print(f"\nfirst-printing sets among the {len(real)} printings: {len(sets)}")
    for s, n in sets.most_common():
        print(f"  {s:6s} {years.get(s, '?'):4s}  {n}")
    print(f"  sets holding exactly one card: {sum(1 for n in sets.values() if n == 1)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
