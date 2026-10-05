#!/usr/bin/env python3
"""The census of how cards are cast, activated and paid for -- every table in
`plans/references/cast-census.md`.

    python plans/references/cast-census.py               # print the tables
    python plans/references/cast-census.py --write       # splice them into cast-census.md between its markers
    python plans/references/cast-census.py --residual    # cost pieces and abilities no bucket caught, for a human read
    python plans/references/cast-census.py --bucket ID   # every clause in one bucket, to read it
    python plans/references/cast-census.py --calibrate   # the registered cards, and every family they use
    python plans/references/cast-census.py --names Q     # the first page of names behind a Scryfall query
    python plans/references/cast-census.py --no-fetch    # caches only; a query the cache lacks is an error
    python plans/references/cast-census.py --refresh     # drop the caches first; the corpus alone takes ~7 minutes

WHY THIS EXISTS
---------------
`cr-coverage-audit.md` asks whether the plan can express the CR. Nothing asked
the same of the cards v1 needs, every Commander-legal card, until SU-7's review
found four families shaped after today's engine and two with no owner
(`codebase-state.md` item 212). The replacement, trigger and "can't" censuses
each found what reading the CR missed; this one does it for casting,
activating and paying, so "did we miss something" is a table to read.

Each family maps to the engine surface that would have to express it and a
status: **built** (the engine plays it), **owned** (a document names it and the
route slots it), **unowned**, or **excluded** (on the record as out of v1). The
statuses and owners are a reading of the tree and the docs by hand, written in
the tables below; the counts are not.

THE UNITS
---------
Four tables partition a unit, most-specific bucket first, so a column sums to
the unit's total:
  * a COST PIECE -- one comma-separated piece of an activated ability's cost,
    of an "As an additional cost to cast this spell" clause, or of a
    "rather than pay this spell's mana cost" clause;
  * an ACTIVATED ABILITY -- a "[cost]: [effect]" line, printed or quoted in a
    grant, by who may activate it, when, from which zone, and whether it is a
    mana ability;
  * a MANA ABILITY -- what it produces, which is what the affordability check
    (`codebase-state.md` item 162) has to read;
  * a CLAUSE -- one sentence naming a casting or playing permission.
Keyword families and mana symbols are counted by card. The question kinds are
read from `ui/choice_types.rs` and `ui/why.rs` and asserted against the enum,
so a new `ChoiceKind` fails this script until its row is written.

WHAT THE NUMBERS ARE NOT
------------------------
Not a work estimate: a family's size says how many cards wait on it, not how
much code it costs. The buckets are regexes over Scryfall's oracle text with
reminder text removed and the card's own name written `~`, so `--residual`
always needs a human read. Every Scryfall query is printed beside its number;
the corpus is `BASE` below, and a family's own query adds to it.

WHEN TO DELETE THIS FILE
------------------------
When the families it counts are built or excluded and the tables have no
customer -- or when the counts are stale enough that a reader would trust a
fresh look over them, which a comment cannot prevent.

Results are cached in this directory (gitignored, `.census-*.json`). Scryfall
asks for a courteous request rate and bans a burst; the delay is deliberate.
"""
import argparse
import datetime as _dt
import json
import os
import re
import subprocess
import sys
import time
import urllib.parse
from collections import Counter, defaultdict

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
DOC = os.path.join(HERE, "cast-census.md")
CACHE = os.path.join(HERE, ".census-cast.json")
COUNTS = os.path.join(HERE, ".census-cast-counts.json")
REGISTERED_CACHE = os.path.join(HERE, ".census-cast-registered.json")
CR = os.path.join(ROOT, "MTG-Rules", "versions", "tmnt.txt")
SRC = os.path.join(ROOT, "mtgsim", "src")
REGISTRY = os.path.join(SRC, "cards", "registry.rs")
CHOICE_TYPES = os.path.join(SRC, "ui", "choice_types.rs")
WHY = os.path.join(SRC, "ui", "why.rs")

UA = "mtgsim-research/1.0 (contact: maiercluke@gmail.com)"
BASE = "legal:commander -t:token game:paper -is:funny"
PAPER = "game:paper -is:funny"
DELAY = 0.35

_no_fetch = False

# ==========================================================================
# Scryfall
# ==========================================================================


def _get(url):
    """curl with a UA header: Scryfall 403s a bare request."""
    for attempt in range(6):
        out = subprocess.run(["curl", "-s", "-A", UA, url], capture_output=True,
                             text=True, encoding="utf-8").stdout
        try:
            d = json.loads(out)
        except json.JSONDecodeError:
            time.sleep(5 * (attempt + 1))
            continue
        if d.get("object") == "error" and d.get("status") == 429:
            time.sleep(8 * (attempt + 1))
            continue
        return d
    raise RuntimeError("rate limited out: " + url)


def _post(url, body):
    for attempt in range(6):
        out = subprocess.run(["curl", "-s", "-A", UA, "-H", "Content-Type: application/json",
                              "-X", "POST", "--data-binary", "@-", url],
                             input=json.dumps(body), capture_output=True, text=True,
                             encoding="utf-8").stdout
        try:
            d = json.loads(out)
        except json.JSONDecodeError:
            time.sleep(5 * (attempt + 1))
            continue
        if d.get("object") == "error" and d.get("status") == 429:
            time.sleep(8 * (attempt + 1))
            continue
        return d
    raise RuntimeError("rate limited out: " + url)


def _card(c):
    faces = c.get("card_faces") or []
    text = c.get("oracle_text")
    if text is None:
        text = "\n".join(f.get("oracle_text", "") for f in faces)
    mana = c.get("mana_cost")
    if mana is None:
        mana = " // ".join(f.get("mana_cost", "") for f in faces)
    names = [c["name"]] + [f["name"] for f in faces if f.get("name")]
    return {"name": c["name"], "names": names, "text": text, "mana": mana,
            "type": c.get("type_line", ""), "kw": c.get("keywords", []),
            "layout": c.get("layout", ""), "produced": c.get("produced_mana", [])}


def corpus():
    """Every card `BASE` matches, cached whole: ~184 pages, ~7 minutes."""
    if os.path.exists(CACHE):
        return json.load(open(CACHE, encoding="utf-8"))
    if _no_fetch:
        raise SystemExit("--no-fetch, and there is no corpus cache")
    url = "https://api.scryfall.com/cards/search?q=%s&unique=cards&order=name" % urllib.parse.quote(BASE)
    cards = []
    while url:
        d = _get(url)
        if d.get("object") == "error":
            raise RuntimeError(d.get("details"))
        cards += [_card(c) for c in d["data"]]
        url = d.get("next_page")
        time.sleep(DELAY)
    out = {"query": BASE, "fetched": _dt.date.today().isoformat(), "cards": cards}
    json.dump(out, open(CACHE, "w", encoding="utf-8"))
    return out


_counts = None


def count(q):
    """`total_cards` for `q`, cached by the query string."""
    global _counts
    if _counts is None:
        _counts = json.load(open(COUNTS, encoding="utf-8")) if os.path.exists(COUNTS) else {}
    if q in _counts:
        return _counts[q]["total"]
    if _no_fetch:
        raise SystemExit("--no-fetch, and the count cache lacks: " + q)
    d = _get("https://api.scryfall.com/cards/search?q=%s&unique=cards" % urllib.parse.quote(q))
    if d.get("object") == "error":
        if d.get("code") != "not_found":
            raise RuntimeError("%s -> %s" % (q, d.get("details")))
        n = 0
    else:
        n = d["total_cards"]
    _counts[q] = {"total": n, "fetched": _dt.date.today().isoformat()}
    json.dump(_counts, open(COUNTS, "w", encoding="utf-8"), indent=1, sort_keys=True)
    time.sleep(DELAY)
    return n


def names(q, limit=60):
    d = _get("https://api.scryfall.com/cards/search?q=%s&unique=cards" % urllib.parse.quote(q))
    if d.get("object") == "error":
        print(d.get("details"))
        return
    print("total:", d.get("total_cards"))
    for c in d.get("data", [])[:limit]:
        t = c.get("oracle_text") or " | ".join(f.get("oracle_text", "") for f in c.get("card_faces", []))
        print("  - %s :: %s" % (c["name"], t.replace("\n", " | ")[:220]))


# ==========================================================================
# Owners -- the pointers a status cell names. One place, so a family's owner
# is changed once.
# ==========================================================================

O = {
    "built": "",
    "CP-1": "`cost-architecture.md` CP-1 (`roadmap-v2.md` B1, any time)",
    "B9": "`codebase-state.md` item 33, mana provenance (`roadmap-v2.md` B9, before C)",
    "162": "`codebase-state.md` item 162's design, the next PR",
    "RS-2": "`cant-effects-architecture.md` RS-2 (beside A, after A6c)",
    "B8": "`backlog.md` §2.24, \"as though\" (`roadmap-v2.md` B8, before C)",
    "CV-5": "`copy-effects-architecture.md` CV-5 (`roadmap-v2.md` B6)",
    "CV-6": "`copy-effects-architecture.md` CV-6, face-down (`roadmap-v2.md` B6)",
    "CV-7": "`copy-effects-architecture.md` CV-7, merging (`roadmap-v2.md` B6, before C)",
    "CV-4": "`copy-effects-architecture.md` CV-4, copies of spells (`roadmap-v2.md` B6)",
    "2.41": "`backlog.md` §2.41, the copy track (`roadmap-v2.md` B6)",
    "A4l": "`roadmap-v2.md` A4l, CR 601.2d (before C)",
    "B2": "`roadmap-v2.md` B2, `GameConfig::commander()`",
    "TR": "`triggers-architecture.md` TR-3 to TR-7 (`roadmap-v2.md` A6d)",
    "item30": "`codebase-state.md` item 30, what paid (`ManaSpent` by type)",
    "item81": "`codebase-state.md` item 81, spree",
    "C": "`roadmap-v2.md` C, Phase 8 breadth: a card is a normal diff over surfaces named in the row",
    "PM": "`permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2)",
    "CP-2": "`cost-architecture.md` CP-2 (`roadmap-v2.md` B1, before C)",
    "C-arm": "`cost-architecture.md` §6: one `Cost` arm with its first card, in C",
    "C-change": "`cost-architecture.md` §6: one `CostChange` arm with its first card, in C",
    "SU-8": "`setup-architecture.md` §8, SU-8",
    # Found unowned. Each is assigned in the census doc's §1; until it is,
    # the cell names the entry that holds it.
    "2.3": "`backlog.md` §2.3, owner none",
    "2.8": "`backlog.md` §2.8, owner none",
    "2.11": "`backlog.md` §2.11, owner none",
    "2.19": "`backlog.md` §2.19, owner none",
    "2.7": "`backlog.md` §2.7, owner none",
    "2.2": "`backlog.md` §2.2, owner none",
    "2.5": "`backlog.md` §2.5, owner none",
    "2.15": "`backlog.md` §2.15, owner none",
    "2.31": "`backlog.md` §2.31, owner none",
    "item2": "`codebase-state.md` \"Before card breadth\" item 2, with the first such card",
    "costs": "no document: `Cost` has no arm",
    "unchecked": "no document: the arm exists, `CannotPay::Unchecked`",
    "none": "no document",
    "excluded": "`roadmap-v2.md` §7, the excluded tail",
}

