#!/usr/bin/env python3
"""state-of-play — the board, generated, plus the check that keeps it honest.

`plans/state-of-play.md` answers one question: **what is done, what is next,
and what the tree is carrying.** ("Carrying", not "owed": `specdb owed` is a
different question and the board says how — see the section it renders.) It
exists because the doc that answered it before —
`roadmap-v2.md` §2 — was three days and eight merged PRs stale the first time
anyone went looking (2026-09-03), and nothing failed when it rotted.

    python plans/check_state_of_play.py            # print the board
    python plans/check_state_of_play.py --write    # regenerate the file
    python plans/check_state_of_play.py --check    # exit 1 if it is stale
    python plans/check_state_of_play.py --slotted <name>   # a phase's intake list
    python plans/check_state_of_play.py --forecast  # the forecast table, as the rows read today

**Named `check_*.py` on purpose.** CI runs the check scripts and `CLAUDE.md`'s
exit criteria cite them as a family, so joining the family is what makes a stale
board fail a pull request rather than sit there. That is the whole mechanism:
`engineering-practices.md#line-budgets`'s rule is that a convention which can fail
silently is not one.

**Every number here is a file read.** No git, no `gh`, no network, no
`spec.sqlite` — a shallow CI checkout has to produce the same bytes as a local
run, and anything derived from history or from a derived database would not.
The two things that need git are printed by `--flight` and deliberately kept
out of the generated file.

# What the check actually catches, and what it does not

1. **A stale file** — regenerate and diff. Catches counts that moved.
2. **The critical path contradicting an architecture doc** — a phase whose
   `####` heading carries ✅ must not be named "next" in `CLAUDE.md`'s critical
   path. This is the failure that happened: RC-5 landed, its heading said so,
   and `CLAUDE.md` still said "RC-5 next".

3. **The floors' table drifting** — each floor's limit
   (`engineering-practices.md#the-floors`) and latest reading
   (`fuzz-record.md#the-latest-floor-readings`) are rendered from the tables
   between their markers, so a reading taken and not regenerated here fails
   the check like any other stale number.

4. **The Deferred Migrations parser drifting** — `selftest()` runs the item
   splitter and the verdict classifier against a fixture before every check,
   so a regex edit that changes what counts as an item fails here rather than
   silently moving the board's numbers.

5. **An open item with no home** — no `**Slotted:**` line, or one naming no
   phase, row or card still to come. A phase reads what is slotted to it at
   its first ticket (`--slotted`), so an item with no slot is one no phase
   will read: the owner's question after #237, "how do we know we're not
   forgetting something". A line naming only landed phases fails too, since
   those phases went without the item.

6. **A number used twice in one run** — three main items collided in
   September, when parallel branches each took the next free number.

7. **A route row with no size, or a size with no row** — `roadmap-v2.md`'s
   Sizing table is what "The way to v0.5" totals, so a row of the critical path
   it does not carry, a row the route has no name for, or a PR cell that is not
   a count or a range would each misstate how far v0.5 is.

8. **A landed row still sized as a range, or a step with no forecast** — a
   step's PRs taken are its landed rows' cells, read against the range the
   route was last forecast at, so a landed row's cell is what it took, and
   every step with rows has a forecast.

9. **The gap census's records misstating a card** — the cards that could be
   written today are the deck cards with no need left, and a need naming no
   row reads as met, so a row the Sizing table renamed fails here until
   `gap_census.py rename` carries it into the records.

It does **not** derive per-phase status for the "can't" and copy tracks,
because those docs record their phases only in sizing tables with no status
marker. The replacement track has heading markers and is checked; the others
are a normalisation nobody has done. Said out loud rather than papered over —
a check that silently covers one track of three is worse than one that says so.
"""

import argparse
import re
import sys
from collections import namedtuple
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "plans" / "state-of-play.md"

# Globbed, not listed. The list form silently omitted `cost-architecture.md`
# from the day that doc was written until 2026-09-07 -- CM-0, CM-1 and CM-2 all
# shipped with the ✅ heading this file looks for and none of them appeared on
# the board, because "add the new subsystem to `ARCH_DOCS`" is a step nobody
# owns. `CLAUDE.md`'s authority row already says a new subsystem *extends* the
# architecture-doc family rather than starting a new one, so the family is
# knowable from the filenames and this asks the filesystem instead of a human.
ARCH_DOCS = sorted(str(p.relative_to(ROOT)).replace("\\", "/")
                   for p in (ROOT / "plans").glob("*-architecture.md"))


def read(rel):
    return (ROOT / rel).read_text(encoding="utf-8")


# --------------------------------------------------------------------------
# Derivation — file reads only
# --------------------------------------------------------------------------

#: The cards with CR 205.4c's basic supertype that a deck may hold any number of.
BASIC_LANDS = {"Plains", "Island", "Swamp", "Mountain", "Forest", "Wastes"}


def deck_lists():
    """Each of v0.5's five lists as `(file, its names less basics, in order)`."""
    out = []
    for f in sorted((ROOT / "mtgsim" / "decks").glob("tdc-*.txt")):
        names = re.findall(r"^\d+ (.+?)\s*$", f.read_text(encoding="utf-8"), re.M)
        out.append((f, [name for name in names if name not in BASIC_LANDS]))
    return out


def deck_cards():
    """The five lists' distinct cards. Read off the lists and not Scryfall's
    `e:tdc`, whose 373 leaves out the 12 printed in `tdm`."""
    return {name for _, names in deck_lists() for name in names}


def registered_cards():
    # `registry.register("` and not `registry.register(`: the latter also
    # matches `performance_pool`'s own loop, which re-registers a name it
    # was handed rather than adding a card, so every board since this
    # script was written has reported one card too many.
    return set(re.findall(r'registry\.register\(\s*"([^"]+)"', read("mtgsim/src/cards/registry.rs")))


def gap_census():
    """`plans/references/card-census/gap_census.py`, which holds the deck
    cards' needs. It imports this file for the lists and the Sizing table, so
    it is loaded on the first call, and finds this file under its own name
    even when this file is the script being run."""
    sys.modules.setdefault("check_state_of_play", sys.modules[__name__])
    census_dir = str(ROOT / "plans" / "references" / "card-census")
    if census_dir not in sys.path:
        sys.path.insert(0, census_dir)
    import gap_census as census
    return census


#: The deck cards not registered, by what stands between each and being
#: written: nothing since the census proved it (its `writable`), nothing since
#: its needs landed (its `ready`), a row or a gap, or no record. The first two
#: together could be written today.
CardsToWrite = namedtuple("CardsToWrite", "proved landed_since waiting unread")


def cards_to_write(records, parsed, unregistered):
    """`unregistered` as a `CardsToWrite` by the gap census's `records` against
    the Sizing table's `parsed` rows, and what is wrong with the records."""
    census = gap_census()
    rows, landed, _ = census.rows_of(parsed)
    counted = census.tally(records, rows, unregistered)
    to_write = CardsToWrite(counted["writable"], counted["ready"], counted["blocked"] + counted["gap"],
                            len(unregistered - set(records)))
    return to_write, census.record_problems(records, rows, landed, unregistered)


def deck_cards_to_write():
    """The five lists' unregistered cards as `cards_to_write` reads them today."""
    census = gap_census()
    unregistered = deck_cards() - registered_cards()
    if not census.RECORDS.exists():
        return CardsToWrite(0, 0, 0, len(unregistered)), [f"{census.RECORDS.relative_to(ROOT).as_posix()} is gone"]
    return cards_to_write(census.read_records(), sizing()[0], unregistered)


def counts():
    reg = read("mtgsim/src/cards/registry.rs")
    pool = re.search(r"PERFORMANCE_POOL: \[&str; (\d+)\]", reg)
    tests = 0
    for base in ("mtgsim/src", "mtgsim/tests"):
        for f in (ROOT / base).rglob("*.rs"):
            tests += f.read_text(encoding="utf-8", errors="replace").count("#[test]")
    registered = registered_cards()
    decks = deck_cards()
    return {
        "cards": len(registered),
        "pool": int(pool.group(1)) if pool else 0,
        "tests": tests,
        "deck_cards": len(decks),
        "deck_cards_registered": len(decks & registered),
    }


