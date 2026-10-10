#!/usr/bin/env python3
"""similar_functions — one fact computed in more than one place.

An instrument for `engineering-practices.md` §9's hygiene pass, not a gate:
card definitions resemble each other by design, so the list is read, never
counted against a threshold. It was written at #225's review (2026-10-06),
which found the permanents a player controls built in six places, and widened
at #239's (A6l, 2026-10-10), where one rule was spelled three times and one
fact split across two files, each copy too short and too differently written
for a token scan.

    python plans/similar_functions.py                  # the report over the tree
    python plans/similar_functions.py --all            # every entry, not the default cut
    python plans/similar_functions.py --against main   # only what is new against a revision

Every non-test function in `mtgsim/src` and `devgui/src` (binaries left out)
is read five ways. The first two compare token 5-grams of functions of 60 or
more tokens, card definitions included, by Jaccard similarity: with names kept
(a near-copy) and with every identifier masked (one shape, different names).
The other three fingerprint every function, however short, outside the card
definitions, and list a key that functions in different files share:

- **same inputs**: the parameter types, `self` as its impl's type. Two or more
  parameters, shared by at most four functions, or by at most eight that
  also return one type. Two functions of the same inputs may be one fact
  projected twice (an object's identity and its characteristics, both "as it
  deals damage").
- **same composition**: two reads in a row, one of them a call. A read is a
  dotted path off a value, with the value's own name and std plumbing dropped:
  `self.events.next_seq()` is `events.next_seq()`, `path::controls(..)` is
  `controls()`. Listed when at most twelve functions make it, the two occur
  together at least ten times as often as chance would put them in one
  function, and the functions sit in three files or more (two with `--all`).
  This is where an inline copy of a helper shows: the helper is one of the
  functions.
- **same test**: an equality test, each side cut to its last name
  (`binding.subject == Some(source)` is `subject == Some(_)`), a literal or an
  enum variant on either side left out. A rule spelled twice usually spells
  its condition twice.

What it cannot see: a copy that shares no input signature, no adjacent pair of
reads and no test with its original — the same fact reached through different
fields, or a rule restated in different terms. Reading is regex over Rust, not
a parse: a macro body, a closure passed as a value and a type behind an alias
are invisible or misread. The per-PR use is `--against main`, which subtracts
everything already on main and leaves the copies the branch made.
"""
import argparse
import collections
import itertools
import pathlib
import re
import subprocess

ROOT = pathlib.Path(__file__).resolve().parent.parent
SOURCE_DIRS = ["mtgsim/src", "devgui/src"]
MIN_TOKENS = 60
NEAR_COPY = 0.6
SAME_SHAPE = 0.85
#: An input signature shared by more functions is a vocabulary, not a fact.
SAME_INPUTS_MAX = 4
SAME_INPUTS_AND_OUTPUT_MAX = 8
COMPOSITION_MAX = 12
COMPOSITION_LIFT = 10
COMPOSITION_FILES = 3
TEST_MAX = 6

FN = re.compile(r"^[ \t]*(pub(\([^)]*\))?\s+)?(const\s+)?(async\s+)?(unsafe\s+)?fn\s+(\w+)", re.M)
IMPL = re.compile(r"^[ \t]*impl\b", re.M)
TEST_MODULE = re.compile(r"#\[cfg\(test\)\]\s*(pub(\([^)]*\))?\s+)?mod\s+\w+\s*\{")
TEST_ITEM = re.compile(r"#\[cfg\(test\)\]\s*$")
LITERALS = re.compile(r'//[^\n]*|/\*.*?\*/|r(#*)".*?"\1|b?"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])\'', re.S)
TOKEN = re.compile(r"[A-Za-z_][A-Za-z0-9_]*|\d+|==|!=|<=|>=|&&|\|\||::|->|=>|[{}()\[\];,.<>!&|*+\-/=?:#']")
WORD = re.compile(r"[A-Za-z_]\w*|\d+|::|->|=>|\.\.=?|[(){}\[\].?]|[^\sA-Za-z_(){}\[\].?]+")
TEST = re.compile(r"([\w.]+(?:\(\))?)\s*(==|!=)\s*([&*]*[\w.:]+(?:\([^()]*\))?)")
KEYWORDS = set("""as break const continue crate else enum extern false fn for if impl in let loop match mod move mut pub
ref return self Self static struct super trait true type unsafe use where while Some None Ok Err Vec Option Result""".split())
#: std's plumbing: how a value is carried, never which fact it is.
PLUMBING = set("""iter into_iter iter_mut map filter filter_map flat_map and_then or_else as_ref as_mut as_deref clone
cloned copied collect unwrap unwrap_or unwrap_or_default unwrap_or_else expect is_some is_none is_some_and is_none_or
is_ok is_err ok err contains get get_mut len is_empty push extend any all find position count first last insert remove
entry or_insert or_insert_with or_default take replace to_string to_owned into from default new with_capacity sort
sort_by sort_by_key sort_unstable sort_unstable_by_key retain drain chain zip enumerate rev skip next peekable max min
max_by_key min_by_key sum fold flatten keys values values_mut join format as_str borrow borrow_mut deref ok_or
ok_or_else map_or map_or_else then then_some dedup truncate split_off pop clear starts_with ends_with trim push_str
fmt write writeln eq ne cmp partial_cmp hash""".split())


