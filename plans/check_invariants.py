#!/usr/bin/env python3
"""
check_invariants - three of `CLAUDE.md`'s invariants, read off the source.

`CLAUDE.md` states the layer-system read, the chokepoint and determinism at the
decision boundary as rules, and until A6q only review held them. The read of
the tree on 2026-10-13 found each broken again within weeks of being written.
Each has a shape a grep can see, and this reads those shapes
(`engineering-practices.md` §11).

    python plans/check_invariants.py      # exit 1 on an untagged site

**printed.** A read of a printed characteristic, `card_data.` followed by
`types`, `subtypes`, `supertypes`, `colors`, `keyword_flags` or `abilities`,
directly or through a local bound to a whole `card_data`. Allowed in `PRINTED`'s
files: the route itself and the cards that write what is printed. Anywhere else
the effective value comes through `oracle::characteristics`.

**unordered.** Iteration over a hash-keyed field of `GameState`: `.iter()`,
`.keys()`, `.values()`, their `_mut` and `into_` forms, `.drain()`, `.retain()`,
or a `for` over the field. The fields are read off the declaration, so a new
`IdMap` is covered the day it is added. Tests are read too: a test that takes
the first of two permanents out of a map passes by the hasher's seed.

**write.** A write to a field whose change is an event (a zone's membership,
life, tapped, damage, attachment, combat, the game's result), or a call to a
store method that makes one, outside the files `PERFORMERS` lists for it.
A write is an assignment, a `&mut` borrow, or a method that changes what the
field holds (`insert`, `push`, `retain`, `take`, ...).

**Tags.** A site the rule does not mean carries a tag, in the comment block
directly above its statement or trailing on its own line. Above, the tag
covers the whole statement: a `let`, an `if` with its `else`, a match arm, a
`fn`. The tag word starts its comment line; one quoted mid-sentence is a
mention. Each check reads its own words, in `TAGS`.

**`#[cfg(test)]` items and `test_support.rs` are not read by printed and
write**: a test builds and inspects a board directly, which is what it is for.

**Known violations** are listed in `KNOWN` by codebase-state item, here rather
than in the source because a source comment carries no item number. An entry
that matches nothing fails too: its item is closed, or its site moved.

**What it does not see.** A `&CardData` handed to a function, which reads it
under a name this cannot follow; iteration through a local bound to the map;
a write through a `&mut PermanentState` returned by a helper and written under
another name; anything in `devgui/`, which reaches the engine through its
snapshot by id. Registry membership (item 233's decision sites, the
static-ability existence rule) is not a shape at all.
"""

import re
import sys
from pathlib import Path

from rust_source import code_only, test_items

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

ROOT = Path(__file__).resolve().parent.parent
CRATE = ROOT / "mtgsim"
SRC = CRATE / "src"
GAME_STATE = SRC / "state" / "game_state.rs"

#: The tag words each check accepts.
TAGS = {
    "printed": ("PRE-LAYER ZONE", "AS PRINTED", "NO ROW REACHES"),
    "unordered": ("ORDER-FREE",),
    "write": ("CAST-ROLLBACK", "NOT REPLACEABLE"),
}

# --- printed -----------------------------------------------------------------

PRINTED_FIELD = r"(types|subtypes|supertypes|colors|keyword_flags|abilities)"
DIRECT_READ = re.compile(r"\bcard_data\s*\.\s*" + PRINTED_FIELD + r"\b")
#: A local bound to a whole card, `let card = &obj.card_data;` or `let Some(card)
#: = ...map(|o| Arc::clone(&o.card_data)) else`, whose `card.types` is the same read.
CARD_LOCAL = re.compile(r"\blet\s+(?:mut\s+)?(?:Some\(\s*)?(\w+)\s*\)?\s*(?::[^=;]*)?=(?!=)([^;]*?)(?:;|\belse\b)")
WHOLE_CARD = re.compile(r"\bcard_data\b(?!\s*\.\s*(?!clone\s*\()\w)")

#: Files that read what is printed by design, relative to `mtgsim/src/`.
PRINTED = {
    "oracle/": "the route: `oracle::characteristics` and the queries asked of a card in hand",
    "cards/": "card definitions, which write what is printed",
}

# --- unordered ---------------------------------------------------------------

#: A field whose walk order is no rule's: a hash-keyed one, whose order the
#: hasher's seed sets, or an `ObjectTable`, which walks in id order, the same
#: in every process and still not the CR's (`clone-architecture.md` decision 7).
HASH_TYPE = re.compile(r"\b(IdMap|IdSet|HashMap|HashSet|ObjectTable)\b")
ITERATING_METHOD = r"(iter|iter_mut|keys|values|values_mut|into_iter|into_keys|into_values|drain|retain|extract_if)"