# ==========================================================================
# Text
# ==========================================================================

SELF_NOUN = (r"this (card|creature|artifact|enchantment|land|permanent|planeswalker|equipment"
             r"|vehicle|aura|spell|token|saga|battle|class|case|room|attraction|contraption)")


def strip_reminder(s):
    return re.sub(r"\s*\([^()]*\)", "", s)


def selfify(card, s):
    """The card's own name, its faces' names and their short forms become `~`."""
    out = s
    every = card.get("names") or [card["name"]] + card["name"].split(" // ")
    for n in sorted(set(every), key=len, reverse=True):
        out = out.replace(n, "~")
        short = n.split(",")[0]
        if short != n and len(short) >= 4:
            out = re.sub(r"\b%s\b" % re.escape(short), "~", out)
    return re.sub(SELF_NOUN, "~", out, flags=re.I)


def lines(card):
    for line in card["text"].split("\n"):
        line = strip_reminder(line).strip()
        if line:
            yield selfify(card, line)


def sentences(card):
    for line in lines(card):
        for s in re.split(r"(?<=[.!])\s+(?=[A-Z~•\"])", line):
            s = s.strip()
            if s:
                yield s


# --------------------------------------------------------------------------
# Activated abilities
# --------------------------------------------------------------------------

# An ability word, a flavor word or a station/level threshold before an
# activated ability: "Channel — {1}{G}, Discard ~: ...", "12+ | {3}{W}, {T}: ...".
PREFIX = re.compile(r"^(?:[^:{\"]{1,70}? — |\d+\+ \| |LEVEL \d+[-+]?\d* )")
LOYALTY = re.compile(r"^[+−-]?(\d+|X)$")


def _split(body, card, granted):
    i = body.find(":")
    if i <= 0:
        return None
    cost = body[:i].strip()
    if '"' in cost or "'" in cost.replace("'s ", "") or len(cost) > 170:
        return None
    # "Choose one —" bullets and the like carry no colon-cost; a cost always
    # starts with a symbol, a loyalty number, or a capitalized verb.
    if not re.match(r"^(\{|[+−-]?(\d|X)|[A-Z~])", cost):
        return None
    if re.match(r"^(Choose|Each|When|Whenever|At |If |As |Target|You |Your |This |Creatures|Spells)", cost):
        return None
    return {"card": card["name"], "cost": cost, "effect": body[i + 1:].strip(), "granted": granted}


def activated(card):
    """Each activated ability on `card`, printed or quoted in a grant."""
    for line in lines(card):
        # A grant quotes the ability it grants; an emblem's grant nests a
        # second quote in single marks ("Mountains you control have '{T}: ...'").
        quoted = re.findall(r"\"([^\"]+)\"", line)
        quoted += [q for outer in quoted for q in re.findall(r"'(\{[^']+)'", outer)]
        for q in quoted:
            r = _split(PREFIX.sub("", q), card, True)
            if r:
                yield r
        if line.startswith('"'):
            continue
        body = PREFIX.sub("", line)
        # A ticket cost gates the ability it prefixes ("{TK}{TK} — {2}, {T}: ...").
        m = re.match(r"^((?:\{TK\})+) — ", body)
        if m:
            body = body[m.end():]
            r = _split(body, card, False)
            if r:
                r["cost"] = m.group(1) + ", " + r["cost"]
                yield r
            continue
        r = _split(body, card, False)
        if r:
            yield r


VERBS = (r"pay|exile|sacrifice|discard|return|tap|untap|reveal|remove|put|collect evidence|blight|behold"
         r"|waterbend|forage|mill|exert|unattach|roll|choose")
# "Pay {4} or sacrifice an artifact", "{T} or {U}": the payer picks one of two
# costs. An "or" inside a filter ("an artifact or creature") is not one.
CHOICE = re.compile(r"\bor (" + VERBS + r")\b|\bor \{", re.I)


def cost_pieces(cost):
    """A cost cut where a new piece starts: at a comma before a symbol or a
    verb. A choice between costs stays whole."""
    cost = cost.strip()
    if LOYALTY.match(cost) or CHOICE.search(cost):
        return [cost]
    parts = re.split(r",\s+(?:and\s+)?(?=\{|(?:" + VERBS + r")\b)", cost, flags=re.I)
    out = []
    for p in parts:
        # "Sacrifice ~ and a creature you control" is two sacrifices.
        m = re.match(r"^(sacrifice ~) and (.+)$", p.strip(), re.I)
        out += [m.group(1), "Sacrifice " + m.group(2)] if m else [p.strip()]
    return [p for p in out if p]


def spell_cost_clauses(card):
    """The cost text of a spell's mandatory additional cost and of a printed
    alternative cost, as (context, cost text)."""
    for s in sentences(card):
        m = re.match(r"^As an additional cost to cast (?:~|this spell), (.+?)\.?$", s, re.I)
        if m:
            yield "additional", m.group(1)
            continue
        m = re.match(r"^You may (.+?) rather than pay (?:~'s|this spell's) mana cost\.?$", s, re.I)
        if m:
            yield "alternative", m.group(1)


def spell_pieces(text):
    """An additional or alternative cost's pieces: "pay 1 life and exile a blue
    card". "You may" marks an optional one (CR 118.8b); "behold a Kithkin and
    exile it" is one piece, since "it" is the beheld card."""
    text = re.sub(r"^you may ", "", text.strip(), flags=re.I)
    if CHOICE.search(text):
        return [text]
    parts = re.split(r",\s+(?:and\s+)?(?=(?:" + VERBS + r")\b|\{)|\s+and\s+(?=(?:" + VERBS + r")\b(?! (?:it|them)\b))",
                     text, flags=re.I)
    return [p.strip() for p in parts if p.strip()]


# ==========================================================================
# Table A -- cost pieces, against `Cost`'s ten arms
# ==========================================================================
# `(id, family, CR, surface, status, owner, regex)` over the lowercased piece.
# Ordered most-specific-first; a piece lands in the first that matches.

N = (r"(?:a|an|one|two|three|four|five|six|seven|eight|nine|ten|eleven|twelve|thirteen|fourteen|fifteen"
     r"|twenty|another|\d+)")
NX = r"(?:x|any number of|one or more|all|half|that many|cards equal|up to)"
MANA = r"^(pay )?(\{[^}]+\})+$"