def balanced(text, i, open_, close):
    """The index of the bracket closing the one at `i`."""
    depth = 0
    while i < len(text):
        if text[i] == open_ and not (open_ == "<" and text[i - 1] == "-"):
            depth += 1
        elif text[i] == close and not (close == ">" and text[i - 1] == "-"):
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return len(text) - 1


def blank(text, strings):
    """Comments blanked, and strings and chars too when `strings`, each
    character to a space with newlines kept, so offsets agree between the
    two and a `{` or a `//` inside a string cannot cut a function short. The
    token scan keeps the strings: a card's oracle text is what tells two card
    definitions apart."""
    def spaces(m):
        if m.group(0)[0] != "/" and not strings:
            return m.group(0)
        return re.sub(r"[^\n]", " ", m.group(0))
    return LITERALS.sub(spaces, text)


def type_name(text):
    """A type as written, without references, lifetimes or module paths:
    `&'a crate::types::triggers::DepartedFrame` is `DepartedFrame`."""
    text = re.sub(r"'\w+\s*", "", text)
    text = re.sub(r"\b(mut|dyn|impl)\b", "", text)
    text = re.sub(r"\b[a-z_]\w*::", "", text)
    return text.replace("&", "").replace(" ", "")


def split_commas(text):
    out, depth, start = [], 0, 0
    for i, c in enumerate(text):
        if c in "<([":
            depth += 1
        elif c in ")]" or (c == ">" and text[i - 1:i] != "-"):
            depth -= 1
        elif c == "," and depth == 0:
            out.append(text[start:i])
            start = i + 1
    out.append(text[start:])
    return [p.strip() for p in out if p.strip()]


def impl_type(header):
    """The type an `impl` block's methods take as `self`."""
    header = header.strip()
    if header.startswith("<"):
        header = header[balanced(header, 0, "<", ">") + 1:]
    header = re.split(r"\bwhere\b", header)[0]
    target = re.split(r"\bfor\b", header)[-1]
    target = re.sub(r"<.*", "", type_name(target))
    return target or "Self"


def reads(body):
    """The values a function reads, in order: each a dotted path off a value,
    the value's own name and std's plumbing dropped, a call marked `()`."""
    words = WORD.findall(body)
    out = []
    levels = [[[], False]]  # per bracket depth: the path so far, and whether `.` may extend it

    def end_path(level):
        names = [n for n in level[0] if n.rstrip("()") not in PLUMBING]
        if names:
            out.append(".".join(names))
        level[0], level[1] = [], False

    for i, w in enumerate(words):
        level = levels[-1]
        before = words[i - 1] if i else ""
        after = words[i + 1] if i + 1 < len(words) else ""
        if w in ("(", "[", "{"):
            if w == "{":
                end_path(level)
            levels.append([[], False])
        elif w in (")", "]", "}"):
            if len(levels) > 1:
                end_path(levels.pop())
            level = levels[-1]
            if w == "}":
                end_path(level)
            else:
                level[1] = True
        elif (w == "." and level[1]) or w == "?" or (w.isdigit() and before == "." and level[1]):
            pass
        elif re.match(r"[A-Za-z_]", w):
            call = "()" if after == "(" else ""
            if before == "." and level[1]:
                level[0].append(w + call)
            elif before == "::":
                # `path::f(..)`, or `path::f` passed as a value; a type or a
                # variant (`Zone::Hand`) is no read.
                if w[0].islower() and after != "::" and w not in KEYWORDS:
                    level[0] = [w + "()"]
            else:
                end_path(level)
                if call and w[0].islower() and w not in KEYWORDS:
                    level[0] = [w + call]
            level[1] = w not in KEYWORDS or w == "self"
        else:
            end_path(level)
    for level in levels:
        end_path(level)
    return out


