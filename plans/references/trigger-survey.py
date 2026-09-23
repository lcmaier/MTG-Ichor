#!/usr/bin/env python3
"""The trigger survey -- every table in `plans/references/trigger-survey.md`.

    python plans/references/trigger-survey.py             # print the tables, the query beside each count
    python plans/references/trigger-survey.py --write     # splice them into trigger-survey.md between its markers
    python plans/references/trigger-survey.py --boundary  # substring against regex-with-\\b, where it matters
    python plans/references/trigger-survey.py --names Q   # the first page of names behind a query, to read a bucket
    python plans/references/trigger-survey.py --refresh   # drop the Scryfall cache first
    python plans/references/trigger-survey.py --no-fetch  # cache only; a query the cache lacks is an error

WHY THIS EXISTS
---------------
`codebase-state.md` "Before Triggered abilities" item 2 asked, in 2026-08-24's
words, whether the event stream carries what triggers need. The triggers
architecture doc (`roadmap-v2.md` row A6) has to answer that before it decides
a shape, and an answer reconstructed from prose is one nobody re-checks. Every
number in the survey doc comes from here: Scryfall's `total_cards` for a
printed count, `plans/atomic-tests/spec.sqlite` for the corpus atoms behind a
rule, and the Rust tree for what the stream is today -- the `GameEvent` and
`GameAction` enums parsed for their variants and fields, the emit sites
counted, and the field a row claims an event carries **asserted against the
enum**, so a rename or an added field fails this script instead of leaving the
doc quietly wrong.

WHAT THE NUMBERS ARE NOT
------------------------
Card counts, not clause counts, and upper bounds: `o:` is a substring match
(`o:"when "` also matches "whenever ", and a card with two triggers counts once
per phrase it happens to contain). The query is printed beside every number
so the over-count is a known one. Scryfall's regex form honored `\\b` on
2026-09-18 -- `--boundary` prints the pairs -- and the tables still use the
substring form on purpose: the brief that commissioned this survey chose a
number anyone can reproduce over a cleaner one that depends on a feature the
API does not document as stable.

It counts; it does not judge. A row's "performed event today" column is a
reading of the tree by hand, checked by assertion where a field is named;
the "no event" verdicts and the gap list in the doc are the reader's.

WHEN TO DELETE THIS FILE
------------------------
When the triggers architecture doc has fixed the event stream's shape and
critical-path item 6 has landed, the survey is a record and this is its
instrument. Delete both together, or when the counts are stale enough that a
reader would trust a fresh look over them -- which a comment cannot prevent.

Results are cached in this directory (gitignored, `.census-*.json`). Scryfall
asks for a courteous request rate and bans a burst; the delay is deliberate.
"""
import argparse
import datetime as _dt
import json
import os
import re
import sqlite3
import subprocess
import sys
import time
import urllib.parse

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
DOC = os.path.join(HERE, "trigger-survey.md")
CACHE = os.path.join(HERE, ".census-triggers.json")
DB = os.path.join(ROOT, "plans", "atomic-tests", "spec.sqlite")
EVENT_RS = os.path.join(ROOT, "mtgsim", "src", "events", "event.rs")
ACTIONS_RS = os.path.join(ROOT, "mtgsim", "src", "engine", "actions.rs")
ZONES_RS = os.path.join(ROOT, "mtgsim", "src", "types", "zones.rs")
SRC = os.path.join(ROOT, "mtgsim", "src")
CARDS = os.path.join(SRC, "cards")

UA = "mtgsim-research/1.0 (contact: maiercluke@gmail.com)"
BASE = ' game:paper -is:funny'        # appended to every query; unique=cards is the API parameter
DELAY = 0.35
W = '(o:"when " or o:"whenever ")'    # "behind a trigger word"
TRIGGER = '(o:"when " or o:"whenever " or o:"at the beginning of")'

# --------------------------------------------------------------------------
# Scryfall
# --------------------------------------------------------------------------

_cache = None
_no_fetch = False


def _load_cache():
    global _cache
    if _cache is None:
        _cache = json.load(open(CACHE, encoding="utf-8")) if os.path.exists(CACHE) else {}
    return _cache


def _save_cache():
    json.dump(_cache, open(CACHE, "w", encoding="utf-8"), indent=1, sort_keys=True)


def _get(url):
    """curl with a UA header: Scryfall 403s a bare urllib request."""
    for attempt in range(6):
        out = subprocess.run(["curl", "-s", "-A", UA, url],
                             capture_output=True, text=True, encoding="utf-8").stdout
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


def count(q):
    """`total_cards` for `q + BASE`, cached by the full query string."""
    full = q + BASE
    c = _load_cache()
    if full in c:
        return c[full]["total"]
    if _no_fetch:
        raise SystemExit("--no-fetch, and the cache lacks: " + full)
    url = "https://api.scryfall.com/cards/search?q=%s&unique=cards" % urllib.parse.quote(full)
    d = _get(url)
    if d.get("object") == "error":
        # "didn't match any cards" is a count of zero, not a failure.
        if d.get("code") == "not_found":
            n = 0
        else:
            raise RuntimeError("%s -> %s" % (full, d.get("details")))
    else:
        n = d["total_cards"]
    c[full] = {"total": n, "fetched": _dt.date.today().isoformat()}
    _save_cache()
    time.sleep(DELAY)
    return n


def names(q, limit=40):
    url = "https://api.scryfall.com/cards/search?q=%s&unique=cards" % urllib.parse.quote(q + BASE)
    d = _get(url)
    if d.get("object") == "error":
        print(d.get("details"))
        return
    print("total:", d.get("total_cards"))
    for card in d.get("data", [])[:limit]:
        text = card.get("oracle_text") or " | ".join(
            f.get("oracle_text", "") for f in card.get("card_faces", []))
        print("  - %s :: %s" % (card["name"], text.replace("\n", " | ")[:200]))


def fetched_on():
    c = _load_cache()
    dates = sorted({v["fetched"] for v in c.values()})
    return dates[-1] if dates else _dt.date.today().isoformat()


# --------------------------------------------------------------------------
# The tree: enums, emit sites, registered triggers
# --------------------------------------------------------------------------