COST_BUCKETS = [
    ("count", "a cost paid any number of times", "601.2b", "601.2b's announcement cannot ask for a number",
     "unowned", "costs", r"any number of times"),
    ("choice", "a choice between two costs (\"pay {4} or sacrifice\")", "118.1", "no arm: a cost is one action",
     "unowned", "costs", r"\bor (" + VERBS + r")\b|\bor \{"),
    ("tap-self", "{T}", "107.5", "`Cost::TapSelf`", "built", "built", r"^\{t\}$"),
    ("untap-self", "{Q}", "107.6", "`Cost::UntapSelf`", "built", "built", r"^\{q\}$"),
    ("loyalty", "a loyalty symbol, [+N] [−N] [0]", "606.4", "no loyalty ability", "unowned", "2.11",
     r"^[+−-]?(\d+|x)$"),
    ("ticket", "ticket counters, {TK}", "107.17a", "no arm: a player's counters", "unowned", "costs",
     r"^(\{tk\})+$"),
    ("mana-colorless-hybrid", "mana with a {C/W}-style hybrid", "107.4e", "no `ManaSymbol` arm", "unowned", "CP-1",
     r"\{c/[wubrg]\}"),
    ("mana-phyrexian", "mana with a Phyrexian symbol", "107.4f", "`ManaSymbol::Phyrexian`, unpaid", "owned", "CP-1",
     r"^(pay )?(\{[^}]+\})*\{[wubrg](/[wubrg])?/p\}"),
    ("mana-hybrid", "mana with a hybrid or {2/W} symbol", "107.4e", "`ManaSymbol::Hybrid`, `MonoHybrid`, unpaid",
     "owned", "CP-1", r"^(pay )?(\{[^}]+\})*\{[wubrg2]/[wubrg]\}"),
    ("mana-snow", "mana with {S}", "107.4h", "`ManaSymbol::Snow`, unpaid", "owned", "B9",
     r"^(pay )?(\{[^}]+\})*\{s\}"),
    ("mana-x", "mana with {X}", "107.3a", "`Cost::Mana`; an ability's X is never asked", "unowned", "none",
     r"^(pay )?(\{[^}]+\})*\{x\}"),
    ("mana", "mana", "601.2g", "`Cost::Mana`", "built", "built", r"^(pay )?(\{(\d+|[wubrgc])\})+$"),
    ("mana-variable", "an amount of mana the board decides", "107.3", "`Cost::Mana(ManaCost)` holds symbols",
     "unowned", "costs", r"^pay \{\d\} for each|^pay .*mana cost|^pay \{x\}"),
    ("energy", "pay {E}", "107.14", "no arm: a player's counters", "unowned", "costs",
     r"^pay .*\{e\}"),
    ("life", "pay N life", "119.4", "`Cost::PayLife`", "built", "built",
     r"^pay (\d+|one|two|three|four|five|six|seven|eight|ten) life$"),
    ("life-variable", "pay X life, half your life", "119.4",
     "`Cost::PayLife(u64)` holds a number; X is announced only from a mana cost (`cast_spell`)", "unowned", "costs",
     r"^pay .*life"),
    ("sacrifice-self", "sacrifice ~", "701.21a", "`Cost::SacrificeSelf`", "built", "built", r"^sacrifice ~$"),
    ("sacrifice-variable", "sacrifice X, any number or half", "701.21a",
     "`Cost::Sacrifice(_, u32)` holds a number; X is announced only from a mana cost (`cast_spell`)",
     "unowned", "costs", r"^sacrifice " + NX),
    ("sacrifice-attached", "sacrifice the enchanted or equipped permanent", "701.21a", "no `ObjectFilter` for it",
     "unowned", "costs", r"^sacrifice (enchanted|equipped|the creature|the permanent|it\b|attached|[a-z]+$)"),
    ("sacrifice", "sacrifice N permanents", "701.21a", "`Cost::Sacrifice`", "built", "built",
     r"^sacrifice " + N),
    ("discard-self", "discard ~", "113.6m", "no arm, and the ability works in a hand", "unowned", "2.8",
     r"^discard ~"),
    ("discard-hand", "discard your hand", "701.9a", "no arm", "unowned", "costs", r"^discard your hand"),
    ("discard", "discard N cards", "701.9a", "`Cost::Discard`, unchecked", "unowned", "unchecked",
     r"^discard "),
    ("exile-self-graveyard", "exile ~ from your graveyard", "113.6j", "no arm, and the ability works in a graveyard",
     "unowned", "2.8", r"^exile ~ from your graveyard"),
    ("exile-graveyard", "exile N cards from a graveyard", "701.13", "`Cost::ExileFromGraveyard`, unchecked",
     "unowned", "unchecked", r"^exile .*(from|of) (your|a|a single) graveyard"),
    ("exile-hand", "exile cards from your hand", "701.13", "no arm", "unowned", "costs", r"^exile .*from your hand"),
    ("exile-library", "exile cards from the top of your library", "701.13", "no arm", "unowned", "costs",
     r"^exile .*of your library"),
    ("exile-self", "exile ~", "701.13", "no arm", "unowned", "costs", r"^exile ~$"),
    ("exile-permanent", "exile a permanent you control", "701.13", "no arm", "unowned", "costs",
     r"^exile .*you control"),
    ("move-exiled", "put an exiled card into a graveyard (processors)", "406.3", "no arm", "unowned", "costs",
     r"^put .*(from exile|exiled with ~) into"),
    ("remove-counters-self", "remove N counters from ~", "122.1", "`Cost::RemoveCounters`, unchecked",
     "unowned", "unchecked", r"^remove (" + N[3:-1] + r") .*counters? from ~$"),
    ("remove-counters-variable", "remove X or any number of counters", "122.1",
     "`Cost::RemoveCounters(_, u32)` holds a number; X is announced only from a mana cost (`cast_spell`)", "unowned", "costs",
     r"^remove (" + NX[3:-1] + r"|one or more) .*counters?"),
    ("remove-counters-other", "remove counters from another permanent", "122.1", "no arm: the arm names the source",
     "unowned", "costs", r"^remove .*counters? from"),
    ("put-counters-self", "put N counters on ~", "122.1", "`Cost::AddCounters`, unchecked", "unowned", "unchecked",
     r"^put .*counters? on ~$"),
    ("put-counters-other", "put counters on another permanent, blight", "701.68", "no arm: the arm names the source",
     "unowned", "costs", r"^put .*counters? on|^blight"),
    ("tap-others", "tap N untapped permanents you control", "701.26a", "no arm", "unowned", "costs",
     r"^tap "),
    ("untap-others", "untap N tapped permanents", "701.26b", "no arm", "unowned", "costs", r"^untap "),
    ("return-self", "return ~ to its owner's hand", "118.1", "no arm", "unowned", "costs", r"^return ~ to"),
    ("return-permanent", "return a permanent you control to its owner's hand", "118.1", "no arm", "unowned", "costs",
     r"^return "),
    ("behold", "behold a [quality]", "701.4a", "no arm", "unowned", "costs", r"^behold|^choose a .* you control or|^reveal .* or choose"),
    ("reveal", "reveal cards from your hand", "701.20a", "no arm", "unowned", "costs", r"^reveal "),
    ("put-library", "put cards on top or bottom of a library", "118.1", "no arm", "unowned", "costs",
     r"^put .*library"),
    ("mill", "mill N", "701.17", "no arm", "unowned", "costs", r"^mill "),
    ("exert", "exert ~", "701.43", "no arm", "unowned", "2.5", r"^exert "),
    ("collect-evidence", "collect evidence N", "701.59", "no arm", "unowned", "costs", r"^collect evidence"),
    ("forage", "forage", "701.61", "no arm", "unowned", "costs", r"^forage"),
    ("waterbend", "waterbend {N}", "701.67", "no arm", "unowned", "costs", r"^waterbend"),
    ("unattach", "unattach an Equipment", "701.3d", "no arm", "unowned", "costs", r"^unattach"),
    ("roll", "roll a die", "706.1", "no arm", "unowned", "2.31", r"^roll "),
    ("choose", "a choice announced as a cost (a type, a number, a keyword)", "601.2b", "no arm", "unowned", "costs",
     r"^choose "),
]

# ==========================================================================
# Table B -- activated abilities, against `AbilityDef`'s activation fields
# ==========================================================================

WHO = [
    ("who-any", "any player may activate", "602.2", "no `AbilityDef` field", "unowned", "item2",
     r"\bany player may activate"),
    ("who-opponents", "only an opponent may activate", "602.2", "no `AbilityDef` field", "unowned", "item2",
     r"\b(only your opponents|only an opponent|any opponent|each opponent) may activate"),
    ("who-controller", "its controller (CR 602.2's default)", "602.2", "`can_activate_its_abilities`", "built", "built",
     r""),
]

WHEN = [
    ("when-once-game", "once a game (exhaust and the like)", "602.5b", "no `ActivationRestriction` arm", "unowned", "2.8",
     r"^exhaust\b|activate (this ability |each exhaust ability )?only once\.|activate only once\b(?! each)"),
    ("when-count", "once or twice each turn", "602.5b", "no `ActivationRestriction` arm", "unowned", "2.8",
     r"only once each turn|only twice each turn|no more than (once|twice|\w+ times) each turn|only (once|twice) each round"),
    ("when-condition", "only if a condition holds", "602.5", "no `ActivationRestriction` arm; `Condition` exists", "unowned", "2.8",
     r"activate (this ability |these abilities |each .{0,30} )?only (if|while)\b"),
    ("when-step", "only during a turn, a step or combat", "602.5", "no `ActivationRestriction` arm", "unowned", "2.8",
     r"activate (this ability )?only (during|before|any time you could cast an instant during)"),
    ("when-sorcery", "only as a sorcery", "602.5d", "`ActivationRestriction::OnlyAsSorcery`", "built", "built",
     r"activate (this ability |each .{0,30} )?only as a sorcery"),
    ("when-instant", "only as an instant", "602.5e", "the default, as `AbilityType::Activated`", "built", "built",
     r"activate (this ability )?only as an instant"),
    ("when-any", "any time with priority (the default)", "602.1", "the default", "built", "built", r""),
]

WHERE = [
    ("where-graveyard", "from a graveyard", "113.6j", "`CannotActivate::NotOnBattlefield`", "unowned", "2.8",
     r"(exile|return|put|cast|shuffle) ~ (onto the battlefield )?from your graveyard|~ is in your graveyard"),
    ("where-hand", "from a hand", "113.6m", "`CannotActivate::NotOnBattlefield`", "unowned", "2.8",
     r"^discard ~|discard ~[,:]|reveal ~ from your hand|(exile|put|return|cast) ~ (onto the battlefield )?from your hand|~ is in your hand"),
    ("where-exile", "from exile", "113.6b", "`CannotActivate::NotOnBattlefield`", "unowned", "2.8",
     r"(put|return|cast) ~ (onto the battlefield )?from exile|activate only if ~ is (exiled|in exile)"),
    ("where-battlefield", "on the battlefield (CR 113.6's default)", "113.6", "the default", "built", "built", r""),
]

# ==========================================================================
# Table C -- mana abilities, by what they produce: what item 162 reads
# ==========================================================================

MANA_EFFECT = re.compile(r"(^|\. )add (\{|one mana|two mana|three mana|x mana|an amount|that much|mana|an additional|"
                         r"\w+ mana)|^double the amount of each type of unspent mana", re.I)

PRODUCTION = [
    # `(id, family, CR, surface, status, owner, pattern, what the affordability check does with it)`.
    # The status is whether the engine plays it; the last column is how
    # `find_mana_sources` counts it, which is item 162's.
    ("prod-restricted", "with \"spend this mana only\"", "106.6", "`ManaRestriction`; `ManaPool::pay` reads the plain pool",
     "owned", "B9", r"spend this mana only|this mana can't be spent", "counts it as unrestricted"),
    ("prod-grant", "mana that does something when spent, or does not empty", "106.6",
     "`ManaGrant`, `ManaPersistence`; no payment path reads them", "owned", "B9",
     r"when (you|that player) spend|spent this way|spend this mana|don't lose this mana|doesn't empty", "counts its mana"),
    ("prod-rider", "with a non-mana instruction in the same ability (painlands)", "605.1a",
     "`resolve_mana_effect` resolves `ProduceMana` atoms alone", "unowned", "2.19",
     r"^add [^.]*\.\s+\S", "counts its mana"),
    ("prod-could-produce", "a type another permanent \"could produce\"", "106.7", "no `ManaOutput` form", "unowned", "162",
     r"could produce", "skips it"),
    ("prod-chosen", "the color or type chosen as it entered", "607.2d", "no record of the choice", "unowned", "2.2",
     r"chosen color|the chosen type|of the chosen|noted type|circled colors|exiled cards?'s colors", "skips it"),
    ("prod-any-color", "one mana of any color or type", "106.1b", "no `ManaOutput` form", "unowned", "2.19",
     r"any color|any one color|any type|any combination of colors|in any combination|any colors|of any color"
     r"|of that color|of either of|any of (the|its|~'s)", "skips it"),
    ("prod-double", "doubles the mana already in the pool", "106.4", "`AmountExpr::UnspentMana`", "built", "built",
     r"^double the amount", "skips it"),
    ("prod-variable", "an amount the board decides", "106.1", "`AmountExpr`, where it names the count", "built", "built",
     r"for each|equal to|an amount of|that much|x mana|\{x\}", "skips it: `Fixed` alone"),
    ("prod-choice", "one of two or three listed types", "106.1", "one ability per type, as Everywhere is written",
     "built", "built", r"^add \{[wubrgc]\}(, \{[wubrgc]\})*,? or \{[wubrgc]\}|^choose a color\b",
     "counts each type as a source"),
    ("prod-several", "several mana at once ({C}{C}, {W}{U})", "106.1", "`ManaOutput`", "built", "built",
     r"^add (\{[wubrgc]\}){2,}|^add (two|three) mana", "counts one"),
    ("prod-one", "one mana of one type", "106.1", "`Primitive::ProduceMana`", "built", "built", r"^add \{[wubrgc]\}\.?$",
     "counts it"),
]

