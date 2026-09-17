#!/usr/bin/env python3
"""
check_rulings - the rulings ledger, and the gate that keeps it honest.

`engineering-practices.md` §3.4 makes every card a phase registers carry a
rulings pass: a card's rulings exist *because players got those cases wrong*,
so they are a free list of the boards a naive implementation misses. §3.4a
says what holds that rule up. **Not a verifier** - a ruling is prose and
nothing compiles prose into an assertion - but a **ledger with a gate**, the
shape `specdb` already proved one level down: the corpus is fetched, the join
is generated, and the check fails when a claim has no linked test.

    python plans/check_rulings.py            # the census, and the queue's head
    python plans/check_rulings.py --check    # the gate; exit 1 on a failure
    python plans/check_rulings.py --queue    # the whole work queue, pool first
    python plans/check_rulings.py --fetch    # NETWORK; rewrite the ledger

`--check` reads files only - no network, no git, no `spec.sqlite` - so a
shallow CI checkout answers the same as a local run. `check_state_of_play.py`
sets that rule and this joins the same family for the same reason: CI runs the
`check_*.py` scripts and `CLAUDE.md`'s Commands fence cites them together, so
joining the family is what makes an unlinked ruling fail a pull request rather
than sit there.

# The unit of the gate is a linked test

Not a disposition (the owner, 2026-09-08, sharpening §3.4a): *"card author
needs to make a test and link it to the ruling for someone to review in PR
review."* So the normal answer edits no ledger at all - it is a comment above
a `#[test]`, the way `// COVERS:` is:

    // RULING: March of the Machines #4 - "If an Equipment becomes a creature,
    //   it can no longer equip a creature. If it's currently attached to a
    //   creature, it becomes unattached (but remains on the battlefield)."
    // COVERS: ATOM-704.5p-001
    #[test]
    fn test_equipment_that_becomes_a_creature_is_unattached() { ... }

One annotation per line, and everything after the number is prose this parser
ignores. A comma list is not available on purpose: 11 of the 161 registered
names carry a comma ("Isamaru, Hound of Konda"), so a comma inside the key
would be a parse ambiguity on 7% of the registry. Several lines above one test
say what one line with a list would have, and several tests may name one
ruling.

# What a ruling is keyed on, and why not its date

`<Card Name> #<n>`, where `n` is assigned at first fetch and **never reused** -
`specdb`'s rule for atom ids, for the same reason: annotations in Rust source
depend on it.

Card plus date was the obvious key and the census refuses it. Scryfall stamps
a ruling with `published_at` and nothing else; **298 of today's 330 rulings -
90% - share a date with another ruling on the same card**, and 70 of the 84
cards carrying more than one carry all of them on a single date. So a date
needs an ordinal, and an ordinal derived from position in the fetched list is
the failure this key exists to avoid: a ruling inserted ahead of another
renumbers it, and every annotation below the insertion silently re-points at a
*different* ruling. A key that silently re-points is worse than one that breaks
loudly.

So `n` is stored, not derived. The fetcher matches what it fetched against
what the ledger holds on the ruling's text, keeps the number it already gave,
and hands a genuinely new ruling the next free one. **An edited ruling is
therefore a removal and an addition**, which breaks its annotation and fails
this check - the right answer, because an edit to a ruling is an instruction
to read it again.

# What drift does, and why it cannot block someone else's pull request

Scryfall adds rulings, so a card that was correct when it was registered can
acquire one later that the engine violates, and nothing else in this project
would ever notice. That is the reason §3.4a gives for building the tool at all.

It fails, rather than reports - but only in the pull request that runs
`--fetch`, because `--check` is offline and cannot see a ruling the ledger does
not hold. The two halves of that decision are usually in tension and here they
are not: the failing form is the one that catches drift, and a manual fetch is
what keeps it off the desk of somebody whose branch touched nothing.

# Scope: the retroactive half is a backlog, the forward half is a rule

A gate that fails on all 330 rulings the day it lands is a gate nothing passes,
and `specdb owed` had the same problem and the same answer (§5.1): scope it, or
it is a report. A card is **in scope** when either
  * it carries `read` - someone did §3.4's pass on it, so its rulings are
    claims this tree is answerable for; or
  * its `first_seen` is later than the ledger's `created` - it was registered
    *after* the ledger existed, so §3.4 already obliged its author.

Everything else is the backlog: counted, printed, queued pool-first, never
failed. Two stamps the fetcher writes and one field an author writes, so
nothing here needs git.

# The escape, and what it has to carry

"Not yet expressible" stays a legal answer - §3.4's third, and the one that
finds gaps from the outside - because a gate with no honest escape gets
satisfied dishonestly. It is structured rather than free text, and `--check`
rejects one that does not name what it owes:

    "disposition": {"kind": "not-expressible", "facility": "...",
                    "owner": "backlog.md §2.19"}
    "disposition": {"kind": "no-registered-card", "needs": "..."}
    "disposition": {"kind": "defect", "item": 82}
    "disposition": {"kind": "format-variant", "format": "Two-Headed Giant"}
    "disposition": {"kind": "no-board", "why": "..."}

**Five kinds and not one free-text field, because each becomes untrue a
different way** - which is the only thing a disposition is for. A
`not-expressible` clears when its facility lands, and `owner` must name a
`plans/` doc that exists so there is somewhere to look. A
`no-registered-card` clears when somebody registers the card; the engine can
already state the ruling, and saying "not expressible" there would be a
recorded untruth. A `defect` clears when a fix lands, and `item` must be a
number `codebase-state.md` carries. A `format-variant` clears when §9 builds
the format. A `no-board` never clears - a ruling about card frames names no
board and never will.

**`format-variant` is a disposition rather than a fetch-time filter**: §3.4
says a Two-Headed Giant ruling "gets no line", but a regex deciding that at
fetch time re-adds it on every run and would drop rulings that mention a
format in passing. Recorded per ruling, it is a judgement somebody made once.

A ruling may not carry both a disposition and an annotation. They are two
answers to a question that has one.
"""

