"""Scryfall census behind `plans/copy-effects-architecture.md` §2.

    python plans/references/copy-census.py            # the mechanism + population tables
    python plans/references/copy-census.py --residual # clauses no bucket caught
    python plans/references/copy-census.py --rules    # just the label audit
    python plans/references/copy-census.py --atoms    # the spec-corpus inventory
    python plans/references/copy-census.py --decompose # the derived numbers §2.4/§7 cite

Sibling of `cant-census.py`, and written for the same reason: the population
sizes decide how much machinery each mechanism earns, and a number
reconstructed from prose is a number nobody re-checks.

WHY THIS ONE HAS TWO TABLES
---------------------------
"Can't" is one word, so one query and a clause classifier covered it. Copying
is not: half the population never prints the word "copy". A transforming
double-faced card is a copiable-values question (CR 712.2/613.2a) and says
"transform"; a morph creature is one (CR 708.2) and says "face down"; a mutate
creature is one (CR 729.2a) and says "mutate". So:

  * the MECHANISM table classifies *clauses* containing copy/copies/copied by
    which engine mechanism would have to produce them -- the `cant-census`
    shape, and it is the only table where the buckets partition;
  * the POPULATION table counts *cards* by layout and keyword, one Scryfall
    query each, for the faces-and-states half that the word search cannot see.

The two overlap by construction (a Clone token of a DFC is in both) and are
never summed. Card counts, not clause counts, are the unit of the second table.

THE LABEL AUDIT IS NOT OPTIONAL
-------------------------------
`rb-review.md` theme J found `cant-census.py` shipping "can't search (701.19)"
where 701.19 is Regenerate -- a label wrong by one rule, in a file whose whole
job is to be citable. Every rule number in a bucket label here is checked
against `MTG-Rules/versions/tmnt.txt` at startup and the CR's own first line is
printed beside it (`--rules`). A mislabelled bucket now fails loudly instead of
being believed.

WHAT THE NUMBERS ARE NOT
------------------------
Not a work estimate. Tier A (token copies) is the largest clause bucket and the
cheapest mechanism; Tier C (continuous copy) is ~1% of the clauses and is the
one that needs a Layer 1 row, a dependency story and CR 613.8. §2.5 of the
design doc is where size is converted to work, and it disagrees with this
table's ordering on purpose.

WHEN TO DELETE THIS FILE
------------------------
When the copy mechanisms have shipped and stopped changing, so §2 has no
remaining customer -- or when these numbers are stale enough that a reader would
trust them over a fresh look, which a comment cannot prevent.

Results are cached in this directory (gitignored). Scryfall asks for a courteous
request rate; the delay and backoff below are deliberate -- do not tighten them.
"""
import argparse
import json
import os
import re
import sqlite3
import time
import urllib.error
import urllib.parse
import urllib.request
from collections import Counter, defaultdict

UA = "mtgsim-research/1.0 (contact: maiercluke@gmail.com)"
HERE = os.path.dirname(os.path.abspath(__file__))
CACHE = os.path.join(HERE, ".census-copy.json")
COUNTS_CACHE = os.path.join(HERE, ".census-copy-counts.json")
CR = os.path.join(HERE, "..", "..", "MTG-Rules", "versions", "tmnt.txt")
SPECDB = os.path.join(HERE, "..", "atomic-tests", "spec.sqlite")
QUERY = "(o:copy or o:copies or o:copied) -is:funny"
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
            # 503 as well as 429: Scryfall sheds load with a 503 when a burst
            # arrives faster than it likes, and the census is a burst by nature.
            if e.code == 429 or 500 <= e.code < 600:
                time.sleep(8 * (attempt + 1))
                continue
            raise
    raise RuntimeError("rate limited out: " + url)


def fetch():
    """Every card whose oracle text says copy/copies/copied."""
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
            out.append({"name": c["name"], "text": txt, "layout": c.get("layout", "")})
        url = d.get("next_page")
        time.sleep(DELAY)
    json.dump(out, open(CACHE, "w", encoding="utf-8"))
    return out


def count(q):
    """`total_cards` for one query -- one request, no pagination."""
    cache = json.load(open(COUNTS_CACHE, encoding="utf-8")) if os.path.exists(COUNTS_CACHE) else {}
    if q in cache:
        return cache[q]
    url = "https://api.scryfall.com/cards/search?q=%s&unique=cards" % urllib.parse.quote(q)
    d = _get(url)
    n = 0 if d is None else d.get("total_cards", 0)
    cache[q] = n
    json.dump(cache, open(COUNTS_CACHE, "w", encoding="utf-8"))
    time.sleep(DELAY)
    return n


