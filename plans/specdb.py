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

Use COVERS-PARTIAL when a test exercises a rule but does not build the atom's
full scenario. The corpus writes atoms as maximal composite scenarios while
the test suite decomposes them into minimal units, so partial is the common
case, not an excuse:

    // COVERS-PARTIAL: ATOM-613.4c-001
    #[test]
    fn test_two_anthems_stack() { ... }   // additive stacking, but no counter

An atom is "covered" only when some test COVERS it fully; partials are
reported separately so they read as work remaining, not work done.

Usage:
    python plans/specdb.py build          # (re)build spec.sqlite
    python plans/specdb.py stats          # coverage by phase
    python plans/specdb.py next           # uncovered atoms on the critical path
    python plans/specdb.py next --phase "Phase 5-Layers" --rule 613
    python plans/specdb.py show ATOM-305.7-001
    python plans/specdb.py orphans        # COVERS ids with no matching atom
    python plans/specdb.py suspicious     # COVERS ids that exist but look wrong
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
CODE_DIRS = [ROOT / "mtgsim" / "src", ROOT / "mtgsim" / "tests"]
DB_PATH = ROOT / "plans" / "atomic-tests" / "spec.sqlite"

# Comprehensive Rules snapshots, one official .txt per version. The filename
# stem is the version label (tmnt, strixhaven, ...). BASELINE_VERSION is the
# snapshot the engine currently targets; `gaps` reports against it.
CR_DIR = ROOT / "MTG-Rules" / "versions"
BASELINE_VERSION = "tmnt"

# Phases on the critical path to v1, in the order they must land. CLAUDE.md's
# "Critical path to v1" owns that ordering; this is a mirror for the corpus's
# phase labels, and follows it rather than deciding anything.
CRITICAL_PATH = ["Phase 5-Layers", "Phase 6", "Phase 7"]

ENTRY_RE = re.compile(r"^\*\*((?:ATOM|BOUNDARY|COMP)-[^*]+)\*\*\s*$")
# A CR rule line: "613.4c Layer 7c: Effects and counters that modify ..."
CR_RULE_RE = re.compile(r"^(\d{3}\.\d+[a-z]?)\.?\s+(.*)$")
# A session classification line: "**100.1** — PURE-DEF. Defines scope ..."
CLASSIFY_RE = re.compile(r"^\*\*(\d{3}\.\d+[a-z]?)\*\*\s*[—–-]\s*([A-Z][A-Z-]+)")
RULE_TOKEN_RE = re.compile(r"\d{3}\.\d+[a-z]?")
FIELD_RE = re.compile(r"^-\s+\*\*([A-Za-z ]+):\*\*\s*(.*)$")
COVERS_RE = re.compile(r"COVERS(-PARTIAL)?:\s*(.+)$")
# Ids are `KIND-<rule>-<seq>`, but a COMP may name its cards instead of a
# second rule: COMP-613-TARMOGOYF-HUMILITY-001. Allow extra `-`-joined
# segments, each required to contain a letter so that the trailing `-<seq>`
# is never swallowed. Only `scan_coverage` uses this; the session parser
# reads ids from their headings.
ATOM_ID_RE = re.compile(
    r"(?:ATOM|BOUNDARY|COMP)-[0-9A-Za-z.+]+(?:-[0-9A-Za-z.+]*[A-Za-z][0-9A-Za-z.+]*)*-\d+"
)
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


# Canonical phase names, in the order they are reported.
PHASE_ORDER = [
    "ALREADY-IMPL",
    "Phase 5-Pre",
    "Phase 5-Layers",
    "Phase 6",
    "Phase 7",
    "Phase 8",
    "Phase 9",
    "Post-v1",
    "Cross-cutting",
    "DEFERRED",
    "COMP-REF",
    "UNKNOWN",
]

_EXPLICIT_PHASE = re.compile(r"^\s*phase\s*\d", re.I)


