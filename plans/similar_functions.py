#!/usr/bin/env python3
"""similar_functions — functions whose token sequences nearly match another's.

An instrument for `engineering-practices.md` §9's hygiene pass, not a gate:
card definitions resemble each other by design, so the list is read, never
counted against a threshold. It was written at #225's review (2026-10-06),
which found the permanents a player controls built in six places, one of them
a helper the engine never called.

    python plans/similar_functions.py

Every non-test function of 60 or more tokens in `mtgsim/src` and `devgui/src`
(binaries left out) is cut into token 5-grams, and pairs are compared by
Jaccard similarity twice: with names kept (a near-copy) and with every
identifier masked (one shape, different names). **A copy written differently
is invisible to it**: the permanents' copies were a filter in one place and a
sort in another. That half of the question is a grep per common board query
(a player's permanents, creatures, opponents, untapped lands), which the pass
runs beside this.
"""
import itertools
import pathlib
import re

ROOT = pathlib.Path(__file__).resolve().parent.parent
SOURCES = [ROOT / "mtgsim" / "src", ROOT / "devgui" / "src"]
MIN_TOKENS = 60
NEAR_COPY = 0.6
SAME_SHAPE = 0.85

FN = re.compile(r"^\s*(pub(\([^)]*\))?\s+)?(const\s+)?fn\s+(\w+)", re.M)
TOKEN = re.compile(r"[A-Za-z_][A-Za-z0-9_]*|\d+|==|!=|<=|>=|&&|\|\||::|->|=>|[{}()\[\];,.<>!&|*+\-/=?:#']")
KEYWORDS = set("""as break const continue crate else enum extern false fn for if impl in let loop match mod move mut pub
ref return self Self static struct super trait true type unsafe use where while Some None Ok Err Vec Option Result""".split())


def functions(path):
    """Each function above the file's test module, as (name, body)."""
    text = path.read_text(encoding="utf-8")
    cut = text.find("#[cfg(test)]")
    if cut >= 0:
        text = text[:cut]
    text = re.sub(r"//[^\n]*", "", text)
    for m in FN.finditer(text):
        start = text.find("{", m.end())
        semi = text.find(";", m.end())
        if start < 0 or (0 <= semi < start):
            continue
        depth, i = 0, start
        while i < len(text):
            if text[i] == "{":
                depth += 1
            elif text[i] == "}":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        yield m.group(4), text[start:i + 1]


def grams(tokens, n=5):
    return {tuple(tokens[i:i + n]) for i in range(len(tokens) - n + 1)}


def main():
    fns = []
    for src in SOURCES:
        for path in sorted(src.rglob("*.rs")):
            if "bin" in path.parts:
                continue
            for name, body in functions(path):
                tokens = TOKEN.findall(body)
                if len(tokens) < MIN_TOKENS:
                    continue
                masked = [t if (t in KEYWORDS or not re.match(r"[A-Za-z_]", t)) else "ID" for t in tokens]
                where = str(path.relative_to(ROOT)).replace("\\", "/")
                fns.append((where, name, grams(tokens), grams(masked), len(tokens)))

    near, shape = [], []
    for a, b in itertools.combinations(fns, 2):
        small, big = sorted((a[4], b[4]))
        if small < 0.6 * big:
            continue
        kept = len(a[2] & b[2]) / max(1, len(a[2] | b[2]))
        if kept >= NEAR_COPY:
            near.append((kept, a, b))
            continue
        masked = len(a[3] & b[3]) / max(1, len(a[3] | b[3]))
        if masked >= SAME_SHAPE:
            shape.append((masked, a, b))

    print(f"{len(fns)} functions of {MIN_TOKENS} tokens or more")
    for title, pairs in ((f"near-copies, names kept (Jaccard >= {NEAR_COPY})", near),
                         (f"one shape, names differ (masked Jaccard >= {SAME_SHAPE})", shape)):
        print(f"\n== {title}: {len(pairs)} pairs")
        for j, a, b in sorted(pairs, key=lambda p: -p[0]):
            print(f"  {j:.2f}  {a[0]}::{a[1]} ({a[4]})  ~  {b[0]}::{b[1]} ({b[4]})")


if __name__ == "__main__":
    main()