# What a mana ability's cost feeds it, beside what it produces: item 162's
# "sources fed by mana or a sacrifice".
FED = [
    ("fed-mana", "costs mana (a filter, a Signet)", "605.1a", "`Cost::Mana`, paid from the pool", "built", "built",
     r"\{(\d+|[wubrgcx]|[wubrg]/[wubrgp])\}", "skips it unless the pool already holds the cost"),
    ("fed-sacrifice", "costs a sacrifice (Ironworks, a Treasure)", "605.1a", "`Cost::Sacrifice`", "built", "built",
     r"sacrifice", "counts it free, the sacrificed source included"),
    ("fed-other", "costs a tap, life or nothing", "605.1a", "`available_mana_sources`", "built", "built", r"",
     "counts it"),
]

# ==========================================================================
# Table D -- casting and playing: permission, owner, timing (CR 601.3, 305.1)
# ==========================================================================
# `(id, family, CR, surface, status, owner, clause regex, Scryfall query)`.
# The clause regex reads one sentence with `~` for the card's own name; the
# query is what a reader pastes, and it counts cards, not clauses.

CAST = [
    ("cast-free", "cast without paying its mana cost", "118.9", "no permission, and no effect-provided alternative cost",
     "unowned", "2.3", r"without paying (its|their|that spell's|~'s) mana costs?",
     'o:"without paying"'),
    ("cast-others-cards", "cast or play a card its caster does not own", "601.3", "`can_begin_to_cast` asks owner = caster",
     "unowned", "2.3", r"(you don't own|your opponents own|an opponent owns|cards? (your )?opponents? own)",
     '(o:"you don\'t own" or o:"opponents own" or o:"an opponent owns")'),
    ("cast-top-library", "cast or play from the top of a library", "601.3", "`CannotCast::NotInHand`",
     "unowned", "2.3", r"(cast|play)[^.]{0,60}from the top of (your|their|a|that player's) library|top card of your library[^.]{0,40}you may (cast|play)",
     'o:/(cast|play)[^.]*from the top of/'),
    ("cast-self-graveyard", "cast ~ from a graveyard (no keyword)", "601.3", "`CannotCast::NotInHand`",
     "unowned", "2.3", r"(cast|play) ~ from (your|a) graveyard",
     'o:/(cast|play) ~ from (your|a) graveyard/'),
    ("cast-graveyard", "cast or play other cards from a graveyard", "601.3", "`CannotCast::NotInHand`",
     "unowned", "2.3", r"(cast|play)[^.]{0,80}from (your|a|their|an opponent's|that player's) graveyards?",
     'o:/(cast|play)[^.]*from (your|a|their) graveyard/'),
    ("cast-exile", "cast or play a card from exile", "601.3", "`CannotCast::NotInHand`",
     "unowned", "2.3", r"(you may|may) (cast|play) (it|that card|those cards|them|the exiled card|cards exiled|one of|a card exiled|spells? from among)|(cast|play)[^.]{0,60}from exile|exiled with ~",
     'o:/may (cast|play) (it|that card|those cards|them|the exiled card|cards exiled|one of|a card exiled|spells? from among)|(cast|play)[^.]*from exile/'),
    ("cast-flash-grant", "cast as though it had flash", "601.3b", "a row granting flash; CR 609.4's permission", "owned", "B8",
     r"as though (it|they) had flash|any time you could cast an instant",
     '(o:"as though it had flash" or o:"as though they had flash")'),
    ("cast-self-timing", "cast ~ only at a time or if a condition holds", "601.3", "RS-2's `Cast` arm, `SourceOnly`", "owned", "RS-2",
     r"cast (~|this spell) only",
     'o:"cast this spell only"'),
    ("cast-cant", "a player can't cast", "601.3", "RS-2's `Cast` arm", "owned", "RS-2",
     r"can't (be )?cast",
     'o:"can\'t cast"'),
    ("play-extra-land", "play an additional land", "305.2", "`PlayerState::lands_per_turn`, nothing sets it", "unowned", "2.15",
     r"play (an|two|three|any number of) additional lands?|play any number of lands",
     'o:"additional land"'),
]

# ==========================================================================
# Table E -- keyword families, by card (CR 702 with 601.3, 113.6, 118.8, 118.9)
# ==========================================================================
# `(id, family, keywords, CR, surface, status, owner)`.

KEYWORDS = [
    ("kw-graveyard-cast", "cast from a graveyard",
     ["Flashback", "Escape", "Retrace", "Jump-start", "Aftermath", "Disturb", "Mayhem", "Harmonize"],
     "601.3", "`AlternativeCost::Flashback`, `Escape`; `CannotCast::NotInHand`", "unowned", "2.3"),
    ("kw-exile-special", "exiled by a special action, cast later",
     ["Foretell", "Plot", "Suspend"], "116.2f", "no special action but a land play", "unowned", "2.3"),
    ("kw-cast-resolving", "cast while something resolves",
     ["Cascade", "Discover", "Ripple", "Rebound", "Madness", "Miracle"], "608.2g", "no cast during a resolution", "unowned", "2.3"),
    ("kw-commander", "cast from the command zone (a commander)", [], "903.8",
     "`CannotCast::NotInHand`; `is_commander` is B2's", "unowned", "2.3"),
    ("kw-face-down", "cast face down", ["Morph", "Megamorph", "Disguise"], "708.4", "no face-down spell", "owned", "CV-6"),
    ("kw-alt-characteristics", "cast with other characteristics",
     ["Prototype"], "718.3", "no alternative characteristics", "owned", "2.41"),
    ("kw-alt-arm", "an alternative cost with an arm",
     ["Evoke", "Dash", "Overload", "Bestow"], "118.9", "`AlternativeCost` arm; each needs a second half", "unowned", "none"),
    ("kw-alt-other", "an alternative cost with no arm",
     ["Emerge", "Spectacle", "Surge", "Prowl", "Blitz", "Mutate", "Freerunning", "Impending", "Warp", "Sneak",
      "Web-slinging", "Awaken", "Cleave"], "118.9", "`AlternativeCost::Custom` names it; each needs a second half", "unowned", "none"),
    ("kw-kicker", "kicker", ["Kicker"], "702.33", "`AdditionalCost::Kicker`, `Condition::SpellWasKicked`", "built", "built"),
    ("kw-add-arm", "an additional cost with an arm",
     ["Buyback", "Entwine", "Casualty", "Bargain", "Strive"], "118.8", "`AdditionalCost` arm; each needs a second half", "unowned", "none"),
    ("kw-add-count", "an additional cost paid any number of times",
     ["Multikicker", "Replicate", "Squad"], "601.2b", "601.2b's announcement cannot ask for a number", "unowned", "none"),
    ("kw-add-other", "an additional cost with no arm",
     ["Splice", "Conspire", "Escalate", "Offspring", "Gift", "Tiered"], "118.8", "`AdditionalCost::Custom` names it", "unowned", "none"),
    ("kw-spree", "spree", ["Spree"], "702.172", "a mode's own additional cost", "owned", "item81"),
    ("kw-pay-with", "pay with permanents or cards",
     ["Convoke", "Improvise", "Delve", "Assist", "Offering", "Waterbend"], "702.51", "no payment but mana", "owned", "162"),
    ("kw-affinity", "a cost reduction", ["Affinity", "Undaunted"], "702.41", "`CostChange::ReduceGeneric`", "built", "built"),
    ("kw-mana-spent", "reads the mana spent",
     ["Sunburst", "Converge", "Adamant"], "702.44", "`ManaSpent` by type", "owned", "item30"),
    ("kw-hand-ability", "an activated ability that works in a hand",
     ["Cycling", "Typecycling", "Landcycling", "Basic landcycling", "Channel", "Ninjutsu", "Commander ninjutsu",
      "Transmute", "Forecast", "Bloodrush", "Reinforce"], "113.6m", "`CannotActivate::NotOnBattlefield`", "unowned", "2.8"),
    ("kw-graveyard-ability", "an activated ability that works in a graveyard",
     ["Unearth", "Embalm", "Eternalize", "Encore", "Scavenge"], "113.6m", "`CannotActivate::NotOnBattlefield`", "unowned", "2.8"),
    ("kw-tap-creatures", "tap creatures with total power N",
     ["Crew", "Saddle", "Station"], "702.122", "no arm: tap N untapped creatures", "unowned", "costs"),
    ("kw-restricted-ability", "an ability with a use limit",
     ["Boast", "Exhaust", "Level Up", "Outlast"], "602.5b", "no `ActivationRestriction` arm", "unowned", "2.8"),
    ("kw-equip", "equip and its kin", ["Equip", "Fortify", "Reconfigure"], "702.6", "`ActivationRestriction::OnlyAsSorcery`", "built", "built"),
    ("kw-loyalty", "a planeswalker's loyalty abilities", [], "606.3", "no loyalty ability", "unowned", "2.11"),
    ("kw-upkeep-costs", "a cost paid as a trigger resolves",
     ["Cumulative upkeep", "Echo"], "118.12", "CR 118.12's payment (TR-2b); age counters, item 3", "owned", "C"),
]

