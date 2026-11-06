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

#: Where each gap is filed and the row that builds it.
GAPS = {
    "target-card-in-a-zone": ("cant-be-targeted", "item 392"),
    "target-an-ability": ("cant-be-targeted", "item 392"),
    "targets-controlled-by-different-players": (DECKS_ROW, "item 393"),
    "modes-target-different-players": (DECKS_ROW, "item 393"),
    "targets-with-a-total": (DECKS_ROW, "item 393"),
    "target-by-kicker": (DECKS_ROW, "item 393"),
    "filter-by-keyword": (DECKS_ROW, "`backlog.md` 2.43"),
    "filter-mana-value": (DECKS_ROW, "item 394"),
    "filter-against-an-amount": (DECKS_ROW, "item 394"),
    "filter-tapped": (DECKS_ROW, "item 394"),
    "condition-counts-permanents": (DECKS_ROW, "item 395"),
    "during-your-turn": (DECKS_ROW, "item 396"),
    "condition-on-the-triggering-object": (DECKS_ROW, "item 396"),
    "condition-most-life": (DECKS_ROW, "item 396"),
    "attacked-you-last-turn": (DECKS_ROW, "item 396"),
    "count-of-attackers-at-you": (DECKS_ROW, "item 396"),
    "count-off-the-battlefield": (DECKS_ROW, "item 397"),
    "value-over-a-set": (DECKS_ROW, "item 398"),
    "amount-difference": (DECKS_ROW, "item 398"),
    "amount-opponents": (DECKS_ROW, "item 398"),
    "amount-spells-targets": (DECKS_ROW, "item 398"),
    "count-colors-among": (DECKS_ROW, "item 398"),
    "amount-cards-put-in-graveyard": (DECKS_ROW, "item 398"),
    "tokens-created-this-turn": (DECKS_ROW, "item 398"),
    "token-with-an-amount": (DECKS_ROW, "item 398"),
    "assigns-damage-by-toughness": (DECKS_ROW, "item 399"),
    "for-each-object": (DECKS_ROW, "item 400"),
    "nth-spell-each-turn": (DECKS_ROW, "item 401"),
    "first-time-each-turn-on-a-zone-change": (DECKS_ROW, "item 401"),
    "monstrosity": (DECKS_ROW, "item 402"),
    "surveil": (DECKS_ROW, "item 402"),
    "proliferate": (DECKS_ROW, "item 402"),
    "connive": (DECKS_ROW, "item 402"),
    "riot": (DECKS_ROW, "item 402"),
    "exchange-life-and-toughness": (DECKS_ROW, "item 402"),
    "protection": (DECKS_ROW, "item 402"),
    "myriad": (DECKS_ROW, "item 402"),
    "untap-on-another-players-untap-step": (DECKS_ROW, "item 403"),
    "mana-keeps-between-steps": (DECKS_ROW, "item 403"),
    "counter-then-exile": (DECKS_ROW, "item 404"),
    "when-you-cycle": (DECKS_ROW, "item 404"),
    "random-opponent": (DECKS_ROW, "item 404"),
    "has-activated-abilities-of-cards": (DECKS_ROW, "item 404"),
    "choose-with-a-total-the-rest-acts": (DECKS_ROW, "item 404"),
    "entry-counters-amount": (DECKS_ROW, "item 389"),
    "amount-differently-named-tokens": (DECKS_ROW, "item 390"),
    "dredge": (DECKS_ROW, "`backlog.md` 2.49"),
    "exert": (DECKS_ROW, "`backlog.md` 2.14"),
}

