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
    python plans/specdb.py owed           # atoms a shipped phase left uncovered
    python plans/specdb.py audit --dark --families   # CR rules NOBODY examined

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
RULE_TOKEN_RE = re.compile(r"\d{3}\.\d+[a-z]?")

# --- Session classification lines -------------------------------------------
#
# **The corpus writes a verdict in three shapes, not one**, and reading only the
# first is what made `audit` report 478 in-scope dark families when the real
# number was far smaller (`cr-coverage-audit.md` §1, defect D1). The shapes are authored
# fact across twelve session files and five months, so the parser learns them
# rather than the files being rewritten to match a regex — `CLAUDE.md` calls the
# corpus authored, never generated.
#
#   1. bold span   `**100.1** — PURE-DEF.`   (sessions 1, 4, 7a, 7b, 9a, 9b, 10)
#   2. heading     `### 609.1 — PURE-DEF`     (session 6)
#   3. heading + `**Classification: PURE-DEF.**` on a later line  (2, 3, 5, 8)
#
# A span may name several rules: a comma list, a `+`, or a range written either
# in full (`903.12a–903.12h`) or with a bare letter tail (`100.4c–d`).
CLASSIFY_VERDICTS = ("TESTABLE", "PURE-DEF", "DEFERRED", "OUT-OF-SCOPE",
                     "BOUNDARY-DEF", "ALREADY-IMPLEMENTED", "ALREADY-IMPL",
                     "META", "LKI")
_V = "(%s)" % "|".join(CLASSIFY_VERDICTS)
# The verdict must come *first*, not merely somewhere on the line: three lines
# in the corpus discuss a verdict mid-sentence ("They are all TESTABLE but
# belong to Session 5") and reading those as classifications would claim rules
# no session actually classified. Precision over recall — over-claiming a
# verdict is the failure this whole table exists to avoid.
# The leading `(?:[-*]\s+)?` is a list bullet: sessions 4, 7a and 9b write the
# same shape as a bullet under a section heading (`- **726.1** — DEFERRED …`).
# Same shape, not a new one, which is why it earns a character rather than a
# branch — it resolves 35 otherwise-dark rules and disagrees with none.
CLASSIFY_BOLD_RE = re.compile(
    r"^(?:[-*]\s+)?\*\*\s*([\d.,+a-z–—\s-]*\d{3}\.\d+[a-z]?[\d.,+a-z–—\s-]*?)\s*:?\s*\*\*"
    r"\s*(?:\([^)]*\)\s*)?[—–:-]?\s*" + _V)
CLASSIFY_HEAD_RE = re.compile(r"^#{2,4}\s+(\d{3}\.\d+[a-z]?)\s*[—–-]?\s*(.*)$")
CLASSIFY_FIELD_RE = re.compile(r"^\*\*Classification:\s*" + _V)
# The pre-2026-08-31 regex, kept as a fallback so that widening the parser can
# never *narrow* it. It took any leading uppercase word as a verdict, which is
# how `108.5` (`PARTIALLY DEFERRED`) and `732.1`/`732.2`
# (`ALREADY-HANDLED-BY-DESIGN`) got classified off-vocabulary. Those three are
# corpus defects to normalise, not rules to re-darken.
CLASSIFY_LEGACY_RE = re.compile(
    r"^\*\*(\d{3}\.\d+[a-z]?)\*\*\s*[—–-]\s*([A-Z][A-Z-]+)")
# `100.4c–d` and `903.12a–903.12h`; the tail may drop the shared `NNN.M`.
CLASSIFY_RANGE_RE = re.compile(
    r"^(\d{3}\.\d+)([a-z]?)\s*[–—-]\s*(?:(\d{3}\.\d+))?([a-z]?)$")
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
    # The word-boundary escape here was a literal backspace until 2026-08-31,
    # so both flags were permanently false. Fixing it changes no output, and
    # that is worth knowing: their only reader is the digit-free branch below,
    # and every corpus phase string carrying an `L##`/`T##` ticket also names
    # its phase in digits ("Phase 5 Layers (L10)"), so the branch is
    # unreachable for a second, independent reason. Left as the fallback it
    # was written to be — promoting it above the digit path would relabel
    # atoms and move what `owed` gates on, which nothing is asking for.
    has_l_ticket = bool(re.search(r"\bL\d+", cleaned))
    has_t_ticket = bool(re.search(r"\bT\d+", cleaned))
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