# --------------------------------------------------------------------------
# The mechanism buckets. Ordered most-specific-first; a clause lands in the
# first arm that matches, so the tiers partition the clause set.
# `(tier, bucket, cr, regex)` -- `cr` is audited against the CR at startup.
# --------------------------------------------------------------------------
BUCKETS = [
    # --- Tier F first: the rules READ a copy here, they do not make one ------
    ("F", "can't be copied", "707.1",
     r"can't be copied|copy of .{0,30}can't"),
    ("F", "refers to \"the copied\" object", "707.11",
     r"the copied (permanent|creature|card|spell|object)|that (permanent|creature) is a copy"),
    # --- Tier B before A and D: "enters as a copy" is the narrowest phrase ---
    ("B", "enters as a copy (707.5/616.1c)", "707.5",
     r"enters? .{0,45}as (a )?cop(y|ies)|enters? .{0,45}that's a copy"
     r"|as a copy of (any|target|another|a |the )|you may have .{0,45}enter"),
    # --- Tier A: a token that is a copy -- the largest bucket ----------------
    ("A", "create a token copy (707.1/111.1)", "707.1",
     r"creates? .{0,80}token.{0,60}cop|token that's a copy|tokens that are copies"
     r"|cop(y|ies) of (it|that|those|them|target|each|up to).{0,80}token"),
    ("A", "the copy becomes a token (707.10f)", "707.10f",
     r"the cop(y|ies) become(s)? (a )?token"),
    # --- Tier D: CR 707.10, a copy of a spell or ability on the stack --------
    ("D", "copy a spell (707.10)", "707.10",
     r"cop(y|ies) (target |that |the |each |a |it|any |up to )?.{0,50}(spell|instant|sorcery)"
     r"|spell .{0,30}is copied|copy it\b|copies? of the spell|cop(y|ies) it\b"),
    ("D", "copy an ability (707.10)", "707.10",
     r"cop(y|ies) .{0,45}abilit|abilit(y|ies) .{0,20}is copied|copy of that abilit"),
    ("D", "retarget the copy (707.10c/d/e)", "707.10c",
     r"choose (a |any )?new targets? for (the|that|their|its|each|the additional) cop"
     r"|(each|the) cop(y|ies) targets"),
    ("D", "an exception to the copy (707.9a-c)", "707.9a",
     r"the cop(y|ies) (gains|has|isn't|is|costs|enters)"),
    # --- Tier E: CR 707.12-14 -- copy a CARD, then cast it -------------------
    ("E", "cast a copy (707.12)", "707.12",
     r"cast(s)? .{0,35}cop(y|ies)|copy (of it |of that card )?without paying"),
    ("E", "copy a card in exile / a graveyard (707.12)", "707.12",
     r"cop(y|ies) (that|the|those|each|a|any|up to|them)\b.{0,60}\bcards?\b"
     r"|cop(y|ies) the exiled card|copy them\b|cop(y|ies) those exiled cards"
     r"|cop(y|ies) that card"),
    ("E", "copy a card named N, from outside the game (707.13/707.14)", "707.13",
     r"cop(y|ies) of the card with the (chosen|noted) name"),
    # --- Tier C: CR 707.4 / 707.2c -- a permanent BECOMES a copy, continuous -
    ("C", "becomes a copy (707.4)", "707.4",
     r"becomes? a copy|become copies|is a copy of"),
]

TIER_NAMES = {
    "A": "A - token copy            (Primitive::CreateToken + copiable values)",
    "B": "B - enters as a copy      (a CR 616.1c replacement -- ReplacementClass::CopyOnEnter)",
    "C": "C - becomes a copy        (a Layer 1a continuous effect)",
    "D": "D - spell / ability copy  (a stack object, no layer involvement)",
    "E": "E - cast a copy           (CR 707.12 -- create in zone, then cast)",
    "F": "F - copy as subject       (the rules read a copy; nothing is produced)",
    "?": "? - unclassified, read with --residual",
}
TIER_ORDER = ["A", "B", "C", "D", "E", "F", "?"]