#: Each gap's code and tests, low and high, an estimate at filing: a plain sum,
#: which `plans/references/sizing-calibration.md` reads at about 3x.
GAP_LINES = {
    "target-card-in-a-zone": (300, 500), "target-an-ability": (100, 150),
    "targets-controlled-by-different-players": (60, 100), "modes-target-different-players": (40, 80),
    "targets-with-a-total": (80, 120), "target-by-kicker": (60, 100),
    "filter-by-keyword": (100, 150), "filter-mana-value": (40, 60), "filter-against-an-amount": (80, 120), "filter-tapped": (30, 40),
    "condition-counts-permanents": (30, 50), "during-your-turn": (15, 25), "condition-on-the-triggering-object": (40, 60),
    "condition-most-life": (30, 40), "attacked-you-last-turn": (40, 60), "count-of-attackers-at-you": (30, 50),
    "count-off-the-battlefield": (80, 120), "value-over-a-set": (80, 120), "amount-difference": (20, 30), "amount-opponents": (15, 25),
    "amount-spells-targets": (30, 40), "count-colors-among": (30, 40), "amount-cards-put-in-graveyard": (40, 60),
    "tokens-created-this-turn": (30, 50), "token-with-an-amount": (50, 80), "assigns-damage-by-toughness": (60, 100),
    "for-each-object": (120, 200), "nth-spell-each-turn": (40, 60), "first-time-each-turn-on-a-zone-change": (30, 50),
    "monstrosity": (80, 120), "surveil": (60, 100), "proliferate": (100, 150), "connive": (60, 100), "riot": (40, 70),
    "exchange-life-and-toughness": (80, 120), "protection": (300, 500), "myriad": (100, 160),
    "untap-on-another-players-untap-step": (40, 60), "mana-keeps-between-steps": (60, 100), "counter-then-exile": (40, 70),
    "when-you-cycle": (30, 50), "random-opponent": (30, 50), "has-activated-abilities-of-cards": (120, 200),
    "choose-with-a-total-the-rest-acts": (80, 120), "entry-counters-amount": (30, 60), "amount-differently-named-tokens": (40, 80),
    "dredge": (300, 500), "exert": (100, 160),
}
PLAIN_SUM_RATIO = 3
LINES_A_PR = 1700


def gap_prs(gaps):
    """A set of gaps as route PRs, low and high, at a plain sum's ratio."""
    low = sum(GAP_LINES[g[4:]][0] for g in gaps) * PLAIN_SUM_RATIO / LINES_A_PR
    high = sum(GAP_LINES[g[4:]][1] for g in gaps) * PLAIN_SUM_RATIO / LINES_A_PR
    return round(low, 1), round(high, 1)


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


def home(need):
    """The row a need is built in: its own name, or a gap's filing."""
    return GAPS[need[4:]][0] if need.startswith("gap:") else need


def last_step(left, rows):
    """The step at which every need is met."""
    return max((rows[home(n)].step for n in left), default=0)


def deck_title(path):
    return Path(path).stem.removeprefix("tdc-").replace("-", " ").title()