def _expand_classify_span(span, known):
    """The rule numbers a classification span names, in CR order.

    Expansion is bounded by `known` — the rule numbers the CR ingest actually
    found — so a range never invents a subrule. `104.3g–k` has to skip `l` and
    `o`, which the CR does not use, and asking the CR beats encoding its
    alphabet here.
    """
    out = []
    for part in re.split(r"[,+]", span):
        part = part.strip()
        if not part:
            continue
        if RULE_TOKEN_RE.fullmatch(part):
            out.append(part)
            continue
        m = CLASSIFY_RANGE_RE.match(part)
        if not m:
            continue
        family, lo_tail, hi_family, hi_tail = m.groups()
        lo, hi = family + lo_tail, (hi_family or family) + hi_tail
        if lo not in known or hi not in known:
            continue
        # Everything between the endpoints, capped at the section (`NNN`) they
        # share. `805.1–805.10f` spans ten families and means all of them; the
        # cap is what stops it running on into 806.
        section = lo.split(".")[0]
        lo_key, hi_key = _rule_sort_key(lo), _rule_sort_key(hi)
        out.extend(sorted(
            (r for r in known
             if r.split(".")[0] == section
             and lo_key <= _rule_sort_key(r) <= hi_key),
            key=_rule_sort_key))
    return out


def parse_rule_mentions(known=()):
    """Rules a session explicitly classified, e.g. `**100.1** - PURE-DEF.`

    A rule with a verdict but no atom was considered and deliberately not
    atomized; that is different from a rule nobody ever looked at. `known` is
    the CR's own rule numbers, so a range expands against the real CR.

    Rule numbers come from the classification's *span* only, never from the
    rest of the line: the prose beside a verdict routinely cross-references
    other rules ("deferred until 702.87"), and harvesting those would classify
    a rule on the strength of someone else's footnote.
    """
    known = set(known)
    rows, loose = [], set()
    for path in sorted(SESSIONS_DIR.glob("session-*.md")):
        session = path.stem.replace("session-", "S")
        text = path.read_text(encoding="utf-8")
        loose.update(RULE_TOKEN_RE.findall(text))
        pending_head = None
        for line in text.splitlines():
            s = line.strip()
            head = CLASSIFY_HEAD_RE.match(s)
            if head:
                # `### 609.1 — PURE-DEF` classifies inline; `### 200.1 — Parts
                # of a card` defers to the `**Classification:**` line below it.
                inline = re.match(_V, head.group(2))
                if inline:
                    rows.append((head.group(1), inline.group(1), session))
                    pending_head = None
                else:
                    pending_head = head.group(1)
                continue
            field = CLASSIFY_FIELD_RE.match(s)
            if field and pending_head:
                rows.append((pending_head, field.group(1), session))
                pending_head = None
                continue
            bold = CLASSIFY_BOLD_RE.match(s)
            if bold:
                for number in _expand_classify_span(bold.group(1), known):
                    rows.append((number, bold.group(2), session))
                continue
            legacy = CLASSIFY_LEGACY_RE.match(s)
            if legacy:
                rows.append((legacy.group(1), legacy.group(2), session))
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
    mentions, loose_mentions = parse_rule_mentions(
        {number for version, number, *_ in rules if version == BASELINE_VERSION})
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


# Phases whose work has shipped. An atom parked in one of these was supposed to
# be built and is not being waited on by anything -- which is the only way debt
# goes quiet. Add a phase here when it lands; that is what makes `owed` a gate
# rather than a report.
SHIPPED_PHASES = ("ALREADY-IMPL", "Phase 5-Pre", "Phase 5-Layers")