# --------------------------------------------------------------------------
# The population table. Card counts by layout / keyword -- the half of the
# corpus that never prints the word "copy". `(label, cr, query)`.
# --------------------------------------------------------------------------
POPULATIONS = [
    ("nonmodal DFC (transform)", "712.2", "layout:transform -is:funny"),
    ("modal DFC", "712.3", "layout:modal_dfc -is:funny"),
    ("meld", "712.4", "layout:meld -is:funny"),
    ("flip card", "710.1", "layout:flip -is:funny"),
    ("face-down producers (morph et al.)", "708.2a",
     "(keyword:morph or keyword:megamorph or keyword:manifest or keyword:disguise "
     "or keyword:cloak) -is:funny"),
    ("mutate (merges with a permanent)", "729.1", "keyword:mutate -is:funny"),
    ("says \"transform\"", "701.27", "o:transform -is:funny"),
    ("says \"face down\"", "708.2", "o:\"face down\" -is:funny"),
]

# Derived numbers §2.4 / §7 lean on. `(label, query)` -- printed, never counted
# by hand. Kept separate from POPULATIONS because these are intersections.
DECOMPOSE = [
    ("Tier B population, printed", "(o:\"as a copy\" or o:\"that's a copy\") -is:funny"),
    ("  ... of which are Commander-legal", "(o:\"as a copy\" or o:\"that's a copy\") f:commander"),
    ("Tier B with a 707.9b exception (\"except\")",
     "(o:\"as a copy\" or o:\"that's a copy\") o:except -is:funny"),
    ("Tier C population, printed", "o:\"becomes a copy\" -is:funny"),
    ("  ... of which are Commander-legal", "o:\"becomes a copy\" f:commander"),
    ("Tier D population, printed", "(o:\"copy target spell\" or o:\"copy that spell\") -is:funny"),
    ("  ... of which are Commander-legal",
     "(o:\"copy target spell\" or o:\"copy that spell\") f:commander"),
    ("token copies, printed", "o:\"token that's a copy\" -is:funny"),
    ("  ... of which are Commander-legal", "o:\"token that's a copy\" f:commander"),
    # `is:dfc` is NOT the rules-relevant DFC population and the difference is
    # 4.6x. Decomposed here because `codebase-state.md`'s 2026-08-27 audit cited
    # the headline number and it is dominated by art cards, which are not
    # playable Magic cards and have no copiable values to model.
    ("DFC headline (is:dfc) -- do NOT cite this", "is:dfc -is:funny"),
    ("  of which layout:art_series (not playable)", "layout:art_series -is:funny"),
    ("  of which layout:double_faced_token", "layout:double_faced_token -is:funny"),
    ("  of which layout:reversible_card", "layout:reversible_card -is:funny"),
    ("  the rules-relevant remainder: transform", "layout:transform -is:funny"),
    ("  the rules-relevant remainder: MDFC", "layout:modal_dfc -is:funny"),
    ("  the rules-relevant remainder: meld", "layout:meld -is:funny"),
    ("  ... of which are Commander-legal MDFC", "layout:modal_dfc f:commander"),
    ("  ... of which are Commander-legal transform", "layout:transform f:commander"),
    ("meld, Commander-legal", "layout:meld f:commander"),
    ("a DFC that can BE a commander: transform", "is:commander layout:transform"),
    ("a DFC that can BE a commander: MDFC", "is:commander layout:modal_dfc"),
    ("face-down producers, Commander-legal",
     "(keyword:morph or keyword:megamorph or keyword:manifest or keyword:disguise "
     "or keyword:cloak) f:commander"),
    ("mutate, Commander-legal", "keyword:mutate f:commander"),
    ("can be a commander AND is a DFC", "is:commander is:dfc"),
]

# The spec-corpus slice this design owes tests for. `(prefix, what)`.
ATOM_SLICE = [
    ("707", "Copying Objects"),
    ("712", "Double-Faced Cards"),
    ("708", "Face-Down Spells and Permanents"),
    ("613.2", "Layer 1 (copiable effects / face-down)"),
    ("729", "Merging with Permanents"),
    ("710", "Flip Cards"),
]


def audit_labels():
    """Every rule number in a bucket or population label, against the CR.

    theme J's failure mode, made structural: a label whose rule number does not
    resolve, or resolves to a rule about something else, is visible here before
    it reaches the doc.
    """
    if not os.path.exists(CR):
        return [("(CR not found at %s -- labels unaudited)" % CR, None)]
    text = open(CR, encoding="utf-8", errors="replace").read()
    out = []
    seen = set()
    for cr in [b[2] for b in BUCKETS] + [p[1] for p in POPULATIONS]:
        if cr in seen:
            continue
        seen.add(cr)
        m = re.search(r"^%s[. ].*$" % re.escape(cr), text, re.M)
        out.append((cr, m.group(0)[:96] if m else None))
    return out


