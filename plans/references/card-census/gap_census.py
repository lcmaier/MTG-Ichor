"""The gap census's tables, read off what its crate's run recorded.

    python plans/references/card-census/gap_census.py ingest <run.jsonl>... --crate <src/cards>
    python plans/references/card-census/gap_census.py report

A card's needs are the record. Everything else here is derived, so a row landing
clears its cards on the next `report` with no card re-read: `ingest` keeps one
line a card in `results/gap-census-records.jsonl`, adding which rows to come
reshape what the card was written in, read off its source; `report` joins those
lines to the five lists and to `roadmap-v2.md`'s Sizing table and writes
`results/gap-census.md`.

A need is a row's name, or `gap:<mechanic>` for a mechanic no row owns. A card is
**blocked** when each need left is a row and a **gap** when one is a mechanic no
row owns.

**Writable is proved, not typed.** With no need left, a card is writable only if
its run shows all of these, and `report` fails on a card that claims it without
them:

- it passed the ten checks of how a card is composed (`registry.rs`'s and the
  lowering test's, which the crate runs on cards outside the registry);
- every printed paragraph is taken by an ability or a keyword flag, and nothing
  is a stand-in, a clause written in place of what the card prints;
- a board passed, and removing each written ability, and each member of a
  sequence, made that board fail, so nothing written went unread;
- its random games ended with no refusal and no panic.

How far a card was written is measured the same way, never typed. What none of
this reaches is whether a clause means what its paragraph says, which a board
asserts only as far as its author read the card: `results/gap-census-composition.txt`
prints each card's tree beside its text for a reader.
"""

import argparse
import json
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent.parent))
from check_state_of_play import deck_lists, registered_cards, sizing  # noqa: E402

HERE = Path(__file__).resolve().parent
RECORDS = HERE / "results" / "gap-census-records.jsonl"
REPORT = HERE / "results" / "gap-census.md"
COMPOSITION = HERE / "results" / "gap-census-composition.txt"
DECKS_ROW = "tarkir-dragonstorm-decks"

#: What a row to come reshapes, read off a card's source: the row, then a pattern
#: of the words it replaces. Each is the row's own statement of its sweep.
RESHAPES = [
    ("any-color-plus-painlands-plus-signets", r"ManaOutput|add_mana|mana_ability_single|tap_for|mana1|filter_land_output"),
    ("resolution-recipients-and-amounts", r"AmountExpr::(Source|Target|Triggering)|TriggeringPlayer|PlayerRef::Owner"),
    ("linked-abilities", r"EnterWith|EnterModsTemplate|enters_tapped"),
    ("permission-design", r"AdditionalCost|AlternativeCost|additional_cost|alternative_cost"),
    ("modal-spells-and-abilities", r"Effect::Modal"),
]


def source_of(functions, name, seen):
    """A function's text and the text of every function it calls, so a card
    written through a helper is read through it."""
    if name in seen or name not in functions:
        return ""
    seen.add(name)
    body = functions[name]
    return body + "".join(source_of(functions, called, seen) for called in re.findall(r"\b([a-z_][a-z0-9_]*)\(", body))


def functions_in(crate):
    found = {}
    for path in Path(crate).glob("*.rs"):
        text = path.read_text(encoding="utf-8")
        starts = [(m.start(), m.group(1)) for m in re.finditer(r"^(?:pub )?fn ([a-z_][a-z0-9_]*)", text, re.M)]
        for (start, name), (end, _) in zip(starts, starts[1:] + [(len(text), "")]):
            found.setdefault(name, text[start:end])
    return found


def ingest(runs, crate):
    functions = functions_in(crate)
    kept = {}
    if RECORDS.exists():
        kept = {r["name"]: r for r in map(json.loads, RECORDS.read_text(encoding="utf-8").splitlines())}
    for run in runs:
        for line in Path(run).read_text(encoding="utf-8").splitlines():
            record = json.loads(line)
            source = source_of(functions, record["made_by"], set())
            record["reshaped"] = [row for row, pattern in RESHAPES if re.search(pattern, source)]
            record["stand_ins"] = len(re.findall(r"\bstand_in\(", source))
            kept[record["name"]] = record
    RECORDS.parent.mkdir(exist_ok=True)
    RECORDS.write_text("".join(json.dumps(kept[n], ensure_ascii=False) + "\n" for n in sorted(kept)), encoding="utf-8")
    print(f"{len(kept)} cards in {RECORDS.name}")


