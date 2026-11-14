"""The gap census's tables, read off what its crate's run recorded.

    python plans/references/card-census/gap_census.py ingest <run.jsonl>... --crate <src/cards>
    python plans/references/card-census/gap_census.py report
    python plans/references/card-census/gap_census.py amend "<card>" --add <need> --remove <need>
    python plans/references/card-census/gap_census.py rename <need> <row>

A card's needs are the record. Everything else here is derived, so a row landing
clears its cards on the next `report` with no card re-read: `ingest` keeps one
line a card in `results/gap-census-records.jsonl`, adding which rows to come
reshape what the card was written in, read off its source; `report` joins those
lines to the five lists and to `roadmap-v2.md`'s Sizing table and writes
`results/gap-census.md`. A row's hunt records what it finds with `amend`, since the
crate that wrote the cards is not kept. A row the Sizing table renames takes its
records with it through `rename`: a need naming no row reads as met, so
`record_problems` fails one, here and on the board (`check_state_of_play.py`),
which counts the cards with no need left. A row split in two is renamed to its
later part, so no card reads as writable early, and a card that needs only the
earlier part is moved to it with `amend`.

**A record's fields.** A run's line holds `name`, `made_by` (the function that
builds it), `needs`, `note`, `loops`, `board` (`passed`, `no board` or `FAILED: …`),
`reached` (copies that left the library and hand over its random games), `errors`,
`panics`, `checks_failed`, `survived` (what its board did not notice losing),
`paragraphs`, `keyword_paragraphs`, `uncovered` (paragraphs nothing took) and
`composition` (each ability's tree). `ingest` adds `reshaped` and `stand_ins`, and
drops what no table reads.

A need is a row's name, or `gap:<mechanic>` for a mechanic no row owned, which
`GAPS` files and homes. A gap clears when its home row lands, or from `BUILT_GAPS`
when it is built ahead of that row. A card is **blocked** when each need left is a
row, a **gap** when one is a gap, and **ready** when its needs have landed since:
nothing stops it, and its registration proves it.

**Writable is proved, not typed.** A card the census recorded with no need is
writable only if its run shows all of these, and `report` fails on one without
them:

- it passed the ten checks of how a card is composed (`registry.rs`'s and the
  lowering test's, which the crate ran on cards outside the registry);
- every printed paragraph is taken by an ability or a keyword flag, and nothing
  is a stand-in, a clause written in place of what the card prints;
- a board passed, and removing each written ability, and each member of a
  sequence, made that board fail, so nothing written went unread;
- its random games reached play and ended with no refusal and no panic.

How far a card was written is measured the same way, never typed. What none of
this reaches is whether a clause means what its paragraph says, which a board
asserts only as far as its author read the card: `results/gap-census-composition.txt`
prints each card's tree under its name for a reader.
"""

import argparse
import json
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path
from typing import NamedTuple

sys.path.insert(0, str(Path(__file__).resolve().parent.parent.parent))
from check_state_of_play import deck_lists, registered_cards, sizing  # noqa: E402

HERE = Path(__file__).resolve().parent
RECORDS = HERE / "results" / "gap-census-records.jsonl"
REPORT = HERE / "results" / "gap-census.md"
COMPOSITION = HERE / "results" / "gap-census-composition.txt"
DECKS_ROW = "tarkir-dragonstorm-decks"
#: The Sizing table's row of the small gaps built at step 9's close, which shares
#: its name with the decks' other rows.
CHEAP_GAPS = "the cheap gaps"
GAP_PREFIX = "gap:"
#: What a run's line carries that no table reads.
UNREAD = ("made_by", "written", "games")

#: What a row to come reshapes, read off a card's source: the row, then a pattern
#: of the words it replaces. Each is the row's own statement of its sweep.
RESHAPES = [
    ("any-color-plus-painlands-plus-signets", r"ManaOutput|add_mana|mana_ability_single|tap_for|mana1|filter_land_output|two_mana|filters_for|\bpain\("),
    ("resolution-recipients-and-amounts", r"AmountExpr::(Source|Target|Triggering)|TriggeringPlayer|PlayerRef::Owner"),
    ("linked-abilities", r"EnterWith|EnterModsTemplate|enters_tapped"),
    ("permission-design", r"AdditionalCost|AlternativeCost|additional_cost|alternative_cost"),
    ("mode-rules-and-costs", r"Effect::Modal"),
]


