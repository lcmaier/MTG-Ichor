#!/usr/bin/env python3
"""
specdb - build and query the atomic-test spec database.

The spec corpus (plans/atomic-tests/sessions/*.md) is the AUTHORED source of
truth: ~1,760 atomic tests derived from a close read of the comprehensive
rules. The Rust test suite is the AUTHORED source of status. This script joins
them and derives everything else.

Nothing here should ever be hand-edited into the database. If a number looks
wrong, fix the session file or the test annotation and rebuild.

Coverage is declared by annotating a Rust test with a COVERS comment:

    // COVERS: ATOM-305.7-001, ATOM-305.6-002
    #[test]
    fn test_blood_moon_grants_intrinsic_mana() { ... }

Usage:
    python plans/specdb.py build          # (re)build spec.sqlite
    python plans/specdb.py stats          # coverage by phase
    python plans/specdb.py next           # uncovered atoms on the critical path
    python plans/specdb.py next --phase "Phase 5-Layers" --rule 613
    python plans/specdb.py show ATOM-305.7-001
    python plans/specdb.py orphans        # COVERS ids with no matching atom
    python plans/specdb.py gaps --chapter 6   # CR rules the corpus never examined

CR snapshots live in MTG-Rules/versions/<version>.txt, one official file per
version. Rule text is stored with a sha256 so a future `sync-rules` can diff
two snapshots. Rule NUMBERS are addresses, not identities - they shift when
WotC inserts a rule (CR 310.8 in The Hobbit bumped 310.8a to 310.9a). Atom IDs
are therefore permanent handles and must never be renumbered to follow the CR;
`// COVERS:` annotations in Rust source depend on them.
"""

import argparse
import hashlib
import importlib.util
import re
import sqlite3
import sys
from pathlib import Path

# Session files contain em-dashes and arrows; the Windows console defaults to
# cp1252 and would raise UnicodeEncodeError on them.
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

ROOT = Path(__file__).resolve().parent.parent
SESSIONS_DIR = ROOT / "plans" / "atomic-tests" / "sessions"
EXTRACT_SCRIPT = ROOT / "plans" / "atomic-tests" / "extract-phase-index.py"
CODE_DIRS = [ROOT / "mtgsim" / "src", ROOT / "mtgsim" / "tests"]
DB_PATH = ROOT / "plans" / "atomic-tests" / "spec.sqlite"

# Comprehensive Rules snapshots, one official .txt per version. The filename
# stem is the version label (tmnt, strixhaven, ...). BASELINE_VERSION is the
# snapshot the engine currently targets; `gaps` reports against it.
CR_DIR = ROOT / "MTG-Rules" / "versions"
BASELINE_VERSION = "tmnt"

# Phases on the critical path to v1, in the order they must land.
CRITICAL_PATH = ["Phase 5-Layers", "Phase 6", "Phase 7"]

ENTRY_RE = re.compile(r"^\*\*((?:ATOM|BOUNDARY|COMP)-[^*]+)\*\*\s*$")
# A CR rule line: "613.4c Layer 7c: Effects and counters that modify ..."
CR_RULE_RE = re.compile(r"^(\d{3}\.\d+[a-z]?)\.?\s+(.*)$")
# A session classification line: "**100.1** — PURE-DEF. Defines scope ..."
CLASSIFY_RE = re.compile(r"^\*\*(\d{3}\.\d+[a-z]?)\*\*\s*[—–-]\s*([A-Z][A-Z-]+)")
RULE_TOKEN_RE = re.compile(r"\d{3}\.\d+[a-z]?")
FIELD_RE = re.compile(r"^-\s+\*\*([A-Za-z ]+):\*\*\s*(.*)$")
COVERS_RE = re.compile(r"COVERS:\s*(.+)$")
ATOM_ID_RE = re.compile(r"(?:ATOM|BOUNDARY|COMP)-[0-9A-Za-z.+]+-\d+")
RUST_FN_RE = re.compile(r"fn\s+([a-zA-Z0-9_]+)")

# Field-name aliases: a handful of entries use variant labels.
FIELD_ALIASES = {
    "rule": "rule", "rules": "rule", "rule pair": "rule", "rules composed": "rule",
    "mechanism": "mechanism", "boundary": "mechanism", "engine constraint": "mechanism",
    "minimal board": "board", "board": "board", "scenario": "board",
    "action": "action",
    "expected result": "expected", "expected": "expected",
    "phase": "phase",
    "ticket": "ticket",
    "tags": "tags",
    "dependencies": "dependencies",
    "composes": "composes", "why composition": "composes",
    "note": "note",
}