def test_side(text):
    text = text.strip().lstrip("&*")
    wrapped = re.fullmatch(r"(Some|Ok|Err)\((.*)\)", text)
    if wrapped:
        return wrapped.group(1) + "(_)"
    text = re.sub(r"\([^()]*\)$", "()", text)
    return text.split(".")[-1].split("::")[-1]


def tests(body):
    """The equality tests a function makes, each side cut to its last name.
    A side that is a literal, an enum variant or a one- or two-letter local
    says nothing about which fact is tested."""
    out = set()
    for m in TEST.finditer(body):
        sides = (test_side(m.group(1)), test_side(m.group(3)))
        if all(len(s) > 2 and not s[0].isdigit() and (not s[0].isupper() or s.endswith("(_)")) for s in sides):
            out.add(" == ".join(sorted(sides)))
    return out


def functions(source, where):
    """Each function outside the file's test code, fingerprinted."""
    worded, text = blank(source, strings=False), blank(source, strings=True)
    for m in reversed(list(TEST_MODULE.finditer(text))):
        end = balanced(text, m.end() - 1, "{", "}")
        text = text[:m.start()] + re.sub(r"[^\n]", " ", text[m.start():end + 1]) + text[end + 1:]
    impls = []
    for m in IMPL.finditer(text):
        open_brace = text.find("{", m.end())
        if open_brace >= 0 and ";" not in text[m.end():open_brace]:
            impls.append((open_brace, balanced(text, open_brace, "{", "}"), impl_type(text[m.end():open_brace])))
    for m in FN.finditer(text):
        if TEST_ITEM.search(text, 0, m.start()):
            continue
        i = m.end()
        while i < len(text) and text[i].isspace():
            i += 1
        if i < len(text) and text[i] == "<":
            i = balanced(text, i, "<", ">") + 1
        open_paren = text.find("(", i)
        close_paren = balanced(text, open_paren, "(", ")")
        start = text.find("{", close_paren)
        semi = text.find(";", close_paren)
        if open_paren < 0 or start < 0 or (0 <= semi < start):
            continue
        end = balanced(text, start, "{", "}")
        tail = re.split(r"\bwhere\b", text[close_paren + 1:start])[0]
        owner = next((ty for s, e, ty in reversed(impls) if s < m.start() < e), "Self")
        params = []
        for p in split_commas(text[open_paren + 1:close_paren]):
            if re.fullmatch(r"&?\s*('\w+\s+)?(mut\s+)?self", p):
                params.append(owner)
            elif ":" in p:
                params.append(type_name(p.split(":", 1)[1]))
        body = text[start:end + 1]
        yield dict(where=where, name=m.group(6), line=text.count("\n", 0, m.start(6)) + 1,
                   params=tuple(sorted(params)), output=type_name(tail.split("->", 1)[1]) if "->" in tail else "",
                   tokens=TOKEN.findall(worded[start:end + 1]), reads=reads(body), tests=tests(body))


def sources(root):
    """(path, text) of each source file under `root`."""
    for d in SOURCE_DIRS:
        for path in sorted((root / d).rglob("*.rs")):
            yield str(path.relative_to(root)).replace("\\", "/"), path.read_text(encoding="utf-8")