ITEM_RE = re.compile(r"^(\d+[a-z]?)\.\s", re.M)
VERDICT_RE = re.compile(r"\*\*Reachability \((\d{4}-\d{2}-\d{2})\):\*\*\s*(.+)")
LEGACY_UNREACHABLE_RE = re.compile(r"[Uu]nreachable|no board to fail on|not reachable|no registered")


def classify_item(body):
    """One of: closed, unreachable, reachable_wrong, reachable_ok, none_owed, unstated.

    A dated `**Reachability (YYYY-MM-DD):**` verdict is authoritative when
    present; its first words name the class. Without one, the pre-triage
    heuristics apply: a ✅/CLOSED/strike-through near the top means closed, and
    the word "unreachable" (or its stock paraphrases) means the item says why it
    cannot bite yet. `sized` is separate — `**Sized:**` anywhere in the item.
    """
    m = VERDICT_RE.search(body)
    if m:
        v = m.group(2).lower()
        if v.startswith("closed"):
            return "closed"
        if v.startswith("unreachable"):
            return "unreachable"
        # "reachable — wrong today" is the bolded row; "reachable but not
        # wrong today" is not, and until 2026-09-15 the substring test read
        # it as one (items 118 and 131 sat on the board as wrong answers).
        head = v[:48]
        if v.startswith("reachable") and "wrong today" in head and "not wrong" not in head:
            return "reachable_wrong"
        if v.startswith("reachable"):
            return "reachable_ok"
        if v.startswith("nothing owed"):
            return "none_owed"
        return "unstated"
    if "CLOSED" in body[:400] or "~~" in body[:120] or "✅" in body[:200]:
        return "closed"
    if LEGACY_UNREACHABLE_RE.search(body):
        return "unreachable"
    return "unstated"


def split_items(block):
    """The numbered items of a Deferred Migrations block, as text bodies.

    An item is a column-0 `N.` or `Na.` line (`46.`, `7a.`, `16b.`); it runs to
    the next such line or the end of the block. Lists in prose must not use that
    marker at column 0 — the section uses `1)` for those, which Markdown renders
    the same way.
    """
    starts = [m.start() for m in ITEM_RE.finditer(block)]
    return [block[s:e] for s, e in zip(starts, starts[1:] + [len(block)])]


def dm_block():
    """`codebase-state.md`'s Deferred Migrations section, and the file line it starts on."""
    lines = read("plans/codebase-state.md").replace("\r\n", "\n").split("\n")
    start = next(i for i, l in enumerate(lines) if l.startswith("## Deferred Migrations"))
    end = len(lines)
    for i in range(start + 1, len(lines)):
        if lines[i].startswith("## "):
            end = i
            break
    return "\n".join(lines[start:end]), start + 1


#: The sections that number a run of their own, by their headings' first
#: words; every other section's items are the main run (the section's header,
#: "Item ids are section-scoped"). Two "Before card breadth" sections share a run.
OWN_RUNS = ("Before Layers", "Before card breadth", "Before Triggered abilities", "Before Commander")


def items_by_run(block):
    """Each item as `(run, number, body, line)`: `run` is `"main"` or the
    section whose run it is in, `line` its offset in `block`'s lines."""
    out = []
    for m in ITEM_RE.finditer(block):
        heading = block.rfind("\n### ", 0, m.start())
        title = block[heading + 5:block.find("\n", heading + 1)] if heading >= 0 else ""
        run = next((r for r in OWN_RUNS if title.startswith(r)), "main")
        out.append((run, m.group(1), m.start(), block.count("\n", 0, m.start())))
    ends = [start for _, _, start, _ in out[1:]] + [len(block)]
    return [(run, n, block[start:end], line) for (run, n, start, line), end in zip(out, ends)]


def cite(run, number):
    """An item as the section's header says to cite one."""
    return f"main item {number}" if run == "main" else f"'{run}' item {number}"


#: A `**Slotted:**` line's text, to the next bold label or the item's end.
SLOTTED_RE = re.compile(r"\*\*Slotted:(.+?)(?=\*\*[A-Z][^*\n]{0,60}:\*\*|\Z)", re.S)
#: A phase or route row still to come, cited by its name hyphenated in
#: backticks (`combat-restrictions`); the backticks are not part of it.
NAME = r"`[a-z0-9]+(?:-[a-z0-9]+)+`"
#: What a Slotted line can name on the route: a name, a landed phase's code,
#: a roadmap row's label (`A6h`, `B10`), or one of the two phases with no rows yet.
ROUTE_TOKEN_RE = re.compile(
    r"(?<=`)[a-z0-9]+(?:-[a-z0-9]+)+(?=`)|\b[A-Z]{2}-[0-9]+[a-z]?\b|\b[ABC][0-9]+[a-z]?\b|\bPhase (?:9|10)\b|§[DE]\b"
)
#: A code whose last letter marks a phase of a family (`TR-4a` of `TR-4`).
LETTERED_CODE_RE = re.compile(r"[A-Z]{2}-[0-9]+[a-z]?")
RECORD_HOME_RE = re.compile(r"^none\s*[—–-]+\s*a record\b")


def route_ids():
    """Every home the route offers, each with whether it has landed: the
    architecture docs' phases (`phase_index`), `roadmap-v2.md` §3a's rows by
    label or name, a row landed when its second cell says ✅, and Phases 9
    and 10."""
    landed, to_build = phase_index()
    route = {code: True for _, _, code, _ in landed if code}
    route.update({code: False for _, code, _ in to_build if code not in route})
    roadmap = read("plans/roadmap-v2.md")
    for m in re.finditer(rf"^\| (?:\*\*)?([AB][0-9]+[a-z]?|{NAME})(?:\*\*)? \| ([^|]*)\|", roadmap, re.M):
        route[m.group(1).strip("`")] = m.group(2).strip().startswith("✅")
    for m in re.finditer(rf"^\*\*(C[0-9]+|{NAME}) — ", roadmap, re.M):
        route[m.group(1).strip("`")] = False
    route.update({"Phase 9": False, "Phase 10": False, "§D": False, "§E": False})
    return route


def route_status(token, route):
    """`True` landed, `False` still to come, `None` unknown. A phase code
    the docs define only by its letters (`TR-4a`) is its family's (`TR-4`),
    and a family (`TR-3`) has landed once every phase of it has."""
    if token in route:
        return route[token]
    if not LETTERED_CODE_RE.fullmatch(token):
        return None
    family = re.sub(r"[a-z]$", "", token)
    if family != token and family in route:
        return route[family]
    members = [landed for code, landed in route.items() if re.fullmatch(re.escape(token) + "[a-z]", code)]
    return all(members) if members else None


def slotted_text(body):
    """The Slotted line's words. An item's text runs to the next item, so a
    section's last item carries the next heading; the line stops before it."""
    m = SLOTTED_RE.search(re.split(r"\n#{2,3} ", body)[0])
    return " ".join(m.group(1).replace("*", " ").split()) if m else None


def home(body, route):
    """`(class, detail)` for an open item's home. `route`: slotted to a phase
    or row still to come; `record`: nothing to build. Not homes: `missing`
    (no Slotted line), `landed` (it names only phases that have landed, so
    they went without it), `unknown` (it names nothing the route has, the
    retired "with the first registered card" among them)."""
    text = slotted_text(body)
    if text is None:
        return "missing", ""
    if RECORD_HOME_RE.match(text):
        return "record", text
    tokens = ROUTE_TOKEN_RE.findall(text)
    status = {t: route_status(t, route) for t in tokens}
    if any(s is False for s in status.values()):
        return "route", text
    if any(status.values()):
        return "landed", ", ".join(t for t, s in status.items() if s)
    unknown = [t for t, s in status.items() if s is None]
    return "unknown", ", ".join(unknown) if unknown else text[:80]


def deferred_migrations():
    """Item counts for `codebase-state.md`'s Deferred Migrations section.

    The section's own audit block carries a dated snapshot of these; this is the
    live one. `unstated` is the number that matters — an item that does not say
    why it cannot bite yet is an unchecked claim, not a deferral — and
    `reachable_wrong` is the list of known wrong answers the pool can reach.
    """
    block, _ = dm_block()
    items = split_items(block)
    classes = [classify_item(b) for b in items]
    open_items = [b for b, c in zip(items, classes) if c != "closed"]
    counts = {k: classes.count(k) for k in
              ("closed", "unreachable", "reachable_wrong", "reachable_ok", "none_owed", "unstated")}
    route = route_ids()
    homes = [home(b, route)[0] for b in open_items]
    return {
        "items": len(items),
        **counts,
        "sized": sum(1 for b in open_items if "Sized:" in b),
        "open": len(open_items),
        **{f"home_{k}": homes.count(k) for k in ("route", "record")},
        "homeless": sum(1 for h in homes if h not in ("route", "record")),
        "duplicates": len(duplicate_numbers(block)),
    }