class Gap(NamedTuple):
    """A mechanic no row owned: the row that builds it, where it is filed, and
    its code and tests, low and high, an estimate at filing."""

    row: str
    filed: str
    low: int
    high: int


GAPS = {
    "target-card-in-a-zone": Gap("cant-be-targeted", "item 392", 300, 500),
    "target-an-ability": Gap("cant-be-targeted", "item 392", 100, 150),
    "targets-controlled-by-different-players": Gap(DECKS_ROW, "item 393", 60, 100),
    "modes-target-different-players": Gap(DECKS_ROW, "item 393", 40, 80),
    "targets-with-a-total": Gap(DECKS_ROW, "item 393", 80, 120),
    "target-by-kicker": Gap(DECKS_ROW, "item 393", 60, 100),
    "filter-by-keyword": Gap(CHEAP_GAPS, "`backlog.md` 2.43", 100, 150),
    "filter-mana-value": Gap(CHEAP_GAPS, "item 394", 40, 60),
    "filter-against-an-amount": Gap(CHEAP_GAPS, "item 394", 80, 120),
    "filter-tapped": Gap(CHEAP_GAPS, "item 394", 30, 40),
    "filter-toughness": Gap(CHEAP_GAPS, "item 394", 20, 30),
    "condition-counts-permanents": Gap(CHEAP_GAPS, "item 395", 30, 50),
    "during-your-turn": Gap(CHEAP_GAPS, "item 396", 15, 25),
    "condition-on-the-triggering-object": Gap(DECKS_ROW, "item 396", 40, 60),
    "condition-most-life": Gap(DECKS_ROW, "item 396", 30, 40),
    "attacked-you-last-turn": Gap(DECKS_ROW, "item 396", 40, 60),
    "count-of-attackers-at-you": Gap(DECKS_ROW, "item 396", 30, 50),
    "count-off-the-battlefield": Gap(CHEAP_GAPS, "item 397", 80, 120),
    "value-over-a-set": Gap(DECKS_ROW, "item 398", 80, 120),
    "amount-difference": Gap(DECKS_ROW, "item 398", 20, 30),
    "amount-opponents": Gap(DECKS_ROW, "item 398", 15, 25),
    "amount-spells-targets": Gap(DECKS_ROW, "item 398", 30, 40),
    "count-colors-among": Gap(DECKS_ROW, "item 398", 30, 40),
    "amount-cards-put-in-graveyard": Gap(DECKS_ROW, "item 398", 40, 60),
    "tokens-created-this-turn": Gap(DECKS_ROW, "item 398", 30, 50),
    "token-with-an-amount": Gap(DECKS_ROW, "item 398", 50, 80),
    "assigns-damage-by-toughness": Gap(CHEAP_GAPS, "item 399", 60, 100),
    "for-each-object": Gap(DECKS_ROW, "item 400", 120, 200),
    "nth-spell-each-turn": Gap(DECKS_ROW, "item 401", 40, 60),
    "first-time-each-turn": Gap(DECKS_ROW, "item 401", 30, 50),
    "monstrosity": Gap(DECKS_ROW, "item 402", 80, 120),
    "surveil": Gap(CHEAP_GAPS, "item 402", 60, 100),
    "proliferate": Gap(DECKS_ROW, "item 402", 100, 150),
    "connive": Gap(DECKS_ROW, "item 402", 60, 100),
    "riot": Gap(DECKS_ROW, "item 402", 40, 70),
    "exchange-life-and-toughness": Gap(DECKS_ROW, "item 402", 80, 120),
    "protection": Gap(DECKS_ROW, "item 402", 120, 200),
    "myriad": Gap(DECKS_ROW, "item 402", 100, 160),
    "untap-on-another-players-untap-step": Gap(DECKS_ROW, "item 403", 40, 60),
    "mana-keeps-between-steps": Gap("mana-kept-and-lost", "item 403", 60, 100),
    "counter-then-exile": Gap(DECKS_ROW, "item 404", 40, 70),
    "when-you-cycle": Gap(DECKS_ROW, "item 404", 30, 50),
    "random-opponent": Gap(DECKS_ROW, "item 404", 30, 50),
    "has-activated-abilities-of-cards": Gap(DECKS_ROW, "item 404", 120, 200),
    "choose-with-a-total-the-rest-acts": Gap(DECKS_ROW, "item 404", 80, 120),
    "whose-controller-was-dealt-damage": Gap(DECKS_ROW, "item 404", 30, 50),
    "exile-top-cards": Gap("casting-from-other-zones", "item 405", 40, 70),
    "damage-by-another-object": Gap(DECKS_ROW, "item 406", 30, 50),
    "entry-counters-amount": Gap(DECKS_ROW, "item 389", 30, 60),
    "amount-differently-named-tokens": Gap(DECKS_ROW, "item 390", 40, 80),
    "dredge": Gap(DECKS_ROW, "`backlog.md` 2.49", 300, 500),
    "exert": Gap(DECKS_ROW, "`backlog.md` 2.14", 100, 160),
}