EXTRA_KW_QUERY = {
    "kw-commander": "is:commander",
    "kw-loyalty": "t:planeswalker",
}

# ==========================================================================
# Table F -- mana symbols in a card's mana cost
# ==========================================================================

SYMBOLS = [
    ("sym-colorless-hybrid", "a {C/W}-style hybrid", "107.4e", "no `ManaSymbol` arm", "unowned", "CP-1",
     r"\{c/[wubrg]\}", "m:/\\{C\\/[WUBRG]\\}/"),
    ("sym-hybrid-phyrexian", "a hybrid Phyrexian symbol", "107.4f", "`ManaSymbol::HybridPhyrexian`, unpaid", "owned", "CP-1",
     r"\{[wubrg]/[wubrg]/p\}", "m:/\\{[WUBRG]\\/[WUBRG]\\/P\\}/"),
    ("sym-phyrexian", "a Phyrexian symbol", "107.4f", "`ManaSymbol::Phyrexian`, unpaid", "owned", "CP-1",
     r"\{[wubrg]/p\}", "is:phyrexian"),
    ("sym-mono-hybrid", "a {2/W} symbol", "107.4e", "`ManaSymbol::MonoHybrid`, unpaid", "owned", "CP-1",
     r"\{2/[wubrg]\}", "m:/\\{2\\/[WUBRG]\\}/"),
    ("sym-hybrid", "a hybrid symbol", "107.4e", "`ManaSymbol::Hybrid`, unpaid", "owned", "CP-1",
     r"\{[wubrg]/[wubrg]\}", "is:hybrid"),
    ("sym-snow", "{S}", "107.4h", "`ManaSymbol::Snow`, unpaid", "owned", "B9", r"\{s\}", "m:{S}"),
    ("sym-x", "{X}", "107.3a", "`ChooseXValue` asked; `find_mana_sources` refuses X, so never offered", "owned", "162",
     r"\{x\}", "m:{X}"),
    ("sym-colorless", "{C}", "107.4c", "`ManaSymbol::Colorless`", "built", "built", r"\{c\}", "m:{C}"),
    ("sym-plain", "colored and generic only", "107.4a", "`ManaSymbol::Colored`, `Generic`", "built", "built", r"", ""),
]

# ==========================================================================
# Table G -- mana by phrase: production and spending the tables above miss
# ==========================================================================

MANA_PHRASES = [
    ("ph-treasure", "makes a Treasure (one mana of any color)", "111.10a", "no `ManaOutput` form", "unowned", "2.19",
     'o:"treasure token"'),
    ("ph-triggered-mana", "a triggered mana ability", "605.1b", "built (TR-1); affordability skips it", "owned", "162",
     '(o:"tapped for mana" o:"adds") or o:"is tapped for mana, its controller adds"'),
    ("ph-mana-replacement", "replaces the mana a permanent produces", "106.12b", "built (RE-9); affordability skips it", "owned", "162",
     '(o:"tapped for mana" o:"instead") or o:"produces twice"'),
    ("ph-ritual", "a spell or non-mana ability that adds mana", "106.3", "`Primitive::ProduceMana` off a resolution", "built", "built",
     '(t:instant or t:sorcery) o:/^add \\{/'),
    ("ph-as-though", "spend mana as though it were any color or type", "609.4b", "no payment path reads it", "owned", "B8",
     'o:"mana as though it were mana of any"'),
    ("ph-persist", "mana that does not empty", "106.4", "`ManaPersistence`, no payment path reads it", "owned", "B9",
     '(o:"don\'t lose this mana" or o:"doesn\'t empty" or o:"don\'t empty")'),
    ("ph-spend-grant", "mana that does something when spent", "106.6", "`ManaGrant`, no payment path reads it", "owned", "B9",
     '(o:"spend this mana" -o:"spend this mana only")'),
]

# ==========================================================================
# Table H -- the rules a card may override (the rules pass, CR 101.1)
# ==========================================================================
# `(CR, the default, the override, query, surface, status, owner)`.

OVERRIDES = [
    ("601.2", "a spell is cast from its caster's hand (\"usually the hand\")",
     "a permission names another zone", 'o:/(cast|play)[^.]* from (your|a|their) (graveyard|exile)|from the top of your library/',
     "`can_begin_to_cast`", "unowned", "2.3"),
    ("601.3", "no player may cast a card that a rule or effect does not permit",
     "\"you may cast\" a card you don't own", '(o:"you don\'t own" or o:"opponents own")',
     "`can_begin_to_cast`", "unowned", "2.3"),
    ("117.1a", "a noninstant spell waits for a main phase and an empty stack",
     "flash", "kw:flash", "`is_instant_or_has_flash`", "built", "built"),
    ("601.3b", "a spell without flash is cast at sorcery timing",
     "\"as though it had flash\"", '(o:"as though it had flash" or o:"as though they had flash")',
     "a permission (CR 609.4)", "owned", "B8"),
    ("305.1", "a land is played from its owner's hand",
     "play lands from a graveyard, a library's top or exile", 'o:/play (lands|land cards)[^.]* from/',
     "`can_play_land`", "unowned", "2.3"),
    ("305.2", "one land a turn", "play additional lands", 'o:"additional land"',
     "`lands_per_turn`", "unowned", "2.15"),
    ("602.2", "only an object's controller activates its abilities (\"unless the object specifically says otherwise\")",
     "any player, or only an opponent, may activate", '(o:"any player may activate" or o:"opponents may activate")',
     "`can_activate_its_abilities`", "unowned", "item2"),
    ("602.5", "an ability may be activated any time its controller has priority",
     "\"Activate only ...\"", 'o:"activate only"', "`ActivationRestriction`, `OnlyAsSorcery` alone", "unowned",
     "2.8"),
    ("602.5a", "a creature's {T} or {Q} ability waits out summoning sickness",
     "haste, or \"as though it had haste\"", '(kw:haste or o:"as though it had haste" or o:"as though they had haste")',
     "`has_summoning_sickness`", "built", "built"),
    ("113.6", "a permanent's abilities work on the battlefield and a spell's on the stack",
     "an ability that names its zone, or whose cost moves its object", 'o:/~ from your (graveyard|hand)/',
     "`CannotActivate::NotOnBattlefield`", "unowned", "2.8"),
    ("106.4", "mana empties between steps and phases", "mana that does not empty",
     '(o:"don\'t lose this mana" or o:"doesn\'t empty" or o:"don\'t empty")', "`ManaPersistence`", "owned", "B9"),
    ("107.4a", "colored mana pays only its own color", "spend mana as though it were any color",
     'o:"as though it were mana of any"', "`ManaPool::pay`", "owned", "B8"),
    ("106.6", "mana is spent on anything", "\"spend this mana only\"", 'o:"spend this mana only"',
     "`ManaRestriction`", "owned", "B9"),
    ("118.9", "a spell's mana cost is paid", "a printed alternative cost (\"rather than pay\")",
     'o:"rather than pay"', "`AlternativeCost::Custom`; its pieces are §3's", "built", "built"),
    ("118.9", "a spell's mana cost is paid", "an effect's \"without paying its mana cost\"",
     'o:"without paying"', "a permission's cost", "unowned", "2.3"),
    ("118.8", "only the costs a spell prints", "an additional cost another object imposes (the Defilers)",
     'o:"additional cost" -o:"as an additional cost to cast this spell"', "`CostChange` has no such arm", "unowned",
     "costs"),
    ("601.6", "the caster chooses the targets",
     "\"an opponent chooses target\"", 'o:"opponent chooses target"', "`ask_select_recipients`'s player", "owned", "C"),
    ("701.9b", "the discarding player chooses", "\"at random\", or another player chooses",
     'o:/discard[^.]* at random|chooses a card[^.]* discards/', "`DiscardChooser`", "unowned", "2.5"),
]

# ==========================================================================
# Table I -- the brief's priors, each beside its query
# ==========================================================================

PRIORS = [
    ("activated abilities (a colon in the text)", 8612, 'o:":"', BASE),
    ("an additional cost", 316, 'o:"additional cost"', BASE),
    ("hybrid or Phyrexian mana", 668, "(is:hybrid or is:phyrexian)", BASE),
    ("convoke, improvise, delve, emerge or affinity", 243,
     "(kw:convoke or kw:improvise or kw:delve or kw:emerge or kw:affinity)", BASE),
    ("flashback, escape, unearth, retrace, jump-start, embalm, eternalize or disturb", 384,
     "(kw:flashback or kw:escape or kw:unearth or kw:retrace or kw:jump-start or kw:embalm or kw:eternalize or kw:disturb)", BASE),
    ("\"spend this mana only\"", 165, 'o:"spend this mana only"', BASE),
    ("\"any player may activate\" (paper)", 40, 'o:"any player may activate"', PAPER),
    ("\"only your opponents may activate\" (paper)", 5, 'o:"only your opponents may activate"', PAPER),
    ("discard as a cost (paper)", 384, "o:/discard [^:]*:/", PAPER),
    ("exile from a graveyard as a cost (paper)", 161, "o:/exile [^:]*from your graveyard[^:]*:/", PAPER),
    ("removing counters as a cost (paper)", 334, "o:/remove [^:]*counters? from [^:]*:/", PAPER),
    ("cycling (paper)", 399, "kw:cycling", PAPER),
]

# ==========================================================================
# Table J -- the question kinds (`ChoiceKind`), what builds each one's options
# ==========================================================================
# `kind: (what the options are, filtered?, what a why would read, owner, slot)`.
# "filtered" means a check drops candidates the question could have named, so
# "why is X not offered" has an answer beyond "X is not what this asks".