import argparse
import json
import re
import subprocess
import sys
import tempfile
import time
from pathlib import Path

# The comments are Scryfall's prose: curly quotes, em-dashes and accented card
# names. The Windows console defaults to cp1252 and would raise on the first
# one. Same guard as `specdb.py` and `check_claude_md.py`.
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

ROOT = Path(__file__).resolve().parent.parent
LEDGER = ROOT / "plans" / "rulings-ledger.json"
REGISTRY = ROOT / "mtgsim" / "src" / "cards" / "registry.rs"
CODE_DIRS = [ROOT / "mtgsim" / "src", ROOT / "mtgsim" / "tests"]

UA = "MTG-Ichor-Research/1.0 (contact: maiercluke@gmail.com)"
CHUNK = 75      # Scryfall's documented maximum per /cards/collection request
SLEEP = 0.15    # <10 req/s is asked for and meant: 80ms earned a 60-second ban

RULING_RE = re.compile(r"//\s*RULING:\s*(.+?)\s*#(\d+)")
RUST_FN_RE = re.compile(r"\bfn\s+([a-z_0-9]+)")
DISPOSITIONS = {"not-expressible": ("facility", "owner"),
                "no-registered-card": ("needs",),
                "defect": ("item",),
                "format-variant": ("format",),
                "no-board": ("why",)}


# --------------------------------------------------------------------------
# The tree, read offline
# --------------------------------------------------------------------------

def registered():
    """Every name `registry.rs` registers, and the subset in PERFORMANCE_POOL.

    `register(\\s*"` and not `register(`: the latter also matches the pool's own
    re-registration loop, which is handed a name rather than adding a card.
    `check_state_of_play.py` learned that the expensive way.
    """
    src = REGISTRY.read_text(encoding="utf-8")
    names = sorted(set(re.findall(r'registry\.register\(\s*"([^"]+)"', src)))
    m = re.search(r"PERFORMANCE_POOL: \[&str; \d+\] = \[(.*?)\n\];", src, re.S)
    pool = set(re.findall(r'^\s*"([^"]+)",', m.group(1), re.M)) if m else set()
    return names, pool