def owed(phase=None, show_all=False):
    """Atoms a shipped phase left behind: no test, and a ticket that promised one.

    This is the query that was missing when Phase 5-Pre closed. That phase
    carried 223 atoms and shipped with none of them covered, including
    ATOM-400.7-001, whose ticket specified a `zone_change_epoch` field on
    `GameObject` down to the name. Nothing asked at close time, so the design was
    lost for two years and rediscovered by hand in 2026-08.

    A `NEW ...` ticket is the strong signal -- the corpus author judged that the
    atom needed infrastructure that did not exist. `--all` drops that filter and
    shows every uncovered atom in a shipped phase, which is a much longer and
    much noisier list.
    """
    db = connect()
    phases = (phase,) if phase else SHIPPED_PHASES
    marks = ",".join("?" * len(phases))
    ticket_filter = "" if show_all else "AND a.ticket LIKE 'NEW%'"
    rows = db.execute(f"""
        SELECT a.id, a.rule_num, a.phase, a.summary, a.ticket FROM atoms a
        WHERE a.phase IN ({marks}) {ticket_filter}
          AND a.id NOT IN (SELECT atom_id FROM coverage)
        ORDER BY a.rule_num, a.id
    """, phases).fetchall()

    total = db.execute(
        f"SELECT COUNT(*) FROM atoms WHERE phase IN ({marks})", phases).fetchone()[0]
    scope = "uncovered" if show_all else "uncovered, ticketed NEW"
    print(f"shipped phases: {', '.join(phases)}  ({total} atoms)")
    print(f"{len(rows)} {scope}\n")
    if not rows:
        print("nothing owed - every atom in a shipped phase is covered or was")
        print("explicitly deferred. That is the state to keep it in.")
        return
    for aid, rule, ph, summary, ticket in rows:
        print("  %-22s %-10s %s" % (aid, rule or "-", (summary or "")[:66]))
        if ticket:
            print("  %-22s %-10s -> %s" % ("", "", ticket[:66]))
    print()
    print("Triage each as a FACT or a FEATURE before scheduling it.")
    print("  fact    - unrecoverable if not captured when it exists (object")
    print("            identity, who cast this, characteristics an instant ago).")
    print("            Costs a re-thread through every system built meanwhile.")
    print("  feature - a normal diff whenever it is added. Defer freely.")
    print("See codebase-state.md, 'Before Replacement effects' item 9.")