def sources_at(rev):
    """(path, text) of each source file at `rev`, read from git."""
    listed = subprocess.run(["git", "ls-tree", "-r", "--name-only", rev, "--", *SOURCE_DIRS], cwd=ROOT,
                            capture_output=True, text=True, check=True).stdout.split()
    names = [n for n in listed if n.endswith(".rs")]
    blobs = subprocess.run(["git", "cat-file", "--batch"], cwd=ROOT, capture_output=True, check=True,
                           input="".join(f"{rev}:{n}\n" for n in names).encode()).stdout
    at = 0
    for name in names:
        header_end = blobs.index(b"\n", at)
        size = int(blobs[at:header_end].split()[2])
        yield name, blobs[header_end + 1:header_end + 1 + size].decode("utf-8")
        at = header_end + 1 + size + 1


def collect(files):
    fns = []
    for where, source in files:
        if "/bin/" in where or where.endswith("/test_support.rs"):
            continue
        fns.extend(functions(source, where))
    return fns


def label(f):
    return f"{f['where']}::{f['name']}"


def grams(tokens, n=5):
    return {tuple(tokens[i:i + n]) for i in range(len(tokens) - n + 1)}


def token_pairs(fns):
    """The token scan: (title, entries), each entry (key, members, note)."""
    big = []
    for f in fns:
        if len(f["tokens"]) >= MIN_TOKENS:
            masked = [t if (t in KEYWORDS or not re.match(r"[A-Za-z_]", t)) else "ID" for t in f["tokens"]]
            big.append((f, grams(f["tokens"]), grams(masked)))
    near, shape = [], []
    for (a, ka, ma), (b, kb, mb) in itertools.combinations(big, 2):
        small, large = sorted((len(a["tokens"]), len(b["tokens"])))
        if small < 0.6 * large:
            continue
        kept = len(ka & kb) / max(1, len(ka | kb))
        if kept >= NEAR_COPY:
            near.append((kept, a, b))
            continue
        masked = len(ma & mb) / max(1, len(ma | mb))
        if masked >= SAME_SHAPE:
            shape.append((masked, a, b))
    sections = []
    for title, pairs in ((f"near-copies, names kept (Jaccard >= {NEAR_COPY})", near),
                         (f"one shape, names differ (masked Jaccard >= {SAME_SHAPE})", shape)):
        entries = [(f"{label(a)}  ~  {label(b)}", (a, b), f"{j:.2f}, {len(a['tokens'])} and {len(b['tokens'])} tokens")
                   for j, a, b in sorted(pairs, key=lambda p: -p[0])]
        sections.append((title, entries))
    return sections, len(big)


def across_files(members):
    return len({f["where"] for f in members})


def same_inputs(fns):
    by_inputs, by_both = collections.defaultdict(list), collections.defaultdict(list)
    for f in fns:
        if len(f["params"]) >= 2:
            by_inputs[f["params"]].append(f)
            if f["output"]:
                by_both[(f["params"], f["output"])].append(f)
    entries = {}
    for params, members in by_inputs.items():
        if len(members) <= SAME_INPUTS_MAX and across_files(members) >= 2:
            entries[", ".join(params)] = (members, "")
    for (params, output), members in by_both.items():
        if len(members) <= SAME_INPUTS_MAX or len(members) > SAME_INPUTS_AND_OUTPUT_MAX:
            continue
        # A larger group lists only the functions that read something in common.
        reading = [f for f in members if any(set(f["reads"]) & set(g["reads"]) for g in members if g is not f)]
        if across_files(reading) >= 2:
            entries[f"{', '.join(params)} -> {output}"] = (reading, f"{len(members)} share it")
    rows = [(key, members, note) for key, (members, note) in entries.items()]
    rows.sort(key=lambda r: (len(r[1]), r[0]))
    return rows