def annotated_fn(lines, n):
    """The test a `// RULING:` at 1-based line `n` annotates.

    Walks forward past comments, attributes and blank lines. A fixed lookahead
    does not work and `specdb.py` says why: an annotation here carries the
    ruling's own text, so the block above a test runs to a dozen lines.
    """
    for look in lines[n:n + 60]:
        s = look.strip()
        if not s or s.startswith("//") or s.startswith("#["):
            continue
        fm = RUST_FN_RE.search(look)
        return fm.group(1) if fm else ""
    return ""


def scan_annotations():
    """(card, n) -> [(test name, file, line)] for every `// RULING:` in the tree."""
    out = {}
    for d in CODE_DIRS:
        if not d.exists():
            continue
        for path in sorted(d.rglob("*.rs")):
            lines = path.read_text(encoding="utf-8", errors="replace").split("\n")
            for i, line in enumerate(lines, 1):
                m = RULING_RE.search(line)
                if not m:
                    continue
                rel = str(path.relative_to(ROOT)).replace("\\", "/")
                key = (m.group(1).strip(), int(m.group(2)))
                out.setdefault(key, []).append((annotated_fn(lines, i), rel, i))
    return out


def load():
    if not LEDGER.exists():
        sys.exit(f"no ledger at {LEDGER} - run --fetch to create it")
    return json.loads(LEDGER.read_text(encoding="utf-8"))


def in_scope(card, created):
    """A card the gate has teeth on. The docstring's two stamps and one field."""
    return bool(card.get("read")) or card.get("first_seen") != created


# --------------------------------------------------------------------------
# The gate
# --------------------------------------------------------------------------

def verify(ledger, names, pool, links):
    """Every failure, as a list of strings. Empty means the gate passes."""
    bad = []
    cards, created = ledger["cards"], ledger["created"]
    known = set(cards) | set(ledger.get("fixtures", []))
    for n in names:
        if n not in known:
            bad.append(f"{n}: registered, but the ledger has never seen it "
                       f"- run `python plans/check_rulings.py --fetch`")

    item_nums = codebase_state_items()
    for name in sorted(cards):
        card = cards[name]
        scoped = in_scope(card, created)
        for r in card["rulings"]:
            key = (name, r["n"])
            linked = links.get(key)
            disp = r.get("disposition")
            if linked and disp:
                bad.append(f"{name} #{r['n']}: carries a disposition "
                           f"({disp.get('kind')}) and a test ({linked[0][0]}). "
                           f"A ruling gets one answer, not two")
            elif disp:
                bad.extend(check_disposition(name, r, disp, item_nums))
            elif not linked and scoped:
                why = "read" if card.get("read") else "registered after the ledger"
                bad.append(f"{name} #{r['n']}: no test names it, and no "
                           f"disposition says why ({why}) - "
                           f"{r['comment'][:60]}...")

    have = {(name, r["n"]) for name, c in cards.items() for r in c["rulings"]}
    for (name, n), where in sorted(links.items()):
        if (name, n) not in have:
            _, path, line = where[0]
            bad.append(f"{path}:{line}: names {name} #{n}, which the ledger "
                       f"does not hold. A ruling whose text changed is a "
                       f"removal and an addition - re-read it and re-link")
    return bad