def route():
    """Each row to come, by name, as `(step, low, high)`; a name's last row wins,
    so `mana-provenance` is its build and not its design."""
    parsed, problems = sizing()
    if problems:
        sys.exit("\n".join(problems))
    rows, landed = {}, set()
    for sized in parsed:
        name = re.match(r"`([^`]+)`", sized.row).group(1)
        if sized.landed:
            landed.add(name)
            rows.pop(name, None)
        else:
            rows[name] = sized
            landed.discard(name)
    return rows, landed, [r for r in parsed if not r.landed]


def written(record):
    """How far a card got, read off its run."""
    if not record["uncovered"] and not record["stand_ins"]:
        return "whole"
    return "body" if len(record["uncovered"]) == record["paragraphs"] - record["keyword_paragraphs"] else "part"


def unproved(record):
    """What stands between a card with no need left and the word writable."""
    reasons = []
    if not record["board"].startswith("pass"):
        reasons.append("no passing board")
    if record["uncovered"]:
        reasons.append(f"{len(record['uncovered'])} paragraphs unwritten")
    if record["stand_ins"]:
        reasons.append(f"{record['stand_ins']} stand-ins")
    if record["checks_failed"]:
        reasons.append("failed: " + "; ".join(record["checks_failed"]))
    if record["survived"]:
        reasons.append("unread by its board: " + "; ".join(record["survived"]))
    if record["errors"] or record["panics"]:
        reasons.append("refused in play")
    return reasons


def needs_left(record, rows):
    return [n for n in record["needs"] if n.startswith("gap:") or n in rows]


def status(left):
    if not left:
        return "writable"
    return "gap" if any(n.startswith("gap:") for n in left) else "blocked"


def last_step(left, rows):
    """The step at which every need is met: a gap waits for the decks' own row."""
    steps = [rows[DECKS_ROW].step if n.startswith("gap:") else rows[n].step for n in left]
    return max(steps, default=0)


def deck_title(path):
    return Path(path).stem.removeprefix("tdc-").replace("-", " ").title()


