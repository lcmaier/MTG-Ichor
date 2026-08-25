"""Scryfall census behind `plans/replacement-architecture.md` §3.2c.

    python plans/references/replacement-census.py

Pulls every card matching o:/would.*instead/ and classifies each
"would ... instead" clause by the event kind it watches, so the load-bearing
claim in §3.2c stays falsifiable: **does the `Rewrite` algebra need a sixth
arm?** (2026-08-24: no -- 0 of 574 clauses.)

WHY THIS EXISTS
---------------
Reconstructing the classifier from prose is the kind of work that does not get
done, so without it the claim would quietly become folklore. A future phase that
changes the algebra has to re-test it.

It counts; it does not judge. The buckets are regexes over oracle text, so the
residual always needs a human read -- that is the point of printing it.

A `sizing` subcommand lived here until 2026-08-24 and was **deleted rather than
fixed**: it answered a one-time bird's-eye question (how large does `GameAction`
get), its numbers are now recorded in §8b, and two of its readings were known to
be wrong (Regenerate=0 because it replaces *destruction*; EnterBattlefield=7211
because that is every ETB creature ever printed). A tool that prints answers
known to be wrong is worse than no tool -- if that question needs re-asking,
write a better instrument for it.

WHEN TO DELETE THIS FILE
------------------------
When the `Rewrite` algebra has shipped and stopped changing, so §3.2c has no
remaining customer -- or when these numbers are stale enough that a reader would
trust them over a fresh look, which a comment cannot prevent.

Its output is prose input, never an artifact. Nothing in the repo is derived
from it -- that is what separates it from `extract-phase-index.py`, which was
deleted for regenerating *authoritative* files from a stale source.

Results are cached in this directory (gitignored). Scryfall asks for a courteous
request rate; the delay and backoff below are deliberate -- do not tighten them.
"""
import argparse
import json
import os
import re
import time
import urllib.error
import urllib.parse
import urllib.request
from collections import Counter, defaultdict

UA = "mtgsim-research/1.0 (contact: maiercluke@gmail.com)"
HERE = os.path.dirname(os.path.abspath(__file__))
DELAY = 0.3


def _get(url):
    req = urllib.request.Request(url, headers={"User-Agent": UA, "Accept": "application/json"})
    for attempt in range(6):
        try:
            with urllib.request.urlopen(req) as r:
                return json.load(r)
        except urllib.error.HTTPError as e:
            if e.code == 404:
                return None
            if e.code == 429:
                time.sleep(5 * (attempt + 1))
                continue
            raise
    raise RuntimeError("rate limited out: " + url)


def search_all(q):
    """Every card matching `q`, following pagination."""
    url = f"https://api.scryfall.com/cards/search?q={urllib.parse.quote(q)}&unique=cards"
    out = []
    while url:
        d = _get(url)
        if d is None:
            break
        out.extend(d["data"])
        url = d.get("next_page")
        time.sleep(DELAY)
    return out


# --------------------------------------------------------------------------
# `clauses` -- §3.2c
# --------------------------------------------------------------------------

# Non-vocabulary kinds are tested first so a broad arm cannot swallow them.
KINDS_MISSING = [
    ("!RollDice", r"would roll"),
    ("!Scry", r"would scry|would surveil"),
    ("!CounterSpell", r"would counter|would be countered"),
    ("!SearchLibrary", r"would search"),
    ("!Sacrifice", r"would (be )?sacrifice"),
    ("!Cast/Play", r"would cast|would play"),
    ("!Copy", r"would copy|would be copied"),
    ("!Attack/Block", r"would (attack|block)"),
    ("!TurnFaceUp", r"turned face"),
    ("!Vote", r"would vote"),
]
KINDS_KNOWN = [
    ("EnterBattlefield*", r"would enter"),
    ("ZoneChange*", r"would be put into|would be exiled|would die|would be destroyed|would leave"
                    r"|would be returned|would be shuffled|would be milled|would mill"
                    r"|would put .*into (a|its|their|your|the) (graveyard|library|hand|exile)"
                    r"|would be discarded|would discard|causes you to discard"),
    ("DealDamage*", r"damage|would be dealt"),
    ("DrawCard*", r"would draw"),
    ("AddCounters*", r"counters?"),
    ("CreateTokens*", r"would create|tokens? would be created|would be created"),
    ("GainLife*", r"would gain .*life"),
    ("LoseLife*", r"would lose .*life|life total would be reduced|would pay life"),
    ("TapUntap*", r"would (become )?(untap|tap)"),
    ("ProduceMana*", r"mana"),
    ("BeginStep*", r"would (begin|skip)|skips? (their|your|the)"),
    ("PlayerLosesWins", r"would (lose|win) the game"),
]


def cmd_clauses(_args):
    cache = os.path.join(HERE, ".census-clauses.json")
    if os.path.exists(cache):
        cards = json.load(open(cache, encoding="utf-8"))
    else:
        cards = search_all("o:/would.*instead/ -is:funny")
        json.dump(cards, open(cache, "w", encoding="utf-8"))

    clauses = []
    for c in cards:
        txt = c.get("oracle_text") or ""
        if not txt and "card_faces" in c:
            txt = "\n".join(f.get("oracle_text", "") for f in c["card_faces"])
        for sent in re.split(r"(?<=\.)\s+|\n", txt):
            if re.search(r"would", sent, re.I) and re.search(r"instead", sent, re.I):
                clauses.append((c["name"], " ".join(sent.split())))

    order = KINDS_MISSING + KINDS_KNOWN

    def kind(s):
        for name, pat in order:
            if re.search(pat, s, re.I):
                return name
        return "UNCLASSIFIED"

    counts, ex = Counter(), defaultdict(list)
    for name, s in clauses:
        k = kind(s)
        counts[k] += 1
        if len(ex[k]) < 30:
            ex[k].append((name, s))

    print(f"cards: {len(cards)}   clauses: {len(clauses)}\n")
    known = missing = unk = 0
    for k, n in counts.most_common():
        print(f"  {k:<20} {n:>4}")
        if k.endswith("*"):
            known += n
        elif k.startswith("!"):
            missing += n
        else:
            unk += n
    print(f"\nin the GameAction vocabulary: {known}")
    print(f"needs a new GameAction:       {missing}")
    print(f"unclassified:                 {unk}   (expect CR 701 keyword actions)")
    print("\nUNCLASSIFIED residual:")
    for name, s in ex["UNCLASSIFIED"]:
        print(f"  - {name}: {s[:165]}")
    print("\nA sixth `Rewrite` arm is needed only if some clause above cannot be")
    print("expressed as Prevent / Amount / Retarget / EnterWith / Instead. Read the")
    print("residual and decide by hand -- this script counts, it does not judge.")


def main():
    argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    ).parse_args()
    cmd_clauses(None)


if __name__ == "__main__":
    main()