QUESTIONS = {
    "PriorityAction": ("lands, spells, abilities, and Pass", "yes",
                       "`can_play_land`, `can_cast`, `can_activate` (SU-7)", "built", ""),
    "DeclareAttackers": ("creatures and what each may attack", "yes", "`can_attack` (SU-7)", "built", ""),
    "DeclareBlockers": ("creatures and what each may block", "yes", "`can_block` (SU-7)", "built", ""),
    "AssignCombatDamage": ("amounts across the creatures blocking it", "bounds",
                           "CR 510.1c's lethal-damage bound", "setup", "SU-8"),
    "AssignTrampleDamage": ("amounts across blockers and the player", "bounds",
                            "CR 702.19b's lethal-damage bound", "setup", "SU-8"),
    "ChooseXValue": ("a number", "bounds", "the bound the enumeration sets on X", "162", "item 162's build"),
    "ChooseAlternativeCost": ("the mana cost and each printed alternative", "no", "", "setup", "SU-8"),
    "ChooseAdditionalCosts": ("each optional additional cost", "no",
                              "a mandatory cost is in the total, not offered (CR 118.8)", "setup", "SU-8"),
    "SelectRecipients": ("the objects or players a target or choice may name", "yes",
                         "`validate_selection`'s reasons, typed", "RS-2", "RS-2"),
    "GenericManaAllocation": ("a split of the generic part across the pool", "bounds",
                              "the clamp that keeps each pip's mana", "162", "item 162's build"),
    "OrderCostReductions": ("an order of every reduction that applies", "no", "", "setup", "SU-8"),
    "ManaAbilityWindow": ("the mana abilities that can pay now", "yes",
                          "`can_pay_costs`'s `CannotPay`, per ability", "162", "item 162's build"),
    "ChooseSacrificeForCost": ("the permanents that can pay the cost", "yes",
                               "CR 701.21a and the cost's filter", "costs", "the cost phase"),
    "ChooseReplacementEffect": ("the replacement effects that apply", "yes",
                                "the gather's candidates, from the trace", "setup", "SU-8"),
    "OrderTriggers": ("an order of one player's pending triggers", "no",
                      "the dispatch's matcher, from the trace", "setup", "SU-8"),
    "ApplyOptionalReplacement": ("yes or no", "no", "", "setup", "SU-8"),
    "ApplyOptionalEffect": ("yes or no", "no", "", "setup", "SU-8"),
    "AllocateNextDamage": ("amounts across the damage sources", "bounds", "CR 615.7's count", "setup", "SU-8"),
    "ChooseDamageSource": ("every permanent and spell", "no", "", "setup", "SU-8"),
    "ChooseEnteringController": ("the opponents still in the game", "yes", "CR 800.4a", "setup", "SU-8"),
    "ChooseAuxiliaryZoneChange": ("the objects CR 614.13a/b and CR 101.2 allow", "yes",
                                  "the filter and the restriction that dropped it", "setup", "SU-8"),
    "ChooseCopySource": ("the permanents the copy effect may copy", "yes", "the copy effect's filter", "setup", "SU-8"),
    "CommanderToCommandZoneSba": ("yes or no", "no", "", "setup", "SU-8"),
    "Discard": ("the cards in the hand", "no", "", "setup", "SU-8"),
    "Scry": ("the cards looked at", "no", "", "setup", "SU-8"),
    "ScryOrder": ("an order of one pile", "no", "", "setup", "SU-8"),
    "LegendRule": ("the legendary permanents sharing the name", "no", "", "setup", "SU-8"),
}

QUESTION_OWNERS = {
    "built": "built (SU-7)",
    "setup": "`setup-architecture.md` §7c",
    "162": "`codebase-state.md` item 162",
    "RS-2": "`cant-effects-architecture.md` RS-2",
    "costs": "the cost phase that pays the cost",
}

# ==========================================================================
# Classification
# ==========================================================================


def first(buckets, s, ix=6):
    low = s.lower()
    for b in buckets:
        if not b[ix] or re.search(b[ix], low):
            return b
    return None


def classify_piece(piece):
    return first(COST_BUCKETS, piece)


def self_piece(card, piece):
    """"Exile Balthor" on Balthor the Defiled: a verb, then the start of the
    card's own name, is the card itself."""
    m = re.match(r"^(\w+) ([A-Z][\w'-]*(?: [A-Z][\w'-]*)*)$", piece)
    if m and any(n.startswith(m.group(2)) for n in [card["name"]] + card["name"].split(" // ")):
        return m.group(1) + " ~"
    return piece


def is_mana_ability(ab):
    eff = ab["effect"].lower()
    if LOYALTY.match(ab["cost"].strip()):
        return False
    if "target" in eff.split(".")[0]:
        return False
    return bool(MANA_EFFECT.search(eff))


def classify_ability(ab):
    eff = ab["effect"]
    whole = ab["cost"] + ": " + eff
    return {
        "who": first(WHO, eff),
        "when": first(WHEN, eff) if not re.match(r"^exhaust", ab.get("word", ""), re.I) else WHEN[0],
        "where": first(WHERE, whole),
    }


INSTRUCTION = re.compile(r"\s*(activate (this ability |these abilities )?only[^.]*|any player may activate[^.]*"
                         r"|only your opponents may activate[^.]*)\.", re.I)


def classify_production(ab):
    # Activation instructions (CR 602.1b) are not part of the effect.
    eff = INSTRUCTION.sub("", ab["effect"]).strip().lower()
    # What the ability produces starts at its first "add" or "choose a color".
    m = re.search(r"(^|\. )((add|choose a color|double the amount)\b.*)$", eff)
    body = m.group(2) if m else eff
    return first(PRODUCTION, body), first(FED, ab["cost"])


def walk(cards):
    """Every unit the tables partition, classified once."""
    out = {"pieces": [], "abilities": [], "mana": [], "clauses": [], "symbols": [], "kw": defaultdict(set)}
    for c in cards:
        for ab in activated(c):
            ab["word"] = ""
            out["abilities"].append((ab, classify_ability(ab)))
            for p in cost_pieces(ab["cost"]):
                p = self_piece(c, p)
                out["pieces"].append((c["name"], "ability", p, classify_piece(p)))
            if is_mana_ability(ab):
                out["mana"].append((ab, classify_production(ab)))
        for ctx, text in spell_cost_clauses(c):
            for p in spell_pieces(text):
                p = self_piece(c, p[0].upper() + p[1:])
                out["pieces"].append((c["name"], ctx, p, classify_piece(p)))
        for s in sentences(c):
            b = first(CAST, s)
            if b:
                out["clauses"].append((c["name"], s, b))
        sym = first(SYMBOLS, c["mana"], ix=6) if c["mana"] else None
        if sym and sym[6]:
            out["symbols"].append((c["name"], sym))
        for k in c["kw"]:
            out["kw"][k].add(c["name"])
    return out


# ==========================================================================
# The tree: the question kinds, and the registered cards
# ==========================================================================


def parse_variants(path, enum):
    raw = open(path, encoding="utf-8").read().split("\n")
    start = next(i for i, l in enumerate(raw) if re.match(r"^pub enum %s\b" % enum, l))
    depth, out = 0, []
    for l in raw[start:]:
        s = l.strip()
        if s.startswith(("//", "#[")):
            continue
        depth += l.count("{") - l.count("}")
        m = re.match(r"^    ([A-Z]\w*)\s*(\{|,|\(|$)", l)
        if m and depth >= 1:
            out.append(m.group(1))
        if depth == 0 and out:
            break
    return out


def explained_kinds():
    """The kinds `ui::why::refusals` answers with a check, read off its arms."""
    src = open(WHY, encoding="utf-8").read()
    body = src[src.index("fn refusals("):]
    body = body[:body.index("\n}\n")]
    return sorted(set(re.findall(r"\(ChoiceKind::(\w+),\s*WhyAbout::", body)))


def registered():
    src = open(REGISTRY, encoding="utf-8").read()
    return re.findall(r'registry\.register\(\s*"([^"]+)",\s*([a-z_0-9]+)::([a-z_0-9]+)', src)


def registered_cards():
    """The registered printings' oracle text, by `/cards/collection`."""
    if os.path.exists(REGISTERED_CACHE):
        return json.load(open(REGISTERED_CACHE, encoding="utf-8"))
    if _no_fetch:
        raise SystemExit("--no-fetch, and there is no registered-cards cache")
    want = [n for n, _, _ in registered()]
    found, missing = [], []
    for i in range(0, len(want), 75):
        d = _post("https://api.scryfall.com/cards/collection",
                  {"identifiers": [{"name": n} for n in want[i:i + 75]]})
        found += [_card(c) for c in d.get("data", [])]
        missing += [x.get("name") for x in d.get("not_found", [])]
        time.sleep(DELAY)
    out = {"fetched": _dt.date.today().isoformat(), "cards": found, "missing": missing}
    json.dump(out, open(REGISTERED_CACHE, "w", encoding="utf-8"))
    return out


# ==========================================================================
# Ownership this census assigned. A family found unowned keeps its finding
# in the tables ("found unowned"), and its owner cell names the new owner.
# `{old owner key: new owner key}`, filled in from the census doc's §1.
# ==========================================================================

ASSIGN = {
    # The permission half (CR 601.3's allow, 602.2, 602.5, 113.6, 305.1-2, 606.3).
    "2.3": "PM", "2.8": "PM", "2.11": "PM", "item2": "PM", "2.15": "PM",
    # Mana production with no owner: item 162's design takes it.
    "2.19": "162", "2.2": "162", "prod-could-produce": "162", "ph-treasure": "162",
    # The cost actions: CP-2 takes the unchecked arms and the shapes arms need.
    "unchecked": "CP-2",
    "choice": "CP-2", "count": "CP-2", "kw-add-count": "CP-2",
    "sacrifice-variable": "CP-2", "remove-counters-variable": "CP-2", "life-variable": "CP-2",
    "mana-variable": "CP-2", "remove-counters-other": "CP-2", "put-counters-other": "CP-2",
    "tap-others": "CP-2", "untap-others": "CP-2", "kw-tap-creatures": "CP-2",
    # A symbol, and an ability's X: the payment slot.
    "mana-colorless-hybrid": "CP-1", "sym-colorless-hybrid": "CP-1", "mana-x": "CP-1",
    # One arm per first card, in Phase 8.
    "costs": "C-arm", "2.5": "C", "2.31": "C",
    "kw-alt-arm": "C", "kw-alt-other": "C", "kw-add-arm": "C", "kw-add-other": "C",
    "118.8": "C-change",
    # Out of v1: ticket counters come from Attractions and stickers (`roadmap-v2.md` §7).
    "ticket": "excluded",
}


def assigned(bid, key):
    """The owner this census gave a family found unowned, if it gave one."""
    return ASSIGN.get(bid) or ASSIGN.get(key)