def homeless(block, start_line):
    """Open items with no home, as problems: `(citation, file line, why)`."""
    route = route_ids()
    out = []
    for run, number, body, line in items_by_run(block):
        if classify_item(body) == "closed":
            continue
        kind, detail = home(body, route)
        why = {
            "missing": "no **Slotted:** line",
            "landed": f"slotted only to {detail}, which landed without it",
            "unknown": f"its **Slotted:** line names no phase, row or card the route has ({detail})",
        }.get(kind)
        if why:
            out.append((cite(run, number), start_line + line, why))
    return out


def duplicate_numbers(block):
    """Numbers used twice in one run, each `(run, number, [offsets])`."""
    seen = {}
    for run, number, _, line in items_by_run(block):
        seen.setdefault((run, number), []).append(line)
    return [(run, n, lines) for (run, n), lines in seen.items() if len(lines) > 1]


def intake_names(token, code):
    """Whether a Slotted line's `token` puts the item on `code`'s list. A
    lettered phase also takes what is slotted to its family as a whole
    ("TR-4" is TR-4a's or TR-4b's to decide), and a family takes what is
    slotted to any of its phases; a sibling's is not its. A name has no
    family: a split names each part."""
    if not LETTERED_CODE_RE.fullmatch(code):
        return token == code
    family = re.sub("[a-z]$", "", code)
    return token in (code, family) or (family == code and re.sub("[a-z]$", "", token) == code)


#: A backlog entry's Owner line, a bullet with the lines indented under it or
#: a row of the entry's table.
OWNER_RE = re.compile(r"^(?:- \*\*Owner\*\* — |\| \*\*Owner\*\* \| )(.+(?:\n  \S.*)*)", re.M)


def backlog_owners(text):
    """Each `backlog.md` entry's Owner line as `(file line, heading, owner)`,
    `owner` being `None` for an entry with no line this reads, which no
    intake can list by its home."""
    out = []
    for entry in re.finditer(r"^### (2\.\d+ .+)$", text, re.M):
        rest = text[entry.end():]
        end = re.search(r"^#{2,3} ", rest, re.M)
        owners = [" ".join(owner.group(1).rstrip("| ").split())
                  for owner in OWNER_RE.finditer(rest[:end.start()] if end else rest)]
        line = text.count("\n", 0, entry.start()) + 1
        out += [(line, entry.group(1), owner) for owner in owners or [None]]
    return out


def slotted(codes):
    """The intake list a phase reads at its first ticket: every open item
    whose Slotted line names one of `codes`, then those naming one only
    elsewhere in their text, which are context rather than its work, then
    every backlog entry whose Owner line names one, since an entry is homed
    by that line and has no item."""
    block, start_line = dm_block()
    route = route_ids()
    for code in codes:
        if route_status(code, route) is None:
            print(f"(no phase or roadmap row is called {code}; matching the text anyway)")
    slotted_here, mentioned = [], []
    for run, number, body, line in items_by_run(block):
        if classify_item(body) == "closed":
            continue
        text = slotted_text(body) or ""
        title = " ".join(body.split("\n")[0].replace("*", "").split()[1:])[:100]
        row = (cite(run, number), start_line + line, title, text)
        if any(intake_names(token, code) for token in ROUTE_TOKEN_RE.findall(text) for code in codes):
            slotted_here.append(row)
        elif any(re.search(rf"\b{re.escape(c)}\b", body) for c in codes):
            mentioned.append(row)
    print(f"Slotted to {', '.join(codes)} — {len(slotted_here)} open items:")
    for citation, line, title, text in slotted_here:
        print(f"\n- {citation} (codebase-state.md:{line}) {title}\n  Slotted: {text}")
    print(f"\nNamed in {len(mentioned)} more, whose Slotted line points elsewhere:")
    for citation, line, title, text in mentioned:
        print(f"- {citation} (codebase-state.md:{line}) {title}")
    entries = backlog_owners(read("plans/backlog.md"))
    owned = [entry for entry in entries if entry[2] is not None
             and any(intake_names(token, code) for token in ROUTE_TOKEN_RE.findall(entry[2]) for code in codes)]
    print(f"\nBacklog entries whose Owner line names {', '.join(codes)} — {len(owned)}:")
    for line, heading, owner in owned:
        print(f"\n- §{heading} (backlog.md:{line})\n  Owner: {owner}")
    # A shipped or graduated entry has left the backlog's care, with or without the line.
    ownerless = [entry for entry in entries if entry[2] is None and "✅" not in entry[1] and "shipped" not in entry[1]]
    print(f"\nBacklog entries with no Owner line, which may be this row's and are not listed above — {len(ownerless)}:")
    for line, heading, _ in ownerless:
        print(f"- §{heading} (backlog.md:{line})")