def check_disposition(name, ruling, disp, item_nums):
    """An escape that does not name what it owes is not an escape."""
    kind = disp.get("kind")
    if kind not in DISPOSITIONS:
        return [f"{name} #{ruling['n']}: unknown disposition {kind!r} "
                f"(one of {', '.join(sorted(DISPOSITIONS))})"]
    bad = []
    for field in DISPOSITIONS[kind]:
        if not disp.get(field) and disp.get(field) != 0:
            bad.append(f"{name} #{ruling['n']}: a {kind} disposition must "
                       f"name its {field!r}")
    owner = disp.get("owner")
    if kind == "not-expressible" and owner:
        docs = re.findall(r"[\w.-]+\.md", owner)
        if not docs:
            bad.append(f"{name} #{ruling['n']}: owner {owner!r} names no doc")
        for doc in docs:
            if not (ROOT / "plans" / doc).exists() and not (ROOT / doc).exists():
                bad.append(f"{name} #{ruling['n']}: owner names {doc}, "
                           f"which is not in the tree")
    if kind == "defect" and disp.get("item") not in item_nums:
        bad.append(f"{name} #{ruling['n']}: item {disp.get('item')} is not "
                   f"one codebase-state.md carries")
    return bad


def codebase_state_items():
    """The item numbers `codebase-state.md` carries, open or archived.

    A `defect` disposition points at one, and a pointer at nothing is the
    dishonest escape this file exists to refuse. Both files, because an item
    keeps its number when it closes and its body moves to the archive.
    """
    nums = set()
    for rel in ("plans/codebase-state.md",
                "plans/archive/codebase-state-closed.md"):
        path = ROOT / rel
        if path.exists():
            nums.update(int(m) for m in re.findall(
                r"^(\d+)\. \*\*", path.read_text(encoding="utf-8"), re.M))
    return nums


# --------------------------------------------------------------------------
# Reports
# --------------------------------------------------------------------------

def census(ledger, names, pool):
    cards, created = ledger["cards"], ledger["created"]
    rulings = sum(len(c["rulings"]) for c in cards.values())
    carrying = sum(1 for c in cards.values() if c["rulings"])
    pooled = {n: c for n, c in cards.items() if n in pool}
    counts = sorted(len(c["rulings"]) for c in cards.values())
    scoped = [n for n, c in cards.items() if in_scope(c, created)]
    print(f"ledger  {LEDGER.relative_to(ROOT)}   created {created}, "
          f"fetched {ledger['fetched']}")
    print(f"  registered names            {len(names)}")
    print(f"    real cards                {len(cards)}")
    print(f"    fixtures (no rulings)     {len(ledger.get('fixtures', []))}"
          f"  {', '.join(ledger.get('fixtures', []))}")
    print(f"  cards carrying >=1 ruling   {carrying} of {len(cards)} "
          f"({round(100 * carrying / max(len(cards), 1))}%)")
    print(f"  total rulings               {rulings}")
    print(f"    on the {len(pool)} pooled cards     "
          f"{sum(len(c['rulings']) for c in pooled.values())}")
    print(f"  median rulings per card     {counts[len(counts) // 2]}")
    print(f"  in scope for the gate       {len(scoped)} card(s), "
          f"{sum(len(cards[n]['rulings']) for n in scoped)} ruling(s)")


def queue(ledger, pool, limit=None):
    """The backlog, pool first - §3.4a's work queue, descending by count.

    Pool first because the pool is what every measurement walks, which is
    where `codebase-state.md` item 82 came from: a live wrong answer sitting
    in the measured pool, found by reading one ruling.
    """
    cards, created = ledger["cards"], ledger["created"]
    rows = [(n in pool, len(c["rulings"]), n)
            for n, c in cards.items()
            if c["rulings"] and not in_scope(c, created)]
    rows.sort(key=lambda r: (not r[0], -r[1], r[2]))
    shown = rows if limit is None else rows[:limit]
    print(f"\nbacklog: {sum(r[1] for r in rows)} ruling(s) over {len(rows)} "
          f"card(s) nobody has read")
    print(f"  {sum(r[1] for r in rows if r[0])} of them on "
          f"{sum(1 for r in rows if r[0])} pooled card(s), which go first")
    for pooled, count, name in shown:
        print(f"  {count:3d}  {'pool' if pooled else '    '}  {name}")
    if limit is not None and len(rows) > limit:
        print(f"  ... and {len(rows) - limit} more (--queue for all)")


# --------------------------------------------------------------------------
# The fetch - the one mode that touches the network
# --------------------------------------------------------------------------