def clauses(cards):
    """Every sentence mentioning copy/copies/copied, paired with its card."""
    for c in cards:
        for sent in re.split(r"(?<=[.)])\s+|\n", c["text"]):
            s = " ".join(sent.split())
            if re.search(r"\bcop(y|ies|ied)\b", s, re.I):
                yield c["name"], s


def classify(s):
    low = s.lower()
    for tier, bucket, _cr, pat in BUCKETS:
        if re.search(pat, low):
            return tier, bucket
    return "?", "unclassified"


def show_atoms():
    if not os.path.exists(SPECDB):
        print("  spec.sqlite not built -- run `python plans/specdb.py build`")
        return
    db = sqlite3.connect(SPECDB)
    print("ATOMS -- the corpus slice this design owes tests for")
    print("  (uncovered = no COVERS annotation anywhere in the test suite)\n")
    print("  %-8s %-42s %6s %6s   phase tags" % ("CR", "section", "atoms", "uncov"))
    tot = unc_tot = 0
    for pref, what in ATOM_SLICE:
        rows = db.execute(
            "select a.id, a.phase, (select count(*) from coverage c where c.atom_id = a.id) "
            "from atoms a where a.rule_num like ?||'%'", (pref,)).fetchall()
        if not rows:
            continue
        unc = sum(1 for r in rows if r[2] == 0)
        tot += len(rows)
        unc_tot += unc
        phases = ", ".join("%s x%d" % (p or "?", n)
                           for p, n in Counter(r[1] for r in rows).most_common())
        print("  %-8s %-42s %6d %6d   %s" % (pref, what, len(rows), unc, phases))
    print("  %-8s %-42s %6d %6d" % ("", "TOTAL", tot, unc_tot))


def main():
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--residual", action="store_true", help="print unclassified clauses")
    ap.add_argument("--rules", action="store_true", help="only the bucket-label audit")
    ap.add_argument("--atoms", action="store_true", help="only the spec-corpus inventory")
    ap.add_argument("--decompose", action="store_true",
                    help="only the derived counts the doc cites")
    args = ap.parse_args()

    # The label audit runs first and always -- a wrong rule number in a label
    # invalidates the table under it, so it is never a separate opt-in step.
    audit = audit_labels()
    bad = [cr for cr, line in audit if line is None]
    if args.rules or bad:
        print("LABEL AUDIT -- every rule number in a bucket label, against tmnt.txt\n")
        for cr, line in audit:
            print("  %-9s %s" % (cr, line if line else "*** NOT FOUND IN THE CR ***"))
        print()
    if bad:
        print("  *** %d label(s) cite a rule that is not in the CR: %s\n" % (len(bad), bad))
    if args.rules:
        return
    if args.atoms:
        show_atoms()
        return
    if args.decompose:
        print("DERIVED COUNTS -- cards, printed rather than hand-counted\n")
        for label, q in DECOMPOSE:
            print("  %-46s %6d   %s" % (label, count(q), q))
        return

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

    print("MECHANISM TABLE -- which engine mechanism must produce this clause")
    print("query:   %s" % QUERY)
    print("cards:   %d      clauses mentioning copy/copies/copied: %d\n" % (len(cards), len(cl)))
    for tier in TIER_ORDER:
        if not by_tier[tier]:
            continue
        print("TIER %-62s clauses %5d   cards %5d"
              % (TIER_NAMES[tier], by_tier[tier], len(tier_cards[tier])))
        for (t, b), n in sorted(by_bucket.items(), key=lambda kv: -kv[1]):
            if t == tier:
                print("       %-62s      %5d         %5d"
                      % (b, n, len(bucket_cards[(t, b)])))
        print()

    print("POPULATION TABLE -- cards, by layout and keyword (the half that never")
    print("says \"copy\"). Overlaps the table above; never summed with it.\n")
    print("  %-40s %-8s %7s %10s" % ("population", "CR", "cards", "commander"))
    for label, cr, q in POPULATIONS:
        cmdr = count(q.replace("-is:funny", "f:commander"))
        print("  %-40s %-8s %7d %10d" % (label, cr, count(q), cmdr))
    print()
    show_atoms()

    if args.residual:
        print("\n--- unclassified (%d) ---" % len(residual))
        for name, s in residual:
            print("  - %s: %s" % (name, s[:150]))


if __name__ == "__main__":
    main()