def parse_enum(path, name):
    """`{variant: [field, ...]}` for `pub enum <name>`, doc comments and
    attributes skipped. A unit variant maps to `[]`."""
    raw = open(path, encoding="utf-8").read().split("\n")
    start = next((i for i, l in enumerate(raw) if re.match(r"^pub enum %s\b" % re.escape(name), l)), None)
    if start is None:
        raise SystemExit("no `pub enum %s` in %s" % (name, path))
    # Comment and attribute lines go before the braces are counted: the doc
    # comments on `GameAction` spell mana symbols, which are braces.
    lines, depth = [], 0
    for l in raw[start:]:
        s = l.strip()
        if s.startswith(("//", "#[")):
            continue
        depth += l.count("{") - l.count("}")
        lines.append(l)
        if depth == 0 and lines[1:]:
            break
    lines = [l for l in lines[1:-1] if l.strip()]
    out, cur = {}, None
    for l in lines:
        s = l.strip()
        mv = re.match(r"^([A-Z]\w*)\s*(\{|,|\()", s)
        if mv and (l.startswith("    ") and not l.startswith("     ")):
            cur = mv.group(1)
            out[cur] = []
            if mv.group(2) == "{" and s.rstrip().endswith("},"):
                # one-line struct variant: `Tapped { object_id: ObjectId },`
                out[cur] = re.findall(r"(\w+)\s*:", s[s.index("{") + 1:s.rindex("}")])
            elif mv.group(2) == "(":
                out[cur] = ["0"]
            continue
        if cur is not None:
            mf = re.match(r"^(?:pub\s+)?(\w+)\s*:", s)
            if mf:
                out[cur].append(mf.group(1))
    return out


def emit_sites():
    """Every call through either door -- `emit_event(` and, since TR-1,
    `emit_event_unstamped(` -- in `mtgsim/src`, with the variant it emits."""
    sites = []
    for dirpath, _, files in os.walk(SRC):
        for f in files:
            if not f.endswith(".rs"):
                continue
            p = os.path.join(dirpath, f)
            lines = open(p, encoding="utf-8").read().split("\n")
            for i, l in enumerate(lines):
                if not re.search(r"emit_event(_unstamped)?\(", l) or "fn emit_event" in l or l.strip().startswith("//"):
                    continue
                window = "\n".join(lines[i:i + 4])
                mv = re.search(r"GameEvent::(\w+)", window)
                rel = os.path.relpath(p, ROOT).replace("\\", "/")
                sites.append((rel, i + 1, mv.group(1) if mv else "?"))
    return sites


def registered_triggers():
    n = 0
    for f in os.listdir(CARDS):
        if f.endswith(".rs"):
            n += open(os.path.join(CARDS, f), encoding="utf-8").read().count("AbilityType::Triggered")
    return n


# --------------------------------------------------------------------------
# The corpus
# --------------------------------------------------------------------------

def db():
    if not os.path.exists(DB):
        sys.path.insert(0, os.path.join(ROOT, "plans"))
        import specdb  # noqa: E402
        specdb.build()
    return sqlite3.connect(DB)


def atoms_for(conn, rules):
    """Atom ids whose `rule_num` is exactly one of `rules`."""
    out = []
    for r in rules:
        out += [a for (a,) in conn.execute(
            "SELECT id FROM atoms WHERE rule_num = ? ORDER BY id", (r,))]
    return out


def corpus_block(conn):
    q = lambda s, *a: conn.execute(s, a).fetchone()[0]
    total = q("SELECT COUNT(*) FROM atoms WHERE phase='Phase 7'")
    sections = q("SELECT COUNT(DISTINCT substr(rule_num,1,3)) FROM atoms WHERE phase='Phase 7'")
    r603 = q("SELECT COUNT(*) FROM atoms WHERE phase='Phase 7' AND rule_num LIKE '603%'")
    r603_all = q("SELECT COUNT(*) FROM atoms WHERE rule_num LIKE '603%'")
    full = q("SELECT COUNT(DISTINCT c.atom_id) FROM coverage c JOIN atoms a ON a.id=c.atom_id "
             "WHERE a.phase='Phase 7' AND c.partial=0")
    partial = conn.execute(
        "SELECT DISTINCT c.atom_id, c.test_name FROM coverage c JOIN atoms a ON a.id=c.atom_id "
        "WHERE a.phase='Phase 7' AND c.partial=1 AND c.atom_id NOT IN "
        "(SELECT atom_id FROM coverage WHERE partial=0) ORDER BY c.atom_id").fetchall()
    return {"total": total, "sections": sections, "r603": r603, "r603_all": r603_all,
            "full": full, "partial": partial}


# --------------------------------------------------------------------------
# Table 1 -- the CR's own vocabulary, CR 603.1b through 603.12a
# --------------------------------------------------------------------------
# (rule(s) for the atom lookup, rule label, the event as the CR words it, shape, query or None, note)