def curl(args):
    """One Scryfall call, retrying once through a rate limit.

    A 429 comes back as a JSON `error` object rather than a non-zero exit, so
    a caller checking only the exit code reads "rate limited" as "no such
    card" - which is how `classify_cards.py` first reported 76 of its 97 cards
    as fixtures.
    """
    for attempt in range(2):
        proc = subprocess.run(["curl", "-s", "-H", f"User-Agent: {UA}", *args],
                              capture_output=True, check=True)
        data = json.loads(proc.stdout.decode("utf-8"))
        if data.get("object") == "error" and "rate-limit" in (data.get("details") or ""):
            if attempt == 0:
                time.sleep(65)
                continue
        return data
    return data


def identify(names):
    """(name -> card object, fixture names). One request per 75 names."""
    found, missing = {}, []
    for i in range(0, len(names), CHUNK):
        batch = names[i:i + CHUNK]
        body = json.dumps({"identifiers": [{"name": n} for n in batch]})
        with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False,
                                         encoding="utf-8") as f:
            f.write(body)
            payload = f.name
        proc = subprocess.run(
            ["curl", "-s", "-H", f"User-Agent: {UA}",
             "-H", "Content-Type: application/json",
             "--data-binary", f"@{payload}",
             "https://api.scryfall.com/cards/collection"],
            capture_output=True, check=True)
        Path(payload).unlink(missing_ok=True)
        data = json.loads(proc.stdout.decode("utf-8"))
        # Scryfall answers with the card's own name, which is not always the
        # spelling that was asked for, and in its own order.
        by_name = {c["name"]: c for c in data.get("data", [])}
        lower = {k.lower(): k for k in by_name}
        for n in batch:
            c = by_name.get(n) or by_name.get(lower.get(n.lower(), ""))
            if c is not None:
                found[n] = c
        missing.extend(nf.get("name") for nf in data.get("not_found", []))
        time.sleep(SLEEP)
    return found, missing


def norm(text):
    """The form two rulings are compared in.

    Whitespace collapsed and Scryfall's curly punctuation folded: a
    typographic sweep on their side is not an edit to the ruling, and matching
    on the raw bytes would burn every number on every card it touched. A real
    wording change still falls through, which is the point.
    """
    text = text.replace("’", "'").replace("‘", "'")
    text = text.replace("“", '"').replace("”", '"')
    text = text.replace("—", "-").replace("–", "-")
    return " ".join(text.split())


def fetch(today):
    """Rewrite the ledger from Scryfall, keeping every number already given."""
    names, _ = registered()
    bad = [n for n in names if "#" in n]
    if bad:
        sys.exit(f"the key is `<name> #<n>` and these names contain '#': {bad}")
    old = json.loads(LEDGER.read_text(encoding="utf-8")) if LEDGER.exists() else {}
    old_cards = old.get("cards", {})
    created = old.get("created", today)

    print(f"identifying {len(names)} registered name(s) ...")
    found, fixtures = identify(names)
    print(f"  {len(found)} printing(s), {len(fixtures)} fixture(s): "
          f"{', '.join(sorted(fixtures)) or '-'}")

    cards, added, removed = {}, [], []
    for i, name in enumerate(sorted(found), 1):
        data = curl([found[name]["rulings_uri"]])
        fetched = data.get("data", []) if data.get("object") != "error" else []
        prev = old_cards.get(name, {})
        by_text = {norm(r["comment"]): r for r in prev.get("rulings", [])}
        used = max((r["n"] for r in prev.get("rulings", [])), default=0)
        rulings, seen = [], set()
        for r in fetched:
            key = norm(r["comment"])
            seen.add(key)
            if key in by_text:
                row = dict(by_text[key])
                row["published_at"] = r["published_at"]
                row["comment"] = r["comment"]
            else:
                used += 1
                row = {"n": used, "published_at": r["published_at"],
                       "comment": r["comment"]}
                if prev:
                    added.append(f"{name} #{used}")
            rulings.append(row)
        removed.extend(f"{name} #{r['n']}" for r in prev.get("rulings", [])
                       if norm(r["comment"]) not in seen)
        card = {"first_seen": prev.get("first_seen", today),
                "rulings": sorted(rulings, key=lambda r: r["n"])}
        if prev.get("read"):
            card["read"] = prev["read"]
        cards[name] = card
        if i % 25 == 0:
            print(f"  ... {i}/{len(found)}", file=sys.stderr)
        time.sleep(SLEEP)

    for name in sorted(set(old_cards) - set(cards)):
        print(f"  dropped (no longer registered): {name}")
    report_drift(added, removed, cards, created)
    write(created, today, cards, sorted(fixtures))
    print(f"\nwrote {LEDGER.relative_to(ROOT)}: {len(cards)} card(s), "
          f"{sum(len(c['rulings']) for c in cards.values())} ruling(s)")