def selftest():
    """The parser, checked against a fixture — so a regex edit cannot silently
    change what the board counts. Runs under --check and --write."""
    fixture = "\n".join([
        "## Deferred Migrations",
        "1. **Done — ✅ closed (2026-01-01).** text",
        "2. **Open, old style.** Unreachable today: no caller. **Sized:** 5 lines.",
        "2a. **Lettered sub-item.** **Reachability (2026-09-03):** reachable — wrong today; x. **Sized:** y.",
        "3. **Verdict wins over the heading ✅.** **Reachability (2026-09-03):** unreachable — no card.",
        "4. **Says nothing.** prose",
        "   1. an indented list is not an item",
        "5. **Record.** **Reachability (2026-09-03):** nothing owed — a record.",
        "6. **Perf.** **Reachability (2026-09-03):** reachable — not wrong; perf only. **Sized:** z.",
        "7. **Struck later.** **Reachability (2026-09-03):** closed — PR #1.",
        "1) a prose list with the paren delimiter is not an item",
        "8. **Reachable, and the log is the only reader.** **Reachability (2026-09-15):** reachable but not wrong today — nothing reads it. **Sized:** 10 lines.",
        "9. **Bolded negation.** **Reachability (2026-09-15):** reachable, and **not wrong today**, and the bound is worth stating.",
    ])
    items = split_items(fixture)
    got = [classify_item(b) for b in items]
    want = ["closed", "unreachable", "reachable_wrong", "unreachable", "unstated",
            "none_owed", "reachable_ok", "closed", "reachable_ok", "reachable_ok"]
    assert len(items) == 10, f"selftest: expected 10 items, parsed {len(items)}"
    assert got == want, f"selftest: {got} != {want}"
    sized = sum(1 for b, c in zip(items, got) if c != "closed" and "Sized:" in b)
    assert sized == 4, f"selftest: sized {sized} != 4"
    # Homes, runs and numbers: a Slotted line ends at the next bold label, a
    # landed phase is no home, a "Before" section numbers its own run, and
    # one number twice in a run is a duplicate even across its sections.
    open_ = "**Reachability (2026-10-09):** unreachable — x."
    runs = "\n".join([
        "## Deferred Migrations",
        "### Found by X",
        f"1. **Phase.** {open_} **Slotted:** TR-3c, its reflexive window.",
        f"2. **Retired slot.** {open_} **Slotted:** with the first registered card that names a counter.",
        "3. **Record.** **Reachability (2026-10-09):** nothing owed — x. **Slotted:** none — a record of x.",
        f"4. **Landed.** {open_} **Slotted:** TR-3b, beside it. **Sized:** with TR-3c.",
        f"5. **Missing.** {open_}",
        f"6. **Unknown.** {open_} **Slotted:** `backlog.md` §2.3's first PR.",
        f"6a. **Named.** {open_} **Slotted:** `combat-restrictions`, beside TR-3b.",
        f"6b. **Backticked, not a name.** {open_} **Slotted:** with the first registered card that `fuzz-games` deals.",
        f"6c. **Mistyped name.** {open_} **Slotted:** `combat-restriction`.",
        f"7. **Lettered family.** {open_} **Slotted: TR-4a** with the frame.",
        "### Before Layers",
        f"1. **Own run.** {open_} **Slotted:** Phase 10, the harness.",
        "### Found by Y",
        "2. **Twice.** **Reachability (2026-10-09):** closed — PR #2.",
    ])
    route = {"TR-3b": True, "TR-3c": False, "TR-4": False, "Phase 10": False, "combat-restrictions": False}
    numbered = items_by_run(runs)
    assert [(r, n) for r, n, _, _ in numbered] == [
        ("main", "1"), ("main", "2"), ("main", "3"), ("main", "4"), ("main", "5"), ("main", "6"),
        ("main", "6a"), ("main", "6b"), ("main", "6c"), ("main", "7"),
        ("Before Layers", "1"), ("main", "2"),
    ], f"selftest: runs {numbered}"
    homes = [home(b, route)[0] for _, _, b, _ in numbered if classify_item(b) != "closed"]
    assert homes == ["route", "unknown", "record", "landed", "missing", "unknown", "route", "unknown", "unknown",
                     "route", "route"], f"selftest: homes {homes}"
    last = slotted_text(numbered[9][2])
    assert last == "TR-4a with the frame.", f"selftest: a section's last Slotted line ran on: {last!r}"
    dupes = [(r, n, len(lines)) for r, n, lines in duplicate_numbers(runs)]
    assert dupes == [("main", "2", 2)], f"selftest: duplicates {dupes}"
    intake = [intake_names(t, c) for t, c in [("TR-4", "TR-4a"), ("TR-4b", "TR-4a"), ("TR-4a", "TR-4"), ("TR-40", "TR-4"),
                                              ("combat-restrictions", "combat-restrictions"), ("combat-restriction", "combat-restrictions")]]
    assert intake == [True, False, True, False, True, False], f"selftest: intake {intake}"
    # The Sizing table: a row lands by the route or by its own cell, half a
    # phase takes nothing from the phase, and a date is no PR count.
    sized = [
        ["1", "`route-review`", "", "1", "", "", ""],
        ["2", "`card-layer-cleanup`", "✅ 2026-11-01", "5", "", "", ""],
        ["3", "`route-review`, a landed part", "", "4", "", "", ""],
        ["3", "`double-faced-cards`, the transform half", "", "2–3", "", "", ""],
        ["3", "`token-copies`", "", "1", "", "", ""],
        ["4", "`A6e`, readiness", "", "2", "", "", ""],
        ["5", "`token-copies`", "", "1", "", "", ""],
    ]
    known = {"route-review": True, "card-layer-cleanup": False, "double-faced-cards": False, "token-copies": False, "A6e": True}
    parsed, problems = sized_rows(sized, known)
    assert problems == [], f"selftest: sizing problems {problems}"
    assert [(r.step, r.landed, r.low, r.high) for r in parsed] == [
        (1, True, 1, 1), (2, True, 5, 5), (3, True, 4, 4), (3, False, 2, 3), (3, False, 1, 1), (4, True, 2, 2), (5, False, 1, 1),
    ], f"selftest: sized {parsed}"
    # A step is past its forecast on what it took and the least it has left,
    # and one the forecast leaves out is read against itself.
    forecast, forecast_problems = forecast_rows([["1", "1"], ["2", "5"], ["3", "5–6"], ["4", "1"], ["9", "2"], ["x", "1"]], {1, 2, 3, 4, 5})
    assert forecast == {1: (1, 1), 2: (5, 5), 3: (5, 6), 4: (1, 1), 9: (2, 2)}, f"selftest: forecast {forecast}"
    assert len(forecast_problems) == 3, f"selftest: forecast problems {forecast_problems}"
    steps, rows_to_come = way_to_v05(parsed, forecast)
    assert [(s.step, s.state, s.taken, s.left, s.forecast, past_forecast(s)) for s in steps] == [
        (1, "landed", 1, (0, 0), (1, 1), False), (2, "landed", 5, (0, 0), (5, 5), False),
        (3, "1 of 3 landed", 4, (3, 4), (5, 6), True), (4, "landed", 2, (0, 0), (1, 1), True),
        (5, "to come", 0, (1, 1), (1, 1), False),
    ], f"selftest: steps {steps}"
    assert steps[2].rows == "`double-faced-cards`, the transform half and 1 more", f"selftest: {steps[2]}"
    assert pr_span(rows_to_come) == "4–5", f"selftest: rows to come {rows_to_come}"
    wrong = [["5", "`not-a-row`", "", "1", "", "", ""], ["6", "`token-copies`", "", "2026-10", "", "", ""], ["7", "`token-copies`", "", "4–2", "", "", ""],
             ["8", "`route-review`", "", "1–2", "", "", ""]]
    assert len(sized_rows(wrong, known)[1]) == 4, f"selftest: {sized_rows(wrong, known)[1]}"
    own = [["1", "`token-copies`, a half", "✅ 2026-11-13 (PRs #321, #322)", "2", "", "", ""], ["1", "`token-copies`, the rest", "✅ soon", "1", "", "", ""]]
    own_rows, own_problems = sized_rows(own, known)
    assert [r.landed for r in own_rows] == [True, False], "selftest: a row's own State cell"
    assert len(own_problems) == 1 and "cannot read" in own_problems[0], f"selftest: an unreadable State cell {own_problems}"
    assert len(forecast_rows([["1", "3–2"], ["1", "1"]], {1})[1]) == 2, "selftest: a forecast running down, and a step twice"
    # A subsystem closes with the last row its doc defines; a row to come that
    # no doc defines closes none.
    closing, undefined = subsystems(parsed, {"card-layer-cleanup": "x.md", "token-copies": "y.md"})
    assert [(doc, landed_rows, rows, last.step, last.landed) for doc, landed_rows, rows, last in closing] == [
        ("x.md", 1, 1, 2, True), ("y.md", 0, 2, 5, False),
    ], f"selftest: subsystems {closing}"
    assert undefined == ["`double-faced-cards`, the transform half"], f"selftest: undefined rows {undefined}"
    # The cards to write: two the census proved, one whose need has landed
    # since, one waiting on a row, one on a gap and one with no record; and
    # the two ways the records misstate a card.
    proved = {"needs": [], "board": "passed", "uncovered": [], "stand_ins": 0, "paragraphs": 1, "keyword_paragraphs": 0,
              "checks_failed": [], "survived": [], "errors": [], "panics": [], "reached": 3}
    records = {
        "Proved": {"name": "Proved", **proved},
        "Also Proved": {"name": "Also Proved", **proved},
        "Landed Since": {"name": "Landed Since", **proved, "needs": ["card-layer-cleanup"]},
        "Waiting": {"name": "Waiting", **proved, "needs": ["card-layer-cleanup", "token-copies"]},
        "Registered": {"name": "Registered", **proved, "needs": ["zz-no-row"], "board": "no board"},
    }
    census = gap_census()
    gap, homed = next((gap, home) for gap, home in census.GAPS.items() if home.row != census.CHEAP_GAPS and gap not in census.BUILT_GAPS)
    records["On A Gap"] = {"name": "On A Gap", **proved, "needs": [census.GAP_PREFIX + gap]}
    with_the_gaps_row = parsed + [SizedRow(6, f"`{homed.row}`", False, 1, 1)]
    unregistered = {"Proved", "Also Proved", "Landed Since", "Waiting", "On A Gap", "Unread"}
    to_write, record_problems = cards_to_write(records, with_the_gaps_row, unregistered)
    assert to_write == CardsToWrite(2, 1, 2, 1), f"selftest: cards to write {to_write}"
    assert len(record_problems) == 1 and "zz-no-row" in record_problems[0], f"selftest: record problems {record_problems}"
    records["Proved"]["board"] = "no board"
    unproved = cards_to_write(records, with_the_gaps_row, unregistered)[1]
    assert len(unproved) == 2 and "Proved writable and not proved" in unproved[1], f"selftest: unproved {unproved}"
    # An entry's Owner line, as a bullet with its continuation and as a table row.
    backlog = "\n".join([
        "## 0. What an entry is", "| **Owner** | where it will be built |",
        "## 2. Entries", "### 2.1 A bullet", "- **Owner** — `token-copies`, which", "  builds it.", "- **Atoms** — none",
        "### 2.2 A table", "| **Owner** | after v0.5, with A6e |", "### 2.3 No owner", "text",
    ])
    assert backlog_owners(backlog) == [
        (4, "2.1 A bullet", "`token-copies`, which builds it."), (8, "2.2 A table", "after v0.5, with A6e"),
        (10, "2.3 No owner", None),
    ], f"selftest: backlog owners {backlog_owners(backlog)}"
    path = ["**After 6, in order** x:", "`route-review`; `token-copies`; `roadmap-v2.md`", "**v0.5 is** `not-yet-sized`"]
    assert unsized_route_rows(path, ["`route-review`"]) == ["token-copies"], "selftest: an unsized route row"
    # The phase index's parser: coded and plain-named ✅ headings, a parent
    # heading and a closed phase, a trace page's ✅, a sizing table's cells.
    phases = "\n".join([
        "### 7a. RS-1 — the spine and Tier 2 — ✅ 2026-08-31",
        "#### RE-1 — skips, and the turn queue (CR 614.1b) — ✅ landed 2026-09-11",
        "### The editor's advanced settings, first part — ✅ landed 2026-10-06",
        "### Phase RB — the pipeline — ✅ landed 2026-08-26",
        "### Phase RE — the remaining event kinds — sized 2026-09-11, nine PRs",
        "#### RD-5 — partial redirection — ❌ gate closed 2026-09-09",
        "#### Trace page — ✅ written at RE-2's close",
        "| **MA-2, X** | §3.7 |",
        "| **CV-3 — token copies** | a constructor |",
        "### TR-3 — delayed, reflexive, and \"until\" — TR-3a and TR-3b (2,800–3,600)",
        "| **`spell-copies` — a copy of a spell on the stack** | `StackEntry` copy |",
        "### `rust-notes` — the Rust notes — ✅ 2026-11-01",
    ])
    landed, defined = phases_in(phases)
    assert landed == [
        ("2026-08-31", "The spine and Tier 2", "RS-1"),
        ("2026-09-11", "Skips, and the turn queue", "RE-1"),
        ("2026-10-06", "The editor's advanced settings, first part", None),
        ("2026-08-26", "The pipeline", "RB"),
        ("2026-11-01", "The Rust notes", "rust-notes"),
    ], f"selftest: landed {landed}"
    assert defined == [
        ("RS-1", "The spine and Tier 2"),
        ("RE-1", "Skips, and the turn queue"),
        ("MA-2", "X"),
        ("CV-3", "Token copies"),
        ("TR-3", 'Delayed, reflexive, and "until"'),
        ("spell-copies", "A copy of a spell on the stack"),
        ("rust-notes", "The Rust notes"),
    ], f"selftest: defined {defined}"


