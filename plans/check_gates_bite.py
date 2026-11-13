#!/usr/bin/env python3
"""
check_gates_bite - a gate that stopped working passes; this is what notices.

Every `plans/check_*.py` gate reports success by finding nothing, so a gate
whose pattern has rotted reads exactly like a tree with nothing wrong. A regex
that no longer matches, a root that moved, a tag read too generously: each one
exits 0 and CI stays green over the rule it stopped enforcing. The family holds
every rule in the tree and had no test of its own.

This plants a violation of each rule and requires the right gate to fail.

    python plans/check_gates_bite.py      # exit 1 if a gate missed a plant

**Where it plants.** In throwaway worktrees at HEAD with the working copy's
`plans/*.py` copied over them, so a gate edit under test is the one run and the
working copy is never touched. It takes tracked files only, as CI's checkout
does, and works in a shallow clone.

**The control comes first.** Every gate must pass unplanted, or the run stops:
a plant failing against a gate that already fails proves nothing.

**What counts as caught.** The gate exits nonzero and its output holds the
plant's expected text, which is what shows it failed for the rule planted and
not for some other reason. The file's bytes go back before the next plant. A
plant that cannot apply, because its anchor is gone, is MISSED: a plant that
did not run must never read as a gate that worked.

**One plant per rule, not per gate.** The point is catching one rotted pattern
among several, so each gate gets a plant for every rule it enforces, its
`check_invariants.py` tags included (`engineering-practices.md#invariant-gates`),
and a run of near-identical patterns gets one for the family. A gate's exit-2
paths count where a file's edit can reach them: "the layout this gate reads has
changed" is the gate's defense against its own rot.

**A few plants edit a gate's own copy.** The two selftests and `KNOWN` read
nothing a tree could plant in, so the throwaway copy of the gate is the only
place to break them.

**Why several trees.** One after another the plants take about two minutes, and
`check_invariants.py` and `check_type_surface.py` cost nearly two seconds a run.
A tree holds one plant at a time, so a plant still sees a clean tree around it.

**What it does not prove.** That a gate refuses what it should, never that it
passes what it should: a pattern grown too eager shows up in the control, as a
failure on the real tree.
"""

import json
import os
import queue
import shutil
import subprocess
import sys
import tempfile
import time
from concurrent.futures import ThreadPoolExecutor
from enum import Enum, auto
from pathlib import Path
from typing import NamedTuple

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

ROOT = Path(__file__).resolve().parent.parent
#: Long paths break deletion on Windows, and this is the short one when it exists.
SHORT_PATHS = Path(r"C:\w")
#: Eight run the plants in about 30 s, against about two minutes for one.
TREES = min(8, os.cpu_count() or 1)
#: Several gates print through the console's encoding unless told otherwise.
ENV = {**os.environ, "PYTHONIOENCODING": "utf-8"}

CLAUDE_MD = "check_claude_md.py"
MODULES = "check_module_layout.py"
GLOSSARY = "check_glossary.py"
BOARD = "check_state_of_play.py --check"
RULINGS = "check_rulings.py --check"
LITERALS = "check_string_literals.py"
SURFACE = "check_type_surface.py"
CLIPPY = "check_clippy_allows.py"
MAP = "check_engine_map.py"
EGUI = "check_egui_only_draws.py"
INVARIANTS = "check_invariants.py"
PRACTICES = "check_practices_page.py"
GATES = [CLAUDE_MD, MODULES, GLOSSARY, BOARD, RULINGS, LITERALS, SURFACE, CLIPPY, MAP, EGUI, INVARIANTS, PRACTICES]


# --- a plant, as data --------------------------------------------------------


class Op(Enum):
    APPEND = auto()
    REPLACE = auto()
    CREATE = auto()
    DELETE = auto()


class Edit(NamedTuple):
    op: Op
    path: str
    anchor: str = ""  # what REPLACE finds, exactly once
    text: str = ""  # what APPEND, REPLACE and CREATE write


class Plant(NamedTuple):
    gate: str
    rule: str
    expect: str
    edits: tuple[Edit, ...]


def add(path, text):
    return Edit(Op.APPEND, path, text=text)


def swap(path, anchor, text):
    return Edit(Op.REPLACE, path, anchor, text)


def after(path, anchor, text):
    return Edit(Op.REPLACE, path, anchor, anchor + text)


def create(path, text):
    return Edit(Op.CREATE, path, text=text)


def delete(path):
    return Edit(Op.DELETE, path)


def plant(gate, rule, expect, *edits):
    return Plant(gate, rule, expect, edits)


# --- what the plants name ----------------------------------------------------

# Every planted name carries `zz`, which nothing in the tree does, so a gate
# cannot be satisfied by a real name that happens to match.

LIB = "mtgsim/src/lib.rs"  # the crate root: in no gate's allowed list, and it never moves
GUI_LIB = "devgui/src/lib.rs"
GUI_TOOLS = "devgui/tests/tools.rs"
TESTS = "mtgsim/tests/determinism_test.rs"
ENGINE_MOD = "mtgsim/src/engine/mod.rs"
ACTIONS = "mtgsim/src/engine/actions.rs"
STATE = "mtgsim/src/state/game_state.rs"
PLAYER = "mtgsim/src/state/player.rs"
BATTLEFIELD = "mtgsim/src/state/battlefield.rs"
REGISTRY = "mtgsim/src/cards/registry.rs"
GLOSSARY_DOC = "plans/glossary.md"
DEBT = "plans/codebase-state.md"
LEDGER = "plans/rulings-ledger.json"
AUDIT = "plans/cr-coverage-audit.md"
ENGINE_MAP = "plans/engine-map.md"
ELISIONS = "plans/elisions.md"
BACKLOG = "plans/backlog.md"
PAGE = "plans/engineering-practices.md"
#: This file is tracked and `check_practices_page.py` reads tracked files, so a
#: citation planted from here is assembled at run time, or it would fail there.
HASH = "#"