DB_COLUMNS = [
    "id", "kind", "rule_num", "summary", "mechanism", "board", "action",
    "expected", "phase", "phase_raw", "ticket", "tags", "dependencies",
    "composes", "session", "source_file", "source_line",
]


def load_phase_normalizer():
    """Reuse normalize_phase from the existing extraction script, with a fix.

    The upstream normalizer tests for the substring "already" before it parses
    a phase number, so a raw value like

        "Phase 5-Pre (already in `GameConfig` via `DeckLimits`)"

    is classified ALREADY-IMPL even though it names an explicit phase. That
    inflated ALREADY-IMPL to 201 against only 32 ALREADY-IMPLEMENTED verdicts
    in the sessions. Here an explicit "Phase N" prefix wins; the upstream
    heuristics still handle everything else.
    """
    if not EXTRACT_SCRIPT.exists():
        return lambda raw: raw.strip() or "UNKNOWN"
    spec = importlib.util.spec_from_file_location("extract_phase_index", EXTRACT_SCRIPT)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    upstream = mod.normalize_phase

    explicit = re.compile(r"^\s*phase\s*\d", re.I)

    def normalize(raw):
        # Strip a trailing parenthetical before deciding, so an aside like
        # "(already in GameConfig)" can't override the stated phase.
        if explicit.match(raw or ""):
            head = re.sub(r"\s*\([^)]*\)\s*$", "", raw).strip()
            if head:
                return upstream(head)
        return upstream(raw)

    return normalize


def parse_cr_versions():
    """Parse every CR snapshot in MTG-Rules/versions/ into rule rows.

    Rule text is the rule line plus any `Example:` lines that follow it - the
    examples are part of the rule, and folding them in before hashing means an
    example-only edit still shows up as a text change on a future sync.
    """
    rows = []
    if not CR_DIR.exists():
        return rows
    for path in sorted(CR_DIR.glob("*.txt")):
        version = path.stem
        lines = path.read_text(encoding="utf-8", errors="replace").split("\n")
        effective = ""
        for line in lines[:40]:
            m = re.search(r"effective as of (.+?)\.\s*$", line.strip())
            if m:
                effective = m.group(1)
                break
        pending = None  # (number, [text parts], line_no)
        for n, line in enumerate(lines, 1):
            s = line.strip()
            m = CR_RULE_RE.match(s)
            if m:
                if pending:
                    rows.append(_finish_rule(version, effective, pending))
                pending = (m.group(1), [m.group(2).strip()], n)
            elif pending and s.startswith("Example:"):
                pending[1].append(s)
        if pending:
            rows.append(_finish_rule(version, effective, pending))
    return rows


def _finish_rule(version, effective, pending):
    number, parts, line_no = pending
    text = "\n".join(p for p in parts if p)
    sha = hashlib.sha256(text.encode("utf-8")).hexdigest()
    return (version, number, text, sha, effective, line_no)


def parse_rule_mentions():
    """Rules a session explicitly classified, e.g. `**100.1** - PURE-DEF.`

    A rule with a verdict but no atom was considered and deliberately not
    atomized; that is different from a rule nobody ever looked at.
    """
    rows, loose = [], set()
    for path in sorted(SESSIONS_DIR.glob("session-*.md")):
        session = path.stem.replace("session-", "S")
        text = path.read_text(encoding="utf-8")
        loose.update(RULE_TOKEN_RE.findall(text))
        for line in text.split("\n"):
            m = CLASSIFY_RE.match(line.strip())
            if m:
                rows.append((m.group(1), m.group(2), session))
    return rows, loose