def table_rows(path, marker, width):
    """The body rows of the `width`-column table between `<!-- {marker}: begin -->`
    and its end marker, each as its cells."""
    text = read(path)
    start = text.index(f"<!-- {marker}: begin -->")
    end = text.index(f"<!-- {marker}: end -->", start)
    rows = []
    for line in text[start:end].split("\n"):
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) != width or cells[0] in ("Floor", "Step") or set(cells[0]) <= set("-"):
            continue
        rows.append(cells)
    assert rows, f"the table between {path}'s {marker} markers is empty"
    return rows


#: `roadmap-v2.md`'s Sizing table: step, row, state, PRs, stop, splits, rests on.
SIZING_COLUMNS = 7
#: A PR cell, whole: a count or a range. Anchored, since a loose number
#: reads a date such as 2026-10 as a range.
PRS_RE = re.compile(r"(\d+)(?:–(\d+))?\Z")
#: A row's own record that it landed, for a row with no heading to carry one:
#: the day, and the PRs that built it where the cell names them.
OWN_LANDED_RE = re.compile(r"✅ \d{4}-\d{2}-\d{2}(?: \(PRs? #\d+(?:, #\d+)*\))?\Z")


#: A Sizing table row. `row` is the cell as written, which may name part of a
#: phase, and `low` and `high` its PRs: what it is sized at, or, once it has
#: landed, what it took.
SizedRow = namedtuple("SizedRow", "step row landed low high")
#: `roadmap-v2.md`'s forecast table: step, PRs.
FORECAST_COLUMNS = 2
#: One step of the route: its rows named, how many have landed, the PRs its
#: landed rows took, the range its rows to come are sized at, and its range
#: at the route's last forecast.
Step = namedtuple("Step", "step rows state taken left forecast")


def sized_rows(rows, route):
    """The Sizing table's rows as `SizedRow`s, and what is wrong with it.

    A row has landed when the route records its name as landed, or, for a row
    with no heading of its own (half a phase, a §C paragraph), when its State
    cell does. Half of a phase takes nothing from the phase still being open."""
    parsed, problems = [], []
    for step, row, state, prs, *_ in rows:
        token = ROUTE_TOKEN_RE.search(row)
        status = route_status(token.group(0), route) if token else None
        size = PRS_RE.match(prs)
        if status is None:
            problems.append(f"the Sizing table's {row} is no phase or row the route names")
        if not size or not step.isdigit():
            problems.append(f"the Sizing table's {row} wants a step and a PR count or range, not {step!r} and {prs!r}")
            continue
        low = int(size.group(1))
        high = int(size.group(2) or low)
        if low > high:
            problems.append(f"the Sizing table's {row} runs {low} down to {high}")
        landed = bool(status) or bool(OWN_LANDED_RE.match(state))
        if state and not OWN_LANDED_RE.match(state):
            problems.append(f"the Sizing table's {row} has a State cell the board cannot read, {state!r}: "
                            "leave it empty, or write `✅ <date>` or `✅ <date> (PR #N)`")
        if landed and low != high:
            problems.append(f"the Sizing table's {row} landed, and its PR cell is still a range: write what it took")
        parsed.append(SizedRow(int(step), row, landed, low, high))
    return parsed, problems


def forecast_rows(rows, steps):
    """The forecast table as `{step: (low, high)}`, and what is wrong with it
    against the Sizing table's `steps`: one it leaves out, or one it holds
    that has no rows."""
    forecast, problems = {}, []
    for step, prs in rows:
        size = PRS_RE.match(prs)
        if not step.isdigit() or not size:
            problems.append(f"the forecast wants a step and a PR count or range, not {step!r} and {prs!r}")
            continue
        low, high = int(size.group(1)), int(size.group(2) or size.group(1))
        if low > high:
            problems.append(f"the forecast's step {step} runs {low} down to {high}")
        if int(step) in forecast:
            problems.append(f"the forecast holds step {step} twice")
        forecast[int(step)] = (low, high)
    problems += [f"step {step} has rows in the Sizing table and no forecast" for step in sorted(steps - set(forecast))]
    problems += [f"the forecast holds step {step}, which has no rows in the Sizing table" for step in sorted(set(forecast) - steps)]
    return forecast, problems


def unsized_route_rows(critical_path_lines, row_cells):
    """Every phase the critical path names between item 6 and v0.5 that no
    Row cell of the Sizing table names. Only that column: a name cited in
    another row's Rests-on cell is not a size for it."""
    text = "\n".join(critical_path_lines)
    route = text[text.index("**After 6, in order**"):text.index("**v0.5 is")]
    named = "\n".join(row_cells)
    return [name.strip("`") for name in dict.fromkeys(re.findall(NAME, route)) if name not in named]


def sizing():
    """The table as `sized_rows` reads it, and every problem with it."""
    rows = table_rows("plans/roadmap-v2.md", "v0.5 sizing", SIZING_COLUMNS)
    parsed, problems = sized_rows(rows, route_ids())
    for name in unsized_route_rows(critical_path(), [cells[1] for cells in rows]):
        problems.append(f"the critical path names {name}, and the Sizing table has no row for it")
    return parsed, problems


def forecast_by_step():
    """The forecast as `forecast_rows` reads it, and every problem with it."""
    rows = table_rows("plans/roadmap-v2.md", "v0.5 forecast", FORECAST_COLUMNS)
    return forecast_rows(rows, {sized.step for sized in sizing()[0]})


def span(low, high):
    return str(low) if low == high else f"{low}–{high}"


def pr_span(rows):
    return span(sum(r.low for r in rows), sum(r.high for r in rows))