GAME_STATE_OPENER = "pub struct GameState {\n"
GAME_STATE_LITERAL = "\n        GameState {\n"
PLAYER_OPENER = "pub struct PlayerState {\n"
GAME_ACTION_OPENER = "pub enum GameAction {\n"
ARM_SITE = "    ) -> Result<(), String> {\n        match action {\n"
MARKED_DAMAGE = "#### Marked damage\n"
REGISTER_HEADING = "## 4c. Concepts and their owners\n"
DEBT_HEADING = "\n## Deferred Migrations\n"

READ = "fn zz_bite(obj: &GameObject) -> usize { obj.card_data.types.len() }\n"
WALK = "fn zz_bite(game: &GameState) -> usize { game.objects.keys().count() }\n"
WRITE = "fn zz_bite(player: &mut PlayerState) { player.life_total = 0; }\n"
SCAR = 'const ZZ_BITE: &str = "a message that lost          its continuation";\n'
TWO_READS = """fn zz_bite(obj: &GameObject) -> usize {{
    {tag_above}let shown = obj.card_data.types.len();{tag_beside}
    shown + obj.card_data.colors.len()
}}
"""
ALLOWED = "clippy::too_many_arguments"
WINDOW_CRATE_SITE = "  (names a window crate)"

CARD = "Zz Bite Card"
REGISTERED = add(REGISTRY, f'fn zz_bite() {{ registry.register("{CARD}", zz_bite); }}\n')


def ledger_card(disposition=None, **stamps):
    """A card of one ruling, written into the ledger whole, so a rulings plant
    leans on no real card and no ruling a later phase may answer."""
    ruling = {"n": 1, "published_at": "2020-01-01", "comment": "planted"}
    if disposition:
        ruling["disposition"] = disposition
    card = {"first_seen": "2099-01-01", **stamps, "rulings": [ruling]}
    return after(LEDGER, '"cards": {\n', f"    {json.dumps(CARD)}: {json.dumps(card)},\n")


def annotated(tag):
    return f"// {tag}\n#[test]\nfn zz_bite() {{}}\n"


RULING_1 = annotated(f"RULING: {CARD} #1")
DEVIATION_1 = annotated(f"RULING-DEVIATION: {CARD} #1 (zz-no-row)")


def debt_item(rest):
    """An open item, closed by a heading: an item's text runs to the next one,
    and the section's intro below it says `**Slotted:**` as prose."""
    return f"\n9991. **Planted.** **Reachability (2026-11-04):** unreachable — planted. {rest}\n\n### Zz planted\n"


def map_line(text):
    return add(ENGINE_MAP, f"\n{text}\n")


def glossary_entry(text):
    return add(GLOSSARY_DOC, f"\n\n{text}\n")


def state_field(field, ty):
    return after(PLAYER, PLAYER_OPENER, f"    pub {field}: {ty},\n")


ROADMAP = "plans/roadmap-v2.md"
SIZING_END = "<!-- v0.5 sizing: end -->"
FORECAST_END = "<!-- v0.5 forecast: end -->"
#: The forecast's first row, a step that landed before the first forecast, so
#: no later forecast rewrites it.
FIRST_FORECAST = "| 1 | 1 |\n"
BOARD_SCRIPT = "plans/check_state_of_play.py"
RECORDS = "plans/references/card-census/results/gap-census-records.jsonl"


def census_record(needs):
    """A gap census record of a card with no board and no game, appended whole."""
    record = {"name": CARD, "needs": needs, "reshaped": [], "loops": False, "note": "", "board": "no board", "reached": 0,
              "errors": [], "panics": [], "checks_failed": [], "survived": [], "paragraphs": 1, "keyword_paragraphs": 0,
              "uncovered": [], "composition": [], "stand_ins": 0}
    return add(RECORDS, json.dumps(record) + "\n")
ZZ_STRUCT = "pub struct ZzBite {\n    pub at: Timestamp,\n}\n"
FLOOR_ROW = "| 9. Zz bite | 1 | 2026-11-04 | planted |\n"


# --- the plants --------------------------------------------------------------