T1 = [
    (["603.1b"], "603.1b", "more than one trigger condition, and \"all\" of them in a period",
     "multi-condition (turn history)", 'o:"done all"',
     "Avatar Aang (2025) prints the form: four bending conditions and \"if you've done all four this turn\", which its ruling reads over the whole turn whether or not Aang was there -- so the tracker is per player per turn, not per source"),
    (["603.2"], "603.2", "a game event or a game state matches the trigger event", "umbrella", TRIGGER,
     "every card that carries a trigger; the base every other row is a share of"),
    (["603.2b"], "603.2b", "a phase or step begins -- \"at the beginning of\"", "per event", 'o:"at the beginning of"',
     "the step rows in table 2 split it"),
    (["603.2c"], "603.2c", "one event with several occurrences -- once per occurrence, or once for \"one or more\"",
     "one-or-more", 'o:"whenever" o:"one or more"', "the batch (`BatchId`) is the boundary"),
    (["603.2d"], "603.2d", "a triggered ability triggers additional times", "count modifier",
     'o:"triggers an additional time"',
     "not an event: a multiplier read as the ability triggers. Panharmonicon's ruling draws its edges -- the object's own triggered abilities only, never CR 603.6d's \"enters\" statics or a replacement effect; the rule's last sentence excludes the delayed and reflexive triggers those abilities create; and an ability that \"triggers only once each turn\" is not doubled (the row below 603.2h)"),
    (["603.2e"], "603.2e", "\"becomes\" -- tapped, untapped, attached, blocked: the transition only", "per transition",
     '(o:"becomes tapped" or o:"becomes untapped" or o:"becomes attached" or o:"becomes blocked")',
     "\"becomes the target\" and \"becomes unattached\" have rows of their own below"),
    (["603.2f"], "603.2f", "the object with the ability is at no time visible to all players -- it does not trigger",
     "visibility gate", None,
     "not an event; a gate on every row. It answers `atomic-tests/supplemental-docs/603-2f-complexity.md`: Guerrilla Tactics discarded onto the library under Library of Leng is never visible and does not trigger, and under Future Sight the top card is revealed and it does -- visibility is per object, not per zone (S1). The gate is a *global* bit, \"visible to all players\" at the instant after the event, a subset of `backlog.md` §2.9's per-viewer query; that entry weighed moving up at RE-8's close and stayed as `roadmap-v2.md` B4 (1-2 PRs, anywhere in A or B, back-stopped before Phase 8's reveal cards), and until a reveal exists no hidden-zone object is visible, so `Zone::is_public()` is exact today and the doc names the predicate per object for B4 to fill"),
    (["603.2g"], "603.2g", "a prevented or replaced event never happened", "stream property", None,
     "why the matcher reads the *performed* stream and nothing upstream of it"),
    (["603.2h"], "603.2h", "\"Do this only once each turn\"", "per-turn action gate",
     'o:"do this only once each turn"',
     "the action-taken gate, per source object: Nykthos Paragon's rulings have every life gain trigger it until the action is taken, one of two instances on the stack act (ATOM-603.2h-002), and two Paragons act twice; Panharmonicon can double it, since the extra instance just does nothing"),
    ([], "(no rule; CR 702.179d's speed is the baseline CR's one use)",
     "\"This ability triggers only once each turn\" -- a cap on triggering, per source object", "per-turn trigger gate",
     'o:"triggers only once each turn"',
     "no rule of its own in the baseline CR, so the earliest printing's ruling is the definition -- Elvish Warmaster (Kaldheim, found with `order:released direction:asc`): \"Once the triggered ability has triggered once during a turn, it can't trigger again, even if the triggered ability is still on the stack, has been countered, or has otherwise left the stack.\" The later rulings fill in the edges (Jin-Gitaxias: once, not once per opponent; Tyvar: once per creature it is granted to; Fang and Stonebinder's Familiar: once for a batch) and the judge literature adds that Panharmonicon cannot double it. A flag set as the ability triggers: a second tracker beside 603.2h's, written by the dispatcher rather than by the resolution -- question 16"),
    (["603.3b"], "603.3b", "another ability triggering -- the second APNAP tier", "per event",
     'o:"causes a triggered ability to trigger"', "Strict Proctor's shape: the trigger event is a trigger"),
    (["603.4"], "603.4", "intervening \"if\" -- the condition read as the event happens and again at resolution",
     "intervening-if", TRIGGER + ' o:", if "',
     "a comma-if anywhere behind a trigger word; the regex reading is §7's pair 6"),
    (["603.5"], "603.5", "\"may\" and \"unless\" -- the choice is made at resolution, the ability stacks regardless",
     "optional", None, "not an event; a `ChoiceKind` the phase adds (A4j)"),
    (["603.6", "603.6a"], "603.6a", "a permanent enters the battlefield", "per event (zone change)",
     W + ' o:"enters"', "`o:\"enters\"` alone also matches CR 603.6d's \"enters\" statics, which are not triggers -- the line Panharmonicon's ruling draws, since CR 603.2d doubles the triggers and never the statics"),
    (["603.6c"], "603.6c", "a permanent leaves the battlefield, including \"dies\" (CR 700.4: put into a graveyard from the battlefield)",
     "look-back (603.10a)", W + ' (o:"leaves the battlefield" or o:"dies" or o:"put into a graveyard from the battlefield")',
     "one row, not two, because CR 700.4 makes the phrases one event"),
    (["603.6c"], "603.6c", "a phased-in permanent leaves the game because its owner leaves", "look-back (603.10a)", None,
     "no printed phrase to search; `GameEvent::LeftTheGame` carries the frame and item 6 owns the qualifier"),
    (["603.6c"], "603.6c", "put into a zone \"from anywhere\" -- never a leaves-the-battlefield ability",
     "per event (zone change)", W + ' o:"from anywhere"',
     "the row that must *not* look back through 603.10a's first class: Guile's ruling has it trigger from the graveyard even when Lignify took the ability on the battlefield, and not when Yixlid Jailer takes it in the graveyard. A library or hand destination is 603.10a's third class instead, and that one does look back"),
    (["603.6e"], "603.6e", "the enchanted permanent leaves the battlefield -- an Aura's own trigger",
     "look-back (400.7e/f)", W + ' o:"enchanted" (o:"dies" or o:"leaves the battlefield")',
     "finds both new objects: the card and the Aura in its graveyard"),
    (["603.7", "603.7a"], "603.7", "a delayed triggered ability -- \"at the beginning of the next ...\", \"when this creature becomes untapped\"",
     "delayed", 'o:"at the beginning of the next"', "created at resolution, never retroactive (603.7a); CR 513.2's next-turn rule"),
    (["603.7b"], "603.7b", "once -- the next time its event occurs -- unless a stated duration; simultaneous events, the controller chooses",
     "delayed", None,
     "not searched: the first draft's `o:\"the next time\"` counts CR 615's shields (\"the next time ... would deal damage ... prevent\"), not delayed triggers, and was withdrawn; ATOM-603.7b-002 is the simultaneous case (Tatsumasa under a doubler)"),
    (["603.7c"], "603.7c", "a delayed trigger tracks its object through characteristic changes, and loses it at a zone change (CR 400.7)",
     "delayed: identity", None, "an object reference, not a filter -- the same identity question as 603.6's zone-change triggers"),
    (["603.7d"], "603.7d", "created by a spell: the source is the spell, the controller whoever controlled it as it resolved",
     "delayed: provenance", None, "the spell's stack object is gone by the time the trigger fires (CR 608.2n), so the source is a remembered identity"),
    (["603.7e"], "603.7e", "created by an activated or triggered ability: the source is that ability's source",
     "delayed: provenance", None, "inherits `AbilityIdentity`'s source half"),
    (["603.7f"], "603.7f", "created by a static ability's replacement effect: the source is the object with the static ability, the controller its controller as the replacement applied",
     "delayed: provenance", None, "the pipeline is the producer and carries no resolution stamp -- question 14"),
    (["603.7g"], "603.7g", "created by a static ability that let a player take an action: the source is that object, the controller its controller as the action was taken",
     "delayed: provenance", None, "a special action is the producer -- question 14"),
    (["603.7h"], "603.7h", "the ability that created it has resolved N times this turn", "delayed, counted",
     'o:"time this ability has resolved this turn"', "Ashling's shape; the count is per instance or per ability (S3)"),
    (["603.8"], "603.8", "a game state matches -- a state trigger", "state",
     '(o:"when you control no" or o:"whenever you control no" or o:"when there are no" or o:"whenever there are no" or o:"when you have no" or o:"whenever you have no")',
     "no event by definition; P1's mid-resolution check"),
    (["603.9"], "603.9 / 603.10f", "a player loses the game, or leaves it other than by a draw", "look-back",
     W + ' o:"loses the game"', "over-count: also matches \"you lose the game\" effects behind an unrelated trigger"),
    (["603.10a"], "603.10a", "a card leaves a graveyard", "look-back", W + ' (o:"leaves your graveyard" or o:"leaves a graveyard" or o:"leave your graveyard" or o:"leave a graveyard")',
     "the second of 603.10a's three classes"),
    (["603.10a"], "603.10a", "an object all players can see is put into a hand or library", "look-back",
     '(o:"put into a library from" or o:"put into your hand from" or o:"is returned to your hand" or o:"is returned to its owner\'s hand" or o:"are put into a library") ' + W,
     "the third class, and printed: Wan Shi Tong and Dutiful Knowledge Seeker (\"put into a library from anywhere\"), Golgari Brownscale (into your hand from your graveyard), Stormfront Riders (returned to your hand from the battlefield -- its ruling has it trigger for itself when bounced with another). A custom card can name the class outright, which is why the row carries a query rather than \"not searched\""),
    (["603.10b"], "603.10b", "a permanent phases out", "look-back", 'o:"phases out"', "phasing is unbuilt"),
    (["603.10c"], "603.10c", "an object becomes unattached", "look-back", 'o:"becomes unattached"', ""),
    (["603.10d"], "603.10d", "a player loses control of an object, or an opponent gains control of it from them",
     "look-back", W + ' (o:"gains control" or o:"gain control" or o:"loses control" or o:"lose control")', ""),
    (["603.10e"], "603.10e", "a spell is countered", "look-back", W + ' o:"countered"',
     "over-count: \"can't be countered\" behind an unrelated trigger"),
    (["603.10g"], "603.10g", "a player planeswalks away from a plane", "out of scope", None, "Planechase is excluded"),
    (["603.11"], "603.11", "a static ability linked to a triggered one -- \"whenever you reveal ... this way\"",
     "per event", 'o:"whenever you reveal"', "CR 607's linkage; the trigger condition sits mid-paragraph"),
    (["603.12"], "603.12", "reflexive -- \"when you do\", \"when [something happens] this way\"", "reflexive",
     'o:"when you do"', "checked immediately after creation, against the creating resolution's own events"),
    (["603.12a"], "603.12a", "\"when you pay [that cost] one or more times\" -- once, however many times", "reflexive",
     'o:"one or more times"', ""),
]