def report():
    records = {r["name"]: r for r in map(json.loads, RECORDS.read_text(encoding="utf-8").splitlines())}
    rows, _, to_come = route()
    unknown = sorted({n for r in records.values() for n in r["needs"] if not n.startswith("gap:") and n not in rows})
    registered = registered_cards()
    lists = [(deck_title(path), names) for path, names in deck_lists()]
    out = ["<!-- GENERATED by gap_census.py report. Do not hand-edit. -->", "", "# The gap census's tables", ""]
    distinct = {n for _, names in lists for n in names if n not in registered}
    out += [f"{len(set(records) & distinct)} of {len(distinct)} unregistered cards read.", ""]

    left = {name: needs_left(r, rows) for name, r in records.items()}
    claims = {name: unproved(records[name]) for name, l in left.items() if not l and unproved(records[name])}
    if claims:
        sys.exit("claimed writable and not proved:\n" + "\n".join(f"  {n}: {', '.join(why)}" for n, why in sorted(claims.items())))
    tally = Counter(status(l) for name, l in left.items() if name in distinct)
    out += ["| | Cards |", "|---|---:|", f"| Writable today | {tally['writable']} |", f"| Blocked by a named row | {tally['blocked']} |",
            f"| A gap no row owned | {tally['gap']} |", ""]

    out += ["## The decks, closest first", "", "A card in several lists counts in each. A deck's last step is where its last card can be written, and PRs are the route's rows up to it.", "",
            "| Deck | Registered | Writable | Left | Read | Last step | Route PRs before it | Rows it needs |", "|---|---:|---:|---:|---:|---:|---:|---|"]
    rated = []
    for title, names in lists:
        cards = set(names)
        read = [n for n in cards if n in records]
        ready = sum(1 for n in read if not left[n])
        waiting = [n for n in read if left[n]]
        step = max((last_step(left[n], rows) for n in waiting), default=0)
        needed = sorted({need for n in waiting for need in left[n] if not need.startswith("gap:")}, key=lambda n: (rows[n].step, n))
        before = [r for r in to_come if r.step <= step and not r.row.startswith(f"`{DECKS_ROW}`, the gap census")]
        low, high = sum(r.low for r in before), sum(r.high for r in before)
        rated.append((step, len(waiting), title, len(cards & registered), ready, len(waiting), len(read), low, high, needed))
    for step, _, title, reg, ready, waiting, read, low, high, needed in sorted(rated):
        out.append(f"| {title} | {reg} | {ready} | {waiting} | {read} | {step} | {low}–{high} | {len(needed)} |")
    out.append("")

    out += ["## Each row's cards", "", "**Needs** counts every card that waits on the row; **unlocks** counts those for which it is the last need, which is what the row registers from step 9's close.", "",
            "| Step | Row | Needs | Unlocks | The cards it unlocks |", "|---:|---|---:|---:|---|"]
    by_row, unlocked = defaultdict(list), defaultdict(list)
    for name, l in left.items():
        for need in l:
            if not need.startswith("gap:"):
                by_row[need].append(name)
        if l and status(l) == "blocked":
            unlocked[max(l, key=lambda n: rows[n].step)].append(name)
    for row in sorted(by_row, key=lambda n: (rows[n].step, n)):
        out.append(f"| {rows[row].step} | `{row}` | {len(by_row[row])} | {len(unlocked[row])} | {', '.join(sorted(unlocked[row]))} |")
    out.append("")

    gaps = defaultdict(list)
    for name, l in left.items():
        for need in l:
            if need.startswith("gap:"):
                gaps[need[4:]].append(name)
    out += ["## Each gap no row owned", "", "| Gap | Cards | Which |", "|---|---:|---|"]
    for gap in sorted(gaps, key=lambda g: (-len(gaps[g]), g)):
        out.append(f"| {gap} | {len(gaps[gap])} | {', '.join(sorted(gaps[gap]))} |")
    out.append("")

    out += ["## Every card", "", "| Card | Written | Today | Needs | Reshaped by | Loop | Board | Reached play | Note |", "|---|---|---|---|---|---|---|---:|---|"]
    for name in sorted(records):
        r = records[name]
        needs = ", ".join(left[name]) or "—"
        board = "passed" if r["board"].startswith("pass") else "failed" if r["board"].startswith("FAIL") else "—"
        refused = " Refused: " + "; ".join(e[:80] for e in r["errors"]) if r["errors"] else ""
        out.append(f"| {name} | {written(r)} | {status(left[name])} | {needs} | {', '.join(r['reshaped']) or '—'} | {'yes' if r['loops'] else 'no'} | {board} | {r['reached']} | {r['note']}{refused} |")
    out.append("")
    REPORT.write_text("\n".join(out), encoding="utf-8")
    composed = []
    for name in sorted(records):
        r = records[name]
        composed.append(f"=== {name} ({written(r)}, {status(left[name])})")
        composed += [f"unwritten: {paragraph}" for paragraph in r["uncovered"]]
        composed += r["composition"]
        composed.append("")
    COMPOSITION.write_text("\n".join(composed), encoding="utf-8")
    print(f"{REPORT.name}: {len(records)} cards; writable {tally['writable']}, blocked {tally['blocked']}, gap {tally['gap']}; {len(gaps)} gaps")
    if unknown:
        print("needs naming no row to come:", ", ".join(unknown))


def main():
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    parser = argparse.ArgumentParser()
    parser.add_argument("command", choices=["ingest", "report"])
    parser.add_argument("runs", nargs="*")
    parser.add_argument("--crate")
    args = parser.parse_args()
    if args.command == "ingest":
        ingest(args.runs, args.crate)
    else:
        report()


if __name__ == "__main__":
    main()