#: Another type's field that shares a name with a field read here, by the name
#: that precedes it.
NOT_GAME_STATE = {
    "objects": ("referred",),  # `TriggerBinding.referred.objects`, a `Vec`
    "tapped": ("reserved",),  # `mana_supply`'s reservation, not a permanent
}

# --- write -------------------------------------------------------------------

#: Fields of `GameState`, `PlayerState`, `PermanentState` and `GameObject`
#: whose change is an event, or part of one.
EVENT_FIELDS = (
    "battlefield", "objects", "stack", "stack_entries", "exile", "command",
    "library", "hand", "graveyard", "zone",
    "life_total", "result", "player_lost",
    "tapped", "damage_marked", "damaged_by_deathtouch", "commander_damage_taken",
    "attached_to", "attached_by", "attacking", "blocking", "blocked_by",
    "phased_out", "face_down", "flipped",
)
#: The store's methods that write those fields for a caller.
STORE_WRITERS = (
    "move_object", "attach", "detach", "add_counters", "remove_counters",
    "remove_from_combat", "clear_combat_state", "insert_battlefield_entity",
    "add_object", "remove_object", "set_stack_entry", "take_stack_entry",
    "shuffle_library", "settle_game_result", "end_game",
)
MUTATING_METHOD = (
    r"(insert|remove|push|pop|retain|retain_mut|clear|extend|drain|truncate|swap_remove|append|"
    r"shuffle|sort\w*|split_off|dedup\w*|reverse|rotate_\w+|entry|take|replace|get_or_insert\w*)"
)

ALL = "every field"

#: Where each write is performed, relative to `mtgsim/src/`: a name is a field
#: from `EVENT_FIELDS` or a method from `STORE_WRITERS`.
PERFORMERS = {
    "engine/actions.rs": (ALL, "`perform_action`'s arms"),
    "engine/zones.rs": (ALL, "`move_object`, the one mover (CR 400.7)"),
    "state/game_state.rs": (ALL, "the store's writers, each called by a performer"),
    "state/battlefield.rs": (ALL, "`PermanentState`'s own methods"),
    "state/player.rs": (ALL, "`PlayerState`'s own methods"),
    "test_support.rs": (ALL, "the boards tests build"),
    "scenario/build.rs": (ALL, "a scenario's board, written before anything is played (`setup-architecture.md`)"),
    "state/game.rs": ({"shuffle_library"}, "CR 103.3's shuffle, in the start-of-game procedure, which calls the performers"),
    "engine/combat/steps.rs": (
        {"attacking", "blocking", "blocked_by"},
        "the declarations (CR 508.1, 509.1), which no effect replaces; announced as `AttackersDeclared`, `BlockersDeclared`",
    ),
    "engine/put_on_stack.rs": (
        {"stack", "add_object", "set_stack_entry", "take_stack_entry", "remove_object"},
        "an activated ability created on the stack (CR 602.2a), and taken back when the activation rewinds (CR 732.1)",
    ),
    "engine/triggers/placement.rs": (
        {"stack", "add_object", "set_stack_entry", "remove_object"},
        "a triggered ability put on the stack (CR 603.3)",
    ),
    "engine/stack.rs": (
        {"stack", "take_stack_entry", "remove_object"},
        "an ability leaving the stack as it resolves or is countered (CR 608.2n, 701.6b)",
    ),
    "engine/resolve.rs": (
        {"stack", "take_stack_entry", "remove_object"},
        "an ability leaving the stack as it resolves (CR 608.2n)",
    ),
}

#: Known violations, by `codebase-state.md` item: (check, file, name) -> (item, owner).
KNOWN = {}


# --- reading -----------------------------------------------------------------