# --------------------------------------------------------------------------
# Table 2 -- the distribution, against the performed event that carries it
# --------------------------------------------------------------------------
# Each row: (trigger event, CR, query, GameEvent variant(s), reads, note)
#   `reads` is a list of (field-as-the-condition-needs-it, where) with `where` one of
#     "V.field"   -- on the record; asserted to exist on variant V
#     "!V.field"  -- NOT on the record; asserted absent (the doc's field gaps)
#     "gap: ..."  -- not on the record, and no single field would carry it
#     "live: ..." -- read off the live state at dispatch, which synchronous detection allows
#     "tracker"   -- item 42's per-player turn summaries, which wait for the doc
#     "stamp"     -- the EventStamp envelope (batch / resolution)

T2 = [
    ("enters the battlefield", "603.6a", W + ' o:"enters"', ["PermanentEnteredBattlefield", "ZoneChange", "TokenCreated"],
     [("the permanent", "PermanentEnteredBattlefield.object_id"), ("under whose control", "PermanentEnteredBattlefield.controller"),
      ("its types, the moment it is there (603.6b)", "live: the layer walk"), ("the zone it came from", "ZoneChange.from"),
      ("created rather than moved (111.13)", "TokenCreated.zone")],
     "carried across two records: the zone change or the creation, then the entry; a token's entry has no `from`"),
    ("attacks / is attacked / attacks with / attacks alone", "508.3a-e", W + ' o:"attacks"', ["AttackersDeclared"],
     [("which creatures", "AttackersDeclared.attackers"), ("whom each attacks", "!AttackersDeclared.defender"),
      ("whom each attacks (today)", "live: `AttackingInfo.target`"), ("the attacking player", "live: the active player"),
      ("alone", "AttackersDeclared.attackers")],
     "**field gap**: 508.3a's \"attacks [a player]\", 508.3b's \"is attacked\" and 508.3e's \"attacks another player\" read the defender, which the record does not carry"),
    ("casts a spell", "601.2i", 'o:"whenever" o:"cast"', ["SpellCast"],
     [("the spell", "SpellCast.spell_id"), ("who cast it", "SpellCast.caster"),
      ("creature spell, mana value, colors", "live: the stack object"), ("the zone it was cast from", "live: the stack entry's `cast_from`"),
      ("first / second spell this turn", "tracker")],
     "carried; a copy is not cast (CR 707.10) and CV's copy path must emit no `SpellCast`"),
    ("dies", "700.4 / 603.6c", W + ' o:"dies"', ["ZoneChange"],
     [("from the battlefield to a graveyard", "ZoneChange.from"), ("why", "ZoneChange.cause"),
      ("what it was, and whose (603.10a)", "ZoneChange.lki")],
     "carried; the frame is CR 603.10a's look-back, captured before CR 611.2a drops the registry rows"),
    ("draws a card", "121.1", 'o:"whenever" o:"draw"', ["CardDrawn"],
     [("who", "CardDrawn.player_id"), ("which card", "CardDrawn.card_id"), ("first or second card this turn (miracle, 702.94a)", "tracker")],
     "carried; CR 121.5 is why this is not the library-to-hand zone change. Transcendent Archaic is the subtlety: its ETB draws X, the colors spent to cast the spell that became it (CR 400.7d, information the permanent keeps about its own casting), and \"if you draw one or more cards this way\" reads the count the performed draws returned, not the stream"),
    ("at the beginning of upkeep", "603.2b / 500.6", 'o:"at the beginning of" o:"upkeep"', ["StepBegin"],
     [("which step", "StepBegin.step"), ("whose turn", "StepBegin.player")],
     "carried since TR-1 (item 10): the performer writes the `player` `GameAction::BeginStep` carries -- \"your upkeep\" and \"each opponent's upkeep\" read it"),
    ("deals damage / deals combat damage", "120.4b", W + ' (o:"deals damage" or o:"deals combat damage")', ["DamageDealt"],
     [("the source", "DamageDealt.source_id"), ("the recipient", "DamageDealt.target"), ("how much", "DamageDealt.amount"),
      ("combat or not", "DamageDealt.is_combat"), ("the source's controller and types", "live: the source, still on the battlefield until SBAs")],
     "carried since TR-1 (item 10); the next row is the share that reads `is_combat`"),
    ("deals combat damage", "510.2 / 120.4b", W + ' o:"deals combat damage"', ["DamageDealt"],
     [("combat or not", "DamageDealt.is_combat")],
     "the share of the row above that reads the field item 10 added"),
    ("at the beginning of the end step", "513.1 / 513.2", 'o:"at the beginning of" o:"end step"', ["StepBegin"],
     [("which step", "StepBegin.step"), ("whose turn", "StepBegin.player")],
     "carried, as the upkeep row is; CR 513.2's next-turn rule is §14's question 1"),
    ("intervening \"if\"", "603.4", TRIGGER + ' o:", if "', ["(any)"],
     [("the condition, as the event is performed", "live: the board then"), ("the condition again, as the ability resolves (608.2a)", "live: the board then")],
     "no event of its own: a predicate the matcher evaluates twice"),
    ("sacrifices", "701.17", 'o:"whenever" o:"sacrifice"', ["ZoneChange"],
     [("that it was a sacrifice", "ZoneChange.cause"), ("who sacrificed it", "ZoneChange.lki")],
     "carried; `ZoneChangeCause::Sacrificed`, and the frame's `controller` is the player who sacrificed"),
    ("one or more (a batch)", "603.2c", 'o:"whenever" o:"one or more"', ["(any)"],
     [("the events performed as one", "stamp")],
     "carried by the envelope: every record in a batch carries one `BatchId`"),
    ("discards", "701.8", 'o:"whenever" o:"discard"', ["ZoneChange"],
     [("that it was a discard", "ZoneChange.cause"), ("who", "ZoneChange.owner")],
     "carried; \"cycles or discards\" (702.29d) waits for cycling"),
    ("at the beginning of the next ... (delayed)", "603.7", 'o:"at the beginning of the next"', ["StepBegin", "PhaseBegin", "TurnBegin"],
     [("the step", "StepBegin.step"), ("whose turn", "StepBegin.player"), ("\"next\" -- not this one (513.2)", "live: when the ability was created")],
     "carried by the step rows; the creation instant is the delayed ability's own field"),
    ("dies with or without counters, or while attached -- the frame's status", "702.79a / 702.93a / 603.6e / 603.10c",
     '(kw:persist or kw:undying or o:"enchanted creature dies" or o:"becomes unattached")', ["ZoneChange"],
     [("what it was, and whose", "ZoneChange.lki"), ("the counters it had, what it was attached to, whether it was tapped",
      "gap: `EffectiveCharacteristics` carries characteristics and the controller, no status")],
     "**field gap**: persist's and undying's intervening-if read the counters the permanent had as it died; an Aura's 603.6e trigger reads what it enchanted"),
    ("leaves the battlefield", "603.6c", W + ' o:"leaves the battlefield"', ["ZoneChange", "LeftTheGame"],
     [("from the battlefield", "ZoneChange.from"), ("what it was (603.10a)", "ZoneChange.lki"), ("left the game with its owner", "LeftTheGame.lki")],
     "carried on both routes; the phased-in qualifier is item 6"),
    ("at the beginning of combat", "603.2b / 506.1", 'o:"beginning of combat"', ["StepBegin", "PhaseBegin"],
     [("the step", "StepBegin.step"), ("whose turn", "StepBegin.player")],
     "carried, as the upkeep row is; \"at end of combat\" (511.2) is the end-of-combat step beginning"),
    ("blocks / blocks a creature / becomes blocked / becomes blocked by", "509.3a-d", W + ' o:"blocks"', ["BlockersDeclared"],
     [("the (blocker, attacker) pairs", "BlockersDeclared.blockers")],
     "carried; the four shapes are four readings of one list (CR 700.1's example: one event or two)"),
    ("gains life", "119.9", 'o:"whenever" o:"gain life"', ["LifeChanged"],
     [("who", "LifeChanged.player_id"), ("how much", "LifeChanged.old"), ("the source", "LifeChanged.source")],
     "carried; one record per source event, which is CR 702.15e's two lifelink triggers"),
    ("loses life", "120.3a", 'o:"whenever" (o:"lose life" or o:"loses life")', ["LifeChanged"],
     [("who", "LifeChanged.player_id"), ("how much", "LifeChanged.new"), ("from damage, a payment or an effect (727.1a)", "LifeChanged.cause")],
     "carried; one record per `LoseLife` proposal -- combat damage from two attackers is two (§14's question 3: per record); `cause` since TR-1 (item 10)"),
    ("becomes the target", "115.1 / 603.2e", 'o:"becomes the target"', [],
     [("the object targeted", "no event"), ("by which spell or ability, whose", "no event")],
     "**no event**: targets are chosen at CR 601.2c and announced to nothing"),
    ("ward -- becomes the target of an opponent's spell or ability", "702.21a", 'kw:ward', [],
     [("the object targeted, and who controls the targeting spell", "no event")],
     "**no event**: the same gap, with a keyword's population behind it"),
    ("becomes tapped", "603.2e", 'o:"becomes tapped"', ["Tapped"],
     [("the permanent", "Tapped.object_id")],
     "carried; emitted on the transition only (CR 603.2e), never for a redundant tap or an entry"),
    ("becomes untapped", "603.2e", 'o:"becomes untapped"', ["Untapped"],
     [("the permanent", "Untapped.object_id")],
     "carried, as tapping is"),
    ("is dealt damage", "120.4b", W + ' o:"is dealt damage"', ["DamageDealt"],
     [("the recipient", "DamageDealt.target"), ("excess damage (120.10)", "live: toughness and marked damage")],
     "carried"),
    ("tapped for mana", "106.12a", 'o:"tapped for mana"', ["ManaAdded"],
     [("that the source was tapped for it", "ManaAdded.tapped_for_mana"), ("what mana", "ManaAdded.mana")],
     "carried; CR 605.1b makes the trigger a mana ability that skips the stack (main item 11)"),
    ("counters are put on / the Nth counter", "122.6 / 122.7", 'o:"whenever" (o:"counter is put" or o:"counters are put" or o:"put one or more")', ["CountersChanged"],
     [("the subject", "CountersChanged.subject"), ("how many", "CountersChanged.added"), ("fewer than N before, N or more after", "live: the count now, minus `added`"),
      ("counters it entered with (122.6)", "gap: the entry announces no `CountersChanged` and `PermanentEnteredBattlefield` carries no `mods`")],
     "carried for a permanent on the battlefield; CR 122.6 makes entry counters \"put on\" it too -- a question"),
    ("activates an ability", "602.2a", W + ' (o:"activate an ability" or o:"activates an ability" or o:"activate a loyalty" or o:"activates a loyalty")', ["AbilityActivated"],
     [("which ability", "AbilityActivated.identity"), ("who", "AbilityActivated.controller")],
     "carried"),
    ("an ability resolves / has resolved N times", "608.2p / 603.7h", 'o:"time this ability has resolved this turn"', ["AbilityResolved", "ZoneChange"],
     [("which ability, durably", "AbilityResolved.identity"), ("a spell resolving", "ZoneChange.cause"), ("this instance or this ability (S3)", "tracker")],
     "carried; the per-turn count is the tracker's and its key is S3's decision"),
    ("is countered", "603.10e", W + ' o:"countered"', ["SpellCountered", "AbilityCountered", "SpellFizzled"],
     [("the spell", "SpellCountered.spell_id"), ("by what", "SpellCountered.countered_by")],
     "carried; CR 608.2b (tmnt) says a spell whose targets are all illegal \"doesn't resolve\" -- `SpellFizzled` is not this trigger's event, a question for the doc"),
    ("loses the game", "603.9", W + ' o:"loses the game"', ["PlayerLost"],
     [("who", "PlayerLost.player_id"), ("why", "PlayerLost.reason")],
     "carried; the 800.4d refusal at placement is item 7"),
    ("creates a token", "701.7a / 111.13", '(o:"whenever you create" or o:"whenever a player creates" or o:"whenever an opponent creates" or o:"token is created" or o:"tokens are created")', ["TokenCreated"],
     [("the token, where", "TokenCreated.zone"), ("whose", "TokenCreated.owner")],
     "carried; item 8 -- keyed on this event, never on `is_token` at entry"),
    ("scries / surveils", "701.22d / 701.25d", '(o:"whenever you scry" or o:"whenever you surveil" or o:"scries" or o:"surveils")', ["Scried"],
     [("who", "Scried.player_id"), ("how many, and how many really (Elrond)", "Scried.looked_at")],
     "carried for scry; surveil is unbuilt"),
    ("shuffles a library", "701.24e", '(o:"whenever you shuffle" or o:"shuffles")', ["LibraryShuffled"],
     [("whose", "LibraryShuffled.player_id")],
     "carried; one record per shuffle (701.24f)"),
    ("mills / put into a graveyard from anywhere / leaves a graveyard", "701.17a / 603.6c / 603.10a", '(o:"whenever you mill" or o:"mills" or o:"from anywhere" or o:"leaves your graveyard" or o:"leaves a graveyard")', ["ZoneChange"],
     [("the move and why", "ZoneChange.cause"), ("what it was, off the battlefield (603.10a's second class)", "gap: `lki` is `None` unless `from` is the battlefield")],
     "carried by cause; the `lki` frame is captured only for a battlefield departure, so 603.10a's graveyard-leaving class has no frame"),
    ("is exiled", "701.13", '(o:"whenever you exile" or o:"whenever a player exiles" or o:"is exiled from" or o:"are exiled from" or o:"card is exiled" or o:"cards are exiled")', ["ZoneChange"],
     [("the move and why", "ZoneChange.cause")],
     "carried; `ZoneChangeCause::Exiled`"),
    ("plays a land", "305.1", W + ' o:"play a land"', ["ZoneChange", "PermanentEnteredBattlefield"],
     [("played rather than put", "ZoneChange.cause")],
     "carried; `ZoneChangeCause::PlayedAsLand`"),
    ("becomes attached", "603.2e / 701.3a", 'o:"becomes attached"', ["Attached"],
     [("the attachment and its host", "Attached.host"), ("moved from another host", "Attached.former_host")],
     "carried on the transition only (701.3b)"),
    ("becomes unattached", "603.10c", 'o:"becomes unattached"', ["Attached", "EquipmentDetached", "ZoneChange"],
     [("left a host for another", "Attached.former_host"), ("dropped by CR 704.5n", "EquipmentDetached.former_host"),
      ("the host left the battlefield", "live: the host's `ZoneChange` and the attachment's own frame")],
     "three routes and no one record; the third is a look-back with no \"unattached\" line -- a question for the doc"),
    ("gains control / loses control", "603.10d", W + ' (o:"gains control" or o:"gain control" or o:"loses control" or o:"lose control")', [],
     [("the object, the old and the new controller", "no event")],
     "**no event**: control changes are Layer 2 continuous effects and a control-changing `Primitive` writes a registry row, which is not a performed event"),
    ("phases out", "603.10b / 702.26", 'o:"phases out"', [],
     [("the permanent", "no event")],
     "**no event, and no action**: phasing is unbuilt (`codebase-state.md`, \"Phasing\")"),
    ("searches a library", "701.23f", '(o:"whenever you search" or o:"whenever a player searches" or o:"searches")', [],
     [("who searched which library", "no event")],
     "**no event, and no action**: a search is not a `GameAction`; the found card's move is"),
    ("damage is prevented", "615.13", W + ' o:"prevented"', [],
     [("that a prevention effect applied", "no event")],
     "**no event**: a shield applying is a CR 616.1 trace record, not a performed event (615.13 wants one per prevention applied)"),
    ("state triggers", "603.8", '(o:"when you control no" or o:"whenever you control no" or o:"when there are no" or o:"whenever there are no" or o:"when you have no" or o:"whenever you have no")', [],
     [("the state", "live: a predicate at every dispatch (P1)")],
     "no event by definition"),
    ("reflexive -- \"when you do\"", "603.12", 'o:"when you do"', ["(the creating resolution's own records)"],
     [("the action, inside the creating resolution", "stamp")],
     "carried: the records a resolution performed carry its `ResolutionStamp`"),
    ("turn history: first / second spell, second card, last turn, this game", "603.1b / item 42",
     '(o:"first spell" or o:"second spell" or o:"second card" or o:"last turn" or o:"this game")', [],
     [("what a player did this turn, last turn, this game", "tracker")],
     "item 42's per-player turn summaries, which wait for the doc (P2-P4). \"This game\" is a scope, not a window: Commander's Insight counts commander casts from the command zone this game, which is CR 903.8's own counter -- question 15"),
    ("keyword actions with no engine action: turned face up, transforms, cycles, explores, crews, expends, commits a crime",
     "701 / 702 / 700.13-14", W + ' (o:"turned face up" or o:"transforms" or o:"cycle" or o:"explores" or o:"becomes crewed" or o:"expend" or o:"commit a crime")', [],
     [("the keyword action", "no event")],
     "**no action, so no event**: Phase 8's breadth, one event per keyword as the keyword lands"),
    ("inherent triggers with no source: the monarch, the initiative, rad counters", "724.2 / 725.2 / 727.1",
     '(o:"the monarch" or o:"the initiative" or o:"rad counter")', ["StepBegin", "DamageDealt"],
     [("the events", "StepBegin.step"), ("the source", "no source -- CR 113.8's exception")],
     "the events exist; the *source* does not, and `AbilityIdentity` has no arm for a rule-owned ability -- a question for the doc"),
]

