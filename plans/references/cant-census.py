"""Scryfall census behind `plans/cant-effects-architecture.md` §2.

    python plans/references/cant-census.py            # the tier table
    python plans/references/cant-census.py --residual # what no bucket caught

Pulls every card matching `o:"can't" -is:funny` and classifies each *clause*
containing "can't" by **where the engine would have to enforce it**, not by what
the card is about. That is the load-bearing claim of the "can't" design: the
1,857 cards do not want one mechanism, they want six, and five of the six are
not the CR 614.17 event block that Phase RB built.

WHY THIS EXISTS
---------------
`replacement-census.py`'s sibling, and written for the same reason: the tier
sizes are what decide how much machinery each enforcement point earns, and a
number reconstructed from prose is a number nobody re-checks. The design doc
cites this file for every count in its §2 table.

It counts; it does not judge. The buckets are regexes over oracle text, so
`--residual` always needs a human read -- that is the point of printing it. The
buckets are ordered most-specific-first and a clause lands in exactly one, so
the tier column sums to the clause total.

WHAT THE NUMBERS ARE NOT
------------------------
A clause count, not a card count -- one card can contribute several clauses to
several tiers (Academic Probation is Tier 1 twice over). Card counts are printed
alongside and are the smaller number. Neither is a *work* estimate: Tier 1a's
~1,300 clauses collapse onto roughly a dozen predicate shapes, which is the
whole reason the design puts a grammar there rather than an enum.

WHEN TO DELETE THIS FILE
------------------------
When the restriction model has shipped and stopped changing, so §2 has no
remaining customer -- or when these numbers are stale enough that a reader would
trust them over a fresh look, which a comment cannot prevent.

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
CACHE = os.path.join(HERE, ".census-cant.json")
QUERY = "o:\"can't\" -is:funny"
DELAY = 0.3


def _get(url):
    req = urllib.request.Request(url, headers={"User-Agent": UA, "Accept": "application/json"})
    for attempt in range(6):
        try:
            with urllib.request.urlopen(req, timeout=60) as r:
                return json.load(r)
        except urllib.error.HTTPError as e:
            if e.code == 404:
                return None
            if e.code == 429:
                time.sleep(8 * (attempt + 1))
                continue
            raise
    raise RuntimeError("rate limited out: " + url)


def fetch():
    if os.path.exists(CACHE):
        return json.load(open(CACHE, encoding="utf-8"))
    url = "https://api.scryfall.com/cards/search?q=%s&unique=cards" % urllib.parse.quote(QUERY)
    out = []
    while url:
        d = _get(url)
        if d is None:
            break
        for c in d["data"]:
            txt = c.get("oracle_text") or ""
            if "card_faces" in c:
                txt = (txt + "\n" + "\n".join(
                    f.get("oracle_text", "") for f in c["card_faces"])).strip()
            out.append({"name": c["name"], "text": txt})
        url = d.get("next_page")
        time.sleep(DELAY)
    json.dump(out, open(CACHE, "w", encoding="utf-8"))
    return out


# Ordered most-specific-first; a clause lands in the first arm that matches, so
# the tiers partition the clause set. `(tier, bucket, regex)`.
BUCKETS = [
    # --- Tier 6 first: these read like other tiers and are not game events ----
    ("6", "announcement cap (601.2b)",
     r"^x can't be\b|can't be greater than|the amount you pay can't be"),
    ("6", "cost-modification floor (601.2f)", r"can't reduce"),
    ("6", "counter cap on the source", r"can't cause the total number"),
    ("6", "draft / outside the game", r"can't draft|booster pack"),
    # --- Tier 5: a query about ability to act, not a restriction (CR 101.3) --
    ("5", "\"if you can't\" (101.3 query)",
     r"^if (you|they|that player|a player|the player|an opponent|each|those)\b.{0,60}can't\b"
     r"|(who|player) can't\b.{0,40}(discard|lose|sacrific|draw|,)"
     r"|^if .{0,40}\bcan't,|unless (you|they) can't"),
    # --- Tier 4: CR 113.11, a layer-system rule ------------------------------
    ("4", "can't have/gain an ability (113.11)",
     r"can't have or gain|can't have (flying|first strike|double strike|deathtouch|hexproof"
     r"|shroud|indestructible|lifelink|menace|protection|reach|trample|vigilance|ward|haste)"),
    # --- Tier 3: withholds a replacement/prevention effect, not an event -----
    ("3", "can't be regenerated (701.19c)", r"can't be regenerated"),
    ("3", "damage can't be prevented (615.12)", r"can't be prevented"),
    # --- Tier 1: choice-time restrictions (CR 613.11 rules modification) -----
    ("1a", "can't be blocked (509.1b)", r"can't be blocked"),
    ("1a", "can't block (509.1b)", r"can't block"),
    ("1a", "can't attack (506.3/508.1a)", r"can't attack|can't be declared as an attacker"),
    ("1b", "can't cast / can't play (601.2)",
     r"can't cast|can't be cast|can't play|can't be played|can't cycle|can't venture"),
    ("1c", "can't activate (602.5)", r"can't activate|can't be activated"),
    ("1d", "can't be targeted (115.6)",
     r"can't be the target|can't be targeted|can't choose an? .{0,30}as this spell's target"),
    ("1e", "can't pay a cost (614.17b)",
     r"can't pay|can't be paid|can't spend|mana can't be spent|can't sacrifice (those|a |any)"),
    # --- Tier 2: CR 614.17 proper -- an engine-proposed event can't happen ---
    ("2", "can't be destroyed (701.8a)", r"can't be destroyed"),
    ("2", "can't enter the battlefield (614.17d)", r"can't enter"),
    ("2", "can't be countered / copied (701.5)", r"can't be countered|can't be copied"),
    ("2", "source-scoped event block (Sigarda)",
     r"can't cause (you|their controller|them|its controller) to (sacrifice|discard|search)"),
    ("2", "can't change zones",
     r"can't be sacrificed|can't be exiled|can't leave|can't be returned|can't be put into"
     r"|can't be shuffled|can't be milled|can't be discarded|can't move|can't be beamed"),
    ("2", "can't tap / untap (701.20/701.26)",
     r"can't be tapped|can't be untapped|can't untap|can't tap|can't become untapped"),
    ("2", "counters can't be put / removed (122)",
     r"counters? can't be put|can't have (\S+ )?counters|can't get (poison )?counters"
     r"|counters? can't be removed|can't be put on|can't have more than \S+ \S+ counters"),
    ("2", "life total can't change (119)",
     r"can't gain life|can't lose life|life total can't change|can't gain or lose life"),
    ("2", "can't win / lose the game (104)", r"can't win the game|can't lose the game"),
    ("2", "can't draw (121)", r"can't draw"),
    ("2", "can't search (701.19)", r"can't search"),
    ("2", "can't be dealt damage (119.3)", r"can't be dealt"),
    ("2", "can't transform / turn face up / phase (701)",
     r"can't transform|can't be turned face up|can't phase|can't become"),
    ("2", "can't be attached (701.3)",
     r"can't be enchanted|can't be equipped|can't be attached|can't attach|can't be attacked"),
]

TIER_NAMES = {
    "1a": "1 - choice-time: combat declaration",
    "1b": "1 - choice-time: casting and playing",
    "1c": "1 - choice-time: activating abilities",
    "1d": "1 - choice-time: targeting",
    "1e": "1 - choice-time: paying costs",
    "2":  "2 - event-time (CR 614.17), the RB shape",
    "3":  "3 - withholds a replacement effect",
    "4":  "4 - ability existence (CR 113.11)",
    "5":  "5 - a query, not a restriction (CR 101.3)",
    "6":  "6 - not a game event",
    "?":  "? - unclassified, read with --residual",
}

TIER_ORDER = ["1a", "1b", "1c", "1d", "1e", "2", "3", "4", "5", "6", "?"]


def clauses(cards):
    """Every sentence containing "can't", paired with its card."""
    for c in cards:
        for sent in re.split(r"(?<=[.)])\s+|\n", c["text"]):
            s = " ".join(sent.split())
            if "can't" in s.lower():
                yield c["name"], s


def classify(s):
    low = s.lower()
    for tier, bucket, pat in BUCKETS:
        if re.search(pat, low):
            return tier, bucket
    return "?", "unclassified"


def main():
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--residual", action="store_true", help="print unclassified clauses")
    args = ap.parse_args()

    cards = fetch()
    cl = list(clauses(cards))

    by_bucket, bucket_cards = Counter(), defaultdict(set)
    by_tier, tier_cards = Counter(), defaultdict(set)
    residual = []
    for name, s in cl:
        tier, bucket = classify(s)
        by_bucket[(tier, bucket)] += 1
        bucket_cards[(tier, bucket)].add(name)
        by_tier[tier] += 1
        tier_cards[tier].add(name)
        if tier == "?":
            residual.append((name, s))

    print("query:   %s" % QUERY)
    print("cards:   %d      clauses containing \"can't\": %d\n" % (len(cards), len(cl)))

    for tier in TIER_ORDER:
        if not by_tier[tier]:
            continue
        print("TIER %-44s clauses %5d   cards %5d"
              % (TIER_NAMES[tier], by_tier[tier], len(tier_cards[tier])))
        for (t, b), n in sorted(by_bucket.items(), key=lambda kv: -kv[1]):
            if t == tier:
                print("       %-44s      %5d         %5d"
                      % (b, n, len(bucket_cards[(t, b)])))
        print()

    if args.residual:
        print("--- unclassified (%d) ---" % len(residual))
        for name, s in residual:
            print("  - %s: %s" % (name, s[:150]))


if __name__ == "__main__":
    main()