PLANTS = [
    # check_claude_md.py
    plant(CLAUDE_MD, "CLAUDE.md is over its line budget", "OVER BUDGET", add("CLAUDE.md", "planted\n" * 250)),
    plant(CLAUDE_MD, "CLAUDE.md is gone", "not found:", delete("CLAUDE.md")),
    # check_module_layout.py
    plant(MODULES, "a mod.rs defines a fn", "pub fn zz_bite", add(ENGINE_MOD, "pub fn zz_bite() {}\n")),
    plant(MODULES, "a restricted visibility hides no item", "pub(crate) const ZZ_BITE",
          add(ENGINE_MOD, "pub(crate) const ZZ_BITE: u32 = 0;\n")),
    plant(MODULES, "a qualifier hides no item", "unsafe impl Send", add(ENGINE_MOD, "unsafe impl Send for ZzBite {}\n")),
    plant(MODULES, "an item below a macro_rules! body still counts", "struct ZzBite",
          add(ENGINE_MOD, "macro_rules! zz_bite { () => { struct Inside; }; }\nstruct ZzBite;\n")),
    plant(MODULES, "the dev GUI's crate is read too", "struct ZzBite", create("devgui/src/mod.rs", "struct ZzBite;\n")),
    # check_glossary.py
    plant(GLOSSARY, "a defined term appears nowhere in the crate", '"zz bite term" is defined but appears nowhere',
          glossary_entry("**zz bite term** — planted.")),
    plant(GLOSSARY, "one word has two entries", '"walk" is defined twice', glossary_entry("**walk** — planted.")),
    plant(GLOSSARY, "an entry's file anchor is gone", "no such file under mtgsim/src - `zz_bite/nowhere.rs`",
          glossary_entry("**zz bite path** — see `zz_bite/nowhere.rs`.")),
    plant(GLOSSARY, "an entry's code anchor is gone", "`ZzBite::Nowhere` no longer resolves",
          glossary_entry("**zz bite name** — see `ZzBite::Nowhere`.")),
    plant(GLOSSARY, "a watched word has no entry", '"sweep" is on the watch-list and is not defined',
          swap(GLOSSARY_DOC, "**sweep** — an iteration", "**swept** — an iteration")),
    plant(GLOSSARY, "a polysemous word is missing a sense", '"census" owes 2 numbered senses',
          swap(GLOSSARY_DOC, "**census** — **(1)** a *card* census", "**census** — a *card* census")),
    plant(GLOSSARY, "a polysemous word has no entry", '"plan" is listed as polysemous and is not defined',
          swap(GLOSSARY_DOC, "**plan** — two, and the older", "**planned** — two, and the older")),
    plant(GLOSSARY, "British spelling in the crate", f"British spelling: {LIB}", add(LIB, "// behaviour\n")),
    plant(GLOSSARY, "British spelling in the dev GUI", f"British spelling: {GUI_LIB}", add(GUI_LIB, "// colour\n")),
    plant(GLOSSARY, "the glossary is gone", "not found:", delete(GLOSSARY_DOC)),
    # check_state_of_play.py --check
    plant(BOARD, "a count moved and the board did not", "is stale", add(LIB, "#[test]\nfn zz_bite() {}\n")),
    plant(BOARD, "the board is gone", "state-of-play.md does not exist", delete("plans/state-of-play.md")),
    plant(BOARD, "a floor's limit moved and the board did not", "is stale",
          swap(PAGE, "| 3. Kilobytes per clone | ≤ 128 |", "| 3. Kilobytes per clone | ≤ 129 |")),
    plant(BOARD, "the floors' limits and readings name different floors", "name different floors",
          swap("plans/fuzz-record.md", "<!-- floor readings: end -->", FLOOR_ROW + "<!-- floor readings: end -->")),
    plant(BOARD, "the critical path says a landed phase is next", "calls RE-1 'next'",
          after("CLAUDE.md", "## Critical path to v1\n", "\nRE-1 is next\n")),
    plant(BOARD, "the critical path bolds a landed phase as next", "calls RE-1 'next'",
          after("CLAUDE.md", "## Critical path to v1\n", "\n**RE-1 next**\n")),
    plant(BOARD, "an open item has no Slotted line", "item 9991 has no home — no **Slotted:** line",
          after(DEBT, DEBT_HEADING, debt_item(""))),
    plant(BOARD, "an open item is slotted only to landed phases", "slotted only to RE-1, which landed without it",
          after(DEBT, DEBT_HEADING, debt_item("**Slotted:** RE-1."))),
    plant(BOARD, "an open item is slotted to nothing the route has", "names no phase, row or card the route has",
          after(DEBT, DEBT_HEADING, debt_item("**Slotted:** with the first registered card."))),
    plant(BOARD, "one number twice in a run", "item 9991 is numbered twice",
          after(DEBT, DEBT_HEADING, debt_item("**Slotted:** Phase 10.") + debt_item("**Slotted:** Phase 10."))),
    plant(BOARD, "a sized row the route has no name for", "is no phase or row the route names",
          swap(ROADMAP, SIZING_END, "| 18 | `zz-no-row` | | 1 | | | |\n" + SIZING_END)),
    plant(BOARD, "a date in a sized row's PR cell", "wants a step and a PR count or range",
          swap(ROADMAP, "| 18 | `mana-payment-solver` | | 1–2 |", "| 18 | `mana-payment-solver` | | 2026-10 |")),
    plant(BOARD, "a row of the critical path has no size", "the critical path names zz-no-row",
          after("CLAUDE.md", "**After 6, in order**", " `zz-no-row`;")),
    plant(BOARD, "a row cited in another row's cell is still unsized", "the critical path names game-formats",
          swap(ROADMAP, "| 9 | `game-formats` |", "| 9 | `token-copies` |")),
    plant(BOARD, "a landed section keeps its body in the live doc", "ZZ-9 is landed but keeps",
          add("plans/layers-architecture.md", "\n### ZZ-9 — planted — ✅ landed 2026-11-04\n" + "planted\n" * 50)),
    plant(BOARD, "the parser's selftest fails a rotted item pattern", "selftest:",
          swap(BOARD_SCRIPT, r'ITEM_RE = re.compile(r"^(\d+[a-z]?)\.\s", re.M)',
               r'ITEM_RE = re.compile(r"^(\d+)\.\s", re.M)')),
    plant(BOARD, "a landed row is still sized as a range", "landed, and its PR cell is still a range",
          swap(ROADMAP, "| 3 | `card-layer-cleanup` | ✅ 2026-11-01 | 5 |", "| 3 | `card-layer-cleanup` | ✅ 2026-11-01 | 4–5 |")),
    plant(BOARD, "a landed row's State cell the board cannot read", "has a State cell the board cannot read",
          swap(ROADMAP, "| 3 | `card-layer-cleanup` | ✅ 2026-11-01 | 5 |", "| 3 | `card-layer-cleanup` | ✅ 2026-11-01, PR #301 | 5 |")),
    plant(BOARD, "a step with rows has no forecast", "step 1 has rows in the Sizing table and no forecast",
          swap(ROADMAP, FIRST_FORECAST, "")),
    plant(BOARD, "the forecast holds a step with no rows", "the forecast holds step 99",
          swap(ROADMAP, FORECAST_END, "| 99 | 1 |\n" + FORECAST_END)),
    plant(BOARD, "a forecast that is no count or range", "the forecast wants a step and a PR count or range",
          swap(ROADMAP, FIRST_FORECAST, "| 1 | soon |\n")),
    plant(BOARD, "a forecast that runs down", "the forecast's step 1 runs 2 down to 1",
          swap(ROADMAP, FIRST_FORECAST, "| 1 | 2–1 |\n")),
    plant(BOARD, "a step forecast twice", "the forecast holds step 1 twice",
          swap(ROADMAP, FIRST_FORECAST, FIRST_FORECAST + FIRST_FORECAST)),
    plant(BOARD, "the selftest fails a forecast no step can pass", "selftest: steps",
          swap(BOARD_SCRIPT, "return step.taken + step.left[0] > step.forecast[1]", "return False")),
    plant(BOARD, "a census record needs a row the route does not have", "which names no row of the Sizing table and no gap",
          census_record(["zz-no-row"])),
    plant(BOARD, "a deck card is claimed writable with no proof", f"claim {CARD} writable and not proved",
          add("mtgsim/decks/tdc-abzan-armor.txt", f"\n1 {CARD}\n"), census_record([])),
    plant(BOARD, "the census's records are gone", "gap-census-records.jsonl is gone", delete(RECORDS)),
    plant(BOARD, "a backlog entry has no Owner line", "§2.999 Zz planted has no Owner line",
          add(BACKLOG, "\n### 2.999 Zz planted\n\nplanted\n")),
    plant(BOARD, "the selftest fails cards to write read off the wrong status", "selftest: cards to write",
          swap(BOARD_SCRIPT, 'CardsToWrite(counted["writable"], counted["ready"],', 'CardsToWrite(counted["ready"], counted["writable"],')),
    # check_rulings.py --check
    plant(RULINGS, "a registered card the ledger has never seen", f"{CARD}: registered, but the ledger has never seen it",
          REGISTERED),
    plant(RULINGS, "a read card's ruling has no answer", "no disposition says why (read)",
          REGISTERED, ledger_card(read="2099-01-01")),
    plant(RULINGS, "a later card's ruling has no answer", "no disposition says why (registered after the ledger)",
          REGISTERED, ledger_card()),
    plant(RULINGS, "a disposition and a test are two answers", "A ruling gets one answer, not two",
          ledger_card({"kind": "no-board", "why": "planted"}), add(LIB, RULING_1)),
    plant(RULINGS, "an annotation above no test answers nothing", "sits above no test",
          ledger_card(), add(LIB, f"// RULING: {CARD} #1\nfn zz_bite() {{}}\n")),
    plant(RULINGS, "a deviation above no test answers nothing", "sits above no test",
          ledger_card(), add(LIB, f"// RULING-DEVIATION: {CARD} #1 (zz-no-row)\nconst ZZ_BITE: u8 = 0;\n")),
    plant(RULINGS, "a test and a deviation are two answers", "and a deviation",
          ledger_card(), add(LIB, RULING_1 + DEVIATION_1)),
    plant(RULINGS, "a deviation names no register row", "under 'zz-no-row', which is not a row",
          ledger_card(), add(LIB, DEVIATION_1)),
    plant(RULINGS, "a test names a ruling the ledger does not hold", f"names {CARD} #2, which the ledger does not hold",
          ledger_card(), add(LIB, annotated(f"RULING: {CARD} #2"))),
    plant(RULINGS, "a test names a card the ledger has not fetched", f"names {CARD}, a card the ledger has not fetched",
          add(LIB, RULING_1)),
    plant(RULINGS, "an unknown disposition kind", "unknown disposition 'zz-kind'", ledger_card({"kind": "zz-kind"})),
    plant(RULINGS, "a disposition leaves out what it owes", "must name its 'needs'",
          ledger_card({"kind": "no-registered-card"})),
    plant(RULINGS, "an owner names no doc", "names no doc",
          ledger_card({"kind": "not-expressible", "facility": "planted", "owner": "zz nowhere"})),
    plant(RULINGS, "an owner names a doc not in the tree", "owner names zz-bite.md, which is not in the tree",
          ledger_card({"kind": "not-expressible", "facility": "planted", "owner": "zz-bite.md"})),
    plant(RULINGS, "a defect names an item nothing carries", "item 999999 is not one codebase-state.md carries",
          ledger_card({"kind": "defect", "item": 999999})),
    plant(RULINGS, "the ledger is gone", "no ledger at", delete(LEDGER)),
    plant(RULINGS, "the gate's selftest fails a rotted verify", "selftest: the gate refused",
          swap("plans/check_rulings.py", "elif not linked and scoped:", "elif False:")),
    # check_string_literals.py
    plant(LITERALS, "a scar in the crate's source", "lost          its", add(LIB, SCAR)),
    plant(LITERALS, "a scar in the crate's tests", "lost          its", add(TESTS, SCAR)),
    plant(LITERALS, "a scar in the dev GUI", "lost          its", add(GUI_TOOLS, SCAR)),
    plant(LITERALS, "a run after a colon with no placeholder is no column", "Label:          value",
          add(LIB, 'const ZZ_BITE: &str = "Label:          value";\n')),
    plant(LITERALS, "a run before a placeholder with no colon is no column", "label          {}",
          add(LIB, 'const ZZ_BITE: &str = "label          {}";\n')),
    # check_type_surface.py
    plant(SURFACE, "a GameState field the audit does not name", "field(s) cr-coverage-audit.md does not name",
          after(STATE, GAME_STATE_OPENER, "    pub zz_bite: u32,\n"), after(STATE, GAME_STATE_LITERAL, "            zz_bite: 0,\n")),
    plant(SURFACE, "the declaration holds a field the literal does not", "declared, not initialized: zz_bite",
          after(STATE, GAME_STATE_OPENER, "    pub zz_bite: u32,\n")),
    plant(SURFACE, "the literal holds a field the declaration does not", "initialized, not declared: zz_bite",
          after(STATE, GAME_STATE_LITERAL, "            zz_bite: 0,\n")),
    plant(SURFACE, "a second GameState literal stops the parse", "could not find one",
          add(STATE, "        GameState {\n        }\n")),
    plant(SURFACE, "a new holder of a concept queried by type", "Timestamp: `PlayerState` holds it 1x and has no line",
          state_field("zz_bite", "Timestamp")),
    plant(SURFACE, "a new holder of a concept queried by word", "Marked damage: `PlayerState` holds it 1x and has no line",
          state_field("zz_damage_marked", "u32")),
    plant(SURFACE, "a holder's count differs from its line", "Timestamp: `PermanentState` holds it 2x and is listed 1x",
          after(BATTLEFIELD, "pub struct PermanentState {\n", "    pub zz_bite: Timestamp,\n")),
    plant(SURFACE, "a listed type no longer holds the concept", "Marked damage: `ZzBite` is listed and holds it no longer",
          after(AUDIT, MARKED_DAMAGE, "- `ZzBite` — planted.\n")),
    plant(SURFACE, "a holder reached through another type", "Timestamp: `ZzBite` holds it 1x and has no line",
          state_field("zz_bite", "ZzBite"), add(PLAYER, ZZ_STRUCT)),
    plant(SURFACE, "a holder reached through an alias", "Timestamp: `ZzBite` holds it 1x and has no line",
          state_field("zz_bite", "ZzAlias"), add(PLAYER, "type ZzAlias = ZzBite;\n" + ZZ_STRUCT)),
    plant(SURFACE, "GameAction's proposals are a root too", "Timestamp: `GameAction` holds it",
          after(ACTIONS, GAME_ACTION_OPENER, "    ZzBite { at: Timestamp },\n")),
    plant(SURFACE, "a concept with no query", "an entry with no **Query:** line",
          swap(AUDIT, MARKED_DAMAGE, "#### Zz bite concept\n\n" + MARKED_DAMAGE)),
    plant(SURFACE, "the register's section is gone", "no `## 4c. Concepts and their owners` section",
          swap(AUDIT, REGISTER_HEADING, "## 4c. Concepts\n")),
    plant(SURFACE, "a type the register reads is declared twice", "is declared twice",
          add(LIB, "pub struct PermanentState {\n    pub zz_bite: u32,\n}\n")),
    # check_clippy_allows.py
    plant(CLIPPY, "an outer attribute", ALLOWED, add(LIB, f"#[allow({ALLOWED})]\nfn zz_bite() {{}}\n")),
    plant(CLIPPY, "an inner attribute", ALLOWED, add(LIB, f"#![allow({ALLOWED})]\n")),
    plant(CLIPPY, "a lint second in a list", ALLOWED, add(LIB, f"#[allow(dead_code, {ALLOWED})]\nfn zz_bite() {{}}\n")),
    plant(CLIPPY, "an allow nested in cfg_attr", ALLOWED, add(LIB, f"#[cfg_attr(test, allow({ALLOWED}))]\nfn zz_bite() {{}}\n")),
    plant(CLIPPY, "an attribute split over lines", ALLOWED, add(LIB, f"#[allow(\n    {ALLOWED},\n)]\nfn zz_bite() {{}}\n")),
    plant(CLIPPY, "the crate's tests are read too", ALLOWED, add(TESTS, f"#[allow({ALLOWED})]\nfn zz_bite() {{}}\n")),
    plant(CLIPPY, "the table this gate reads is gone", "has no [lints.clippy] table",
          swap("mtgsim/Cargo.toml", "[lints.clippy]\n", "[lints.zz_bite]\n")),
    # check_engine_map.py
    plant(MAP, "a variant with no row in the arm table", "GameAction::ZzBite has no row in the map's arm table",
          after(ACTIONS, GAME_ACTION_OPENER, "    ZzBite,\n"), after(ACTIONS, ARM_SITE, "            GameAction::ZzBite => {}\n")),
    plant(MAP, "a row with no variant", "the map's arm table lists ZzBite",
          after(ENGINE_MAP, "<!-- arms:begin -->\n", "| `ZzBite` | planted | |\n")),
    plant(MAP, "a catch-all arm in perform_action", "perform_action has a catch-all arm",
          after(ACTIONS, ARM_SITE, "            _ => {}\n")),
    plant(MAP, "the enum and the arms disagree", "the two parses of actions.rs disagree",
          after(ACTIONS, GAME_ACTION_OPENER, "    ZzBite,\n")),
    plant(MAP, "the enum's opener moved", "the layout this gate reads has changed",
          swap(ACTIONS, GAME_ACTION_OPENER, "pub(crate) enum GameAction {\n")),
    plant(MAP, "a file that does not exist", "`engine/zz_bite.rs` does not exist", map_line("`engine/zz_bite.rs`")),
    plant(MAP, "a directory that does not exist", "`engine/zz_bite/` does not exist", map_line("`engine/zz_bite/`")),
    plant(MAP, "a glob that matches nothing", "`engine/zz_bite*.rs` does not exist", map_line("`engine/zz_bite*.rs`")),
    plant(MAP, "a link that goes nowhere", "the link zz-bite.md goes nowhere", map_line("[planted](zz-bite.md)")),
    plant(MAP, "a location in a missing file", "engine/zz_bite.rs:1 names a file that does not exist",
          map_line("`engine/zz_bite.rs:1`")),
    plant(MAP, "a location past the file's end", "engine/actions.rs:9999999 is past the file's end",
          map_line("`engine/actions.rs:9999999`")),
    plant(MAP, "a cited name not in its file", "`zz_bite` does not occur in engine/actions.rs's code",
          map_line("`zz_bite` `engine/actions.rs:1`")),
    plant(MAP, "a cited name only in a comment", "`zz_bite` does not occur in engine/actions.rs's code",
          add(ACTIONS, "// zz_bite\n"), map_line("`zz_bite` `engine/actions.rs:1`")),
    plant(MAP, "a cited line past the file's end", "engine/actions.rs:9999999 is past the file's",
          map_line("`perform_action` `engine/actions.rs:9999999`")),
    plant(MAP, "a name cited in a missing file", "`perform_action` is cited in engine/zz_bite.rs, which does not exist",
          map_line("`perform_action` `engine/zz_bite.rs:1`")),
    plant(MAP, "a code name nowhere in the crate", "`zz_bite_name` occurs nowhere in the crate's code",
          map_line("`zz_bite_name`")),
    plant(MAP, "a section another doc does not have", "layers-architecture.md has no §9999",
          map_line("`layers-architecture.md` §9999")),
    plant(MAP, "a section the page does not have", "engine-map.md has no §9999", map_line("§9999")),
    plant(MAP, "elisions.md is read the same way", "elisions.md:", add(ELISIONS, "\n`zz_bite_name`\n")),
    # check_egui_only_draws.py
    plant(EGUI, "the window's own crate outside a drawing file", "egui" + WINDOW_CRATE_SITE, add(GUI_LIB, "use egui::Ui;\n")),
    plant(EGUI, "a window crate of its family", "winit" + WINDOW_CRATE_SITE, add(GUI_LIB, "use winit::event::Event;\n")),
    plant(EGUI, "the dev GUI's tests are read too", "eframe" + WINDOW_CRATE_SITE, add(GUI_TOOLS, "use eframe::App;\n")),
    plant(EGUI, "a drawing file names the engine", "mtgsim  (a drawing file names the engine)",
          add("devgui/src/app.rs", "use mtgsim::state::GameState;\n")),
    plant(EGUI, "a drawing file that no longer exists", "DRAWING names devgui/tests/screenshots.rs, which does not exist",
          delete("devgui/tests/screenshots.rs")),
    # check_invariants.py: printed
    plant(INVARIANTS, "printed: a direct read", "card_data.types", add(LIB, READ)),
    plant(INVARIANTS, "printed: a read through a local bound to the card", "card.colors, `card` being a `card_data`",
          add(LIB, "fn zz_bite(obj: &GameObject) -> bool { let card = &obj.card_data; card.colors.is_empty() }\n")),
    plant(INVARIANTS, "printed: a read through a let-else binding", "zz.subtypes, `zz` being a `card_data`",
          add(LIB, "fn zz_bite(found: Option<&GameObject>) -> usize {\n"
                   "    let Some(zz) = found.map(|o| Arc::clone(&o.card_data)) else { return 0 };\n"
                   "    zz.subtypes.len()\n}\n")),
    # check_invariants.py: unordered
    plant(INVARIANTS, "unordered: a chained iteration", "objects.keys", add(LIB, WALK)),
    plant(INVARIANTS, "unordered: a for over the field", "for over trigger_sources",
          add(LIB, "fn zz_bite(game: &GameState) { for id in &game.trigger_sources { drop(id); } }\n")),
    plant(INVARIANTS, "unordered: the crate's tests are read too", f"{TESTS}:",
          add(TESTS, "fn zz_bite(game: &GameState) { let _ = game.objects.iter(); }\n")),
    plant(INVARIANTS, "unordered: a cfg(test) item is read too", "objects.values",
          add(LIB, "#[cfg(test)]\nfn zz_bite(game: &GameState) { let _ = game.objects.values(); }\n")),
    plant(INVARIANTS, "unordered: a new hash field is covered off the declaration", "zz_bite.iter",
          after(STATE, GAME_STATE_OPENER, "    pub zz_bite: IdMap<ObjectId, u32>,\n"),
          add(LIB, "fn zz_bite(game: &GameState) { let _ = game.zz_bite.iter(); }\n")),
    # check_invariants.py: write
    plant(INVARIANTS, "write: an assignment", "life_total", add(LIB, WRITE)),
    plant(INVARIANTS, "write: a compound assignment", "damage_marked",
          add(LIB, "fn zz_bite(permanent: &mut PermanentState) { permanent.damage_marked += 1; }\n")),
    plant(INVARIANTS, "write: a mutating method", "graveyard",
          add(LIB, "fn zz_bite(player: &mut PlayerState, id: ObjectId) { player.graveyard.push(id); }\n")),
    plant(INVARIANTS, "write: a &mut borrow", "commander_damage_taken",
          add(LIB, "fn zz_bite(player: &mut PlayerState) { let _zz = &mut player.commander_damage_taken; }\n")),
    plant(INVARIANTS, "write: a store method", "settle_game_result",
          add(LIB, "fn zz_bite(game: &mut GameState) { game.settle_game_result(); }\n")),
    plant(INVARIANTS, "write: a narrow performer row covers only its own fields", "mtgsim/src/engine/combat/steps.rs:",
          add("mtgsim/src/engine/combat/steps.rs", "fn zz_bite(permanent: &mut PermanentState) { permanent.tapped = true; }\n")),
    # check_invariants.py: tags
    plant(INVARIANTS, "tags: printed does not read another check's word", "card_data.types",
          add(LIB, "// ORDER-FREE: planted.\n" + READ)),
    plant(INVARIANTS, "tags: unordered does not read another check's word", "objects.keys",
          add(LIB, "// AS PRINTED: planted.\n" + WALK)),
    plant(INVARIANTS, "tags: write does not read another check's word", "life_total",
          add(LIB, "// ORDER-FREE: planted.\n" + WRITE)),
    plant(INVARIANTS, "tags: a word quoted mid-sentence is a mention", "card_data.types",
          add(LIB, "// The words PRE-LAYER ZONE: are quoted here.\n" + READ)),
    plant(INVARIANTS, "tags: a word without its colon is no tag", "card_data.types",
          add(LIB, "// PRE-LAYER ZONE planted.\n" + READ)),
    plant(INVARIANTS, "tags: a blank line ends the comment block", "card_data.types",
          add(LIB, "// PRE-LAYER ZONE: planted.\n\n" + READ)),
    plant(INVARIANTS, "tags: a tag inside a string is no tag", "card_data.types",
          add(LIB, 'fn zz_bite(obj: &GameObject) -> usize { let _s = "// AS PRINTED: x"; obj.card_data.types.len() }\n')),
    plant(INVARIANTS, "tags: a tag above covers one statement", "card_data.colors",
          add(LIB, TWO_READS.format(tag_above="// AS PRINTED: planted.\n    ", tag_beside=""))),
    plant(INVARIANTS, "tags: a trailing tag covers its own line", "card_data.colors",
          add(LIB, TWO_READS.format(tag_above="", tag_beside=" // AS PRINTED: planted."))),
    # check_invariants.py: its other two failures
    plant(INVARIANTS, "a known violation that matches nothing", "take the entry off",
          swap("plans/check_invariants.py", "KNOWN = {", 'KNOWN = {("printed", "zz_bite.rs", "card_data.types"): (1, "planted"), ')),
    plant(INVARIANTS, "GameState's declaration no longer parses", "did not parse into hash fields",
          after(STATE, GAME_STATE_OPENER, "}\n")),
    # check_practices_page.py
    plant(PRACTICES, "the page is over its budget", "lines, over budget by", add(PAGE, "planted\n" * 2000)),
    plant(PRACTICES, "a citation by section sign", f"cites {PAGE} by section number", add(BACKLOG, f"\n{PAGE} §3.4\n")),
    plant(PRACTICES, "a citation by section word, in a .rs file", f"cites {PAGE} by section number",
          add(LIB, f"// {PAGE} section 4\n")),
    plant(PRACTICES, "a dotted number across a comment's line break", f"cites {PAGE} by section number",
          add(LIB, f"// `{PAGE}`'s\n// 3.4\n")),
    plant(PRACTICES, "an anchor that is no heading of its doc", f"{HASH}zz-no-heading` is no heading of",
          add(BACKLOG, f"\n`{PAGE}{HASH}zz-no-heading`\n")),
    plant(PRACTICES, "an anchor on a doc that is not tracked", "`zz-bite.md` names no single tracked doc",
          add(BACKLOG, f"\n`zz-bite.md{HASH}anchor`\n")),
    plant(PRACTICES, "a local link to no heading", f"`{HASH}zz-no-heading` is no heading here",
          add(BACKLOG, f"\n[planted]({HASH}zz-no-heading)\n")),
    plant(PRACTICES, "a heading inside a code fence is no heading", f"`{HASH}zz-fenced` is no heading here",
          add(BACKLOG, f"\n```\n## Zz fenced\n```\n\n[planted]({HASH}zz-fenced)\n")),
]