# --------------------------------------------------------------------------
# Rendering
# --------------------------------------------------------------------------

def fmt(n):
    return "{:,}".format(n) if isinstance(n, int) else n


def esc(s):
    return s.replace("|", "\\|")


def check_fields(events):
    """Every `V.field` in T2 exists on the enum; every `!V.field` does not."""
    problems = []
    for row in T2:
        for _, where in row[4]:
            if where.startswith(("live:", "gap:", "no source")) or where in ("tracker", "stamp", "no event"):
                continue
            absent = where.startswith("!")
            v, f = where.lstrip("!").split(".", 1)
            if v not in events:
                problems.append("%s: no variant %s" % (row[0], v))
            elif absent and f in events[v]:
                problems.append("%s: %s.%s exists now -- the doc's field gap has closed, re-read the row" % (row[0], v, f))
            elif not absent and f not in events[v]:
                problems.append("%s: %s has no field %s" % (row[0], v, f))
        for v in row[3]:
            if v not in events and not v.startswith("("):
                problems.append("%s: no variant %s" % (row[0], v))
    if problems:
        raise SystemExit("event.rs no longer matches the survey:\n  " + "\n  ".join(problems))


def render_stream(events, actions, causes, sites, ntrig):
    emitted = {}
    for _, _, v in sites:
        emitted[v] = emitted.get(v, 0) + 1
    never = sorted(v for v in events if v not in emitted)
    L = []
    L.append("| what | count | reproduced by |")
    L.append("|---|---:|---|")
    L.append("| `GameEvent` variants | %d | `pub enum GameEvent` in `mtgsim/src/events/event.rs`, parsed |" % len(events))
    L.append("| ... emitted somewhere | %d | an emit call naming the variant within four lines |" % (len(events) - len(never)))
    L.append("| ... never emitted | %d | %s |" % (len(never), ", ".join("`%s`" % v for v in never)))
    L.append("| emit call sites | %d | `emit_event(` and `emit_event_unstamped(` in `mtgsim/src`, the definitions and comment lines excluded |" % len(sites))
    L.append("| `GameAction` variants | %d | `pub enum GameAction` in `mtgsim/src/engine/actions.rs`, parsed |" % len(actions))
    L.append("| `ZoneChangeCause` arms | %d | `pub enum ZoneChangeCause` in `mtgsim/src/types/zones.rs`, parsed |" % len(causes))
    L.append("| registered cards with a triggered ability | %d | `AbilityType::Triggered` in `mtgsim/src/cards/*.rs` |" % ntrig)
    return "\n".join(L)


