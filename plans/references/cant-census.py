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
     # The `(?!\S+ (do|don't|does|doesn't),)` guard is load-bearing. "If you do,
     # target creature can't block this turn" is a conditional *effect* whose
     # payload is a restriction; CR 101.3's query is about whether an
     # instruction can be carried out, and there the "can't" belongs to the
     # `if` clause itself. Without the guard this arm swallowed 21 such
     # clauses, 19 of them Tier 1a. Gilded Drake's "If you don't or can't make
     # an exchange" is the real thing and survives it -- no comma after "don't".
     r"^if (?!\S+ (do|don't|does|doesn't),)"
     r"(you|they|that player|a player|the player|an opponent|each|those)\b.{0,60}can't\b"
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
    # Before the Tier 2 attachment arm, which used to swallow these two on the
    # shared substring "attach"/"attack". Island Sanctuary and The Aetherspark
    # both restrict *attack declaration*, which is CR 508.1a, not CR 701.3.
    ("1a", "can't be attacked (508.1a)", r"can't be attacked"),
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
    ("2", "can't search (701.23)", r"can't search"),
    ("2", "can't be dealt damage (119.3)", r"can't be dealt"),
    ("2", "can't transform / turn face up / phase / gain a designation (701, 724, 730)",
     r"can't transform|can't be turned face up|can't phase|can't become"),
    ("2", "can't be attached (701.3)",
     r"can't be enchanted|can't be equipped|can't be attached|can't attach"),
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


# --- Tier 1a's internal decomposition (`--decompose`) -----------------------
#
# `cant-effects-architecture.md` §4.2 asks a different question of Tier 1a than
# the tier table does: not "where is this enforced" but "does it couple two
# creatures", because only the coupling ones need a solver. Those numbers were
# hand counts until 2026-08-30 and disagreed with §2.4's by 28 (rb-review J3);
# they are measured here so the two sections cannot drift again.
#
# First match wins, and the shapes are ordered so the coupling ones are asked
# first -- "can't attack alone" is also a per-creature sentence.
N = r"(one|two|three|four|five|\d+)"
SHAPES_1A = [
    ("cross-creature (needs a solver)",
     r"can't attack alone|can't block alone"
     r"|can't attack unless .{0,60}(other|another) creature"),
    ("per-attacker blocker count",
     r"can't be blocked by more than " + N
     + r"|can't be blocked except by " + N + r" or more"
     + r"|can't block more than " + N),
    ("per-creature predicate", r""),
]


def normalize_tail(clause):
    """What follows "can't be blocked", reduced to the filter it names.

    The point of counting these is §2.4's claim that every tail but the
    counting ones is `PermanentFilter`-expressible, so the normalization drops
    what a filter would not carry: reminder text, durations, and the
    singular/plural of "creature".
    """
    m = re.search(r"can't be blocked(.*)$", clause)
    if not m:
        return None
    t = m.group(1)
    t = re.sub(r"\(.*?\)", " ", t)
    t = re.sub(r"\b(this turn|this combat|next turn|during .{0,30})\b", " ", t)
    t = re.sub(r"\bcreatures\b", "creature", t)
    t = re.sub(r"[^a-z ]", " ", t)
    return " ".join(t.split()).strip() or "(no tail -- unblockable)"


DURATION_RE = re.compile(
    r"this turn|this combat|until end of turn|until your next turn"
    r"|until end of combat|next turn")
UNLESS_RE = re.compile(r"\bunless\b")


def decompose(cl):
    """Print §4.2's Tier-1a decomposition and §2.4's three shapes."""
    cl_by_tier = defaultdict(list)
    for name, s in cl:
        cl_by_tier[classify(s)[0]].append((name, s.lower()))
    t1a = cl_by_tier["1a"]
    counts, cards_in = Counter(), defaultdict(set)
    for name, s in t1a:
        for shape, pat in SHAPES_1A:
            if not pat or re.search(pat, s):
                counts[shape] += 1
                cards_in[shape].add(name)
                break

    print("TIER 1a decomposition -- what actually couples two creatures\n")
    for shape, _ in reversed(SHAPES_1A):
        print("       %-38s      %5d         %5d"
              % (shape, counts[shape], len(cards_in[shape])))
    print("       %-38s      %5d\n" % ("total (= tier 1a)", sum(counts.values())))
    print("  Global caps (Silent Arbiter, Dueling Grounds) are NOT in this sum:")
    print('  they print "no more than one creature can attack", never "can\'t",')
    print("  so they are outside the census query entirely (sec. 2.2).\n")

    blocked = [(n, s) for n, s in t1a if "can't be blocked" in s]
    tails = Counter(normalize_tail(s) for _, s in blocked)
    print('  "can\'t be blocked" clauses %5d   distinct tails %5d'
          % (len(blocked), len(tails)))
    print("  most common tails:")
    for tail, n in tails.most_common(8):
        print("      %5d  %s" % (n, tail[:66]))

    # The other two shapes in sec. 2.4: what a restriction needs beyond a
    # predicate. A duration means a registry row (`Primitive::Restrict`); an
    # "unless" means `Effect::Conditional`, which is Phase 6.
    print("\n  What each tier needs beyond a predicate")
    print("       %-14s %8s %14s %10s" % ("tier", "clauses", "turn-scoped", "unless"))
    for tier in ["1a", "1b", "1c", "1d", "1e", "2", "3"]:
        v = [s for _, s in cl_by_tier.get(tier, [])]
        if not v:
            continue
        dur = sum(1 for s in v if DURATION_RE.search(s))
        unl = sum(1 for s in v if UNLESS_RE.search(s))
        print("       %-14s %8d %8d (%2d%%) %10d"
              % (tier, len(v), dur, 100 * dur // len(v), unl))


def main():
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--residual", action="store_true", help="print unclassified clauses")
    ap.add_argument("--decompose", action="store_true",
                    help="Tier 1a by constraint shape (architecture doc §4.2)")
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

    if args.decompose:
        decompose(cl)
        print()

    if args.residual:
        print("--- unclassified (%d) ---" % len(residual))
        for name, s in residual:
            print("  - %s: %s" % (name, s[:150]))


if __name__ == "__main__":
    main()
