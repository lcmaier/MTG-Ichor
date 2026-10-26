#!/usr/bin/env python3
"""
check_type_surface - the type surface the audit reads stays the one it read.

Three checks, each of `plans/cr-coverage-audit.md`:

    python plans/check_type_surface.py      # exit 1 if any check fails

**1. Every field `GameState` holds is named in the audit (§4b).** §4 promised
that a new type joins the type-surface sweep the day it joins `GameState`.
Nothing held the promise, and it lapsed: between `3c322e5` and 2026-09-24
`GameState` went from 28 fields to 52, and the sweep read none of the new ones
until the re-sweep caught up (§4a). The rule: each field of `pub struct
GameState` appears in backticks in the audit. The declaration and
`GameState::new`'s struct literal each list every field, since Rust refuses a
literal that leaves one out, so the gate reads both and fails if they differ:
a regex that drops a field and still reports a confident answer is the defect
§1 records twice over.

**2. Every holder of a registered concept is on the register (§4c).** Each
concept there has a query, a type of its own or a shared type and a word in a
field's name, and a line per type that holds it, with a count. The gate runs
each query over the types `GameState` holds, transitively, and `GameAction`'s
proposals, stopping at the register's card-text types and at a borrow, and
fails a holding type the register does not list, or a count that differs.

**3. Every word a card's text spells has one spelling, or a reason
(`cr-coverage-audit.md#4d-words-and-their-spellings`).** The second check
stops at the types a card's text is written in. This one walks them: every
enum they hold, transitively, is in one of the register's classes, and every
arm of an enum in its `words` class has a line with a verdict. A new enum
fails until it is classed, a new arm until it has a line, and a **folds**
line whose item is not open, so a landed fold rewrites its lines. Every
field elsewhere in the walk whose type is a word type has a line too, so a new
`Option<PlayerRef>` on a trigger event fails as a new arm does.

**Why per type and never per field.** On 2026-09-24 the owner turned down a
gate on the fields of the record structs: they are the most-edited in the
tree, and such a gate "would fail every rename there". This one counts a
type's holders and never reads a field's name for the count, so a renamed
field passes; a new holder, a removed one or a renamed type is a line to
change.

**What it does not see.** A concept held in a shared type under a word its
query lacks (a controller in a field named `by`); a type outside the state
types (a dispatch's working values, which cannot hold outcome-bearing state
across a decision, `CLAUDE.md`); and a field the half-1 parse or the holder
parse in `rust_source.py` drops, which the half-1 agreement check guards only
for `GameState`.
"""

import re
import sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from rust_source import aliases, holders, owned_types, type_declarations  # noqa: E402
from check_state_of_play import classify_item, dm_block, items_by_run  # noqa: E402

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "mtgsim" / "src"
STATE = SRC / "state" / "game_state.rs"
AUDIT = ROOT / "plans" / "cr-coverage-audit.md"

DECLARATION = re.compile(r"^pub struct GameState \{$")
#: A field at the struct's own indentation. A doc comment starts with `/` and
#: an attribute with `#`, so neither is mistaken for one.
DECLARED = re.compile(r"^    (?:pub(?:\([^)]*\))?\s+)?([a-z_][a-z0-9_]*)\s*:")

LITERAL = re.compile(r"^        GameState \{$")
#: A field one level inside the literal: `resolving: None,` or the shorthand
#: `starting_life,`. A nested block's own lines sit deeper and are skipped.
INITIALIZED = re.compile(r"^            ([a-z_][a-z0-9_]*)\s*[:,]")

REGISTER = "## 4c. Concepts and their owners"
#: The roots the register's scope starts from.
ROOTS = ("GameState", "GameAction")
CONCEPT = re.compile(r"^#### (.+)$")
QUERY = re.compile(r"type `([A-Za-z][A-Za-z0-9_]*)`(?: named `([a-z_]+)`)?")
#: A holding type's line: "- `Type` — role" or "- `Type` ×3 — role".
HOLDING = re.compile(r"^- `([A-Z][A-Za-z0-9_]*)`(?: ×(\d+))? —")