def render_corpus(c):
    L = []
    L.append("| what | count | reproduced by |")
    L.append("|---|---:|---|")
    L.append("| Phase 7 atoms | %d | `SELECT COUNT(*) FROM atoms WHERE phase='Phase 7'` |" % c["total"])
    L.append("| ... CR sections they span | %d | `COUNT(DISTINCT substr(rule_num,1,3))`, same filter |" % c["sections"])
    L.append("| ... under CR 603 | %d | `rule_num LIKE '603%%'`, same filter (%d in the corpus at any phase) |" % (c["r603"], c["r603_all"]))
    L.append("| ... fully covered | %d | `coverage.partial = 0` |" % c["full"])
    L.append("| ... partially covered, by a test that does not build the atom | %d | `coverage.partial = 1` and no full row |" % len(c["partial"]))
    for aid, test in c["partial"]:
        L.append("| | | `%s` -- `%s` |" % (aid, test))
    return "\n".join(L)


def render_t1(conn):
    L = []
    L.append("| CR | the event, as the CR words it | shape | corpus atoms | cards | Scryfall query (plus `%s`, `unique=cards`) |" % BASE.strip())
    L.append("|---|---|---|---|---:|---|")
    for rules, label, event, shape, q, note in T1:
        atoms = atoms_for(conn, rules)
        n = fmt(count(q)) if q else "--"
        qq = "`%s`" % esc(q) if q else "not searched"
        cell = ", ".join(a.replace("ATOM-", "") for a in atoms) if atoms else "none"
        ev = esc(event) + (" -- " + esc(note) if note else "")
        L.append("| %s | %s | %s | %s | %s | %s |" % (label, ev, shape, cell, n, qq))
    return "\n".join(L)