def way_to_v05(parsed, forecast):
    """Each step as a `Step`, and every row to come. A step the forecast
    leaves out is read against its own range, so it is never past it."""
    steps = {}
    for sized in parsed:
        steps.setdefault(sized.step, []).append(sized)
    out = []
    for step, rows in steps.items():
        to_come = [r for r in rows if not r.landed]
        shown = to_come or rows
        also = len(shown) - 1
        state = "landed" if not to_come else "to come" if len(to_come) == len(rows) else f"{len(rows) - len(to_come)} of {len(rows)} landed"
        named = f"{shown[0].row} and {also} more" if also else shown[0].row
        taken = sum(r.low for r in rows if r.landed)
        left = (sum(r.low for r in to_come), sum(r.high for r in to_come))
        out.append(Step(step, named, state, taken, left, forecast.get(step, (taken + left[0], taken + left[1]))))
    return out, [r for r in parsed if not r.landed]


def past_forecast(step):
    """Whether what a step took and the least it has left pass its forecast's top."""
    return step.taken + step.left[0] > step.forecast[1]


def defined_in():
    """The architecture doc that defines each phase, by its code or name."""
    landed, to_build = phase_index()
    docs = {code: doc for _, _, code, doc in landed if code}
    docs.update({code: doc for _, code, doc in to_build if code not in docs})
    return docs


def subsystems(parsed, defined_in):
    """Each architecture doc that defines a row of the Sizing table, as `(doc,
    rows landed, rows, the row that closes it)`, the last of its rows in the
    table's order, soonest to close first; and the rows to come no doc
    defines, which close none until a design names them in one."""
    by_doc, undefined = {}, []
    for sized in parsed:
        token = ROUTE_TOKEN_RE.search(sized.row)
        doc = defined_in.get(token.group(0)) if token else None
        if doc:
            by_doc.setdefault(doc, []).append(sized)
        elif not sized.landed:
            undefined.append(sized.row)
    closing = [(doc, sum(r.landed for r in rows), len(rows), rows[-1]) for doc, rows in by_doc.items()]
    return sorted(closing, key=lambda subsystem: (subsystem[3].step, subsystem[0])), undefined


def floors():
    """Each floor's limit and latest reading, with the room it leaves.

    The limits are `engineering-practices.md#the-floors`'s, a rule, and the
    readings `fuzz-record.md#the-latest-floor-readings`, a measurement; the two
    tables name the same floors in the same order, and this joins them. The room
    is how far the reading is from its limit, as a fraction of the limit's side:
    a reading above a "≥" floor by half is 50%, a reading under a "≤" floor by a
    third of itself is 33%.
    """
    limits = table_rows("plans/engineering-practices.md", "floors", 4)
    readings = table_rows("plans/fuzz-record.md", "floor readings", 4)
    named = [row[0] for row in limits], [row[0] for row in readings]
    assert named[0] == named[1], f"the floors' limits and readings name different floors: {named}"
    rows = []
    for (floor, limit, _definition, _instrument), (_, reading, read_on, where) in zip(limits, readings):
        sign, bound = limit.split()
        value, bound = float(reading), float(bound)
        room = value / bound - 1 if sign == "≥" else bound / value - 1
        rows.append((floor, limit, reading, read_on, where, room))
    return rows


def landed_phases():
    """Phase codes whose architecture-doc heading records them as landed.

    A `###`/`####` heading carrying ✅, optionally behind the doc's own section
    number. **This is the whole list, not a lower bound** — it stopped being
    one on 2026-09-07, when the "can't" and copy tracks got the marker the
    other three already used and `CLAUDE.md` gave up carrying a second copy
    (`engineering-practices.md#line-budgets`). A doc that does not follow the convention
    is invisible here, which is the one way this can under-report and the
    reason the convention is written down rather than inferred.
    """
    out = {}
    for doc in ARCH_DOCS:
        try:
            text = read(doc)
        except FileNotFoundError:
            continue
        for m in re.finditer(
            rf"^#{{2,4}} (?:[0-9]+[a-z]?\. )?(?:Phase )?([A-Z]{{2}}-?[0-9]*[a-z]?|{NAME}) — (.+)$", text, re.M
        ):
            code, rest = m.group(1).strip("`"), m.group(2)
            if "✅" in rest:
                out[code] = doc
    return out


#: A phase's code: a family's letters, and a number where the family has
#: phases (`SU-1`, `RC-4b`), or the letters alone for a one-phase family
#: (`RB`); or a phase named since codes closed, in backticks.
CODE_RE = re.compile(rf"[A-Z]{{2}}(?:-[0-9]+[a-z]?)?|{NAME}")
#: A landed phase's heading: its title, then ✅ and the day it landed.
LANDED_HEADING_RE = re.compile(
    r"^#{2,4} (?:[0-9]+[a-z]?\. )?(?:Phase )?(.+?) — ✅ (?:landed )?(\d{4}-\d{2}-\d{2})\s*$", re.M
)
#: A phase the doc defines: a heading, or a sizing table's bold first cell
#: (`| **MA-2, X** |`, `` | **`token-copies` — a second constructor** | ``).
DEFINED_RE = re.compile(
    rf"^(?:#{{2,5}} (?:[0-9]+[a-z]?\. )?(?:Phase )?|\| \*\*)([A-Z]{{2}}-[0-9]+[a-z]?|{NAME})(?: —|,) ([^*|\n]+)", re.M
)


def plain_name(text):
    """A title as the index shows it: its words, without the parentheses
    that size it or cite its rules, capitalized."""
    name = re.sub(r"\s*\([^)]*\)", "", text).strip()
    return name[:1].upper() + name[1:]


def phases_in(text):
    """One doc's phases: those its ✅ headings record as landed, each
    `(date, name, code or None)`, a plain-named heading's code `None`; and
    the numbered ones it defines, `(code, name)`, closed (❌) ones left out,
    in the doc's order."""
    landed = []
    for m in LANDED_HEADING_RE.finditer(text):
        first, _, rest = m.group(1).partition(" — ")
        code, title = (first.strip("`"), rest) if CODE_RE.fullmatch(first) and rest else (None, m.group(1))
        landed.append((m.group(2), plain_name(title.split(" — ")[0]), code))
    defined = [(m.group(1).strip("`"), plain_name(m.group(2).split(" — ")[0]))
               for m in DEFINED_RE.finditer(text) if "❌" not in m.group(2)]
    return landed, defined


def phase_index():
    """Every phase the architecture docs name, by name: the landed ones in the
    order they landed, `(date, name, code, doc)`; and the numbered ones no ✅
    heading records, `(name, code, doc)`, each by its first definition. A
    family's letters alone name a phase only in a ✅ heading, since a parent
    heading (`Phase RE`) and the superseded plan's (`Phase LA`) use them too."""
    landed, defined = [], []
    for doc in ARCH_DOCS:
        here, named = phases_in(read(doc))
        landed += [(date, name, code, doc) for date, name, code in here]
        defined += [(code, name, doc) for code, name in named]
    landed.sort(key=lambda phase: phase[0])
    seen = {code for _, _, code, _ in landed if code}
    to_build = []
    for code, name, doc in defined:
        if code not in seen:
            seen.add(code)
            to_build.append((name, code, doc))
    return landed, to_build


def doc_name(doc):
    return doc.removeprefix("plans/")


def critical_path():
    """`CLAUDE.md`'s critical-path section, verbatim.

    Lifted rather than summarised: that section *is* the ordering authority
    (it says so), and a board that paraphrased it would be a second answer to
    a question that is supposed to have one.
    """
    text = read("CLAUDE.md")
    start = text.index("## Critical path to v1")
    end = text.index("\n## ", start + 10)
    body = text[start:end].split("\n")
    return [l for l in body[1:] if l.strip()]


def open_handoffs():
    """`plans/handoffs/*.md` — the project's own marker for a half-finished phase.

    `CLAUDE.md`'s authority table says these are deleted when the work lands, so
    a file here is unfinished work by construction. Nothing else surfaces them,
    which is why they are on the board.
    """
    d = ROOT / "plans" / "handoffs"
    return sorted(p.name for p in d.glob("*.md")) if d.is_dir() else []


# --------------------------------------------------------------------------
# Render
# --------------------------------------------------------------------------