#: Gaps built ahead of the row that homes them, as a PR that builds one says here.
BUILT_GAPS = set()

#: A plain sum of `**Sized:**` lines lands at about 3x, by
#: `plans/references/sizing-calibration.md`.
PLAIN_SUM_RATIO = 3
#: A phase PR ran about 1,700 lines (`roadmap-v2.md`'s Sizing section). The cards'
#: 2,500 is a card PR's top, and a gap is engine work.
LINES_A_PR = 1700


def is_gap(need):
    return need.startswith(GAP_PREFIX)


def gap_of(need):
    return GAPS[need.removeprefix(GAP_PREFIX)]


def home(need):
    """The row a need is built in: its own name, or a gap's home."""
    return gap_of(need).row if is_gap(need) else need


def gap_prs(gaps):
    """What a set of gaps adds to the route, low and high: a gap homed in a sized
    row is already in that row's PRs, so only the decks' own count."""
    added = [gap_of(g) for g in gaps if gap_of(g).row == DECKS_ROW]
    return tuple(round(sum(side) * PLAIN_SUM_RATIO / LINES_A_PR, 1) for side in ([g.low for g in added], [g.high for g in added]))


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


def read_records():
    return {r["name"]: r for r in map(json.loads, RECORDS.read_text(encoding="utf-8").splitlines())}


def write_records(records):
    RECORDS.parent.mkdir(exist_ok=True)
    RECORDS.write_text("".join(json.dumps(records[n], ensure_ascii=False) + "\n" for n in sorted(records)), encoding="utf-8", newline="\n")


def ingest(runs, crate):
    functions = functions_in(crate)
    kept = read_records() if RECORDS.exists() else {}
    for run in runs:
        for line in Path(run).read_text(encoding="utf-8").splitlines():
            record = json.loads(line)
            source = source_of(functions, record["made_by"], set())
            record["reshaped"] = [row for row, pattern in RESHAPES if re.search(pattern, source)]
            record["stand_ins"] = len(re.findall(r"\bstand_in\(", source))
            for key in UNREAD:
                record.pop(key, None)
            kept[record["name"]] = record
    write_records(kept)
    print(f"{len(kept)} cards in {RECORDS.name}")


def amend(card, add, remove):
    """A row's hunt, recorded: a need found behind a row the census could not see past."""
    records = read_records()
    record = records[card]
    record["needs"] = [n for n in record["needs"] if n not in remove] + [n for n in add if n not in record["needs"]]
    write_records(records)
    print(f"{card}: {', '.join(record['needs']) or 'no needs'}")


def rename(old, new):
    """A row renamed in the Sizing table, carried into every record. A split
    row goes to its later part, and `amend` moves a card that needs only the
    earlier one."""
    records = read_records()
    renamed = 0
    for record in records.values():
        for key in ("needs", "reshaped"):
            if old in record[key]:
                record[key] = list(dict.fromkeys(new if n == old else n for n in record[key]))
                renamed += 1
    write_records(records)
    print(f"{old} is {new} in {renamed} lists")


def row_name(sized):
    """A Sizing row's name: its first backticked word, or the cell as written
    for a row with none, which no record can name."""
    if CHEAP_GAPS in sized.row:
        return CHEAP_GAPS
    named = re.match(r"`([^`]+)`", sized.row)
    return named.group(1) if named else sized.row


def route():
    """`rows_of` the Sizing table, which must read clean."""
    parsed, problems = sizing()
    if problems:
        sys.exit("\n".join(problems))
    return rows_of(parsed)


def rows_of(parsed):
    """The rows to come by name as `SizedRow`s, the names landed, and the rows to
    come in order. A name's last row wins, so `mana-provenance` is its build and
    not its design."""
    rows, landed = {}, set()
    for sized in parsed:
        name = row_name(sized)
        if sized.landed:
            landed.add(name)
            rows.pop(name, None)
        else:
            rows[name] = sized
            landed.discard(name)
    return rows, landed, [r for r in parsed if not r.landed]