def orphaned(chapter=None, limit=25, show_all=False, out_path=None):
    """Behavior a shipped phase promised, that no test covers and no doc claims.

    **This is the query the five motivating gaps needed; `audit --dark` is not
    it** (`cr-coverage-audit.md` section 1). The two sound alike and
    are not:

      darkness   "has anyone *looked* at this rule?"   -> examination
      ownership  "does anyone *own* it?"                -> a home in the plan

    Five of the six motivating clusters had atoms - CR 107, 118, 202, 601 and
    607 were all examined years ago and then orphaned - so a darkness filter
    removes them before the sweep starts. Only voting (CR 701.38) was dark.
    Getting that backwards is what made A-0's sweep come back empty.

    Three filters, all mechanical:

      1. the atom is filed under a **shipped** phase - somebody promised this
         already works, which is exactly what `owed` selects
      2. **no test covers it** - the promise is unkept
      3. **no plan doc cites its rule** - and no design has claimed it since

    Citations are matched per *rule*, never per section. `plans/*.md` say
    "CR 702" constantly, and a section-level test hands every keyword ability
    to five documents at once - which is how an ownership query silently turns
    back into a darkness one.

    **What it cannot separate** is a missing `// COVERS:` annotation on code
    that already exists from genuinely missing behavior. Only reading the code
    does that, which is why the session plan budgets a per-cluster read rather
    than trusting this list.
    """
    db = connect()
    marks = ",".join("?" * len(SHIPPED_PHASES))
    rows = db.execute(f"""
        SELECT a.id, a.rule_num, a.phase, a.summary FROM atoms a
        WHERE a.phase IN ({marks})
          AND a.rule_num <> ''
          AND a.id NOT IN (SELECT atom_id FROM coverage)
        ORDER BY a.rule_num, a.id
    """, SHIPPED_PHASES).fetchall()

    _src_cites, doc_cites = _scan_citations()
    texts = {n: t for n, t in db.execute(
        "SELECT number, text FROM rules WHERE cr_version = ?", (BASELINE_VERSION,))}

    # An atom is owned when *any* rule it cites is claimed by a plan doc: the
    # atom is one scenario, and one doc claiming any part of it means the
    # mechanic has a home. Over-claiming ownership is the safe direction here -
    # it shrinks the list rather than inventing work.
    orphans = []
    for aid, rule_num, phase, summary in rows:
        tokens = RULE_TOKEN_RE.findall(rule_num)
        if not tokens or any(t in doc_cites for t in tokens):
            continue
        section = tokens[0].split(".")[0]
        if chapter and section[:1] != str(chapter):
            continue
        orphans.append((section, tokens[0], aid, summary or ""))

    by_section = {}
    for section, rule, aid, summary in orphans:
        by_section.setdefault(section, []).append((rule, aid, summary))

    lines = []
    emit = lines.append
    scope = "CR %s" % chapter if chapter else "all chapters"
    emit("ORPHANED - promised by a shipped phase, untested, unowned by any plan doc")
    emit("  %s, baseline CR %s" % (scope, BASELINE_VERSION))
    emit("  shipped phases: %s" % ", ".join(SHIPPED_PHASES))
    emit("")
    ordered = sorted(by_section.items(), key=lambda kv: (-len(kv[1]), kv[0]))
    shown = ordered if (show_all or chapter) else ordered[:limit]
    for section, items in shown:
        rule = items[0][0]
        head = texts.get(rule, "").split("\n")[0]
        emit("  CR %-5s %3d atoms    %-8s %s" % (section, len(items), rule, head[:52]))
    if len(ordered) > len(shown):
        emit("  ... %d more sections (use --all)" % (len(ordered) - len(shown)))
    emit("")
    emit("%d sections, %d atoms." % (len(by_section), len(orphans)))
    emit("")
    emit("Triage each CLUSTER, not each atom, and on two questions in order:")
    emit("  1. is this a missing `// COVERS:` on code that exists, or missing")
    emit("     behavior? Only reading the code answers it.")
    emit("  2. if behavior: does it need a new field on an existing type, or an")
    emit("     existing assumption to become false? Yes -> a FACT, escalate.")
    emit("See cr-coverage-audit.md section 2 for the fact/feature question.")

    text = "\n".join(lines) + "\n"
    if out_path:
        Path(out_path).write_text(text, encoding="utf-8")
        print("wrote %d lines to %s" % (len(lines), out_path))
    else:
        print(text, end="")


# --- audit: is the plan complete against the frozen CR? ----------------------
#
# `plans/cr-coverage-audit.md` owns the method; this is
# its generator. Everything below is derived - nothing here is hand-maintained.

# CR sections outside the engine's scope, as *data* so that extending the list
# is a one-line diff with its reason beside it. CR 903 (Commander) is
# deliberately absent: CLAUDE.md names 4-player Commander a v1 target, so it is
# in scope and its dark rules are real work.
OUT_OF_SCOPE = [
    ("901.", "Planechase"),
    ("902.", "Vanguard"),
    ("904.", "Archenemy"),
    ("905.", "Conspiracy Draft"),
    ("407.", "ante"),
    ("801.", "limited range of influence - an OPTIONAL multiplayer rule; "
             "CR 800 and 802 are on the critical path and stay in scope"),
    ("100.6", "tournament rules"),
    ("100.7", "casual / acorn cards"),
]

# A CR citation in prose or code: "CR 613.7a", "rule 702.33". Deliberately does
# not match a bare number, which is ambiguous with damage amounts and years.
CITE_RE = re.compile(r"\b(?:CR|rule)\s+(\d{3}(?:\.\d+[a-z]?)?)", re.I)
PLAN_DOCS_DIR = ROOT / "plans"