def owner_of(key, bid=None):
    return O[assigned(bid, key) or key]


def status_of(status, key, bid=None):
    new = assigned(bid, key)
    if status == "unowned" and new:
        return "excluded" if new == "excluded" else "owned"
    return status


# ==========================================================================
# The CR labels, audited
# ==========================================================================


def every_cr():
    out = []
    for table in (COST_BUCKETS, WHO, WHEN, WHERE, PRODUCTION, FED, CAST, SYMBOLS, MANA_PHRASES):
        out += [b[2] for b in table]
    out += [k[3] for k in KEYWORDS] + [o[0] for o in OVERRIDES]
    return sorted(set(out), key=lambda r: [int(x) if x.isdigit() else x for x in re.split(r"(\d+)", r)])


def audit_labels():
    """Every rule number a row cites, against the CR. A label whose number does
    not resolve fails loudly instead of being believed (`copy-census.py`)."""
    if not os.path.exists(CR):
        return [], ["the CR is not at %s; labels unaudited" % CR]
    text = open(CR, encoding="utf-8", errors="replace").read()
    found, bad = [], []
    for cr in every_cr():
        m = re.search(r"^%s[. ].*$" % re.escape(cr), text, re.M)
        (found if m else bad).append((cr, m.group(0)[:100]) if m else cr)
    return found, bad


# ==========================================================================
# Rendering
# ==========================================================================


def fmt(n):
    return "{:,}".format(n) if isinstance(n, int) else str(n)


def esc(s):
    return s.replace("|", "\\|")


def status_cell(status, key, bid=None):
    now = status_of(status, key, bid)
    if now != status:
        return "**%s** (found unowned)" % now
    return status


def render_costs(w):
    rows = defaultdict(lambda: [0, set(), Counter()])
    for name, ctx, piece, b in w["pieces"]:
        r = rows[b[0] if b else "?"]
        r[0] += 1
        r[1].add(name)
        r[2][ctx] += 1
    total = sum(r[0] for r in rows.values())
    L = ["| family | CR | surface | status | owner | pieces | cards | in a spell's cost |",
         "|---|---|---|---|---|---:|---:|---:|"]
    for bid, fam, cr, surface, status, key, _ in COST_BUCKETS:
        n, cards, ctx = rows[bid]
        if not n:
            continue
        L.append("| %s | %s | %s | %s | %s | %s | %s | %s |" % (
            esc(fam), cr, surface, status_cell(status, key, bid), owner_of(key, bid), fmt(n), fmt(len(cards)),
            fmt(ctx["additional"] + ctx["alternative"]) or "--"))
    if rows["?"][0]:
        L.append("| unclassified, `--residual` | | | | | %s | %s | |" % (fmt(rows["?"][0]), fmt(len(rows["?"][1]))))
    L.append("| **all pieces** | | | | | **%s** | | |" % fmt(total))
    L.append("")
    L.append("Each row's pattern, matched against the lowercased piece, in row order:")
    L.append("")
    for bid, fam, *_rest, pat in COST_BUCKETS:
        if rows[bid][0]:
            L.append("- %s: `%s`" % (esc(fam), esc(pat)))
    return "\n".join(L)


def render_axis(w, axis, buckets):
    rows = defaultdict(lambda: [0, set()])
    for ab, c in w["abilities"]:
        r = rows[c[axis][0]]
        r[0] += 1
        r[1].add(ab["card"])
    L = ["| %s | CR | surface | status | owner | abilities | cards | pattern |" % axis,
         "|---|---|---|---|---|---:|---:|---|"]
    for bid, fam, cr, surface, status, key, pat in buckets:
        n, cards = rows[bid]
        L.append("| %s | %s | %s | %s | %s | %s | %s | %s |" % (
            esc(fam), cr, surface, status_cell(status, key, bid), owner_of(key, bid), fmt(n), fmt(len(cards)),
            "`%s`" % esc(pat) if pat else "the rest"))
    return "\n".join(L)


def render_abilities(w):
    n = len(w["abilities"])
    granted = sum(1 for ab, _ in w["abilities"] if ab["granted"])
    cards = len({ab["card"] for ab, _ in w["abilities"]})
    head = ("%s activated abilities on %s cards, %s of them quoted in a grant; %s are mana abilities (CR 605.1a). "
            "Who and when read the effect's activation instructions (CR 602.1b); where reads the cost and the effect."
            % (fmt(n), fmt(cards), fmt(granted), fmt(len(w["mana"]))))
    return "\n\n".join([head, render_axis(w, "who", WHO), render_axis(w, "when", WHEN), render_axis(w, "where", WHERE)])


def render_mana(w):
    prod = defaultdict(lambda: [0, set()])
    fed = defaultdict(lambda: [0, set()])
    for ab, (p, f) in w["mana"]:
        for table, b in ((prod, p), (fed, f)):
            r = table[b[0] if b else "?"]
            r[0] += 1
            r[1].add(ab["card"])
    out = []
    for title, table, buckets in (("what it produces", prod, PRODUCTION), ("what its cost feeds it", fed, FED)):
        L = ["| %s | CR | surface | plays | owner | abilities | cards | the affordability check (item 162) |" % title,
             "|---|---|---|---|---|---:|---:|---|"]
        for bid, fam, cr, surface, status, key, _, aff in buckets:
            n, cards = table[bid]
            L.append("| %s | %s | %s | %s | %s | %s | %s | %s |" % (
                esc(fam), cr, surface, status_cell(status, key, bid), owner_of(key, bid), fmt(n), fmt(len(cards)), aff))
        if table["?"][0]:
            L.append("| unclassified | | | | | %s | %s | |" % (fmt(table["?"][0]), fmt(len(table["?"][1]))))
        L.append("")
        L.append("Patterns, in row order, against the effect from its first \"add\" or \"choose a color\", activation "
                 "instructions removed (for the cost, against the cost):")
        L.append("")
        for bid, fam, *_rest, pat, aff in buckets:
            L.append("- %s: %s" % (esc(fam), "`%s`" % esc(pat) if pat else "the rest"))
        out.append("\n".join(L))
    return "\n\n".join(out)


def render_cast(w):
    rows = defaultdict(lambda: [0, set()])
    for name, s, b in w["clauses"]:
        rows[b[0]][0] += 1
        rows[b[0]][1].add(name)
    L = ["| family | CR | surface | status | owner | clauses | cards | Scryfall, cards | query (plus `BASE`) |",
         "|---|---|---|---|---|---:|---:|---:|---|"]
    for bid, fam, cr, surface, status, key, pat, q in CAST:
        n, cards = rows[bid]
        L.append("| %s | %s | %s | %s | %s | %s | %s | %s | `%s` |" % (
            esc(fam), cr, surface, status_cell(status, key, bid), owner_of(key, bid), fmt(n), fmt(len(cards)),
            fmt(count(q + " " + BASE)), esc(q)))
    L.append("")
    L.append("The clause patterns, in row order, against one sentence with `~` for the card's own name; "
             "a sentence lands in the first that matches, so the clause column partitions:")
    L.append("")
    for bid, fam, *_rest, pat, q in CAST:
        L.append("- %s: `%s`" % (esc(fam), esc(pat)))
    return "\n".join(L)


def kw_query(k):
    if k[0] in EXTRA_KW_QUERY:
        return EXTRA_KW_QUERY[k[0]]
    return "(" + " or ".join('kw:"%s"' % x.lower() for x in k[2]) + ")"


def render_keywords(w):
    L = ["| family | keywords, cards each | CR | surface | status | owner | cards | Scryfall | query (plus `BASE`) |",
         "|---|---|---|---|---|---|---:|---:|---|"]
    for k in KEYWORDS:
        kid, fam, kws, cr, surface, status, key = k
        cards = set()
        for x in kws:
            cards |= w["kw"].get(x, set())
        each = ", ".join("%s %s" % (x.lower(), fmt(len(w["kw"].get(x, set())))) for x in kws) or "--"
        q = kw_query(k)
        live = count(q + " " + BASE)
        L.append("| %s | %s | %s | %s | %s | %s | %s | %s | `%s` |" % (
            esc(fam), esc(each), cr, surface, status_cell(status, key, kid), owner_of(key, kid),
            fmt(len(cards)) if kws else "--", fmt(live), esc(q)))
    return "\n".join(L)


def render_symbols(w, n_cards):
    rows = defaultdict(set)
    for name, b in w["symbols"]:
        rows[b[0]].add(name)
    plain = n_cards - sum(len(v) for v in rows.values())
    L = ["| a mana cost with | CR | surface | status | owner | cards | Scryfall | query (plus `BASE`) |",
         "|---|---|---|---|---|---:|---:|---|"]
    for bid, fam, cr, surface, status, key, pat, q in SYMBOLS:
        n = len(rows[bid]) if pat else plain
        live = fmt(count(q + " " + BASE)) if q else "--"
        L.append("| %s | %s | %s | %s | %s | %s | %s | %s |" % (
            esc(fam), cr, surface, status_cell(status, key, bid), owner_of(key, bid), fmt(n), live,
            "`%s`" % esc(q) if q else "the rest"))
    L.append("")
    L.append("A card lands in the first row its mana cost matches, so the cards column partitions the corpus; "
             "the Scryfall column counts every card the query matches, overlaps included.")
    return "\n".join(L)


def render_phrases():
    L = ["| family | CR | surface | status | owner | cards | query (plus `BASE`) |", "|---|---|---|---|---|---:|---|"]
    for pid, fam, cr, surface, status, key, q in MANA_PHRASES:
        L.append("| %s | %s | %s | %s | %s | %s | `%s` |" % (
            esc(fam), cr, surface, status_cell(status, key, pid), owner_of(key, pid), fmt(count(q + " " + BASE)), esc(q)))
    return "\n".join(L)


def render_overrides():
    L = ["| CR | the default | what a card says instead | cards | query (plus `BASE`) | surface | status | owner |",
         "|---|---|---|---:|---|---|---|---|"]
    for cr, default, instead, q, surface, status, key in OVERRIDES:
        L.append("| %s | %s | %s | %s | `%s` | %s | %s | %s |" % (
            cr, esc(default), esc(instead), fmt(count(q + " " + BASE)), esc(q), surface,
            status_cell(status, key, cr), owner_of(key, cr)))
    return "\n".join(L)