def parse_sessions():
    """Parse every session file into atom dicts."""
    normalize_phase = load_phase_normalizer()
    if not SESSIONS_DIR.exists():
        sys.exit("error: %s not found" % SESSIONS_DIR)

    raw_atoms = []
    for path in sorted(SESSIONS_DIR.glob("session-*.md")):
        session = path.stem.replace("session-", "S")
        lines = path.read_text(encoding="utf-8").split("\n")
        current = None
        for n, line in enumerate(lines, 1):
            m = ENTRY_RE.match(line.strip())
            if m:
                if current:
                    raw_atoms.append(current)
                aid = m.group(1).strip()
                current = {
                    "id": aid,
                    "kind": aid.split("-", 1)[0],
                    "session": session,
                    "source_file": str(path.relative_to(ROOT)).replace("\\", "/"),
                    "source_line": n,
                }
                continue
            if current is None:
                continue
            fm = FIELD_RE.match(line)
            if fm:
                key = FIELD_ALIASES.get(fm.group(1).strip().lower())
                if key:
                    current[key] = (current.get(key, "") + " " + fm.group(2).strip()).strip()
        if current:
            raw_atoms.append(current)

    out, seen = [], set()
    for a in raw_atoms:
        raw_phase = a.get("phase", "")
        a["phase_raw"] = raw_phase
        a["phase"] = normalize_phase(raw_phase) if raw_phase else "UNKNOWN"
        rule_line = a.get("rule", "")
        # "305.7 - Blood Moon replaces subtypes" -> rule "305.7", summary the rest
        parts = re.split(r"\s+[—–-]\s+", rule_line, maxsplit=1)
        a["rule_num"] = parts[0].strip() if parts else ""
        a["summary"] = parts[1].strip() if len(parts) > 1 else a.get("mechanism", "")
        if a["id"] in seen:
            continue
        seen.add(a["id"])
        out.append(a)
    return out


def scan_coverage():
    """Find COVERS: annotations in the Rust tree."""
    rows = []
    for d in CODE_DIRS:
        if not d.exists():
            continue
        for path in d.rglob("*.rs"):
            lines = path.read_text(encoding="utf-8", errors="replace").split("\n")
            for n, line in enumerate(lines, 1):
                cm = COVERS_RE.search(line)
                if not cm:
                    continue
                ids = ATOM_ID_RE.findall(cm.group(1))
                if not ids:
                    continue
                # the nearest following `fn name` is the test being annotated
                test_name = ""
                for look in lines[n:n + 8]:
                    fm = RUST_FN_RE.search(look)
                    if fm:
                        test_name = fm.group(1)
                        break
                rel = str(path.relative_to(ROOT)).replace("\\", "/")
                for aid in ids:
                    rows.append((aid, test_name, rel, n))
    return rows


def build():
    atoms = parse_sessions()
    cov = scan_coverage()
    rules = parse_cr_versions()
    mentions, loose_mentions = parse_rule_mentions()
    DB_PATH.parent.mkdir(parents=True, exist_ok=True)
    # Drop and recreate in place rather than unlinking: on Windows an open
    # SQLite browser holds a lock on the file and unlink() raises WinError 32.
    db = sqlite3.connect(DB_PATH)
    db.executescript("""
        DROP TABLE IF EXISTS atoms;
        DROP TABLE IF EXISTS coverage;
        DROP TABLE IF EXISTS rules;
        DROP TABLE IF EXISTS rule_mentions;
        DROP TABLE IF EXISTS rule_seen;
        CREATE TABLE atoms (
            id TEXT PRIMARY KEY, kind TEXT, rule_num TEXT, summary TEXT,
            mechanism TEXT, board TEXT, action TEXT, expected TEXT,
            phase TEXT, phase_raw TEXT, ticket TEXT, tags TEXT,
            dependencies TEXT, composes TEXT, session TEXT,
            source_file TEXT, source_line INTEGER
        );
        CREATE TABLE coverage (
            atom_id TEXT, test_name TEXT, file TEXT, line INTEGER
        );
        -- One row per rule per CR snapshot. text_sha is what a future
        -- sync-rules diffs on; never key anything on `number` alone, it is an
        -- address and it moves between versions.
        CREATE TABLE rules (
            cr_version TEXT, number TEXT, text TEXT, text_sha TEXT,
            effective_date TEXT, source_line INTEGER,
            PRIMARY KEY (cr_version, number)
        );
        CREATE TABLE rule_mentions (
            number TEXT, verdict TEXT, session TEXT
        );
        -- Rule numbers appearing anywhere in a session, even in prose.
        CREATE TABLE rule_seen (number TEXT PRIMARY KEY);
        CREATE INDEX idx_atoms_phase ON atoms(phase);
        CREATE INDEX idx_atoms_rule ON atoms(rule_num);
        CREATE INDEX idx_cov_atom ON coverage(atom_id);
        CREATE INDEX idx_rules_num ON rules(number);
        CREATE INDEX idx_mentions_num ON rule_mentions(number);
    """)
    db.executemany(
        "INSERT INTO atoms VALUES (%s)" % ",".join("?" * len(DB_COLUMNS)),
        [tuple(a.get(c, "") for c in DB_COLUMNS) for a in atoms],
    )
    db.executemany("INSERT INTO coverage VALUES (?,?,?,?)", cov)
    db.executemany("INSERT INTO rules VALUES (?,?,?,?,?,?)", rules)
    db.executemany("INSERT INTO rule_mentions VALUES (?,?,?)", mentions)
    db.executemany("INSERT OR IGNORE INTO rule_seen VALUES (?)",
                   [(r,) for r in sorted(loose_mentions)])
    db.commit()

    linked = db.execute(
        "SELECT COUNT(DISTINCT atom_id) FROM coverage "
        "WHERE atom_id IN (SELECT id FROM atoms)"
    ).fetchone()[0]
    orphan = db.execute(
        "SELECT COUNT(DISTINCT atom_id) FROM coverage "
        "WHERE atom_id NOT IN (SELECT id FROM atoms)"
    ).fetchone()[0]
    versions = db.execute(
        "SELECT cr_version, effective_date, COUNT(*) FROM rules "
        "GROUP BY cr_version ORDER BY cr_version"
    ).fetchall()
    print("built %s" % DB_PATH.relative_to(ROOT))
    print("  atoms parsed:      %d" % len(atoms))
    print("  COVERS rows found: %d" % len(cov))
    print("  atoms covered:     %d" % linked)
    for v, eff, n in versions:
        marker = "  <- baseline" if v == BASELINE_VERSION else ""
        print("  CR %-12s     %d rules (effective %s)%s" % (v, n, eff or "?", marker))
    if not versions:
        print("  NO CR SNAPSHOTS found in %s" % CR_DIR.relative_to(ROOT))
    if orphan:
        print("  ORPHAN ids:        %d  (run: python plans/specdb.py orphans)" % orphan)
    db.close()