def report():
    records = {r["name"]: r for r in map(json.loads, RECORDS.read_text(encoding="utf-8").splitlines())}
    rows, _, to_come = route()
    unknown = sorted({n for r in records.values() for n in r["needs"] if not n.startswith("gap:") and n not in rows})
    registered = registered_cards()
    lists = [(deck_title(path), names) for path, names in deck_lists()]
    out = ["<!-- GENERATED by gap_census.py report. Do not hand-edit. -->", "", "# The gap census's tables", "",
           "Each card written against the tree and played; `plans/references/card-census/README.md#the-gap-census` says how far to trust it. **The census sees a card's blockers only up to its first row not built**, so a list below is not complete past that row, and a row's hunt finishes it.", ""]
    distinct = {n for _, names in lists for n in names if n not in registered}
    out += [f"{len(set(records) & distinct)} of {len(distinct)} unregistered cards read.", ""]

    left = {name: needs_left(r, rows) for name, r in records.items()}
    claims = {name: unproved(records[name]) for name, l in left.items() if not l and unproved(records[name])}
    if claims:
        sys.exit("claimed writable and not proved:\n" + "\n".join(f"  {n}: {', '.join(why)}" for n, why in sorted(claims.items())))
    tally = Counter(status(l) for name, l in left.items() if name in distinct)
    out += ["| | Cards |", "|---|---:|", f"| Writable today | {tally['writable']} |", f"| Blocked by a named row | {tally['blocked']} |",
            f"| A gap no row owned | {tally['gap']} |", ""]

    out += ["## The decks, closest first", "", "A card in several lists counts in each. **Last step** is where a deck's last card can be written on the route as it stands, a gap counting at the row that builds it, and **PRs** the route's rows up to it. **Rows done** is the step its last row closes, with its gaps built beside the rows, and **to it** the route's PRs to that step; **gap PRs** is what its gaps add, at a plain sum's ratio. Closest is the fewest of those two together.", "",
            "| Deck | Registered | Writable | Left | Gaps | Gap PRs | Last step | PRs | Rows done | To it | Rows it needs |", "|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|"]
    rated = []
    for title, names in lists:
        cards = set(names)
        read = [n for n in cards if n in records]
        ready = sum(1 for n in read if not left[n])
        waiting = [n for n in read if left[n]]
        step = max((last_step(left[n], rows) for n in waiting), default=0)
        aside = max((last_step([x for x in left[n] if not x.startswith("gap:")], rows) for n in waiting), default=0)
        needed = {need for n in waiting for need in left[n] if not need.startswith("gap:")}
        deck_gaps = {need for n in waiting for need in left[n] if need.startswith("gap:")}
        before = [r for r in to_come if r.step <= step]
        low, high = sum(r.low for r in before), sum(r.high for r in before)
        rows_done = [r for r in to_come if r.step <= aside]
        to_low, to_high = sum(r.low for r in rows_done), sum(r.high for r in rows_done)
        gap_low, gap_high = gap_prs(deck_gaps)
        cost = (to_low + to_high + gap_low + gap_high) / 2
        rated.append((cost, len(deck_gaps), title, len(cards & registered), ready, len(waiting), gap_low, gap_high, step, low, high, aside, to_low, to_high, len(needed), cards, sorted(deck_gaps)))
    rated.sort(key=lambda r: r[:3])
    for _, gap_count, title, reg, ready, waiting, gap_low, gap_high, step, low, high, aside, to_low, to_high, needed, _, _ in rated:
        out.append(f"| {title} | {reg} | {ready} | {waiting} | {gap_count} | {gap_low}–{gap_high} | {step} | {low}–{high} | {aside} | {to_low}–{to_high} | {needed} |")
    out.append("")

    steps = sorted({r.step for r in to_come})
    out += ["## Each deck's cards by step", "", "The share of a deck's cards that could be written once a step closes, registered cards included, a gap counting at the row that builds it. Step 0 is today.", "",
            "| Deck | Cards | " + " | ".join(["0"] + [str(s) for s in steps]) + " |", "|---|---:|" + "---:|" * (len(steps) + 1)]
    for _, _, title, *rest in rated:
        cards = rest[-2]
        counts = [sum(1 for n in cards if n in registered or (n in left and last_step(left[n], rows) <= step)) for step in [0] + steps]
        out.append(f"| {title} | {len(cards)} | " + " | ".join(str(c) for c in counts) + " |")
    out.append("")

    out += ["## The gaps each deck needs", "", "| Deck | Gaps |", "|---|---|"]
    for _, _, title, *rest in rated:
        out.append(f"| {title} | {', '.join(g[4:] for g in rest[-1])} |")
    out.append("")

    out += ["## Each row's cards", "", "**Needs** counts every card that waits on the row; **unlocks** counts those for which it is the last need, which is what the row registers from step 9's close.", "",
            "| Step | Row | Needs | Unlocks | The cards it unlocks |", "|---:|---|---:|---:|---|"]
    by_row, unlocked = defaultdict(list), defaultdict(list)
    for name, l in left.items():
        for need in l:
            if not need.startswith("gap:"):
                by_row[need].append(name)
        if l:
            unlocked[max((home(n) for n in l), key=lambda n: rows[n].step)].append(name)
    for row in sorted(set(by_row) | set(unlocked), key=lambda n: (rows[n].step, n)):
        out.append(f"| {rows[row].step} | `{row}` | {len(by_row[row])} | {len(unlocked[row])} | {', '.join(sorted(unlocked[row]))} |")
    out.append("")
    unnamed = [r.row for r in to_come if re.match(r"`([^`]+)`", r.row).group(1) not in set(by_row) | set(unlocked)]
    out += ["**No card named these rows.** The census stops at a card's first row not built, so this is what it saw and no more: " + ", ".join(unnamed) + ".", ""]

    gaps = defaultdict(list)
    for name, l in left.items():
        for need in l:
            if need.startswith("gap:"):
                gaps[need[4:]].append(name)
    out += ["## Each gap no row owned", "", "A gap builds in the row named here, its filing says what it is, and each was tried once more before it was filed.", "",
            "| Gap | Cards | Built in | Filed as | Which |", "|---|---:|---|---|---|"]
    for gap in sorted(gaps, key=lambda g: (-len(gaps[g]), g)):
        row, filed = GAPS[gap]
        out.append(f"| {gap} | {len(gaps[gap])} | `{row}` | {filed} | {', '.join(sorted(gaps[gap]))} |")
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