# --- the trees ---------------------------------------------------------------


def git(*args):
    return subprocess.run(["git", "-C", str(ROOT), *args], capture_output=True, encoding="utf-8", errors="replace")


def throwaway_dir() -> Path:
    if os.name == "nt" and SHORT_PATHS.is_dir():
        base = SHORT_PATHS / "gates-bite"
        remove_trees(base)  # what a killed run left behind
        base.mkdir()
        return base
    return Path(tempfile.mkdtemp(prefix="gates-bite-"))


def make_tree(tree: Path):
    made = git("worktree", "add", "--quiet", "--detach", str(tree), "HEAD")
    if made.returncode != 0:
        raise RuntimeError(f"git worktree add {tree} failed:\n{made.stderr}")
    for script in (ROOT / "plans").glob("*.py"):
        shutil.copy2(script, tree / "plans" / script.name)


def remove_trees(base: Path):
    if base.is_dir():
        for tree in base.iterdir():
            git("worktree", "remove", "--force", str(tree))
        shutil.rmtree(base, ignore_errors=True)
    git("worktree", "prune")


def in_a_free_tree(free: queue.Queue, work, item):
    tree = free.get()
    try:
        return work(tree, item)
    finally:
        free.put(tree)


# --- one plant ---------------------------------------------------------------