def render():
    c, dm, hand = counts(), deferred_migrations(), open_handoffs()
    index_landed, index_to_build = phase_index()
    L = []
    L.append("<!-- GENERATED by plans/check_state_of_play.py --write. Do not hand-edit:")
    L.append("     the check regenerates this and fails CI on any difference. To change")
    L.append("     what it says, change the tree it reads or the script that reads it. -->")
    L.append("")
    L.append("# State of play")
    L.append("")
    L.append("**What is done, what is next, and what the tree is carrying.** Generated from")
    L.append("the tree, so it")
    L.append("cannot be stale without CI saying so. It carries no opinions: everything here")
    L.append("is a number or a quotation, and the reasoning lives where it always did —")
    L.append("`codebase-state.md` for state, the architecture docs for design,")
    L.append("`roadmap-v2.md` for the route narrative.")
    L.append("")
    L.append("Refresh with `python plans/check_state_of_play.py --write`.")
    L.append("")
    L.append("## The critical path")
    L.append("")
    L.append("Quoted verbatim from `CLAUDE.md`, which owns the ordering.")
    L.append("")
    L.extend(critical_path())
    L.append("")
    L.append("## The way to v0.5")
    L.append("")
    L.append("Each step of `roadmap-v2.md` §3a, from the Sizing table (`roadmap-v2.md` §8),")
    L.append("which says what each range rests on. **Taken** is what a step's landed rows")
    L.append("took, **left** what its rows to come are sized at, and **forecast** the")
    L.append("step's range when the route was last forecast, in the table under the Sizing")
    L.append("table. A row has landed when a ✅ heading or roadmap row records it under its")
    L.append("name, or, with neither, when its own State cell does. Route PRs only: §8")
    L.append("says what the audits, the dev GUI and the detours beside them have cost.")
    L.append("")
    L.append("| Step | Rows | State | PRs taken | PRs left | Forecast |")
    L.append("|---:|---|---|---:|---:|---:|")
    steps, rows_to_come = way_to_v05(sizing()[0], forecast_by_step()[0])
    for step in steps:
        cells = [str(step.taken), span(*step.left) if step.left[1] else "—", span(*step.forecast)]
        if past_forecast(step):
            cells = [f"**{cell}**" for cell in cells]
        L.append(f"| {step.step} | {step.rows} | {step.state} | {' | '.join(cells)} |")
    whole = span(sum(s.forecast[0] for s in steps), sum(s.forecast[1] for s in steps))
    L.append(f"| | **To v0.5** | **{len(rows_to_come)} rows to come** | **{sum(s.taken for s in steps)}** "
             f"| **{pr_span(rows_to_come)}** | **{whole}** |")
    L.append("")
    past = [str(step.step) for step in steps if past_forecast(step)]
    if past:
        L.append(f"**Past its forecast: step {', '.join(past)}.** What it took and the least it has left")
        L.append("come to more than its forecast's top, so the ranges ahead are due a new")
        L.append("reading: `python plans/check_state_of_play.py --forecast` prints the table")
        L.append("as the rows read today.")
    else:
        L.append("No step is past its forecast: one is when what it took and the least it has")
        L.append("left come to more than its forecast's top, and the route is forecast again then.")
    L.append("")
    to_write = deck_cards_to_write()[0]
    L.append(f"**Deck cards registered: {c['deck_cards_registered']} of {c['deck_cards']}**, the five lists' distinct cards less")
    L.append(f"basics. **Could be written today: {to_write.proved + to_write.landed_since} more**, with no need left on the route:")
    L.append(f"{to_write.proved} the gap census proved writable and {to_write.landed_since} whose needs have landed since, which a")
    L.append(f"registration proves. {to_write.waiting} wait on a row or a gap, and the census left {to_write.unread} unread")
    L.append("(`plans/references/card-census/README.md#the-gap-census`).")
    L.append("")
    L.append("### When each subsystem closes")
    L.append("")
    L.append("A subsystem closes when the last row its architecture doc defines lands, and")
    L.append("the hunt is the next PR (`spine-phase-audit.md#when-it-runs`). A doc defines a")
    L.append("row by naming it in a heading or in a sizing table's first cell, in bold.")
    L.append("")
    L.append("| Architecture doc | Rows landed | Closes with | Step |")
    L.append("|---|---:|---|---:|")
    closing, undefined = subsystems(sizing()[0], defined_in())
    for doc, landed_rows, rows, last in closing:
        L.append(f"| `{doc_name(doc)}` | {landed_rows} of {rows} | {'closed, with ' if last.landed else ''}{last.row} | {last.step} |")
    L.append("")
    L.append("No architecture doc defines these rows to come, so none closes a subsystem")
    L.append(f"until its design names it in one: {'; '.join(undefined)}.")
    L.append("")
    L.append("## Phases by name")
    L.append("")
    L.append("Every phase the architecture docs (`plans/*-architecture.md`, globbed) name,")
    L.append("by its plain name, with what text cites it by: a landed phase's code, or a")
    L.append("phase still to come's name, hyphenated (`engineering-practices.md#naming-a-phase`).")
    L.append("The codes those names replaced are in `plans/archive/route-codes-renamed.md`.")
    L.append("")
    L.append("### Landed")
    L.append("")
    L.append("A `###`/`####` heading carrying ✅ and the day, coded or plain-named, in the")
    L.append("order they landed. **This is where landed status lives** — `CLAUDE.md` owns")
    L.append("the ordering and says nothing about progress, so there is one answer and it")
    L.append("is derived.")
    L.append("")
    L.append("| Landed | Phase | Cited as | Doc |")
    L.append("|---|---|---|---|")
    for date, name, code, doc in index_landed:
        L.append(f"| {date} | {name} | {f'`{code}`' if code else '—'} | `{doc_name(doc)}` |")
    L.append("")
    L.append("### Named, not yet landed")
    L.append("")
    L.append("A phase a doc defines by code or name in a heading or a sizing table's first")
    L.append("cell, and no ✅ heading records, in the docs' order.")
    L.append("")
    L.append("| Phase | Cited as | Doc |")
    L.append("|---|---|---|")
    for name, code, doc in index_to_build:
        L.append(f"| {name} | `{code}` | `{doc_name(doc)}` |")
    L.append("")
    L.append("## Counts")
    L.append("")
    L.append("| | |")
    L.append("|---|---:|")
    L.append(f"| Cards registered | {c['cards']} |")
    L.append(f"| …of them in `PERFORMANCE_POOL` | {c['pool']} |")
    L.append(f"| `#[test]` functions | {c['tests']} |")
    L.append("")
    L.append("Coverage is a separate query and stays one: `python plans/specdb.py stats`.")
    L.append("")
    L.append("## Performance floors")
    L.append("")
    L.append("Each floor's limit (`engineering-practices.md#the-floors`) and latest reading")
    L.append("(`fuzz-record.md#the-latest-floor-readings`), and how much room it leaves. A floor")
    L.append("with less than 20% room is bolded.")
    L.append("")
    L.append("| Floor | Limit | Latest | Room | Read on |")
    L.append("|---|---:|---:|---:|---|")
    for floor, limit, reading, read_on, _where, room in floors():
        cell = f"{room:.0%}"
        if room < 0.20:
            floor, cell = f"**{floor}**", f"**{cell}**"
        L.append(f"| {floor} | {limit} | {reading} | {cell} | {read_on} |")
    L.append("")
    L.append("## Debt — `codebase-state.md`'s Deferred Migrations")
    L.append("")
    L.append("| | |")
    L.append("|---|---:|")
    L.append(f"| Numbered items | {dm['items']} |")
    L.append(f"| …closed, still recorded | {dm['closed']} |")
    L.append(f"| …open — unreachable, and says why | {dm['unreachable']} |")
    L.append(f"| **…open — reachable, wrong today** | **{dm['reachable_wrong']}** |")
    L.append(f"| …open — reachable, not wrong (perf, a name, a harness) | {dm['reachable_ok']} |")
    L.append(f"| …open — nothing to build, a record for a later phase | {dm['none_owed']} |")
    L.append(f"| **…open — reachability *not* stated** | **{dm['unstated']}** |")
    L.append(f"| …open, carrying an explicit `**Sized:**` | {dm['sized']} of {dm['open']} |")
    L.append("")
    L.append("Two bolded rows. \"Not stated\" is the one to act on: an item that does not say")
    L.append("why it cannot bite yet is an unchecked claim rather than a deferral. \"Wrong")
    L.append("today\" is the list of known wrong answers a fuzz game can reach — bug")
    L.append("reports filed as deferrals, each named in the section. A dated")
    L.append("`**Reachability (YYYY-MM-DD):**` line is what the board reads; the date says")
    L.append("when the verdict was last derived against the tree, because reachability")
    L.append("only ever grows.")
    L.append("")
    L.append("Where each open item is homed, read off its `**Slotted:**` line:")
    L.append("")
    L.append("| | |")
    L.append("|---|---:|")
    L.append(f"| …slotted to a phase or roadmap row still to come | {dm['home_route']} |")
    L.append(f"| …a record, nothing to build | {dm['home_record']} |")
    L.append(f"| **…open with no home** | **{dm['homeless']}** |")
    L.append(f"| **A number used twice in one run** | **{dm['duplicates']}** |")
    L.append("")
    L.append("**Both bolded rows fail the check.** A home is a `**Slotted:**` line naming a")
    L.append("phase or `roadmap-v2.md` §3a row still to come (Phases 9 and 10 by number,")
    L.append("having no rows yet), or \"none — a record\". \"With the first registered card")
    L.append("that …\" is no home since #294's review. A line naming only phases that have landed is")
    L.append("no home: they went without it. A phase reads its intake list at its first")
    L.append("ticket: `python plans/check_state_of_play.py --slotted cant-be-targeted`.")
    L.append("")
    L.append("### This is not `specdb owed`, and the two overlap nowhere")
    L.append("")
    L.append("| | `specdb owed` | Deferred Migrations |")
    L.append("|---|---|---|")
    L.append("| Unit | an **atom** — one scenario from the spec corpus | a **migration** — one code change |")
    L.append("| Looks | **backwards**, at phases already shipped | **forwards**, at systems not yet built |")
    L.append("| Lists | a scenario a shipped phase was ticketed to test and did not | scaffolding that will lie to whatever is built on it |")
    L.append("| Reachable now | yes, by construction — the behaviour shipped | usually not, which is why it is easy to forget |")
    L.append("| Read | as a report; a phase reads its rules' atoms when it starts, and no close waits on it | before a system's first ticket |")
    L.append("")
    L.append("**The seam between them is real.** A defect in shipped behaviour that has no")
    L.append("atom is in neither list — RC-5's item 61 is one, because the ruling it violates")
    L.append("was never written into the corpus. And `owed` reads only atoms ticketed `NEW`")
    L.append("in `SHIPPED_PHASES` unless `--all` is passed, and a covered atom proves one")
    L.append("scenario, not its rule, so an empty list is no evidence a rule is right: the")
    L.append("hunt checks a subsystem at its close. → `engineering-practices.md#the-spec-database`.")
    L.append("")
    L.append("## Half-finished work")
    L.append("")
    L.append("`plans/handoffs/*.md`. These are deleted when the work lands, so a file here")
    L.append("is an open plate.")
    L.append("")
    if hand:
        for h in hand:
            L.append(f"- `plans/handoffs/{h}`")
    else:
        L.append("- (none — nothing half-finished)")
    L.append("")
    L.append("## What this file deliberately does not know")
    L.append("")
    L.append("Branches and pull requests. They come from git and `gh`, which a shallow CI")
    L.append("checkout cannot answer the same way a local clone does, so they would make")
    L.append("this file un-checkable. Ask directly:")
    L.append("")
    L.append("```bash")
    L.append("python plans/check_state_of_play.py --flight")
    L.append("```")
    L.append("")
    return "\n".join(L)