def written(record):
    """How far a card got, read off its run. A two-faced card's rules text is its
    front face, so its other face is unwritten and it is never whole."""
    unwritten = record["uncovered"] or record["stand_ins"] or " // " in record["name"]
    if not unwritten:
        return "whole"
    return "face" if len(record["uncovered"]) == record["paragraphs"] - record["keyword_paragraphs"] else "part"


def unproved(record):
    """What stands between a card with no need left and the word writable."""
    reasons = []
    if not record["board"].startswith("pass"):
        reasons.append("no passing board")
    if written(record) != "whole":
        reasons.append(f"written {written(record)}: {len(record['uncovered'])} paragraphs unwritten, {record['stand_ins']} stand-ins")
    if record["checks_failed"]:
        reasons.append("failed: " + "; ".join(record["checks_failed"]))
    if record["survived"]:
        reasons.append("unread by its board: " + "; ".join(record["survived"]))
    if record["errors"] or record["panics"]:
        reasons.append("refused in play")
    if not record["reached"]:
        reasons.append("never reached play")
    return reasons


def needs_left(record, rows):
    """What a card still waits on: a row to come, or a gap whose home is to come
    and that was not built ahead of it."""
    def waiting(need):
        built = is_gap(need) and need.removeprefix(GAP_PREFIX) in BUILT_GAPS
        return not built and home(need) in rows
    return [need for need in record["needs"] if waiting(need)]


def status(needs, record):
    """A card the census proved is writable. One whose needs have since landed is
    ready: nothing stops it, and its registration is what proves it."""
    if not needs:
        return "ready" if record["needs"] else "writable"
    return "gap" if any(is_gap(n) for n in needs) else "blocked"


def record_problems(records, rows, landed, unregistered):
    """What makes the records misstate a card: a need naming no row and no gap,
    which `needs_left` reads as met, and a card claimed writable with no proof."""
    def known(need):
        return need.removeprefix(GAP_PREFIX) in GAPS if is_gap(need) else need in rows or need in landed
    unknown = sorted({need for record in records.values() for need in record["needs"] if not known(need)})
    problems = [f"the gap census's records need {need}, which names no row of the Sizing table and no gap; "
                f"`gap_census.py rename {need} <row>` carries a renamed row into them, a split row's later part" for need in unknown]
    for name in sorted(set(records) & unregistered):
        if not records[name]["needs"] and unproved(records[name]):
            problems.append(f"the gap census's records claim {name} writable and not proved: {', '.join(unproved(records[name]))}")
    return problems


def tally(records, rows, unregistered):
    """How many unregistered deck cards stand at each `status` today."""
    return Counter(status(needs_left(records[name], rows), records[name]) for name in set(records) & unregistered)


def last_step(needs, rows):
    """The step at which every need is met."""
    return max((rows[home(n)].step for n in needs), default=0)


def deck_title(path):
    return Path(path).stem.removeprefix("tdc-").replace("-", " ").title()


class DeckRating(NamedTuple):
    """One deck against the route. Each card counts once: registered, writable,
    or waiting on a need."""

    title: str
    cards: set
    registered: int
    writable: int
    waiting: int
    on_a_gap: int
    gaps: list
    gap_low: float
    gap_high: float
    last_step: int
    prs_low: int
    prs_high: int
    rows_done_at: int
    rows_low: int
    rows_high: int
    rows_needed: int

    @property
    def cost(self):
        """Route PRs to its rows' end and its gaps' PRs, at each range's midpoint."""
        return (self.rows_low + self.rows_high + self.gap_low + self.gap_high) / 2


def rate(title, names, registered, needs_by_card, rows, to_come):
    cards = set(names)
    unregistered = [n for n in cards if n in needs_by_card and n not in registered]
    waiting = [n for n in unregistered if needs_by_card[n]]
    last = max((last_step(needs_by_card[n], rows) for n in waiting), default=0)
    rows_done_at = max((last_step([x for x in needs_by_card[n] if not is_gap(x)], rows) for n in waiting), default=0)
    gaps = sorted({need for n in waiting for need in needs_by_card[n] if is_gap(need)})
    named_rows = {need for n in waiting for need in needs_by_card[n] if not is_gap(need)}
    up_to_last = [r for r in to_come if r.step <= last]
    up_to_rows_done = [r for r in to_come if r.step <= rows_done_at]
    gap_low, gap_high = gap_prs(gaps)
    return DeckRating(
        title=title, cards=cards, registered=len(cards & registered), writable=len(unregistered) - len(waiting), waiting=len(waiting),
        on_a_gap=sum(1 for n in waiting if any(is_gap(x) for x in needs_by_card[n])), gaps=gaps, gap_low=gap_low, gap_high=gap_high,
        last_step=last, prs_low=sum(r.low for r in up_to_last), prs_high=sum(r.high for r in up_to_last),
        rows_done_at=rows_done_at, rows_low=sum(r.low for r in up_to_rows_done), rows_high=sum(r.high for r in up_to_rows_done),
        rows_needed=len(named_rows),
    )