class CouldNotApply(Exception):
    pass


def apply(tree: Path, edit: Edit):
    path = tree / edit.path
    if edit.op is Op.CREATE:
        if path.exists():
            raise CouldNotApply(f"{edit.path} already exists")
        path.write_bytes(edit.text.encode("utf-8"))
        return
    if not path.is_file():
        raise CouldNotApply(f"{edit.path} is gone")
    if edit.op is Op.DELETE:
        path.unlink()
        return
    text = path.read_bytes().decode("utf-8")
    if edit.op is Op.APPEND:
        text += edit.text
    elif (found := text.count(edit.anchor)) != 1:
        raise CouldNotApply(f"{edit.anchor!r} is in {edit.path} {found} times, not once")
    else:
        text = text.replace(edit.anchor, edit.text)
    path.write_bytes(text.encode("utf-8"))


def run_gate(tree: Path, gate: str):
    script, *args = gate.split()
    return subprocess.run(
        [sys.executable, str(tree / "plans" / script), *args],
        cwd=tree, env=ENV, capture_output=True, encoding="utf-8", errors="replace",
    )


def why_missed(tree: Path, planted: Plant) -> str | None:
    """Apply a plant, run its gate and restore the tree: `None` if the gate
    caught it, otherwise what went wrong."""
    saved = {e.path: (tree / e.path).read_bytes() if (tree / e.path).is_file() else None for e in planted.edits}
    try:
        for edit in planted.edits:
            apply(tree, edit)
        done = run_gate(tree, planted.gate)
    except CouldNotApply as e:
        return f"the plant could not apply: {e}"
    finally:
        for name, data in saved.items():
            if data is None:
                (tree / name).unlink(missing_ok=True)
            else:
                (tree / name).write_bytes(data)
    if done.returncode == 0:
        return "the gate exited 0"
    if planted.expect not in done.stdout + done.stderr:
        return f"the gate exited {done.returncode}, and its output never says {planted.expect!r}"
    return None