VOCABULARY = "## 4d. Words and their spellings"
VOCABULARY_ANCHOR = "cr-coverage-audit.md#4d-words-and-their-spellings"
#: A class's bullet: "- **values** — reason: `A`, `B`", wrapping.
CLASS = re.compile(r"^- \*\*([a-z]+)\*\* —")
WORD_ENTRY = re.compile(r"^#### `([A-Z][A-Za-z0-9_]*)`$")
#: An arm's line: "- `Arm` — **verdict** ...".
ARM = re.compile(r"^- `([A-Z][A-Za-z0-9_]*)` — (.*)$")
FIELDS_HEADING = "### The fields"
#: A field's line: "- `Owner.field` — ..." or "- `Owner::Variant.0` — ...".
FIELD = re.compile(r"^- `([A-Z][A-Za-z0-9_]*(?:::[A-Z][A-Za-z0-9_]*)?\.[A-Za-z0-9_]+)` — (.*)$")
#: The types a word is held in; a field of one of them is lined.
WORD_TYPES = {"PlayerRef", "PlayerSet", "Antecedent", "ObjectFilter"}
VERDICTS = ("**owns**", "**holds**", "**kept**", "**folds**", "**not a word**")
#: A fold's items: "(item 340)" or "(items 340, 341)".
FOLD_ITEMS = re.compile(r"\(items? (\d+(?:, \d+)*)\)")
#: Where the vocabulary walk stops: the game's state, which the second check
#: reads, and the keys and containers, which spell no word.
NOT_VOCABULARY = {
    "GameState", "GameObject", "PermanentState", "PlayerState", "ObjectId", "ObjectRef", "PlayerId",
    "AbilityId", "ZoneChangeEpoch", "EventId", "BatchId", "String", "Vec", "Option", "Box", "Arc", "Rc",
}


def fields(lines, opener, field):
    """The fields inside the one block whose first line matches `opener`.

    `None` unless exactly one line matches, so a second `GameState {` literal
    stops the gate rather than being read in place of the first.
    """
    starts = [i for i, line in enumerate(lines) if opener.match(line)]
    if len(starts) != 1:
        return None
    start = starts[0]
    indent = lines[start][: len(lines[start]) - len(lines[start].lstrip())]
    for end in range(start + 1, len(lines)):
        if lines[end].startswith(indent + "}"):
            body = lines[start + 1 : end]
            return [m.group(1) for line in body if (m := field.match(line))]
    return None


class Concept:
    """One register entry: its heading, its queries, and the count each
    holding type is listed with."""

    def __init__(self, name, line):
        self.name, self.line = name, line
        self.queries: list[tuple[str, str | None]] = []
        self.listed: dict[str, int] = {}

    def claims(self, holder) -> bool:
        """Whether `holder` holds this concept, by one of its queries. A word
        matches whole between underscores: `owner` is in `source_owner`."""
        types = holder.types()
        return any(
            ty in types and (word is None or re.search(rf"(^|_){word}(_|$)", holder.field))
            for ty, word in self.queries
        )


def read_register(text: str):
    """The register's card-text stops and its concepts, or `None` without
    the section."""
    lines = text.splitlines()
    starts = [i for i, line in enumerate(lines) if line.strip() == REGISTER]
    if len(starts) != 1:
        return None
    body = []
    for line in lines[starts[0] + 1 :]:
        if line.startswith("## "):
            break
        body.append(line)
    stops_text, in_stops = [], False
    for line in body:
        if line.startswith("**Stops at:**"):
            in_stops = True
        if in_stops:
            if not line.strip():
                break
            stops_text.append(line)
    stops = set(re.findall(r"`([A-Z][A-Za-z0-9_]*)`", " ".join(stops_text)))
    concepts, current, query = [], None, None
    for k, line in enumerate(body + [""]):
        # A query runs from its label to the first holding or blank line, and
        # may wrap, so its lines are joined before it is read.
        if query is not None and (not line.strip() or HOLDING.match(line) or CONCEPT.match(line)):
            current.queries = [(ty, word or None) for ty, word in QUERY.findall(" ".join(query))]
            query = None
        if m := CONCEPT.match(line):
            current = Concept(m.group(1).strip(), starts[0] + 2 + k)
            concepts.append(current)
        elif current is None:
            continue
        elif m := HOLDING.match(line):
            current.listed[m.group(1)] = int(m.group(2) or 1)
        elif "**Query:**" in line:
            query = [line.split("**Query:**", 1)[1]]
        elif query is not None:
            query.append(line)
    return stops, concepts