_SORT_RE = re.compile(r"^(\d+)\.(\d+)([a-z]?)$")
_FAMILY_RE = re.compile(r"^(\d+\.\d+)")


def _rule_sort_key(number):
    """Numeric ordering, so 702.10 follows 702.9 instead of preceding 702.2."""
    m = _SORT_RE.match(number)
    return (int(m.group(1)), int(m.group(2)), m.group(3)) if m else (9999, 9999, number)


def _family(number):
    """The triage unit: 702.10a and 702.10b both belong to family 702.10."""
    m = _FAMILY_RE.match(number)
    return m.group(1) if m else number


def _out_of_scope(number):
    for prefix, reason in OUT_OF_SCOPE:
        if number.startswith(prefix):
            return reason
    return None


def _scan_citations():
    """Rule numbers cited in Rust source/tests, and in plan docs.

    Two sets, because they answer different questions. A citation in *source*
    means the code already encodes an assumption about that rule; one in a
    *plan doc* means a design has considered it. Neither is coverage - a cited
    rule can still have no atom and no test - but either one means the rule is
    not dark, which is the only claim these two columns make.
    """
    src, docs = set(), set()
    for directory in CODE_DIRS:
        for path in sorted(directory.rglob("*.rs")):
            text = path.read_text(encoding="utf-8", errors="ignore")
            src.update(m.group(1) for m in CITE_RE.finditer(text))
    for path in sorted(PLAN_DOCS_DIR.glob("*.md")):
        text = path.read_text(encoding="utf-8", errors="ignore")
        docs.update(m.group(1) for m in CITE_RE.finditer(text))
    return src, docs