# --- the run -----------------------------------------------------------------


def main() -> int:
    started = time.monotonic()
    named = {gate.split()[0] for gate in GATES}
    unlisted = sorted(p.name for p in (ROOT / "plans").glob("check_*.py")
                      if p.name not in named and p.name != Path(__file__).name)
    unplanted = sorted(named - {planted.gate.split()[0] for planted in PLANTS})
    if unlisted or unplanted:
        print(f"gates with no plant, whose rot nothing here would notice: {', '.join(unlisted + unplanted)}")
        return 2
    base = throwaway_dir()
    trees = [base / f"tree{n}" for n in range(TREES)]
    free = queue.Queue()
    missed = 0
    try:
        with ThreadPoolExecutor(TREES) as pool:
            list(pool.map(make_tree, trees))
            for tree in trees:
                free.put(tree)

            controls = list(pool.map(lambda gate: in_a_free_tree(free, run_gate, gate), GATES))
            failing = [(gate, done) for gate, done in zip(GATES, controls) if done.returncode != 0]
            if failing:
                for gate, done in failing:
                    print(f"control: {gate} exits {done.returncode} with nothing planted:\n{done.stdout}{done.stderr}")
                print(f"\n{len(failing)} gate(s) fail on the unplanted tree, so no plant can prove anything. Stopping.")
                return 2
            print(f"control: {len(GATES)} gates pass unplanted at HEAD, with the working copy's plans/*.py.\n")

            results = pool.map(lambda planted: in_a_free_tree(free, why_missed, planted), PLANTS)
            for planted, why in zip(PLANTS, results):
                gate = planted.gate.split()[0]
                if why is None:
                    print(f"caught  {gate}: {planted.rule}")
                else:
                    missed += 1
                    print(f"MISSED  {gate}: {planted.rule} - {why}")
    finally:
        remove_trees(base)

    print(f"\n{len(PLANTS)} plants: {len(PLANTS) - missed} caught, {missed} missed, in {time.monotonic() - started:.0f} s.")
    return 1 if missed else 0


if __name__ == "__main__":
    sys.exit(main())