def render_t2(events):
    L = []
    L.append("| trigger event | CR | cards | performed event today | what the condition reads, and where it is | note |")
    L.append("|---|---|---:|---|---|---|")
    for event, cr, q, variants, reads, note in T2:
        n = fmt(count(q))
        pe = ", ".join(v if v.startswith("(") else "`%s`" % v for v in variants) if variants else "**none**"
        parts = []
        for need, where in reads:
            if where.startswith("!"):
                v, f = where[1:].split(".", 1)
                parts.append("%s: **not on the record** (`%s` has no `%s`)" % (need, v, f))
            elif where.startswith("live:"):
                parts.append("%s: live (%s)" % (need, where[5:].strip()))
            elif where.startswith("gap:"):
                parts.append("%s: **not on the record** (%s)" % (need, where[4:].strip()))
            elif where == "tracker":
                parts.append("%s: a turn tracker (item 42)" % need)
            elif where == "stamp":
                parts.append("%s: the `EventStamp` envelope" % need)
            elif where == "no event" or where.startswith("no source"):
                parts.append("%s: %s" % (need, where))
            else:
                parts.append("%s: `%s`" % (need, where))
        L.append("| %s | %s | %s | %s | %s | %s |" % (esc(event), cr, n, pe, esc("; ".join(parts)), esc(note)))
    L.append("")
    L.append("Queries, in row order (each plus `%s`, `unique=cards`):" % BASE.strip())
    L.append("")
    for event, _, q, *_ in T2:
        L.append("- %s: `%s`" % (esc(event), esc(q)))
    return "\n".join(L)