class Tree:
    """Every holder outside test code, by the type it is in; the files each
    type name is declared in; and each type alias's right side."""

    def __init__(self):
        self.by_type, self.declared_in, self.alias_of = defaultdict(list), defaultdict(set), {}
        for path in sorted(SRC.rglob("*.rs")):
            rel = path.relative_to(ROOT).as_posix()
            text = path.read_text(encoding="utf-8")
            for holder in holders(rel, text):
                self.by_type[holder.owner].append(holder)
                self.declared_in[holder.owner].add(rel)
            for name, names in aliases(text).items():
                self.alias_of[name] = names
                self.declared_in[name].add(rel)

    def scope(self, stops) -> set[str]:
        """The types the register reads: what the roots hold, transitively."""
        return set().union(*(owned_types(self.by_type, root, stops, self.alias_of) for root in ROOTS))


def holdings(concept, by_type, in_scope) -> dict[str, int]:
    """How many fields of each type in scope hold `concept`."""
    counts: dict[str, int] = {}
    for name in sorted(in_scope):
        n = sum(1 for holder in by_type[name] if concept.claims(holder))
        if n:
            counts[name] = n
    return counts


def check_game_state_fields() -> int:
    lines = STATE.read_text(encoding="utf-8").splitlines()
    declared = fields(lines, DECLARATION, DECLARED)
    initialized = fields(lines, LITERAL, INITIALIZED)

    if not declared or not initialized:
        print(
            f"type surface: could not find one `pub struct GameState` and one "
            f"`GameState {{` literal in {STATE.relative_to(ROOT)}.\nThe layout "
            "this gate reads has changed; update its two openers."
        )
        return 2
    if set(declared) != set(initialized):
        print("type surface: the declaration and `GameState::new` disagree.")
        for name in sorted(set(declared) - set(initialized)):
            print(f"  declared, not initialized: {name}")
        for name in sorted(set(initialized) - set(declared)):
            print(f"  initialized, not declared: {name}")
        print("\nOne of the two parses dropped a field; fix the gate's patterns.")
        return 2

    audit = AUDIT.read_text(encoding="utf-8")
    missing = [name for name in declared if f"`{name}`" not in audit]

    print(f"type surface: {len(declared)} GameState fields, read twice and in agreement")
    if not missing:
        print(f"every field is named in {AUDIT.relative_to(ROOT)}.")
        return 0

    print(f"\n{len(missing)} field(s) {AUDIT.name} does not name:\n")
    for name in missing:
        print(f"  {name}")
    print(
        "\nAsk the type-surface question of each (cr-coverage-audit.md §2): what "
        "can the CR\nrequire here that this cannot represent? Then name it in "
        "§4b, beside the answer."
    )
    return 1