def report_drift(added, removed, cards, created):
    """What moved since the last fetch, loudest first.

    A removal is nearly always an *edit*: a reworded ruling cannot match on
    its text, so it leaves as one number and returns as another. The
    annotation that named the old number then fails `--check`, which is the
    re-read this reports.
    """
    if not added and not removed:
        print("  no drift: every ruling is the one the ledger already held")
        return
    scoped = {n for n, c in cards.items() if in_scope(c, created)}
    for label, rows in (("gone (edited, or withdrawn)", removed),
                        ("new since the last fetch", added)):
        if rows:
            print(f"\n  {len(rows)} {label}:")
            for row in rows:
                name = row.rsplit(" #", 1)[0]
                print(f"    {row}{'   <- in scope' if name in scoped else ''}")


def write(created, fetched, cards, fixtures):
    """The ledger, one line per ruling.

    Pretty-printed per card and compact per ruling on purpose: drift is read
    out of `git diff`, and a ruling that owns a line makes an addition one
    added line rather than five.
    """
    out = ['{', '  "_": [',
           '    "GENERATED by plans/check_rulings.py --fetch. The rulings are",',
           '    "Scryfall\'s text, verbatim. Hand-edit two things and nothing",',
           '    "else: a ruling\'s `disposition`, and a card\'s `read` stamp.",',
           '    "See the script\'s docstring for what each one has to carry."',
           '  ],',
           f'  "created": "{created}",', f'  "fetched": "{fetched}",',
           '  "fixtures": %s,' % json.dumps(fixtures, ensure_ascii=False),
           '  "cards": {']
    for i, name in enumerate(sorted(cards)):
        card = cards[name]
        tail = "" if i == len(cards) - 1 else ","
        out.append("    %s: {" % json.dumps(name, ensure_ascii=False))
        out.append('      "first_seen": "%s",' % card["first_seen"])
        if card.get("read"):
            out.append('      "read": "%s",' % card["read"])
        if not card["rulings"]:
            out.append('      "rulings": []')
        else:
            out.append('      "rulings": [')
            for j, r in enumerate(card["rulings"]):
                comma = "" if j == len(card["rulings"]) - 1 else ","
                # Explicit key order, not `sort_keys`: the number and the date
                # are what a reader scans a 1,000-line diff by, and sorted
                # keys put the 1,031-character comment in front of them.
                ordered = {k: r[k] for k in
                           ("n", "published_at", "comment", "disposition")
                           if k in r}
                out.append("        %s%s"
                           % (json.dumps(ordered, ensure_ascii=False), comma))
            out.append("      ]")
        out.append("    }%s" % tail)
    out += ["  }", "}"]
    LEDGER.write_text("\n".join(out) + "\n", encoding="utf-8")


# --------------------------------------------------------------------------