def same_composition(fns, every):
    alone, together, pairs_of = collections.Counter(), collections.Counter(), []
    for f in fns:
        alone.update(set(f["reads"]))
        r = f["reads"]
        pairs = {tuple(sorted(p)) for p in itertools.chain(zip(r, r[1:]), zip(r, r[2:]))
                 if p[0] != p[1] and (p[0].endswith("()") or p[1].endswith("()"))}
        together.update(pairs)
        pairs_of.append(pairs)
    by_members = collections.defaultdict(list)
    for f, pairs in zip(fns, pairs_of):
        for p in pairs:
            if 2 <= together[p] <= COMPOSITION_MAX:
                by_members[p].append(f)
    facts = collections.defaultdict(list)
    for p, members in by_members.items():
        lift = together[p] * len(fns) / (alone[p[0]] * alone[p[1]])
        if lift >= COMPOSITION_LIFT and across_files(members) >= (2 if every else COMPOSITION_FILES):
            facts[tuple(sorted(id(f) for f in members))].append((p, lift, members))
    rows = []
    for found in facts.values():
        members = found[0][2]
        key = "; ".join(f"{a} + {b}" for (a, b), _, _ in sorted(found))
        rows.append((key, members, f"{max(lift for _, lift, _ in found):.0f}x chance"))
    rows.sort(key=lambda r: (-across_files(r[1]), -float(r[2].split("x")[0]), r[0]))
    return rows


def same_test(fns):
    counts = collections.Counter(t for f in fns for t in f["tests"])
    by_test = collections.defaultdict(list)
    for f in fns:
        for t in f["tests"]:
            if 2 <= counts[t] <= TEST_MAX:
                by_test[t].append(f)
    rows = [(t, members, "") for t, members in by_test.items() if across_files(members) >= 2]
    rows.sort(key=lambda r: (-across_files(r[1]), r[0]))
    return rows


def report(files, every):
    """Every section over `files`: (title, rows), each row (key, members,
    note), the first two sections' members a pair; and the header's counts."""
    fns = collect(files)
    sections, big = token_pairs(fns)
    engine = [f for f in fns if "/cards/" not in f["where"]]
    sections += [
        (f"same inputs (two or more parameters, at most {SAME_INPUTS_MAX} functions, or "
         f"{SAME_INPUTS_AND_OUTPUT_MAX} with one output)", same_inputs(engine)),
        (f"same composition (at most {COMPOSITION_MAX} functions, together >= {COMPOSITION_LIFT}x chance, "
         f"{2 if every else COMPOSITION_FILES} files or more)", same_composition(engine, every)),
        (f"same test (at most {TEST_MAX} functions)", same_test(engine))]
    return sections, (len(fns), big, len(engine))


def report_at(rev):
    """The report over `rev`'s sources, every entry, so a key the default cut
    hides there is not read as new here."""
    sections, _ = report(sources_at(rev), every=True)
    seen = {}
    for title, rows in sections:
        atoms = seen.setdefault(title.split(" (")[0], collections.defaultdict(set))
        for key, members, _ in rows:
            for atom in key.split("; "):
                atoms[atom] |= {label(f) for f in members}
    return seen


def old_members(seen, key):
    """The functions a row's key already named at the other revision: a
    composition is several pairs, any of which may have."""
    return set().union(*(seen.get(atom, set()) for atom in key.split("; ")))


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--all", action="store_true", help="compositions in two files as well as three")
    parser.add_argument("--against", metavar="REV", help="list only what is new against this revision")
    parser.add_argument("--at", metavar="REV", help="read this revision instead of the working tree")
    args = parser.parse_args()

    files = sources_at(args.at) if args.at else sources(ROOT)
    sections, (total, big, engine) = report(files, args.all or bool(args.against))
    before = report_at(args.against) if args.against else None

    print(f"{total} functions at {args.at or 'the working tree'}; {big} of {MIN_TOKENS} tokens or more; "
          f"{engine} outside the card definitions")
    if before is not None:
        print(f"only what is new against {args.against}: a new key, or a function new to an old one (+)")
    for title, rows in sections:
        pairs = title.startswith(("near-copies", "one shape"))
        seen = before[title.split(" (")[0]] if before is not None else None
        if seen is not None:
            rows = [r for r in rows if not {label(f) for f in r[1]} <= old_members(seen, r[0])]
        print(f"\n== {title}: {len(rows)}")
        for key, members, note in rows:
            if pairs:
                print(f"  {note}  {key}")
                continue
            print(f"  {key}" + (f"  ({note})" if note else ""))
            old = old_members(seen, key) if seen is not None else set()
            for f in sorted(members, key=label):
                mark = "+" if seen is not None and label(f) not in old else " "
                print(f"     {mark} {f['where']}:{f['line']} {f['name']}")


if __name__ == "__main__":
    main()