def deck_tables(ratings, registered, needs_by_card, rows, to_come):
    out = ["## The decks, closest first", "",
           "A card in several lists counts in each. **Left** is the cards still waiting and **on a gap** those waiting on a gap. **Last step** is where a deck's last card can be written on the route as it stands, a gap counting at the row that builds it, and **PRs** the route's rows up to it. **Rows done** is the step its last row closes, with its gaps built beside the rows, and **to it** the route's PRs to that step; **gap PRs** is what its gaps add, at a plain sum's ratio. Closest is the fewest of those two together.", "",
           "| Deck | Registered | Writable | Left | On a gap | Gaps | Gap PRs | Last step | PRs | Rows done | To it | Rows it needs |",
           "|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|"]
    for d in ratings:
        out.append(f"| {d.title} | {d.registered} | {d.writable} | {d.waiting} | {d.on_a_gap} | {len(d.gaps)} | {d.gap_low}–{d.gap_high} | {d.last_step} | {d.prs_low}–{d.prs_high} | {d.rows_done_at} | {d.rows_low}–{d.rows_high} | {d.rows_needed} |")
    out.append("")

    steps = sorted({r.step for r in to_come})
    out += ["## Each deck's cards by step", "", "How many of a deck's cards could be written once a step closes, registered cards included, a gap counting at the row that builds it. Step 0 is today.", "",
            "| Deck | Cards | " + " | ".join(["0"] + [str(s) for s in steps]) + " |", "|---|---:|" + "---:|" * (len(steps) + 1)]
    for d in ratings:
        counts = [sum(1 for n in d.cards if n in registered or (n in needs_by_card and last_step(needs_by_card[n], rows) <= step)) for step in [0] + steps]
        out.append(f"| {d.title} | {len(d.cards)} | " + " | ".join(str(c) for c in counts) + " |")
    out.append("")

    out += ["## The gaps each deck needs", "", "| Deck | Gaps |", "|---|---|"]
    for d in ratings:
        out.append(f"| {d.title} | {', '.join(g.removeprefix(GAP_PREFIX) for g in d.gaps)} |")
    out.append("")
    return out


def row_table(needs_by_card, rows, to_come):
    needs_it, unlocks = defaultdict(list), defaultdict(list)
    for name, needs in needs_by_card.items():
        for row in {home(need) for need in needs}:
            needs_it[row].append(name)
        if needs:
            unlocks[max((home(n) for n in needs), key=lambda n: rows[n].step)].append(name)
    out = ["## Each row's cards", "", "**Needs** counts every card that waits on the row, its gaps included; **unlocks** counts those for which it is the last need, which is what the row registers from step 9's close.", "",
           "| Step | Row | Needs | Unlocks | The cards it unlocks |", "|---:|---|---:|---:|---|"]
    for row in sorted(needs_it, key=lambda n: (rows[n].step, n)):
        out.append(f"| {rows[row].step} | `{row}` | {len(needs_it[row])} | {len(unlocks[row])} | {', '.join(sorted(unlocks[row]))} |")
    out.append("")
    unnamed = [r.row for r in to_come if row_name(r) not in needs_it]
    out += ["**No card named these rows.** The census stops at a card's first row not built, so this is what it saw and no more: " + ", ".join(unnamed) + ".", ""]
    return out


def gap_table(needs_by_card):
    cards_on = defaultdict(list)
    for name, needs in needs_by_card.items():
        for need in needs:
            if is_gap(need):
                cards_on[need.removeprefix(GAP_PREFIX)].append(name)
    out = ["## Each gap no row owned", "", "A gap builds in the row named here, its filing says what it is, and each was tried once more before it was filed.", "",
           "| Gap | Cards | Built in | Filed as | Which |", "|---|---:|---|---|---|"]
    for gap in sorted(cards_on, key=lambda g: (-len(cards_on[g]), g)):
        out.append(f"| {gap} | {len(cards_on[gap])} | `{GAPS[gap].row}` | {GAPS[gap].filed} | {', '.join(sorted(cards_on[gap]))} |")
    out.append("")
    return out, len(cards_on)