def normalize_phase(raw):
    """Map a session's raw phase label to a canonical phase name.

    Strategy: extract every phase number mentioned and assign the entry to the
    LATEST one, because that is when the test can actually be built. L-tickets
    imply Phase 5-Layers, T-tickets Phase 5-Pre.

    Lived in `atomic-tests/extract-phase-index.py` until 2026-08-24, imported
    from here by path. That script also generated the markdown indexes *from
    `summaries/`* while this one built the database from `sessions/`, so the two
    tiers drifted apart — see the module docstring. It is archived; this is the
    only copy now.

    One deliberate difference from the original, kept: an explicit "Phase N"
    prefix wins over a trailing parenthetical. The original tested for the
    substring "already" first, so "Phase 5-Pre (already in `GameConfig`)"
    classified as ALREADY-IMPL and inflated that bucket to 201 against 32 real
    ALREADY-IMPLEMENTED verdicts.
    """
    raw = raw or ""
    if _EXPLICIT_PHASE.match(raw):
        head = re.sub(r"\s*\([^)]*\)\s*$", "", raw).strip()
        if head:
            raw = head

    cleaned = raw.strip()
    lowered = cleaned.lower()

    # A COMP whose "phase" field lists ATOM ids instead of a phase label.
    if re.match(r"^ATOM-", cleaned):
        return "COMP-REF"
    if "already" in lowered or lowered == "impl" or lowered.startswith("partial impl"):
        return "ALREADY-IMPL"
    if "post" in lowered or "pre-phase" in lowered:
        return "Post-v1"
    if lowered == "deferred":
        return "DEFERRED"
    if "per-system" in lowered or "cross" in lowered:
        return "Cross-cutting"

    phase_nums = re.findall(r"phase\s*(\d+)", lowered)
    combo_nums = re.findall(r"(\d+)", lowered)
    has_l_ticket = bool(re.search(r"L\d+", cleaned))
    has_t_ticket = bool(re.search(r"T\d+", cleaned))
    has_pre = "5-pre" in lowered or "5 pre" in lowered or "pre" in lowered
    has_layers = "5-layer" in lowered or "5 layer" in lowered or "layer" in lowered

    if not phase_nums and not combo_nums:
        if has_l_ticket:
            return "Phase 5-Layers"
        if has_t_ticket:
            return "Phase 5-Pre"
        return "UNKNOWN"

    nums = [int(n) for n in (phase_nums or combo_nums)]
    nums = [n for n in nums if 1 <= n <= 9]
    if not nums:
        return "UNKNOWN"
    # Phases 1-4 shipped long ago; if nothing later is named, it is done.
    if all(n <= 4 for n in nums):
        return "ALREADY-IMPL"

    latest = max(nums)
    if latest == 5:
        if has_layers or has_l_ticket:
            return "Phase 5-Layers"
        if has_pre or has_t_ticket:
            return "Phase 5-Pre"
        return "Phase 5-Pre"
    return {6: "Phase 6", 7: "Phase 7", 8: "Phase 8", 9: "Phase 9"}.get(latest, "UNKNOWN")


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


def split_entry_id(raw):
    """Split an entry heading into (id, title).

    Sessions write a COMP's heading as `COMP-702-001: Deathtouch + First Strike`
    or `COMP-9A-001 — SBA cascade ...`, so a naive capture swallows the title
    into the id. 21 entries were stored that way, which is why `specdb show
    COMP-7A-005` reported "no such atom" and why the ids never lined up with
    the markdown indexes. Ids stay bare; the title becomes the summary when the
    entry has no `Rule` line to derive one from.
    """
    m = re.match(r"^((?:ATOM|BOUNDARY|COMP)-[^\s:—–]+)(?:\s*[:—–-]\s*(.*))?$", raw.strip())
    if not m:
        return raw.strip(), ""
    return m.group(1).strip(), (m.group(2) or "").strip()