def connect():
    if not DB_PATH.exists():
        sys.exit("error: no database. run: python plans/specdb.py build")
    return sqlite3.connect(DB_PATH)


def stats():
    db = connect()
    rows = db.execute("""
        SELECT a.phase, COUNT(*) AS total, COUNT(DISTINCT c.atom_id) AS covered
        FROM atoms a LEFT JOIN coverage c ON c.atom_id = a.id
        GROUP BY a.phase ORDER BY total DESC
    """).fetchall()
    print("%-16s %7s %8s %7s" % ("PHASE", "ATOMS", "COVERED", "PCT"))
    print("-" * 42)
    ta = tc = 0
    for phase, total, covered in rows:
        ta += total
        tc += covered
        print("%-16s %7d %8d %6.1f%%" % (phase, total, covered, 100.0 * covered / total))
    print("-" * 42)
    print("%-16s %7d %8d %6.1f%%" % ("TOTAL", ta, tc, 100.0 * tc / ta if ta else 0))


def next_up(phase, rule, limit):
    db = connect()
    q = ("SELECT a.id, a.rule_num, a.summary, a.ticket, a.phase "
         "FROM atoms a LEFT JOIN coverage c ON c.atom_id = a.id "
         "WHERE c.atom_id IS NULL")
    args = []
    if phase:
        q += " AND a.phase = ?"
        args.append(phase)
    else:
        q += " AND a.phase IN (%s)" % ",".join("?" * len(CRITICAL_PATH))
        args += CRITICAL_PATH
    if rule:
        q += " AND a.rule_num LIKE ?"
        args.append(rule + "%")
    q += " ORDER BY a.phase, a.rule_num, a.id LIMIT ?"
    args.append(limit)
    rows = db.execute(q, args).fetchall()
    if not rows:
        print("nothing uncovered matches that filter.")
        return
    for aid, rule_num, summary, ticket, _ph in rows:
        print("%-26s %-9s %-14s %s" % (aid, rule_num, (ticket or "")[:14], (summary or "")[:70]))
    print("\n(%d shown)" % len(rows))