def card_table(records, needs_by_card):
    out = ["## Every card", "",
           "**Written** is measured: *whole* is every printed paragraph taken with no stand-in, *face* the printed face alone with no paragraph written but keywords, and *part* anything between. A two-faced card's other face is unwritten. **Reached play** counts copies that left the library and hand over its random games, and a refusal ends a game early, so it can read 0. **Board** is whether an asserting board passed, and a blocked card's may fail by design.", "",
           "| Card | Written | Today | Needs | Reshaped by | Loop | Board | Reached play | Note |", "|---|---|---|---|---|---|---|---:|---|"]
    for name in sorted(records):
        r = records[name]
        board = "passed" if r["board"].startswith("pass") else "failed" if r["board"].startswith("FAIL") else "—"
        refused = " Refused: " + "; ".join(e[:80] for e in r["errors"]) if r["errors"] else ""
        out.append(f"| {name} | {written(r)} | {status(needs_by_card[name], r)} | {', '.join(needs_by_card[name]) or '—'} | {', '.join(r['reshaped']) or '—'} | {'yes' if r['loops'] else 'no'} | {board} | {r['reached']} | {r['note']}{refused} |")
    out.append("")
    return out


def report():
    records = read_records()
    rows, landed, to_come = route()
    registered = registered_cards()
    lists = [(deck_title(path), names) for path, names in deck_lists()]
    unregistered = {n for _, names in lists for n in names if n not in registered}
    problems = record_problems(records, rows, landed, unregistered)
    if problems:
        sys.exit("\n".join(problems))
    needs_by_card = {name: needs_left(r, rows) for name, r in records.items()}

    counted = tally(records, rows, unregistered)
    out = ["<!-- GENERATED by gap_census.py report. Do not hand-edit. -->", "", "# The gap census's tables", "",
           "Each card written against the tree and played; `plans/references/card-census/README.md#the-gap-census` says how far to trust it. **The census sees a card's blockers only up to its first row not built**, so a list below is not complete past that row, and a row's hunt finishes it.", "",
           f"{len(set(records) & unregistered)} of {len(unregistered)} unregistered cards read.", "",
           "| | Cards |", "|---|---:|", f"| Writable today, proved | {counted['writable']} |", f"| Ready: its needs have landed, not yet proved | {counted['ready']} |",
           f"| Blocked by a named row | {counted['blocked']} |", f"| A gap no row owned | {counted['gap']} |", ""]
    ratings = sorted((rate(title, names, registered, needs_by_card, rows, to_come) for title, names in lists), key=lambda d: (d.cost, len(d.gaps), d.title))
    out += deck_tables(ratings, registered, needs_by_card, rows, to_come)
    out += row_table(needs_by_card, rows, to_come)
    gaps, gap_count = gap_table(needs_by_card)
    out += gaps
    out += card_table(records, needs_by_card)
    REPORT.write_text("\n".join(out), encoding="utf-8", newline="\n")

    composed = []
    for name in sorted(records):
        r = records[name]
        composed.append(f"=== {name} ({written(r)}, {status(needs_by_card[name], r)})")
        composed += [f"unwritten: {paragraph}" for paragraph in r["uncovered"]]
        composed += r["composition"]
        composed.append("")
    COMPOSITION.write_text("\n".join(composed), encoding="utf-8", newline="\n")
    print(f"{REPORT.name}: {len(records)} cards; writable {counted['writable']}, ready {counted['ready']}, blocked {counted['blocked']}, gap {counted['gap']}; {gap_count} gaps")


def main():
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    parser = argparse.ArgumentParser()
    parser.add_argument("command", choices=["ingest", "report", "amend", "rename"])
    parser.add_argument("runs", nargs="*", help="`ingest`'s run files, `amend`'s card, or `rename`'s need and its row")
    parser.add_argument("--crate")
    parser.add_argument("--add", action="append", default=[])
    parser.add_argument("--remove", action="append", default=[])
    args = parser.parse_args()
    if args.command == "ingest":
        ingest(args.runs, args.crate)
    elif args.command == "amend":
        amend(args.runs[0], args.add, args.remove)
    elif args.command == "rename":
        rename(*args.runs)
    else:
        report()


if __name__ == "__main__":
    main()