def selftest():
    """The gate, against a fixture ledger - so an edit here cannot quietly
    stop it failing. Runs under `--check`, the way `check_state_of_play.py`
    runs its parser selftest. The unlinked ruling is the case that matters."""
    ledger = {"created": "2026-01-01", "fetched": "2026-01-01", "fixtures": [],
              "cards": {
        "Read And Linked": {"first_seen": "2026-01-01", "read": "2026-02-01",
                            "rulings": [{"n": 1, "published_at": "2020-01-01",
                                         "comment": "linked"}]},
        "Read And Unlinked": {"first_seen": "2026-01-01", "read": "2026-02-01",
                              "rulings": [{"n": 4, "published_at": "2020-01-01",
                                           "comment": "nobody names this"}]},
        "Backlog": {"first_seen": "2026-01-01",
                    "rulings": [{"n": 1, "published_at": "2020-01-01",
                                 "comment": "unread, so not owed"}]},
        "Registered Later": {"first_seen": "2026-06-01",
                             "rulings": [{"n": 1, "published_at": "2020-01-01",
                                          "comment": "in scope by its stamp"}]},
        "Disposed": {"first_seen": "2026-01-01", "read": "2026-02-01",
                     "rulings": [{"n": 1, "published_at": "2020-01-01",
                                  "comment": "x", "disposition": {
                                      "kind": "not-expressible",
                                      "facility": "f", "owner": "backlog.md §9"}}]},
        "Empty Escape": {"first_seen": "2026-01-01", "read": "2026-02-01",
                         "rulings": [{"n": 1, "published_at": "2020-01-01",
                                      "comment": "x", "disposition": {
                                          "kind": "not-expressible",
                                          "facility": "", "owner": "backlog.md"}}]},
        "Both Answers": {"first_seen": "2026-01-01", "read": "2026-02-01",
                         "rulings": [{"n": 2, "published_at": "2020-01-01",
                                      "comment": "x", "disposition": {
                                          "kind": "format-variant",
                                          "format": "2HG"}}]}}}
    links = {("Read And Linked", 1): [("test_a", "tests/x.rs", 1)],
             ("Both Answers", 2): [("test_b", "tests/x.rs", 2)],
             ("Ghost", 9): [("test_c", "tests/x.rs", 3)]}
    names = sorted(ledger["cards"]) + ["Never Fetched"]
    bad = verify(ledger, names, set(), links)
    # Each failure named by its subject, which is the text before the first
    # colon. The list is the whole contract: everything else must pass.
    got = sorted(b.split(":")[0] for b in bad)
    want = ["Both Answers #2", "Empty Escape #1", "Never Fetched",
            "Read And Unlinked #4", "Registered Later #1", "tests/x.rs"]
    assert got == want, "selftest: the gate refused\n  " + "\n  ".join(got)
    # The two that must NOT fail: an unread card, and an honest escape.
    joined = "\n".join(bad)
    assert "Backlog" not in joined, "selftest: the backlog is not owed"
    assert "Disposed #" not in joined, "selftest: an honest escape is an answer"


def main():
    ap = argparse.ArgumentParser(description="The rulings ledger and its gate.")
    ap.add_argument("--check", action="store_true",
                    help="exit 1 on a ruling in scope that no test names")
    ap.add_argument("--queue", action="store_true",
                    help="the whole backlog, pool first")
    ap.add_argument("--fetch", action="store_true",
                    help="NETWORK: rewrite the ledger from Scryfall")
    ap.add_argument("--today", default=None, help="date stamp for --fetch")
    args = ap.parse_args()

    if args.fetch:
        import datetime
        fetch(args.today or datetime.date.today().isoformat())
        return 0

    selftest()
    ledger = load()
    names, pool = registered()
    census(ledger, names, pool)
    queue(ledger, pool, None if args.queue else 8)

    bad = verify(ledger, names, pool, scan_annotations())
    print()
    if bad:
        print(f"{len(bad)} ruling(s) the gate refuses:\n")
        for b in bad:
            print(f"  {b}")
        print("\nAnswer each with a test that names it (`// RULING: <card> #<n>`)")
        print("or with a disposition in the ledger. See this file's docstring.")
        return 1 if args.check else 0
    print("every ruling in scope is named by a test or answered by a "
          "disposition.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