def audit(chapter, dark_only, families, out_path):
    """Every CR rule joined against the four places it could have been examined.

    The columns, strongest claim first:

      atom     an atom in the corpus cites the rule
      verdict  a session classified it (TESTABLE, PURE-DEF, DEFERRED, ...)
      src      a Rust source or test file cites it - the code assumes something
      doc      a plan doc cites it - a design has considered it

    A rule with **none** of the four is *dark*: nobody has looked at it in any
    recorded way. `--families` collapses dark rules to the triage unit, because
    a dark subrule of an examined rule is a depth gap rather than a blind spot
    (CR 702: 180 of its 190 keyword families already have an examined subrule).

    Reports what is unexamined, never what is wrong - a rule with an atom and a
    verdict can still be mis-modelled. `cr-coverage-audit.md` section 3.
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
    verdicts = {r: v for r, v in db.execute("SELECT number, verdict FROM rule_mentions")}
    src_cites, doc_cites = _scan_citations()

    numbers = {n for n, _ in rules}
    known = atomized | set(verdicts) | (src_cites & numbers) | (doc_cites & numbers)

    rows = [(n, t) for n, t in rules if not chapter or n[:1] == str(chapter)]
    rows.sort(key=lambda nt: _rule_sort_key(nt[0]))
    dark = [(n, t) for n, t in rows if n not in known]

    lines = []
    emit = lines.append
    scope = "CR %s" % chapter if chapter else "all chapters"
    emit("CR coverage audit - %s, baseline CR %s" % (scope, BASELINE_VERSION))
    emit("  rules in scope of this run : %d" % len(rows))
    emit("  has a corpus atom          : %d" % sum(1 for n, _ in rows if n in atomized))
    emit("  has a classification       : %d" % sum(1 for n, _ in rows if n in verdicts))
    emit("  cited in Rust source/tests : %d" % sum(1 for n, _ in rows if n in src_cites))
    emit("  cited in a plan doc        : %d" % sum(1 for n, _ in rows if n in doc_cites))
    emit("  DARK (none of the four)    : %d" % len(dark))

    if families:
        by_family = {}
        for n, _t in dark:
            by_family.setdefault(_family(n), []).append(n)
        # A family survives only if *no* rule in it is known anywhere. A dark
        # subrule of an examined family is a depth gap - Pass B's, not Pass A's.
        #
        # **Membership is `_family`, not a string prefix.** The prefix test this
        # replaces asked whether a known rule started with `613.4.`, which no
        # subrule ever does - they are `613.4a`, not `613.4.a` - so a family
        # whose subrules were all examined still reported wholly dark. That is
        # `cr-coverage-audit.md` section 1 defect D2, and it inflated that
        # sweep's worklist by roughly sevenfold.
        known_families = {_family(k) for k in known}
        orphans = [f for f in by_family if f not in known_families]
        texts = {n: t for n, t in rules}
        in_scope = [f for f in orphans if not _out_of_scope(f)]
        emit("  dark families                     : %d" % len(by_family))
        emit("    with an examined sibling (depth) : %d" % (len(by_family) - len(orphans)))
        emit("    wholly dark                      : %d" % len(orphans))
        emit("    ... of which out of scope        : %d" % (len(orphans) - len(in_scope)))
        emit("    IN-SCOPE AUDIT SURFACE           : %d" % len(in_scope))
        emit("")
        emit("in-scope wholly-dark families - the Pass A worklist:")
        emit("  %-10s %5s  %s" % ("family", "subs", "rule text"))
        for f in sorted(in_scope, key=_rule_sort_key):
            head = texts.get(f, "").split("\n")[0]
            emit("  %-10s %5d  %s" % (f, len(by_family[f]), head[:66]))
    else:
        listing = dark if dark_only else rows
        label = "dark rules" if dark_only else "every rule"
        emit("")
        emit("%s - flags are atom/verdict/src/doc:" % label)
        for n, t in listing:
            flags = "".join([
                "a" if n in atomized else "-",
                "v" if n in verdicts else "-",
                "s" if n in src_cites else "-",
                "d" if n in doc_cites else "-",
            ])
            oos = _out_of_scope(n)
            tag = "  [out of scope]" if oos else ""
            emit("  %-10s %s  %-14s %s%s"
                 % (n, flags, verdicts.get(n, "")[:14],
                    t.split("\n")[0][:56], tag))

    text = "\n".join(lines) + "\n"
    if out_path:
        Path(out_path).write_text(text, encoding="utf-8")
        print("wrote %d lines to %s" % (len(lines), out_path))
    else:
        print(text, end="")


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
    o = sub.add_parser("owed",
                       help="atoms a shipped phase left uncovered (the phase-exit gate)")
    o.add_argument("--phase", help="one phase instead of every shipped one")
    o.add_argument("--all", action="store_true", dest="owed_all",
                   help="every uncovered atom, not just those ticketed NEW")
    r = sub.add_parser("orphaned",
                       help="shipped-phase behavior no test covers and no doc owns")
    r.add_argument("--chapter", type=int, help="CR chapter 1-9")
    r.add_argument("--limit", type=int, default=25)
    r.add_argument("--all", action="store_true", dest="orph_all")
    r.add_argument("--out", help="write the table here instead of stdout")
    u = sub.add_parser("audit",
                       help="CR rules nobody has examined (cr-coverage-audit.md)")
    u.add_argument("--chapter", type=int, help="CR chapter 1-9")
    u.add_argument("--dark", action="store_true",
                   help="only rules with no atom, verdict, source cite or doc cite")
    u.add_argument("--families", action="store_true",
                   help="collapse to NNN.M, the Pass A triage unit")
    u.add_argument("--out", help="write the table here instead of stdout")
    a = ap.parse_args()
    {"build": lambda: build(),
     "stats": lambda: stats(),
     "next": lambda: next_up(a.phase, a.rule, a.limit),
     "show": lambda: show(a.atom_id),
     "orphans": lambda: orphans(),
     "suspicious": lambda: suspicious(a.threshold),
     "gaps": lambda: gaps(a.chapter, a.limit, a.show_all),
     "owed": lambda: owed(a.phase, a.owed_all),
     "audit": lambda: audit(a.chapter, a.dark, a.families, a.out),
     "orphaned": lambda: orphaned(a.chapter, a.limit, a.orph_all, a.out)}[a.cmd]()


if __name__ == "__main__":
    main()