# --------------------------------------------------------------------------
# Checks
# --------------------------------------------------------------------------

def contradictions():
    """A phase its doc records as landed must not be 'next' on the critical path."""
    landed = landed_phases()
    path = "\n".join(critical_path())
    bad = []
    for code in landed:
        if re.search(rf"\*\*{re.escape(code)} (?:is )?next\*\*|\b{re.escape(code)} is next\b", path):
            bad.append(code)
    return bad


LANDED_STUB_MAX = 40


def oversized_landed():
    """Landed sections that kept their body in the live doc.

    A ✅ heading marks a shipped phase, and what a shipped phase's section
    holds is a record — the design as sized, what the building changed, the
    measurement. Records are append-only, and by 2026-09-11 they were 28% of
    the replacement doc, which is how two answers to one question (skips) and a
    row that was wrong for sixteen days (discard) survived unread. The rule is
    `codebase-state.md`'s eviction rule applied to the architecture docs: the
    body moves to `plans/archive/<doc>-landed.md`, the heading stays with a stub
    and a pointer, and this is the gate (`engineering-practices.md#landing-a-phase`). A
    section's extent runs to the next heading of the same or a higher level, so
    a `#####` inside it cannot hide the overrun.
    """
    out = []
    for doc in ARCH_DOCS:
        try:
            lines = read(doc).split("\n")
        except FileNotFoundError:
            continue
        heads = [(i, l) for i, l in enumerate(lines) if re.match(r"^#{2,5} ", l)]
        for k, (i, l) in enumerate(heads):
            if "✅" not in l:
                continue
            level = len(l.split(" ")[0])
            end = len(lines)
            for i2, l2 in heads[k + 1:]:
                if len(l2.split(" ")[0]) <= level:
                    end = i2
                    break
            if end - i > LANDED_STUB_MAX:
                out.append((doc, l.lstrip("# ").split(" — ")[0], end - i))
    return out


def flight():
    # Each label is flushed before its child runs: `gh` and `git` inherit this
    # process's stdout and write to the fd directly, so an unflushed label sits
    # in Python's buffer while the child's output goes out ahead of it. Piped --
    # into `head`, a log, a terminal pane -- that printed the branch list under
    # "Open PRs:" and left "Branches not in origin/main:" empty at the end.
    import subprocess
    print("Open PRs:", flush=True)
    subprocess.run(["gh", "pr", "list", "--state", "open"], cwd=ROOT)
    print("\nBranches not in origin/main:", flush=True)
    subprocess.run(["git", "branch", "-a", "--no-merged", "origin/main"], cwd=ROOT)


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--write", action="store_true", help="regenerate plans/state-of-play.md")
    ap.add_argument("--check", action="store_true", help="exit 1 if the board is stale")
    ap.add_argument("--flight", action="store_true", help="branches and PRs (needs git/gh)")
    ap.add_argument("--slotted", nargs="+", metavar="NAME",
                    help="the open items slotted to a phase or roadmap row, and the backlog entries it owns: its intake list")
    ap.add_argument("--forecast", action="store_true",
                    help="roadmap-v2.md's forecast table as the Sizing table reads today, to paste at a new forecast")
    args = ap.parse_args()

    if args.flight:
        flight()
        return 0
    if args.forecast:
        print("| Step | PRs |\n|---|---|")
        for step in way_to_v05(sizing()[0], {})[0]:
            print(f"| {step.step} | {span(*step.forecast)} |")
        return 0
    if args.slotted:
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
        slotted(args.slotted)
        return 0

    selftest()
    fresh = render()

    if args.write:
        OUT.write_text(fresh, encoding="utf-8", newline="\n")
        print(f"wrote {OUT.relative_to(ROOT)} ({len(fresh.splitlines())} lines)")
        return 0

    if args.check:
        problems = []
        if not OUT.exists():
            problems.append(f"{OUT.relative_to(ROOT)} does not exist")
        elif OUT.read_text(encoding="utf-8").replace("\r\n", "\n") != fresh:
            problems.append(
                f"{OUT.relative_to(ROOT)} is stale — the tree moved and the board did not"
            )
        for code in contradictions():
            problems.append(
                f"CLAUDE.md's critical path calls {code} 'next', but an architecture "
                f"doc records it as landed"
            )
        block, start_line = dm_block()
        for citation, line, why in homeless(block, start_line):
            problems.append(f"codebase-state.md:{line}: {citation} has no home — {why}")
        for run, number, lines in duplicate_numbers(block):
            where = ", ".join(str(start_line + l) for l in lines)
            problems.append(f"codebase-state.md: {cite(run, number)} is numbered twice (lines {where})")
        problems += sizing()[1] + forecast_by_step()[1] + deck_cards_to_write()[1]
        for doc, code, n in oversized_landed():
            problems.append(
                f"{doc}: {code} is landed but keeps {n} lines in the live doc "
                f"(max {LANDED_STUB_MAX}) — evict the body to plans/archive/<doc>-landed.md "
                f"and leave a stub"
            )
        if problems:
            print("state-of-play: FAILED")
            for p in problems:
                print(f"  - {p}")
            print("\nFix with: python plans/check_state_of_play.py --write")
            print("(and if a phase landed, say so on the critical path in CLAUDE.md)")
            return 1
        print("state-of-play: current.")
        return 0

    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    print(fresh)
    return 0


if __name__ == "__main__":
    sys.exit(main())