class Source:
    """One file: its lines, its code with comments and strings blanked, and
    which lines a tag covers for each check."""

    def __init__(self, path: Path):
        self.path = path
        self.text = path.read_text(encoding="utf-8")
        self.lines = self.text.splitlines()
        self.code = code_only(self.text)
        self.code_lines = self.code.split("\n")
        self.starts = [0]
        for line in self.code_lines:
            self.starts.append(self.starts[-1] + len(line) + 1)
        self.tests = test_items(self.code)
        self.covered = {check: self.tagged_lines(words) for check, words in TAGS.items()}

    def line_of(self, offset: int) -> int:
        lo, hi = 0, len(self.starts) - 1
        while lo < hi:
            mid = (lo + hi + 1) // 2
            if self.starts[mid] <= offset:
                lo = mid
            else:
                hi = mid - 1
        return lo + 1

    def in_test(self, offset: int) -> bool:
        return any(a <= offset < b for a, b in self.tests)

    def tagged_lines(self, words) -> set[int]:
        block = re.compile(r"^\s*//[/!]?\s*(?:" + "|".join(map(re.escape, words)) + "):")
        trailing = re.compile(r"//[/!]?\s*(?:" + "|".join(map(re.escape, words)) + "):")
        covered = set()
        for i, raw in enumerate(self.lines):
            if block.match(raw):
                start = self.statement_below(i)
                if start is not None:
                    covered.update(range(start + 1, self.statement_end(start) + 1))
            elif self.code_lines[i].strip() and (m := trailing.search(raw)) and not self.code_lines[i][m.start():].strip():
                covered.add(i + 1)
        return covered

    def statement_below(self, i: int):
        """The 0-based index of the first code line below the comment block
        holding line `i`, or `None` when a blank line ends the block first."""
        for j in range(i + 1, len(self.lines)):
            raw = self.lines[j].strip()
            if not raw:
                return None
            if raw.startswith("//") or raw.startswith("#[") or raw.startswith("#!["):
                continue
            return j
        return None

    def statement_end(self, start: int) -> int:
        """The 1-based line where the statement starting on 0-based line
        `start` ends: a `;` or `,` at its own depth, a block closing at its own
        depth with no `else`, `.`, `?` after it, or its enclosing block closing."""
        code, depth = self.code, 0
        i = self.starts[start]
        while i < len(code):
            c = code[i]
            if c in "([{":
                depth += 1
            elif c in ")]}":
                depth -= 1
                if depth < 0:
                    return self.line_of(i)
                if depth == 0 and c == "}":
                    k = i + 1
                    while k < len(code) and code[k].isspace():
                        k += 1
                    if not (code.startswith("else", k) or code[k : k + 1] in (".", "?", ";", ",", ")")):
                        return self.line_of(i)
            elif c in ";," and depth == 0:
                return self.line_of(i)
            i += 1
        return len(self.lines)

    def block_end(self, offset: int) -> int:
        """The offset where the block enclosing `offset` closes."""
        depth = 0
        for i in range(offset, len(self.code)):
            depth += {"{": 1, "}": -1}.get(self.code[i], 0)
            if depth < 0:
                return i
        return len(self.code)


def rust_files(root: Path):
    return sorted(p for p in root.rglob("*.rs") if "target" not in p.relative_to(CRATE).parts)


def relative(path: Path) -> str:
    return path.relative_to(SRC).as_posix() if path.is_relative_to(SRC) else path.relative_to(CRATE).as_posix()


def hash_fields() -> list[str]:
    """`GameState`'s fields whose type is hash-keyed, off its declaration."""
    text = GAME_STATE.read_text(encoding="utf-8")
    body = text[text.index("pub struct GameState {") :]
    body = body[: body.index("\n}")]
    field = re.compile(r"^    (?:pub(?:\([^)]*\))?\s+)?([a-z_][a-z0-9_]*)\s*:\s*(.+)$", re.M)
    return [m.group(1) for m in field.finditer(body) if HASH_TYPE.search(m.group(2))]


# --- the checks --------------------------------------------------------------


def printed_reads(src: Source):
    name = relative(src.path)
    if any(name.startswith(prefix) for prefix in PRINTED):
        return
    for m in DIRECT_READ.finditer(src.code):
        if not src.in_test(m.start()):
            yield src.line_of(m.start()), f"card_data.{m.group(1)}"
    for m in CARD_LOCAL.finditer(src.code):
        # `card_data` itself is read above; a tag on the binding covers every read through it.
        if m.group(1) == "card_data" or not WHOLE_CARD.search(m.group(2)):
            continue
        if src.in_test(m.start()) or src.line_of(m.start()) in src.covered["printed"]:
            continue
        alias = re.compile(r"(?<![\w.])" + m.group(1) + r"\s*\.\s*" + PRINTED_FIELD + r"\b")
        for read in alias.finditer(src.code, m.end(), src.block_end(m.end())):
            yield src.line_of(read.start()), f"{m.group(1)}.{read.group(1)}, `{m.group(1)}` being a `card_data`"


def receiver(code: str, field_start: int) -> str:
    """The name just before the `.` that precedes the field at `field_start`:
    `referred` in `binding.referred.objects`."""
    i = code.rfind(".", 0, field_start) - 1
    while i >= 0 and code[i].isspace():
        i -= 1
    end = i + 1
    while i >= 0 and (code[i].isalnum() or code[i] == "_"):
        i -= 1
    return code[i + 1 : end]