def check_register() -> int:
    register = read_register(AUDIT.read_text(encoding="utf-8"))
    if register is None:
        print(f"\nconcept register: no `{REGISTER}` section in {AUDIT.relative_to(ROOT)}.")
        return 2
    stops, concepts = register
    tree = Tree()
    by_type, declared_in = tree.by_type, tree.declared_in
    in_scope = tree.scope(stops)

    ambiguous = sorted(name for name in in_scope if len(declared_in[name]) > 1)
    if ambiguous:
        print("\nconcept register: a type the register reads is declared twice, and its holders would merge:")
        for name in ambiguous:
            print(f"  {name}: {', '.join(sorted(declared_in[name]))}")
        return 2
    unreadable = [concept.name for concept in concepts if not concept.queries]
    if unreadable:
        print("\nconcept register: an entry with no **Query:** line, which nothing can check:")
        for name in unreadable:
            print(f"  {name}")
        return 2

    holders_read = sum(len(by_type[name]) for name in in_scope)
    print(f"\nconcept register: {len(concepts)} concepts over {len(in_scope)} types and {holders_read} holders")
    failures = []
    for concept in concepts:
        found = holdings(concept, by_type, in_scope)
        for name in sorted(set(found) | set(concept.listed)):
            held, listed = found.get(name, 0), concept.listed.get(name, 0)
            if held == listed:
                continue
            if not listed:
                what = f"holds it {held}x and has no line"
            elif not held:
                what = "is listed and holds it no longer, or is no longer read"
            else:
                what = f"holds it {held}x and is listed {listed}x"
            fields_held = ", ".join(holder.name() for holder in by_type[name] if concept.claims(holder))
            failures.append(f"  {concept.name}: `{name}` {what}" + (f" ({fields_held})" if fields_held else ""))
    if not failures:
        print("every holder of a registered concept is on the register.")
        return 0
    print(f"\n{len(failures)} holding(s) the register does not match:\n")
    print("\n".join(failures))
    print(
        "\nA new holder of a concept that has an owner is a finding (cr-coverage-audit.md §4c):"
        "\nkeep it only with a reason, as a line under its concept, or hold the key instead."
    )
    return 1


def read_vocabulary(text: str):
    """§4d's set-aside classes, as {class: enums}, its word entries, as
    {enum: {arm: line}}, and its field lines, as {field: line}; `None`
    without the section."""
    lines = text.splitlines()
    starts = [i for i, line in enumerate(lines) if line.strip() == VOCABULARY]
    if len(starts) != 1:
        return None
    bullet_lines: dict[str, list[str]] = {}
    words: dict[str, dict[str, str]] = {}
    fields: dict[str, str] = {}
    current_class, current_enum, in_fields = None, None, False
    for line in lines[starts[0] + 1 :]:
        if line.startswith("## ") or line.startswith("---"):
            break
        if line.startswith("### "):
            in_fields = line.strip() == FIELDS_HEADING
        if in_fields:
            if m := FIELD.match(line):
                fields[m.group(1)] = m.group(2)
            continue
        if m := CLASS.match(line):
            current_class, current_enum = m.group(1), None
            bullet_lines[current_class] = [line]
        elif m := WORD_ENTRY.match(line):
            current_class, current_enum = None, m.group(1)
            words[current_enum] = {}
        elif current_enum and (m := ARM.match(line)):
            words[current_enum][m.group(1)] = m.group(2)
        elif not line.strip() or line.startswith("#") or line.startswith("- "):
            current_class = None
        elif current_class:
            bullet_lines[current_class].append(line)
    # A class's reason may name types too; its members are the names after
    # the colon that ends the reason.
    classes = {
        name: set(re.findall(r"`([A-Z][A-Za-z0-9_]*)`", re.split(r":\s+(?=`)", " ".join(lines), maxsplit=1)[-1]))
        for name, lines in bullet_lines.items()
    }
    return classes, words, fields


def read_type_declarations():
    """Every struct and enum outside test code, by name; a name declared
    twice keeps its first, the vocabulary being read and not its doubles."""
    found = {}
    for path in sorted(SRC.rglob("*.rs")):
        for decl in type_declarations(path.relative_to(ROOT).as_posix(), path.read_text(encoding="utf-8")):
            found.setdefault(decl.name, decl)
    return found


def vocabulary_scope(declared, roots) -> list[str]:
    """The structs and enums `roots` hold, transitively, stopping at the
    game's state and the keys."""
    seen, todo = set(), sorted(roots)
    while todo:
        name = todo.pop()
        if name in seen or name in NOT_VOCABULARY or name not in declared:
            continue
        seen.add(name)
        todo.extend(sorted(declared[name].mentions() - seen))
    return sorted(seen)


def open_item_numbers() -> set[int]:
    """The main run's open items in `codebase-state.md`'s Deferred
    Migrations, as the board classes them (`check_state_of_play.py`): a
    register line cites a main item, and an item is closed by its verdict."""
    block, _ = dm_block()
    return {
        int(number)
        for run, number, body, _ in items_by_run(block)
        if run == "main" and number.isdigit() and classify_item(body) != "closed"
    }