def render_priors():
    L = ["| the brief's prior | brief | reproduced | query | Commander-legal, `BASE` |",
         "|---|---:|---:|---|---:|"]
    for label, brief, q, base in PRIORS:
        n = count(q + " " + base)
        L.append("| %s | %s | %s | `%s` | %s |" % (esc(label), fmt(brief), fmt(n) + ("" if n == brief else " ≠"),
                                                     esc(q + " " + base), fmt(count(q + " " + BASE))))
    return "\n".join(L)


def render_questions():
    kinds = parse_variants(CHOICE_TYPES, "ChoiceKind")
    if set(kinds) != set(QUESTIONS):
        raise SystemExit("ChoiceKind no longer matches the census:\n  new: %s\n  gone: %s" % (
            sorted(set(kinds) - set(QUESTIONS)), sorted(set(QUESTIONS) - set(kinds))))
    explained = explained_kinds()
    claimed = sorted(k for k, v in QUESTIONS.items() if v[3] == "built")
    if explained != claimed:
        raise SystemExit("ui::why::refusals explains %s; the census says %s" % (explained, claimed))
    L = ["| `ChoiceKind` | its options | filtered | what a why reads | owner | slot |", "|---|---|---|---|---|---|"]
    for k in kinds:
        opts, filt, reads, key, slot = QUESTIONS[k]
        L.append("| `%s` | %s | %s | %s | %s | %s |" % (k, esc(opts), filt, esc(reads) or "--",
                                                     QUESTION_OWNERS[key], slot or "--"))
    rest = Counter(QUESTIONS[k][1] for k in kinds if k not in explained)
    head = ("%d kinds, parsed from `pub enum ChoiceKind` in `mtgsim/src/ui/choice_types.rs`; `ui::why::refusals` "
            "answers %d with a check (%s), read off its arms in `mtgsim/src/ui/why.rs`. Of the %d it does not, "
            "%d are **filtered**: a check drops a candidate the question could have named. %d are **bounds**: the "
            "options are amounts, and the why explains a limit. %d are **no**: every candidate is offered."
            % (len(kinds), len(explained), ", ".join("`%s`" % k for k in explained), len(kinds) - len(explained),
               rest["yes"], rest["bounds"], rest["no"]))
    return head + "\n\n" + "\n".join(L)


def calibrate():
    """The registered printings through the same classifiers: every family they
    use, and each one the engine is not recorded as playing."""
    reg = registered_cards()
    w = walk(reg["cards"])
    uses = defaultdict(set)
    for name, ctx, p, b in w["pieces"]:
        uses[("cost", b[0] if b else "?", b[4] if b else "?", b[5] if b else "")].add(name)
    for ab, c in w["abilities"]:
        for axis in ("who", "when", "where"):
            b = c[axis]
            uses[(axis, b[0], b[4], b[5])].add(ab["card"])
    for ab, (p, f) in w["mana"]:
        for b in (p, f):
            uses[("mana", b[0] if b else "?", b[4] if b else "?", b[5] if b else "")].add(ab["card"])
    for name, s, b in w["clauses"]:
        uses[("cast", b[0], b[4], b[5])].add(name)
    for name, b in w["symbols"]:
        uses[("symbol", b[0], b[4], b[5])].add(name)
    for k in KEYWORDS:
        cards = set()
        for x in k[2]:
            cards |= w["kw"].get(x, set())
        if cards:
            uses[("keyword", k[0], k[5], k[6])] |= cards
    return reg, uses


def render_calibration():
    reg, uses = calibrate()
    names_registered = [n for n, _, _ in registered()]
    L = ["%d names registered in `mtgsim/src/cards/registry.rs`; Scryfall knows %d of them, and %s." % (
        len(names_registered), len(reg["cards"]),
        "the rest are fixtures: " + ", ".join(reg["missing"]) if reg["missing"] else "none is a fixture")]
    L.append("")
    L.append("| table | family | status | registered cards that use it |")
    L.append("|---|---|---|---|")
    for (table, bid, status, key), cards in sorted(uses.items(), key=lambda kv: (kv[0][2] == "built", kv[0][0], kv[0][1])):
        shown = ", ".join(sorted(cards)[:6]) + (" and %d more" % (len(cards) - 6) if len(cards) > 6 else "")
        L.append("| %s | `%s` | %s | %s |" % (table, bid, status, esc(shown)))
    return "\n".join(L)


def summary(w):
    """Families by status, table by table."""
    tables = [("activation costs", COST_BUCKETS), ("who activates", WHO), ("when", WHEN), ("where", WHERE),
              ("mana production", PRODUCTION), ("what feeds a mana ability", FED), ("casting and playing", CAST),
              ("keyword families", KEYWORDS), ("mana symbols", SYMBOLS), ("mana by phrase", MANA_PHRASES)]
    L = ["| table | built | owned and slotted | found unowned | ... given an owner here | ... excluded here | still unowned |",
         "|---|---:|---:|---:|---:|---:|---:|"]
    tot = Counter()
    for title, rows in tables:
        c = Counter()
        for r in rows:
            status, key = (r[5], r[6]) if rows is KEYWORDS else (r[4], r[5])
            c[status] += 1
            if status == "unowned":
                new_owner = assigned(r[0], key)
                c["excluded here" if new_owner == "excluded" else "owned here" if new_owner else "still"] += 1
        tot.update(c)
        L.append("| %s | %d | %d | %d | %d | %d | %d |" % (title, c["built"], c["owned"], c["unowned"],
                                                         c["owned here"], c["excluded here"], c["still"]))
    L.append("| **all** | **%d** | **%d** | **%d** | **%d** | **%d** | **%d** |" % (
        tot["built"], tot["owned"], tot["unowned"], tot["owned here"], tot["excluded here"], tot["still"]))
    return "\n".join(L)


def splice(doc_text, key, body):
    begin, end = "<!-- cast-census: begin %s -->" % key, "<!-- cast-census: end %s -->" % key
    i, j = doc_text.index(begin) + len(begin), doc_text.index(end)
    return doc_text[:i] + "\n" + body + "\n" + doc_text[j:]


def residual(w):
    print("--- cost pieces no bucket caught ---")
    for name, ctx, p, b in w["pieces"]:
        if not b:
            print("  - %s (%s): %s" % (name, ctx, p))
    print("--- mana abilities no production bucket caught ---")
    for ab, (p, f) in w["mana"]:
        if not p:
            print("  - %s: %s: %s" % (ab["card"], ab["cost"], ab["effect"][:150]))


def show_bucket(w, bid):
    for name, ctx, p, b in w["pieces"]:
        if b and b[0] == bid:
            print("  - %s (%s): %s" % (name, ctx, p))
    for ab, c in w["abilities"]:
        if any(c[a][0] == bid for a in ("who", "when", "where")):
            print("  - %s: %s: %s" % (ab["card"], ab["cost"], ab["effect"][:150]))
    for ab, (p, f) in w["mana"]:
        if (p and p[0] == bid) or f[0] == bid:
            print("  - %s: %s: %s" % (ab["card"], ab["cost"], ab["effect"][:150]))
    for name, s, b in w["clauses"]:
        if b[0] == bid:
            print("  - %s: %s" % (name, s[:170]))
    for name, b in w["symbols"]:
        if b[0] == bid:
            print("  - %s" % name)


def main():
    global _no_fetch
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0],
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--write", action="store_true", help="splice the tables into cast-census.md")
    ap.add_argument("--residual", action="store_true", help="print what no bucket caught")
    ap.add_argument("--bucket", metavar="ID", help="print every clause in one bucket")
    ap.add_argument("--calibrate", action="store_true", help="the registered cards' families only")
    ap.add_argument("--names", metavar="Q", help="print the first page of names behind a Scryfall query")
    ap.add_argument("--refresh", action="store_true", help="drop the caches first")
    ap.add_argument("--no-fetch", action="store_true", help="never hit the network")
    args = ap.parse_args()
    _no_fetch = args.no_fetch
    if args.refresh:
        for p in (CACHE, COUNTS, REGISTERED_CACHE):
            if os.path.exists(p):
                os.remove(p)
    if args.names:
        names(args.names)
        return

    found, bad = audit_labels()
    if bad:
        raise SystemExit("a label cites a rule that is not in the CR: %s" % bad)

    d = corpus()
    w = walk(d["cards"])
    if args.residual:
        residual(w)
        return
    if args.bucket:
        show_bucket(w, args.bucket)
        return
    if args.calibrate:
        print(render_calibration())
        return

    blocks = {
        "CORPUS": "`%s`: %s cards (`total_cards` %s)." % (BASE, fmt(len(d["cards"])), fmt(count(BASE))),
        "SUMMARY": summary(w),
        "COSTS": render_costs(w),
        "ABILITIES": render_abilities(w),
        "MANA": render_mana(w),
        "CAST": render_cast(w),
        "KEYWORDS": render_keywords(w),
        "SYMBOLS": render_symbols(w, len(d["cards"])),
        "PHRASES": render_phrases(),
        "OVERRIDES": render_overrides(),
        "PRIORS": render_priors(),
        "QUESTIONS": render_questions(),
        "CALIBRATION": render_calibration(),
        "LABELS": "Every rule number the tables cite, resolved in `MTG-Rules/versions/tmnt.txt`: %s." % ", ".join(
            cr for cr, _ in found),
    }
    blocks["DATE"] = "Counted %s: the corpus fetched %s, the counts cache's newest entry %s, the tree as of the run." % (
        _dt.date.today().isoformat(), d["fetched"], max((v["fetched"] for v in (_counts or {}).values()), default="--"))
    if args.write:
        raw = open(DOC, "rb").read()
        eol = "\r\n" if raw.count(b"\r\n") > raw.count(b"\n") // 2 else "\n"
        text = raw.decode("utf-8").replace("\r\n", "\n")
        for k, v in blocks.items():
            text = splice(text, k, v)
        open(DOC, "wb").write(text.replace("\n", eol).encode("utf-8"))
        print("wrote", os.path.relpath(DOC, ROOT))
    else:
        for k, v in blocks.items():
            print("## %s\n\n%s\n" % (k, v))


if __name__ == "__main__":
    main()
