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

**Named `check_*.py` on purpose.** CI runs the check scripts and `CLAUDE.md`'s
exit criteria cite them as a family, so joining the family is what makes a stale
board fail a pull request rather than sit there. That is the whole mechanism:
`engineering-practices.md` §1's rule is that a convention which can fail
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

3. **The Deferred Migrations parser drifting** — `selftest()` runs the item
   splitter and the verdict classifier against a fixture before every check,
   so a regex edit that changes what counts as an item fails here rather than
   silently moving the board's numbers.

It does **not** derive per-phase status for the "can't" and copy tracks,
because those docs record their phases only in sizing tables with no status
marker. The replacement track has heading markers and is checked; the others
are a normalisation nobody has done. Said out loud rather than papered over —
a check that silently covers one track of three is worse than one that says so.
"""

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "plans" / "state-of-play.md"

ARCH_DOCS = [
    "plans/replacement-architecture.md",
    "plans/cant-effects-architecture.md",
    "plans/copy-effects-architecture.md",
    "plans/layers-architecture.md",
]


def read(rel):
    return (ROOT / rel).read_text(encoding="utf-8")


# --------------------------------------------------------------------------
# Derivation — file reads only
# --------------------------------------------------------------------------

def counts():
    reg = read("mtgsim/src/cards/registry.rs")
    pool = re.search(r"PERFORMANCE_POOL: \[&str; (\d+)\]", reg)
    tests = 0
    for base in ("mtgsim/src", "mtgsim/tests"):
        for f in (ROOT / base).rglob("*.rs"):
            tests += f.read_text(encoding="utf-8", errors="replace").count("#[test]")
    return {
        "cards": reg.count("registry.register("),
        "pool": int(pool.group(1)) if pool else 0,
        "tests": tests,
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
        if v.startswith("reachable") and "wrong today" in v[:40]:
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


def deferred_migrations():
    """Item counts for `codebase-state.md`'s Deferred Migrations section.

    The section's own audit block carries a dated snapshot of these; this is the
    live one. `unstated` is the number that matters — an item that does not say
    why it cannot bite yet is an unchecked claim, not a deferral — and
    `reachable_wrong` is the list of known wrong answers the pool can reach.
    """
    lines = read("plans/codebase-state.md").split("\n")
    start = next(i for i, l in enumerate(lines) if l.startswith("## Deferred Migrations"))
    end = len(lines)
    for i in range(start + 1, len(lines)):
        if lines[i].startswith("## "):
            end = i
            break
    block = "\n".join(lines[start:end])
    items = split_items(block)
    classes = [classify_item(b) for b in items]
    open_items = [b for b, c in zip(items, classes) if c != "closed"]
    counts = {k: classes.count(k) for k in
              ("closed", "unreachable", "reachable_wrong", "reachable_ok", "none_owed", "unstated")}
    return {
        "lines": end - start,
        "of": len(lines),
        "items": len(items),
        **counts,
        "sized": sum(1 for b in open_items if "Sized:" in b),
        "open": len(open_items),
    }


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
    ])
    items = split_items(fixture)
    got = [classify_item(b) for b in items]
    want = ["closed", "unreachable", "reachable_wrong", "unreachable", "unstated",
            "none_owed", "reachable_ok", "closed"]
    assert len(items) == 8, f"selftest: expected 8 items, parsed {len(items)}"
    assert got == want, f"selftest: {got} != {want}"
    sized = sum(1 for b, c in zip(items, got) if c != "closed" and "Sized:" in b)
    assert sized == 3, f"selftest: sized {sized} != 3"


def landed_phases():
    """Phase codes whose architecture-doc heading records them as landed.

    A `###`/`####` heading carrying ✅ is the convention every shipped
    replacement phase already follows. It is a *lower bound* elsewhere: the
    "can't" and copy tracks record phases only in sizing tables, so their
    landed phases do not appear here. See this module's docstring.
    """
    out = {}
    for doc in ARCH_DOCS:
        try:
            text = read(doc)
        except FileNotFoundError:
            continue
        for m in re.finditer(r"^#{2,4} (?:Phase )?([A-Z]{2}-?[0-9]*[a-z]?) — (.+)$", text, re.M):
            code, rest = m.group(1), m.group(2)
            if "✅" in rest:
                out[code] = doc
    return out


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
    c, dm, landed, hand = counts(), deferred_migrations(), landed_phases(), open_handoffs()
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
    L.append("## Phases their architecture doc records as landed")
    L.append("")
    L.append("A `####` heading carrying ✅. **A lower bound** — the \"can't\" and copy tracks")
    L.append("record phases in sizing tables with no status marker, so RS and CV phases are")
    L.append("absent here whether or not they shipped.")
    L.append("")
    if landed:
        for code in sorted(landed):
            L.append(f"- `{code}` — {landed[code]}")
    else:
        L.append("- (none found)")
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
    L.append("## Debt — `codebase-state.md`'s Deferred Migrations")
    L.append("")
    L.append("| | |")
    L.append("|---|---:|")
    L.append(f"| Section size | {dm['lines']} of {dm['of']} lines ({100 * dm['lines'] // dm['of']}%) |")
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
    L.append("### This is not `specdb owed`, and the two overlap nowhere")
    L.append("")
    L.append("| | `specdb owed` | Deferred Migrations |")
    L.append("|---|---|---|")
    L.append("| Unit | an **atom** — one scenario from the spec corpus | a **migration** — one code change |")
    L.append("| Looks | **backwards**, at phases already shipped | **forwards**, at systems not yet built |")
    L.append("| Catches | a phase that closed without testing its own spec | scaffolding that will lie to whatever is built on it |")
    L.append("| Reachable now | yes, by construction — the behaviour shipped | usually not, which is why it is easy to forget |")
    L.append("| Gate | a phase does not close until it is clean | read before a system's first ticket |")
    L.append("")
    L.append("**The seam between them is real.** A defect in shipped behaviour that has no")
    L.append("atom is in neither list — RC-5's item 61 is one, because the ruling it violates")
    L.append("was never written into the corpus. And `owed`'s default scope is `SHIPPED_PHASES`,")
    L.append("which lists three phases and not Phase 6, so a replacement phase closing against")
    L.append("\"owed is clean\" is making a claim about *other* phases; what actually gated it")
    L.append("was the `// COVERS:` annotation discipline. → `engineering-practices.md` §5.")
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


def flight():
    import subprocess
    print("Open PRs:")
    subprocess.run(["gh", "pr", "list", "--state", "open"], cwd=ROOT)
    print("\nBranches not in origin/main:")
    subprocess.run(["git", "branch", "-a", "--no-merged", "origin/main"], cwd=ROOT)


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--write", action="store_true", help="regenerate plans/state-of-play.md")
    ap.add_argument("--check", action="store_true", help="exit 1 if the board is stale")
    ap.add_argument("--flight", action="store_true", help="branches and PRs (needs git/gh)")
    args = ap.parse_args()

    if args.flight:
        flight()
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