def check_vocabulary() -> int:
    text = AUDIT.read_text(encoding="utf-8")
    register, vocabulary = read_register(text), read_vocabulary(text)
    if register is None or vocabulary is None:
        print(f"\nvocabulary: no `{VOCABULARY}` section in {AUDIT.relative_to(ROOT)}.")
        return 2
    stops, _ = register
    classes, words, field_lines = vocabulary
    declared = read_type_declarations()
    walked = set(vocabulary_scope(declared, stops))
    reached = {name for name in walked if declared[name].kind == "enum"}
    classed = dict(classes, words=set(words))
    failures = []
    for name in sorted(reached):
        homes = [c for c, members in classed.items() if name in members]
        if len(homes) != 1:
            failures.append(f"  `{name}` is reached and in {len(homes)} classes ({', '.join(homes) or 'none'})")
    for c, members in sorted(classed.items()):
        for name in sorted(members - reached):
            failures.append(f"  `{name}` is in `{c}` and no longer reached")
    still_open = open_item_numbers()
    lines_read = 0
    for name, lines in sorted(words.items()):
        if name not in declared:
            continue
        arms = [arm for arm, _, _ in declared[name].members]
        for arm in arms:
            if arm not in lines:
                failures.append(f"  `{name}::{arm}` has no line")
        for arm, line in lines.items():
            lines_read += 1
            if arm not in arms:
                failures.append(f"  `{name}::{arm}` has a line and is no arm")
            elif not any(line.startswith(v) for v in VERDICTS):
                failures.append(f"  `{name}::{arm}` has no verdict")
            if line.startswith("**folds**"):
                cited = [n for group in FOLD_ITEMS.findall(line) for n in group.split(", ")]
                for number in cited or ["none"]:
                    if number == "none" or int(number) not in still_open:
                        failures.append(f"  `{name}::{arm}` folds through item {number}, which is not an open main item")
    # A field elsewhere that holds a word type: a role naming a word through
    # a field, not an arm. `holders` names a tuple payload `..0`; the
    # register writes `.0`.
    held = {}
    for path in sorted(SRC.rglob("*.rs")):
        for holder in holders(path.relative_to(ROOT).as_posix(), path.read_text(encoding="utf-8")):
            if holder.owner in walked and holder.owner not in WORD_TYPES and holder.owner not in words:
                if holder.types() & WORD_TYPES:
                    held[holder.name().replace("..", ".")] = holder
    for name in sorted(set(held) - set(field_lines)):
        failures.append(f"  `{name}` holds {held[name].ty} and has no line")
    for name, line in sorted(field_lines.items()):
        if name not in held:
            failures.append(f"  `{name}` has a line and holds no word type")
        elif not any(line.startswith(v) for v in VERDICTS):
            failures.append(f"  `{name}` has no verdict")
        elif line.startswith("**folds**"):
            cited = [n for group in FOLD_ITEMS.findall(line) for n in group.split(", ")]
            for number in cited or ["none"]:
                if number == "none" or int(number) not in still_open:
                    failures.append(f"  `{name}` folds through item {number}, which is not an open main item")
    arms_total = sum(len(declared[n].members) for n in words if n in declared)
    print(
        f"\nvocabulary: {len(reached)} enums reached, {len(words)} word entries, {lines_read} lines for "
        f"{arms_total} arms, {len(field_lines)} for {len(held)} fields"
    )
    if not failures:
        print("every reached enum is classed, and every word arm and word-holding field has its line.")
        return 0
    print(f"\n{len(failures)} problem(s) with {VOCABULARY_ANCHOR}:\n")
    print("\n".join(failures))
    print(
        "\nClass a new enum, or give a new arm its line: does it spell a word another arm"
        "\nalready spells? Hold the shared word type, keep it with its reason, or file its fold."
    )
    return 1


def main() -> int:
    first = check_game_state_fields()
    second = check_register()
    third = check_vocabulary()
    return max(first, second, third)


if __name__ == "__main__":
    sys.exit(main())