BOUNDARY = [
    ('o:"dies"', 'o:/\\bdies\\b/'),
    ('o:"when "', 'o:/\\bwhen\\b/'),
    (TRIGGER, 'o:/\\b(when|whenever|at the beginning of)\\b/'),
    (W + ' o:"enters"', 'o:/\\b(when|whenever)\\b[^.]*\\benters\\b/'),
    (W + ' o:"attacks"', 'o:/\\bwhenever\\b[^.]*\\battacks\\b/'),
    (TRIGGER + ' o:", if "', 'o:/(when|whenever|at the beginning of)[^.]*, if /'),
    ('o:"whenever" o:"one or more"', 'o:/\\b(when|whenever)\\b[^.]*one or more/'),
]


def render_boundary():
    # The regex queries carry `|`, which a table cell would have to escape and
    # a reader of the raw file would then misread; they are listed under the
    # table instead, by pair number.
    L = ["| pair | substring query | cards | cards, regex reading |", "|---:|---|---:|---:|"]
    for i, (a, b) in enumerate(BOUNDARY, 1):
        L.append("| %d | `%s` | %s | %s |" % (i, esc(a), fmt(count(a)), fmt(count(b))))
    L.append("")
    L.append("The regex readings, by pair (each plus `%s`, `unique=cards`):" % BASE.strip())
    L.append("")
    for i, (_, b) in enumerate(BOUNDARY, 1):
        L.append("%d. `%s`" % (i, b))
    return "\n".join(L)


def splice(doc_text, key, body):
    begin, end = "<!-- trigger-survey: begin %s -->" % key, "<!-- trigger-survey: end %s -->" % key
    i, j = doc_text.index(begin) + len(begin), doc_text.index(end)
    return doc_text[:i] + "\n" + body + "\n" + doc_text[j:]


def main():
    global _no_fetch
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0],
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--write", action="store_true", help="splice the tables into trigger-survey.md")
    ap.add_argument("--boundary", action="store_true", help="the substring-vs-regex table only")
    ap.add_argument("--names", metavar="Q", help="print the first page of names behind a query")
    ap.add_argument("--refresh", action="store_true", help="drop the Scryfall cache first")
    ap.add_argument("--no-fetch", action="store_true", help="never hit the network")
    args = ap.parse_args()
    _no_fetch = args.no_fetch
    if args.refresh and os.path.exists(CACHE):
        os.remove(CACHE)
    if args.names:
        names(args.names)
        return
    if args.boundary:
        print(render_boundary())
        return

    events = parse_enum(EVENT_RS, "GameEvent")
    actions = parse_enum(ACTIONS_RS, "GameAction")
    causes = parse_enum(ZONES_RS, "ZoneChangeCause")
    check_fields(events)
    sites = emit_sites()
    conn = db()

    blocks = {
        "STREAM": render_stream(events, actions, causes, sites, registered_triggers()),
        "CORPUS": render_corpus(corpus_block(conn)),
        "TABLE-1": render_t1(conn),
        "TABLE-2": render_t2(events),
        "BOUNDARY": render_boundary(),
        "DATE": "Counted %s (Scryfall fetch date from the cache; the tree and the corpus as of the run)." % fetched_on(),
    }
    if args.write:
        raw = open(DOC, "rb").read()
        eol = "\r\n" if raw.count(b"\r\n") > raw.count(b"\n") // 2 else "\n"
        text = raw.decode("utf-8").replace("\r\n", "\n")
        for k, v in blocks.items():
            text = splice(text, k, v)
        open(DOC, "wb").write(text.replace("\n", eol).encode("utf-8"))
        print("wrote", os.path.relpath(DOC, ROOT))
    else:
        for k in ("DATE", "STREAM", "CORPUS", "TABLE-1", "TABLE-2", "BOUNDARY"):
            print("## %s\n\n%s\n" % (k, blocks[k]))


if __name__ == "__main__":
    main()