def parse_sessions():
    """Parse every session file into atom dicts."""
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
                aid, title = split_entry_id(m.group(1))
                current = {
                    "id": aid,
                    "title": title,
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
        if not a["summary"]:
            a["summary"] = a.get("title", "")
        if a["id"] in seen:
            continue
        seen.add(a["id"])
        out.append(a)
    return out


def find_annotated_fn(lines, n):
    """The name of the test a COVERS comment at 1-based line `n` annotates.

    Walks forward past comments, blank lines and attributes to the first real
    line, which is the `fn`. A fixed lookahead does not work: annotations here
    carry their reasoning, and the corpus asks them to — the block above
    `test_tarmogoyf_pt_is_layer_7a_and_an_ability_strip_removes_it` runs twelve
    lines, so an 8-line window recorded that atom as covered by a test with no
    name.
    """
    for look in lines[n:n + 60]:
        stripped = look.strip()
        if not stripped or stripped.startswith("//") or stripped.startswith("#["):
            continue
        fm = RUST_FN_RE.search(look)
        return fm.group(1) if fm else ""
    return ""


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
                partial = 1 if cm.group(1) else 0
                ids = ATOM_ID_RE.findall(cm.group(2))
                if not ids:
                    continue
                test_name = find_annotated_fn(lines, n)
                rel = str(path.relative_to(ROOT)).replace("\\", "/")
                for aid in ids:
                    rows.append((aid, test_name, rel, n, partial))
    return rows


def session_sort_key(session):
    """S1 < S2 < ... < S7a < S7b < ... < S10, not lexicographic."""
    body = (session or "").lstrip("S")
    num = "".join(c for c in body if c.isdigit())
    suffix = "".join(c for c in body if not c.isdigit())
    return (int(num) if num else 999, suffix)


def write_indexes(atoms):
    """Regenerate the markdown test indexes from the corpus.

    These are derived files. They used to be written by
    `atomic-tests/extract-phase-index.py`, which read `summaries/` while this
    script read `sessions/` — two parsers over two tiers, and by 2026-08-24 they
    disagreed by 27 entries and on the size of three phases (Phase 7: 202 vs
    133). `sessions/` is the authored tier, corrections land there, so the
    indexes now come from the same parse as the database and cannot drift from
    it again. `summaries/` is the authoring trail and generates nothing.
    """
    out_dir = SESSIONS_DIR.parent
    by_phase = {}
    for a in atoms:
        by_phase.setdefault(a["phase"], []).append(a)
    for rows in by_phase.values():
        rows.sort(key=lambda a: (session_sort_key(a.get("session")), a.get("source_line", 0)))

    def table(rows):
        out = ["| ID | Rule | Summary | Ticket | Session | Tags |",
               "|----|------|---------|--------|---------|------|"]
        for a in rows:
            cells = [a["id"], a.get("rule_num", ""), a.get("summary", ""),
                     a.get("ticket", ""), a.get("session", ""), a.get("tags", "")]
            out.append("| %s |" % " | ".join(c.replace("|", "\\|") for c in cells))
        return out

    written = []
    ordered = ([p for p in PHASE_ORDER if p in by_phase]
               + sorted(p for p in by_phase if p not in PHASE_ORDER))

    lines = ["# Global Atomic Test Index", "",
             "> Generated by `python plans/specdb.py build` from"
             " `plans/atomic-tests/sessions/*.md`. Do not hand-edit — fix the"
             " session file and rebuild.",
             "> Total entries: %d" % len(atoms), "", "---", "", "## Phase Counts", "",
             "| Phase | ATOMs | BOUNDARYs | COMPs | Total |",
             "|-------|-------|-----------|-------|-------|"]
    for phase in ordered:
        rows = by_phase[phase]
        counts = {k: sum(1 for a in rows if a["kind"] == k) for k in ("ATOM", "BOUNDARY", "COMP")}
        lines.append("| %s | %d | %d | %d | %d |" % (
            phase, counts["ATOM"], counts["BOUNDARY"], counts["COMP"], len(rows)))
    lines += ["", "---", ""]
    for phase in ordered:
        lines += ["## %s" % phase, "", "**%d entries**" % len(by_phase[phase]), ""]
        lines += table(by_phase[phase]) + [""]
    path = out_dir / "global-test-index.md"
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    written.append(path)

    for phase in ordered:
        rows = by_phase[phase]
        slug = phase.lower().replace(" ", "-").replace("/", "-")
        path = out_dir / ("phase-index-%s.md" % slug)
        body = ["# %s — Test Index" % phase, "",
                "> Generated by `python plans/specdb.py build` from"
                " `plans/atomic-tests/sessions/*.md`. Do not hand-edit.",
                "> %d entries" % len(rows), "", "---", "",
                "## %s" % phase, "", "**%d entries**" % len(rows), ""]
        body += table(rows) + [""]
        path.write_text("\n".join(body) + "\n", encoding="utf-8")
        written.append(path)

    # Sweep indexes for phases that no longer have entries. Leaving one behind
    # is the same failure this rewrite exists to fix: a generated file that
    # nothing regenerates is a stale claim with a plausible filename.
    # `phase-index-deferred.md` was exactly that -- the DEFERRED bucket emptied
    # and the file stayed.
    keep = {q.name for q in written}
    for stale in out_dir.glob("phase-index-*.md"):
        if stale.name not in keep:
            stale.unlink()
            print("  removed stale index:  %s" % stale.relative_to(ROOT))
    return written


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
            atom_id TEXT, test_name TEXT, file TEXT, line INTEGER,
            partial INTEGER DEFAULT 0
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
    db.executemany("INSERT INTO coverage VALUES (?,?,?,?,?)", cov)
    db.executemany("INSERT INTO rules VALUES (?,?,?,?,?,?)", rules)
    db.executemany("INSERT INTO rule_mentions VALUES (?,?,?)", mentions)
    db.executemany("INSERT OR IGNORE INTO rule_seen VALUES (?)",
                   [(r,) for r in sorted(loose_mentions)])
    db.commit()

    linked = db.execute(
        "SELECT COUNT(DISTINCT atom_id) FROM coverage "
        "WHERE partial = 0 AND atom_id IN (SELECT id FROM atoms)"
    ).fetchone()[0]
    part = db.execute(
        "SELECT COUNT(DISTINCT atom_id) FROM coverage c WHERE partial = 1 "
        "AND atom_id IN (SELECT id FROM atoms) AND atom_id NOT IN "
        "(SELECT atom_id FROM coverage WHERE partial = 0)"
    ).fetchone()[0]
    orphan = db.execute(
        "SELECT COUNT(DISTINCT atom_id) FROM coverage "
        "WHERE atom_id NOT IN (SELECT id FROM atoms)"
    ).fetchone()[0]
    versions = db.execute(
        "SELECT cr_version, effective_date, COUNT(*) FROM rules "
        "GROUP BY cr_version ORDER BY cr_version"
    ).fetchall()
    index_files = write_indexes(atoms)
    print("built %s" % DB_PATH.relative_to(ROOT))
    print("  atoms parsed:      %d" % len(atoms))
    print("  indexes written:   %d (%s)" % (
        len(index_files), index_files[0].parent.relative_to(ROOT)))
    print("  COVERS rows found: %d" % len(cov))
    print("  atoms covered:     %d full, %d partial-only" % (linked, part))
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
        SELECT a.phase, COUNT(*) AS total,
               SUM(CASE WHEN f.atom_id IS NOT NULL THEN 1 ELSE 0 END) AS full_cov,
               SUM(CASE WHEN f.atom_id IS NULL AND p.atom_id IS NOT NULL
                        THEN 1 ELSE 0 END) AS part_cov
        FROM atoms a
        LEFT JOIN (SELECT DISTINCT atom_id FROM coverage WHERE partial = 0) f
               ON f.atom_id = a.id
        LEFT JOIN (SELECT DISTINCT atom_id FROM coverage WHERE partial = 1) p
               ON p.atom_id = a.id
        GROUP BY a.phase ORDER BY total DESC
    """).fetchall()
    print("%-16s %7s %6s %8s %8s" % ("PHASE", "ATOMS", "FULL", "PARTIAL", "FULL%"))
    print("-" * 50)
    ta = tf = tp = 0
    for phase, total, full_cov, part_cov in rows:
        ta += total
        tf += full_cov
        tp += part_cov
        print("%-16s %7d %6d %8d %7.1f%%"
              % (phase, total, full_cov, part_cov, 100.0 * full_cov / total))
    print("-" * 50)
    print("%-16s %7d %6d %8d %7.1f%%"
          % ("TOTAL", ta, tf, tp, 100.0 * tf / ta if ta else 0))
    print("\nFULL = a test builds the atom's whole scenario. PARTIAL = a test")
    print("exercises the rule but not that scenario; work remains.")


def next_up(phase, rule, limit):
    db = connect()
    q = ("SELECT a.id, a.rule_num, a.summary, a.ticket, "
         "       CASE WHEN p.atom_id IS NULL THEN ' ' ELSE '~' END AS mark "
         "FROM atoms a "
         "LEFT JOIN (SELECT DISTINCT atom_id FROM coverage WHERE partial = 0) f "
         "       ON f.atom_id = a.id "
         "LEFT JOIN (SELECT DISTINCT atom_id FROM coverage WHERE partial = 1) p "
         "       ON p.atom_id = a.id "
         "WHERE f.atom_id IS NULL")
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
    for aid, rule_num, summary, ticket, mark in rows:
        print("%s %-26s %-9s %-13s %s"
              % (mark, aid, rule_num, (ticket or "")[:13], (summary or "")[:66]))
    print("\n(%d shown; ~ = partially covered already)" % len(rows))


def show(atom_id):
    db = connect()
    row = db.execute("SELECT * FROM atoms WHERE id = ?", (atom_id,)).fetchone()
    if not row:
        sys.exit("no such atom: %s" % atom_id)
    for k, v in zip(DB_COLUMNS, row):
        if v not in ("", None):
            print("%-13s %s" % (k + ":", v))
    cov = db.execute(
        "SELECT test_name, file, line, partial FROM coverage WHERE atom_id = ? "
        "ORDER BY partial, test_name", (atom_id,)
    ).fetchall()
    print("%-13s %s" % ("covered_by:", "NOTHING" if not cov else ""))
    for name, f, ln, part in cov:
        print("              %s%s  (%s:%d)"
              % (name, "  [PARTIAL]" if part else "", f, ln))


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


# Words that appear in every atom and every test and so carry no signal.
_STOPWORDS = set("""
a an and are as at be been but by can cant does doesnt for from has have if in
into is it its no not of on or should so than that the their then there they
this to under until up was were when which while with without you your
test tests fn let mut assert asserteq game state card cards player players
rule cr atom effect effects one two both new same other another
""".split())


def _words(text):
    out = set()
    for w in re.findall(r"[A-Za-z]+", (text or "").lower()):
        if len(w) > 3 and w not in _STOPWORDS:
            out.add(w)
            # crude stem, so "creature"/"creatures" and "taps"/"tapped" meet
            for suffix in ("ing", "ed", "es", "s"):
                if w.endswith(suffix) and len(w) - len(suffix) > 3:
                    out.add(w[: -len(suffix)])
                    break
    return out


def _test_source(path, line):
    """The annotated test's own text — its signature and body, nothing else.

    Deliberately excludes every `COVERS` line in the window. The annotation
    names the atom, the atom id contains the rule number, so a window that
    includes the comment matches the atom it is supposed to be checked against.
    That circularity made the first version of this check pass everything,
    including a poison-counter atom pinned to a mana-formatting test.
    """
    full = ROOT / path
    if not full.exists():
        return ""
    lines = full.read_text(encoding="utf-8", errors="replace").split("\n")
    # The annotated item is the next `fn` after the comment block.
    fn_line = None
    for i in range(line - 1, min(len(lines), line + 60)):
        stripped = lines[i].strip()
        if i >= line and (not stripped or stripped.startswith("//")
                          or stripped.startswith("#[")):
            continue
        if RUST_FN_RE.search(lines[i]):
            fn_line = i
            break
        if i >= line:
            break
    if fn_line is None:
        return ""
    body, depth, opened = [], 0, False
    for i in range(fn_line, min(len(lines), fn_line + 80)):
        text = lines[i]
        if "COVERS" not in text:
            body.append(text)
        depth += text.count("{") - text.count("}")
        if "{" in text:
            opened = True
        if opened and depth <= 0:
            break
    return "\n".join(body)


def suspicious(threshold):
    """Flag COVERS links whose test looks unrelated to the atom it claims.

    `orphans` catches an id that does not exist. Nothing caught an id that
    exists and is *wrong* — and a wrong link is worse than a blank, because a
    blank reads as work remaining while a wrong one reads as done. This is the
    cheap approximation: compare the vocabulary of the atom's scenario against
    the vocabulary of the annotated test. A real link almost always shares the
    nouns (`aura`, `sacrifice`, `poison`, `equipment`); a mismatched one usually
    shares nothing but boilerplate.

    It is a smell detector, not a proof. Low overlap on a correct link happens
    when a test builds a scenario in different words, so a hit means "read this
    one", not "this is wrong". Silence does not mean every link is right.
    """
    db = connect()
    rows = db.execute(
        "SELECT c.atom_id, c.test_name, c.file, c.line, c.partial, "
        "       a.summary, a.mechanism, a.board, a.action, a.expected, a.rule_num "
        "FROM coverage c JOIN atoms a ON a.id = c.atom_id "
        "ORDER BY c.file, c.line"
    ).fetchall()
    if not rows:
        print("no COVERS annotations to check")
        return

    flagged = []
    for aid, test, path, line, partial, summary, mech, board, action, expected, rule in rows:
        atom_words = _words(" ".join([summary or "", mech or "", board or "",
                                      action or "", expected or ""]))
        src = _test_source(path, line)
        test_words = _words(test + " " + src)
        if not atom_words:
            continue
        shared = atom_words & test_words
        score = len(shared) / len(atom_words)
        # A test naming the rule number is strong evidence on its own.
        names_rule = bool(rule) and rule in src
        if score < threshold and not names_rule:
            flagged.append((score, aid, test, path, line, partial, sorted(shared)[:6]))

    print("checked %d COVERS link(s) against the atoms they claim" % len(rows))
    if not flagged:
        print("nothing below the %.0f%% vocabulary-overlap threshold." % (threshold * 100))
        print("that is a smell test, not a proof — it cannot see a link that is")
        print("plausible and still wrong.")
        return
    print("")
    print("%d link(s) share little vocabulary with their atom. Read these:" % len(flagged))
    for score, aid, test, path, line, partial, shared in sorted(flagged):
        kind = "COVERS-PARTIAL" if partial else "COVERS"
        print("")
        print("  %s  %s" % (aid, kind))
        print("    test:   %s" % (test or "<none found>"))
        print("    at:     %s:%d" % (path, line))
        print("    shared: %.0f%% %s" % (score * 100, shared or "(nothing)"))
    print("")
    print("Overlap is a heuristic: a correct link can score low when the test")
    print("builds the scenario in different words. Check, do not delete blind.")


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
    q = sub.add_parser("suspicious")
    q.add_argument("--threshold", type=float, default=0.15,
                   help="flag links below this vocabulary overlap (default 0.15)")
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
     "suspicious": lambda: suspicious(a.threshold),
     "gaps": lambda: gaps(a.chapter, a.limit, a.show_all)}[a.cmd]()


if __name__ == "__main__":
    main()
