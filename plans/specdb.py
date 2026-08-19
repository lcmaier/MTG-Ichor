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
"""

import argparse
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

# Phases on the critical path to v1, in the order they must land.
CRITICAL_PATH = ["Phase 5-Layers", "Phase 6", "Phase 7"]

ENTRY_RE = re.compile(r"^\*\*((?:ATOM|BOUNDARY|COMP)-[^*]+)\*\*\s*$")
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
    """Reuse normalize_phase from the existing extraction script."""
    if not EXTRACT_SCRIPT.exists():
        return lambda raw: raw.strip() or "UNKNOWN"
    spec = importlib.util.spec_from_file_location("extract_phase_index", EXTRACT_SCRIPT)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod.normalize_phase


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
    DB_PATH.parent.mkdir(parents=True, exist_ok=True)
    if DB_PATH.exists():
        DB_PATH.unlink()
    db = sqlite3.connect(DB_PATH)
    db.executescript("""
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
        CREATE INDEX idx_atoms_phase ON atoms(phase);
        CREATE INDEX idx_atoms_rule ON atoms(rule_num);
        CREATE INDEX idx_cov_atom ON coverage(atom_id);
    """)
    db.executemany(
        "INSERT INTO atoms VALUES (%s)" % ",".join("?" * len(DB_COLUMNS)),
        [tuple(a.get(c, "") for c in DB_COLUMNS) for a in atoms],
    )
    db.executemany("INSERT INTO coverage VALUES (?,?,?,?)", cov)
    db.commit()

    linked = db.execute(
        "SELECT COUNT(DISTINCT atom_id) FROM coverage "
        "WHERE atom_id IN (SELECT id FROM atoms)"
    ).fetchone()[0]
    orphan = db.execute(
        "SELECT COUNT(DISTINCT atom_id) FROM coverage "
        "WHERE atom_id NOT IN (SELECT id FROM atoms)"
    ).fetchone()[0]
    print("built %s" % DB_PATH.relative_to(ROOT))
    print("  atoms parsed:      %d" % len(atoms))
    print("  COVERS rows found: %d" % len(cov))
    print("  atoms covered:     %d" % linked)
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
    a = ap.parse_args()
    {"build": lambda: build(),
     "stats": lambda: stats(),
     "next": lambda: next_up(a.phase, a.rule, a.limit),
     "show": lambda: show(a.atom_id),
     "orphans": lambda: orphans()}[a.cmd]()


if __name__ == "__main__":
    main()