def unordered_iterations(src: Source, fields: list[str]):
    alternatives = "|".join(fields)
    chained = re.compile(r"\.\s*(" + alternatives + r")\s*\.\s*" + ITERATING_METHOD + r"\s*\(")
    looped = re.compile(r"\bfor\b[^{;]*?\bin\s+&?(?:mut\s+)?[\w.()\[\]&]*?\.\s*(" + alternatives + r")\s*\{")
    for pattern in (chained, looped):
        for m in pattern.finditer(src.code):
            if receiver(src.code, m.start(1)) in NOT_GAME_STATE.get(m.group(1), ()):
                continue
            how = f"{m.group(1)}.{m.group(2)}" if pattern is chained else f"for over {m.group(1)}"
            yield src.line_of(m.start(1)), how


FIELD = "(" + "|".join(EVENT_FIELDS) + ")"
ASSIGNED = re.compile(r"\.\s*" + FIELD + r"\b(?:\s*\[[^\]]*\])?\s*(?:[-+*/%|&^]|<<|>>)?=(?![=>])")
MUTATED = re.compile(r"\.\s*" + FIELD + r"\b(?:\s*\[[^\]]*\])?\s*\.\s*" + MUTATING_METHOD + r"\s*\(")
BORROWED = re.compile(r"&\s*mut\s+[\w.()\[\]&*]*?\.\s*" + FIELD + r"\b(?!\s*[.(\[])")
CALLED = re.compile(r"\.\s*(" + "|".join(STORE_WRITERS) + r")\s*\(")


def writes(src: Source):
    name = relative(src.path)
    allowed, _ = PERFORMERS.get(name, (set(), ""))
    seen = set()
    for pattern in (ASSIGNED, MUTATED, BORROWED, CALLED):
        for m in pattern.finditer(src.code):
            written = m.group(1)
            if receiver(src.code, m.start(1)) in NOT_GAME_STATE.get(written, ()):
                continue
            if allowed is ALL or written in allowed or src.in_test(m.start()):
                continue
            line = src.line_of(m.start(1))
            if (line, written) not in seen:
                seen.add((line, written))
                yield line, written


# --- the run -----------------------------------------------------------------


def main() -> int:
    fields = hash_fields()
    if len(fields) < 2:
        print("invariants: GameState's declaration did not parse into hash fields; the layout this gate reads has changed.")
        return 2
    sources = [Source(p) for p in rust_files(CRATE)]
    in_src = [s for s in sources if s.path.is_relative_to(SRC)]
    found = {
        "printed": [(s, line, what) for s in in_src for line, what in printed_reads(s)],
        "unordered": [(s, line, what) for s in sources for line, what in unordered_iterations(s, fields)],
        "write": [(s, line, what) for s in in_src for line, what in writes(s)],
    }

    failures, known_hit = {}, set()
    for check, hits in found.items():
        for src, line, what in hits:
            name = relative(src.path)
            if line in src.covered[check]:
                continue
            key = (check, name, what)
            if key in KNOWN:
                known_hit.add(key)
                continue
            failures.setdefault(check, []).append(f"{src.path.relative_to(ROOT).as_posix()}:{line}  {what}")
    stale = sorted(set(KNOWN) - known_hit)

    print(
        f"invariants: {len(in_src)} files under mtgsim/src, {len(sources) - len(in_src)} more read for iteration; "
        f"{len(fields)} hash fields on GameState; {len(known_hit)} of {len(KNOWN)} known violations seen"
    )
    for check, hits in found.items():
        tagged = sum(1 for s, line, _ in hits if line in s.covered[check])
        print(f"  {check}: {len(hits)} sites, {tagged} tagged ({', '.join(TAGS[check])})")
    if not failures and not stale:
        print("\nevery site is tagged, performed where PERFORMERS says, or a known violation.")
        return 0

    advice = {
        "printed": "Read the effective value through oracle::characteristics, or tag the statement\n"
        "`// PRE-LAYER ZONE:` (asked before the layers can answer for it), `// AS PRINTED:` (shown, never\n"
        "ruled on) or `// NO ROW REACHES:` (no continuous effect reaches it, so printed is effective).",
        "unordered": "Iterate through battlefield_ordered / battlefield_ids_ordered or sort by timestamp first,\n"
        "or tag the statement `// ORDER-FREE:` with why no order can be observed.",
        "write": "Propose it with execute_action / change_zone, or tag it `// NOT REPLACEABLE:` with the rule\n"
        "that leaves no replacement effect to watch for it (CR 614.1). A new performer is a row in PERFORMERS, which a review reads in the diff.",
    }
    for check, lines in failures.items():
        print(f"\n{check}: {len(lines)} untagged site(s):\n")
        for line in lines:
            print(f"  {line}")
        print("\n" + advice[check])
    for check, name, what in stale:
        item, why = KNOWN[(check, name, what)]
        print(f"\n  KNOWN lists item {item} ({why}) at {name}, {what}, and no site matches: take the entry off.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
