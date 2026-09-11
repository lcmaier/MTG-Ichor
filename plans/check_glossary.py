#!/usr/bin/env python3
"""
check_glossary - the codebase's invented vocabulary, as a gate rather than a doc.

`plans/glossary.md` states facts about the tree: that `Member` is a batch
member, that `ToEffectSource` is one of two things called a source. A doc that
states facts and has nothing re-reading it is `codebase-state.md` item 89 with
a new name - it goes stale in the commit that renames something and nobody
finds out. This is the re-reader.

    python plans/check_glossary.py        # exit 1 on any of three failures

Three assertions:

  1. every term the glossary defines, and every code anchor an entry names,
     still appears in `mtgsim/src` - so a rename fails here in the commit that
     renames;
  2. every word on `WATCHLIST` below is defined in the glossary - so a new
     coinage cannot land undefined. The list is this check's *input*: coin a
     word, add it here, and the gate makes you define it;
  3. every word in `POLYSEMOUS` carries all of its senses, numbered. One sense
     per word is what let `Rewrite::Retarget` reach the build with arms called
     `ToSource` and `ToSourceController` - two different sources, adjacent -
     and both had to be renamed mid-PR.

**What assertion 1 does and does not prove.** It is a text search over every
`.rs` file, comments included, so a rename that leaves a "this used to be X"
note behind still passes. That is deliberate: such a note is the pointer a
reader following the old name needs, and four of them exist today. What fails
is a name that vanishes from the crate entirely, which is the case the glossary
cannot survive.

`check_claude_md.py` and `check_module_layout.py` are the template, and
CLAUDE.md's Commands fence runs all three.
"""

import re
import sys
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

ROOT = Path(__file__).resolve().parent.parent
GLOSSARY = ROOT / "plans" / "glossary.md"
CRATE_SRC = ROOT / "mtgsim" / "src"

# Every word this project uses in a sense a reader cannot recover from ordinary
# English plus one rule number. Adding one here is the whole cost of coining it.
WATCHLIST = [
    "applied set", "arm", "atom", "batch", "blocked", "bucket", "candidate",
    "ceiling", "census", "chokepoint", "donor", "emitter", "epoch", "frame",
    "gate", "host", "instance", "ladder", "leg", "member", "memo", "performer",
    "pool", "proposal", "rider", "shield", "source", "step", "subject",
    "subject group", "sweep",
]

# Words that name more than one thing, and how many senses the glossary owes
# each. A new collision is a line here and a numbered sense there, together.
POLYSEMOUS = {
    "source": 3, "shield": 3, "census": 2, "pool": 2,
    "step": 2, "blocked": 2, "gate": 2,
}

# A definition paragraph opens with its term(s) in bold, then an em-dash:
#   `**subject group** — ...`, `**performer** / **emitter** — ...`
TERM = re.compile(r"^((?:\*\*[^*]+\*\*)(?:\s*/\s*\*\*[^*]+\*\*)*)\s+—\s")
BOLD = re.compile(r"\*\*([^*]+)\*\*")
SENSE = re.compile(r"\*\*\((\d+)\)\*\*")

TICKED = re.compile(r"`([^`]+)`")
RS_PATH = re.compile(r"^[a-z0-9_]+(?:/[a-z0-9_]+)*\.rs$")
IDENT = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)*$")
# A backticked word that is just lowercase English (`gather`, `owed`) is prose,
# not an anchor. An anchor has a `::`, an `_`, or a capital in it.
LOOKS_LIKE_CODE = re.compile(r"::|_|[A-Z]")


def entries(text):
    """Every definition paragraph under a `## ` heading: (terms, paragraph)."""
    out, para, in_body = [], [], False
    for line in text.splitlines() + [""]:
        if line.startswith("## "):
            in_body = True
        if line.strip():
            para.append(line)
            continue
        if in_body and para and (m := TERM.match(para[0])):
            terms = [t.strip().lower() for t in BOLD.findall(m.group(1))]
            out.append((terms, "\n".join(para)))
        para = []
    return out


def anchors(paragraph):
    """The backticked spans in a paragraph that name code rather than prose."""
    for span in TICKED.findall(paragraph):
        if span.endswith((".md", ".py", ".txt", ".sqlite")):
            continue
        if RS_PATH.match(span):
            yield span, "path"
        elif IDENT.match(span) and LOOKS_LIKE_CODE.search(span):
            yield span, "ident"


def main() -> int:
    if not GLOSSARY.exists():
        print(f"not found: {GLOSSARY}", file=sys.stderr)
        return 2
    if not CRATE_SRC.is_dir():
        print(f"no crate source at {CRATE_SRC}", file=sys.stderr)
        return 2

    rs = sorted(CRATE_SRC.rglob("*.rs"))
    blob = "\n".join(p.read_text(encoding="utf-8") for p in rs)
    words = set(re.findall(r"[A-Za-z_][A-Za-z0-9_]*", blob))
    paths = {p.as_posix() for p in (q.relative_to(CRATE_SRC) for q in rs)}

    def in_src(phrase):
        pattern = r"\b" + r"\s+".join(re.escape(w) for w in phrase.split()) + r"\b"
        return re.search(pattern, blob, re.IGNORECASE | re.DOTALL) is not None

    defined, failures = {}, []
    for terms, paragraph in entries(GLOSSARY.read_text(encoding="utf-8")):
        for term in terms:
            # Two entries for one word is the failure this file warns about,
            # and it would also hide a sense count from assertion 3.
            if term in defined:
                failures.append(f'"{term}" is defined twice; one word, one entry')
            defined[term] = paragraph
            # 1a - the term itself.
            if not in_src(term):
                failures.append(f'"{term}" is defined but appears nowhere in mtgsim/src')
        # 1b - the code each entry points at.
        for span, kind in anchors(paragraph):
            if kind == "path":
                if not any(p == span or p.endswith("/" + span) for p in paths):
                    failures.append(f'{terms[0]}: no such file under mtgsim/src - `{span}`')
            elif missing := [s for s in span.split("::") if s not in words]:
                failures.append(
                    f"{terms[0]}: `{span}` no longer resolves - "
                    f"{', '.join(missing)} is in no .rs file"
                )

    # 2 - the watch-list is the input; every word on it owes a definition.
    for word in WATCHLIST:
        if word not in defined:
            failures.append(f'"{word}" is on the watch-list and is not defined')

    # 3 - a word with more than one meaning carries all of them, numbered.
    for word, owed in POLYSEMOUS.items():
        if word not in defined:
            failures.append(f'"{word}" is listed as polysemous and is not defined')
            continue
        senses = sorted(int(n) for n in SENSE.findall(defined[word]))
        if senses != list(range(1, owed + 1)):
            failures.append(
                f'"{word}" owes {owed} numbered senses; its entry carries {senses or "none"}'
            )

    print(f"  terms defined       {len(defined):>4}")
    print(f"  watch-list          {len(WATCHLIST):>4}")
    print(f"  polysemous          {len(POLYSEMOUS):>4}")
    print(f"  .rs files searched  {len(rs):>4}")
    print()

    if failures:
        print("glossary: the tree moved and plans/glossary.md did not.\n")
        for line in failures:
            print(f"  {line}")
        print(
            "\nFix the glossary, not this script - unless the word really is gone,\n"
            "in which case drop it from WATCHLIST in the same commit. A rename that\n"
            "argues for itself belongs in `plans/codebase-state.md` and its own PR."
        )
        return 1

    print("glossary: every term resolves, every watched word is defined, every sense is numbered.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