def show(atom_id):
    db = connect()
    row = db.execute("SELECT * FROM atoms WHERE id = ?", (atom_id,)).fetchone()
    if not row:
        sys.exit("no such atom: %s" % atom_id)
    for k, v in zip(DB_COLUMNS, row):
        if v not in ("", None):
            print("%-13s %s" % (k + ":", v))
    cov = db.execute(
        "SELECT test_name, file, line FROM coverage WHERE atom_id = ?", (atom_id,)
    ).fetchall()
    print("%-13s %s" % ("covered_by:", "NOTHING" if not cov else ""))
    for name, f, ln in cov:
        print("              %s  (%s:%d)" % (name, f, ln))


def gaps(chapter, limit, show_all):
    """CR rules the spec corpus never accounted for.

    Three dispositions per rule, from strongest to weakest:
      atomized   - at least one ATOM/BOUNDARY/COMP cites it
      classified - a session gave it a verdict (PURE-DEF, DEFERRED, ...) but
                   produced no atom: considered and deliberately skipped
      unseen     - the rule number appears nowhere in any session
    Only `unseen` is a real blind spot.
    """
    db = connect()
    rules = db.execute(
        "SELECT number, text FROM rules WHERE cr_version = ? ORDER BY number",
        (BASELINE_VERSION,),
    ).fetchall()
    if not rules:
        sys.exit("no rules for baseline version %r - check %s"
                 % (BASELINE_VERSION, CR_DIR))

    atomized = set()
    for (r,) in db.execute("SELECT DISTINCT rule_num FROM atoms WHERE rule_num <> ''"):
        atomized.update(RULE_TOKEN_RE.findall(r))
    classified = {r: v for r, v in db.execute("SELECT number, verdict FROM rule_mentions")}
    seen = {r for (r,) in db.execute("SELECT number FROM rule_seen")}

    def chap(number):
        return number[:1]

    rows = [(n, t) for n, t in rules if not chapter or chap(n) == str(chapter)]
    unseen = [(n, t) for n, t in rows if n not in atomized and n not in seen]
    only_classified = [(n, t) for n, t in rows
                       if n not in atomized and n in classified]

    total = len(rows)
    n_atom = sum(1 for n, _ in rows if n in atomized)
    scope = "CR %s" % chapter if chapter else "all chapters"
    print("%s - baseline CR %s" % (scope, BASELINE_VERSION))
    print("  rules:                 %d" % total)
    print("  atomized:              %d  (%.1f%%)" % (n_atom, 100.0 * n_atom / total))
    print("  classified, no atom:   %d" % len(only_classified))
    print("  NEVER SEEN:            %d  (%.1f%%)" % (len(unseen), 100.0 * len(unseen) / total))
    if not unseen:
        print("\nno blind spots in this scope.")
        return
    print("\nrules no session ever mentioned:")
    shown = unseen if show_all else unseen[:limit]
    for n, t in shown:
        print("  %-10s %s" % (n, t.split("\n")[0][:74]))
    if len(unseen) > len(shown):
        print("  ... %d more (use --all)" % (len(unseen) - len(shown)))


def orphans():
    db = connect()
    rows = db.execute("""
        SELECT DISTINCT atom_id, file, line FROM coverage
        WHERE atom_id NOT IN (SELECT id FROM atoms) ORDER BY atom_id
    """).fetchall()
    if not rows:
        print("no orphans - every COVERS id matches a real atom.")
        return
    print("COVERS ids with no matching atom (typo, or spec entry missing):")
    for aid, f, ln in rows:
        print("  %-28s %s:%d" % (aid, f, ln))


def main():
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("build")
    sub.add_parser("stats")
    n = sub.add_parser("next")
    n.add_argument("--phase")
    n.add_argument("--rule")
    n.add_argument("--limit", type=int, default=30)
    s = sub.add_parser("show")
    s.add_argument("atom_id")
    sub.add_parser("orphans")
    g = sub.add_parser("gaps")
    g.add_argument("--chapter", type=int, help="CR chapter 1-9")
    g.add_argument("--limit", type=int, default=25)
    g.add_argument("--all", action="store_true", dest="show_all")
    a = ap.parse_args()
    {"build": lambda: build(),
     "stats": lambda: stats(),
     "next": lambda: next_up(a.phase, a.rule, a.limit),
     "show": lambda: show(a.atom_id),
     "orphans": lambda: orphans(),
     "gaps": lambda: gaps(a.chapter, a.limit, a.show_all)}[a.cmd]()


if __name__ == "__main__":
    main()
