#!/usr/bin/env python3
"""
fuzz_ab - one sitting of the fuzz A/B, sized to what each number needs.

    python plans/fuzz_ab.py --arm main=../mtgsim_v2_main/mtgsim/target/release/fuzz_games.exe \
                            --arm new=mtgsim/target/release/fuzz_games.exe [--require "Cytoshape"]

Three kinds of number come out of `fuzz_games`, and they cost different
amounts to get right. This script runs each at the cheapest setting that is
still the same number, which is what makes a sitting fit in a few minutes
instead of most of an hour:

- **Counters** (turns, spells, deaths, layer walks, gathers, ...) are a pure
  function of the seed and the pool, and identical at every `--threads` value
  (`codebase-state.md`, "Determinism holds"). One threaded run per arm per
  pool, 200 games. Seconds.
- **Fixture rows** for `fuzz-record.md` are the same counters at
  50 games. One threaded run per arm per pool. Printed as the table.
- **Time** is the only number that needs `--threads 1`, interleaving and
  medians, and only `performance` measures a delta - `stress` milliseconds
  are a threshold, never compared (§3.1). So: `--rounds` rounds of every arm
  in turn, 200 games, serial, performance only. That block is nearly all of
  the wall time; `--rounds 3` and `--games 200` are the defaults because
  medians of three at 200 games is where the run-to-run spread sits at ~2.4%,
  and cutting either is what raises it.

Determinism falls out for free: every timing round's output outside
`=== Timing ===` must equal the threaded counter run's, which checks both
thread-independence and run-to-run identity without extra runs. The first
arm is the baseline; every other arm's counters are diffed against it, so a
"registered but not pooled" arm that should reproduce `main` on
`performance` is checked by construction — on `stress` registration changes
the decks, and the engine's reading there is an arm with the cards
*unregistered* (`replacement-architecture.md` §11 item 80). Checked by
construction.

Reads the counters, not the milliseconds: the timing table prints ms per
1,000 walks and per 1,000 queries (walks plus memo hits) beside the median,
because "the game got longer", "the walk got slower" and "fewer questions
walked" are three different findings. It prints us per decision for a fourth:
the ratchet's own unit (`engineering-practices.md` 3.1), which is what says
whether a change cost the agent anything rather than the machine.
"""

import argparse
import os
import re
import statistics
import subprocess
import sys
import tempfile
import time

POOLS = ["performance", "stress"]

ROWS = [
    ("Errors", r"^Errors:\s+(\d+)"),
    ("Panics", r"^Panics:\s+(\d+)"),
    ("Uncast resolved", r"^Uncast resolved:\s+(\d+)"),
    ("Hit turn limit", r"^Hit turn limit:\s+(\d+)"),
    ("Wins by seat", None),
    ("Wins by effect", None),
    ("Avg turns", r"^Avg turns/game:\s+([\d.]+)"),
    ("Max turns", r"^Max turns seen:\s+(\d+)"),
    ("Spells cast", r"^\s+Spells cast:\s+([\d.]+)"),
    ("Lands played", r"^\s+Lands played:\s+([\d.]+)"),
    ("Combat w/ atk", r"^\s+Combat w/ atk:\s+([\d.]+)"),
    ("Creatures died", r"^\s+Creatures died:\s+([\d.]+)"),
    ("Damage events", r"^\s+Damage events:\s+([\d.]+)"),
    ("Total damage", r"^\s+Total damage:\s+([\d.]+)"),
    ("Life changes", r"^\s+Life changes:\s+([\d.]+)"),
    # Printed by `fuzz_games` only above two seats, so absent from every
    # two-player run and skipped rather than shown as "?" — see OPTIONAL_ROWS.
    ("Turns after a departure", r"^\s+Turns after a departure:\s+([\d.]+)"),
    ("Departed-owned permanents", r"^\s+Departed-owned permanents:\s+([\d.]+)"),
    ("Layer walks", r"^\s+Layer walks:\s+(\d+)"),
    ("Board walks", r"^\s+Board walks:\s+(\d+)"),
    ("Memo hits", r"^\s+Memo hits:\s+(\d+)"),
    ("Layer frames", r"^\s+Layer frames:\s+(\d+)"),
    ("Frames/walk", r"^\s+Frames/walk:\s+([\d.]+)"),
    ("Dependency checks", r"^\s+Dependency checks:\s+(\d+)"),
    ("Replacement gathers", r"^\s+Replacement gathers:\s+(\d+)"),
    ("Restriction queries", r"^\s+Restriction queries:\s+(\d+)"),
    # `GameAction::ProduceMana` performed, per game: the denominator the
    # gathers row is read against on the hottest path (RE-9).
    ("Mana productions", r"^\s+Mana productions:\s+(\d+)"),
    ("Prevention allocations", r"^\s+Prevention allocations:\s+([\d.]+)"),
    ("Replacement prompts", r"^\s+Replacement prompts:\s+([\d.]+)"),
    ("Max batch depth", r"^\s+Max batch depth:\s+(\d+)"),
    # Item 138's two: prompts with two or more legal answers, and their
    # priority share. A fixture row like the rest, and the denominator the
    # timing table's `µs / decision` divides CPU/game by.
    ("Decisions", r"^\s+Decisions:\s+(\d+)"),
    ("Priority decisions", r"^\s+Priority decisions:\s+(\d+)"),
]
# Rows that must read zero, flagged loudly when they do not. The fuzz harness
# asserts nothing, so a row pinned at zero is the only way a 200-game run can
# fail rather than merely print — which is why a *fixed* bug keeps its row:
# "Uncast resolved" has read 0 since item 16c, and "Departed-owned permanents"
# has read 0 since RE-7. A regression in either is a line here, not a diff
# somebody has to notice.
ZERO_ROWS = [
    "Errors", "Panics", "Uncast resolved", "Hit turn limit",
    "Departed-owned permanents",
]
# What stays out of `fuzz-record.md`'s fixture table: the four
# process rows, which are about the run rather than about the game.
# **Not the same list as `ZERO_ROWS` any more**, and they were one list only
# because they happened to coincide until RE-7: a row can be a threshold *and*
# a number the table records, and "Departed-owned permanents" is both — its
# 32.2 → 0.0 is the record of a fix and its 0.0 is the guard on it.
FIXTURE_EXCLUDE = ["Errors", "Panics", "Uncast resolved", "Hit turn limit", "Max turns"]
# Rows a two-player run does not print at all. Dropped from a table where no
# arm has them, and shown wherever one does — **by name, not by "every value
# is '?'"**: a row that vanished because the harness stopped printing it is a
# regression, and only these two are legitimately absent.
OPTIONAL_ROWS = {"Turns after a departure", "Departed-owned permanents"}
# The §3 table's rows, in its order.
FIXTURE_ROWS = [r for r, _ in ROWS if r not in FIXTURE_EXCLUDE]
BOLD = {"Layer walks", "Board walks", "Memo hits", "Layer frames", "Frames/walk", "Dependency checks", "Replacement gathers", "Restriction queries"}


def run(binary, args, out_path):
    with open(out_path, "w", encoding="utf-8") as f:
        subprocess.run([binary] + args, stdout=f, stderr=subprocess.STDOUT, check=False)
    return open(out_path, encoding="utf-8", errors="replace").read()


def strip_timing(text):
    out, skip = [], False
    for line in text.splitlines():
        if line.strip() == "=== Timing ===":
            skip = True
            continue
        if skip and line.strip() == "":
            skip = False
            continue
        if not skip:
            out.append(line)
    # The header names the pool size and the thread count; neither is a counter.
    out = [l for l in out if not l.startswith("Card pool: ") and not l.startswith("Threads: ")]
    return "\n".join(out)


def grab(text, pat):
    m = re.search(pat, text, re.M)
    return m.group(1) if m else "?"


def seats(text):
    # The harness prints the line only off its default of two.
    m = re.search(r"^Players: (\d+)", text, re.M)
    return int(m.group(1)) if m else 2


def wins(text):
    """Every seat's wins, in seat order — two cells at two seats, four at four."""
    cells = []
    for p in range(seats(text)):
        m = re.search(rf"^\s+P{p} wins\s+(\d+) \(([\d.]+)%\)", text, re.M)
        cells.append(f"{m.group(1)} ({m.group(2)}%)" if m else "0 (0.0%)")
    return " / ".join(cells)


def wins_by_effect(text):
    """Games ended by a `PlayerWon` (CR 104.2b), summed over seats."""
    return str(sum(int(n) for n in re.findall(r"^\s+P\d+ wins by effect\s+(\d+) \(", text, re.M)))


SPECIAL = {"Wins by seat": wins, "Wins by effect": wins_by_effect}


def counters(text):
    return {name: (SPECIAL[name](text) if pat is None else grab(text, pat)) for name, pat in ROWS}


ROW_LABEL = re.compile(r"^\s*([A-Za-z][A-Za-z /-]*?):\s", re.M)


def comparable(text, base):
    """`text` without the rows the baseline binary does not print at all.

    A phase that adds a diagnostic row (RE-9's `Mana productions`) prints a
    line `main` never will, and a whole-text compare would call every arm
    `differ` for the sitting that introduces it. A row absent from the
    baseline cannot be evidence of a changed game; it is the new row itself.
    The table still shows it as `?` on the baseline, which is the honest cell.
    """
    labels = {m.group(1) for m in ROW_LABEL.finditer(base)}
    keep = []
    for line in text.split("\n"):
        m = ROW_LABEL.match(line)
        if m and m.group(1) not in labels:
            continue
        keep.append(line)
    return "\n".join(keep)


def fmt(name, value):
    if name in ("Layer walks", "Board walks", "Memo hits", "Layer frames", "Dependency checks") and value.isdigit():
        return f"{int(value):,}"
    return value


def table(title, arms, rows_by_arm, rows):
    width = max(len(a) for a in arms) + 2
    print(f"\n{title}")
    print(f"{'':<26}" + "".join(f"{a:>{max(width, 18)}}" for a in arms))
    for name in rows:
        if name in OPTIONAL_ROWS and all(rows_by_arm[a][name] == "?" for a in arms):
            continue
        print(f"{name:<26}" + "".join(f"{fmt(name, rows_by_arm[a][name]):>{max(width, 18)}}" for a in arms))


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--arm", action="append", required=True, metavar="LABEL=PATH",
                    help="a fuzz_games binary; the first is the baseline. Repeatable")
    ap.add_argument("--games", type=int, default=200)
    ap.add_argument("--rounds", type=int, default=3, help="timing rounds (medians); 0 skips timing")
    ap.add_argument("--seed", type=int, default=12345)
    ap.add_argument("--threads", type=int, default=os.cpu_count() or 8,
                    help="for the counter and fixture runs; timing is always --threads 1")
    ap.add_argument("--require", default=None, help="also run --require NAMES on every arm (performance, threaded)")
    ap.add_argument("--no-fixtures", action="store_true", help="skip the 50-game §3 rows")
    ap.add_argument("--players", type=int, default=None,
                    help="seats at the table, passed to every run (default: the binary's, two). "
                         "The four-player run is its own table in §3, diffed RE-7 against RE-6, never against a two-player arm")
    ap.add_argument("--deck-size", type=int, default=None,
                    help="cards per deck, passed to every run (default: the binary's, 60). "
                         "100 with --life 40 --players 4 is the Commander-scale board")
    ap.add_argument("--life", type=int, default=None,
                    help="starting life, passed to every run (default: the binary's, 20)")
    ap.add_argument("--out", default=None, help="directory for the raw outputs (default: a temp dir)")
    args = ap.parse_args()

    arms = []
    for spec in args.arm:
        label, _, path = spec.partition("=")
        if not path or not os.path.isfile(path):
            sys.exit(f"--arm {spec!r}: not a file")
        arms.append((label, os.path.abspath(path)))
    labels = [a for a, _ in arms]
    out = args.out or tempfile.mkdtemp(prefix="fuzz_ab_")
    os.makedirs(out, exist_ok=True)
    common = ["--seed", str(args.seed)]
    if args.players is not None:
        common += ["--players", str(args.players)]
    if args.deck_size is not None:
        common += ["--deck-size", str(args.deck_size)]
    if args.life is not None:
        common += ["--life", str(args.life)]
    t0 = time.time()
    print(f"outputs: {out}")

    # ---- counters, both pools, threaded ----------------------------------
    counted = {}   # (arm, pool) -> (counters, stripped text)
    for pool in POOLS:
        for label, path in arms:
            text = run(path, ["--games", str(args.games), "--threads", str(args.threads), "--pool", pool] + common,
                       os.path.join(out, f"counters_{label}_{pool}.txt"))
            counted[(label, pool)] = (counters(text), strip_timing(text))
    for pool in POOLS:
        table(f"=== counters, {pool}, {args.games} games / seed {args.seed} ===", labels,
              {a: counted[(a, pool)][0] for a in labels}, [r for r, _ in ROWS])
        base = counted[(labels[0], pool)][1]
        for a in labels[1:]:
            same = comparable(counted[(a, pool)][1], base) == base
            print(f"  {a} vs {labels[0]} outside Timing: {'IDENTICAL' if same else 'differ'}")
        for a in labels:
            # "0.0" as readily as "0": the two-seat-only rows are averages.
            bad = [
                t for t in ZERO_ROWS
                if counted[(a, pool)][0][t] not in ("0", "0.0", "?")
            ]
            if bad:
                print(f"  !! {a}: {', '.join(bad)} nonzero")

    # ---- reachability -----------------------------------------------------
    if args.require:
        print(f"\n=== --require {args.require}, performance, {args.games} games ===")
        for label, path in arms:
            text = run(path, ["--games", str(args.games), "--threads", str(args.threads), "--require", args.require] + common,
                       os.path.join(out, f"require_{label}.txt"))
            block = text[text.find("=== Reachability"):] if "=== Reachability" in text else "(no reachability block)"
            print(f"-- {label}")
            print("\n".join(l for l in block.splitlines()[1:] if l.strip()))

    # ---- §3 fixture rows, 50 games, threaded ------------------------------
    if not args.no_fixtures:
        fixed = {}
        for pool in POOLS:
            for label, path in arms:
                text = run(path, ["--games", "50", "--threads", str(args.threads), "--pool", pool] + common,
                           os.path.join(out, f"fixture_{label}_{pool}.txt"))
                fixed[(label, pool)] = counters(text)
        for label in labels:
            print(f"\n=== §3 fixture rows, {label}, 50 games / seed {args.seed} ===")
            print("| | performance | stress |")
            print("|---|---|---|")
            for name in FIXTURE_ROWS:
                cells = [fmt(name, fixed[(label, pool)][name]) for pool in POOLS]
                if name in BOLD:
                    print(f"| **{name}** | **{cells[0]}** | **{cells[1]}** |")
                else:
                    print(f"| {name} | {cells[0]} | {cells[1]} |")

    # ---- timing, performance only, serial, interleaved ---------------------
    if args.rounds > 0:
        print(f"\n=== timing, performance, {args.games} games, --threads 1, {args.rounds} interleaved rounds ===")
        cpu = {a: [] for a in labels}
        p50 = {a: [] for a in labels}
        p99 = {a: [] for a in labels}
        turn_p50 = {a: [] for a in labels}
        det_ok = {a: True for a in labels}
        for r in range(1, args.rounds + 1):
            for label, path in arms:
                text = run(path, ["--games", str(args.games), "--threads", "1", "--pool", "performance"] + common,
                           os.path.join(out, f"timing_{label}_r{r}.txt"))
                if strip_timing(text) != counted[(label, "performance")][1]:
                    det_ok[label] = False
                cpu[label].append(float(grab(text, r"^CPU/game:\s+([\d.]+)ms")))
                p50[label].append(float(grab(text, r"^CPU/game tail:\s+([\d.]+) p50")))
                p99[label].append(float(grab(text, r"^CPU/game tail:\s+[\d.]+ p50 / ([\d.]+) p99")))
                turn_p50[label].append(float(grab(text, r"^CPU/turn tail:\s+([\d.]+) p50")))
                print(f"  round {r} {label:<12} CPU/game {cpu[label][-1]:8.2f} ms")
        walks = {a: float(counted[(a, "performance")][0]["Layer walks"]) for a in labels}
        # Questions asked: walks plus memo hits. A binary from before the
        # epoch memo prints no hits row, and every question it asked walked.
        hits = {a: counted[(a, "performance")][0]["Memo hits"] for a in labels}
        queries = {a: walks[a] + (float(hits[a]) if hits[a].isdigit() else 0.0) for a in labels}
        med = {a: statistics.median(cpu[a]) for a in labels}
        per_walk = {a: med[a] / walks[a] * 1000 for a in labels}
        per_query = {a: med[a] / queries[a] * 1000 for a in labels}
        # Item 138's unit. A binary from before the two counters prints no such
        # row, so its cell is `?` rather than a number divided by a zero.
        dec = {a: counted[(a, "performance")][0]["Decisions"] for a in labels}
        per_dec = {
            a: (med[a] / float(dec[a]) * 1000 if dec[a].isdigit() and float(dec[a]) > 0 else None)
            for a in labels
        }
        base = labels[0]
        print(f"\n{'':<22}" + "".join(f"{a:>18}" for a in labels))
        print(f"{'CPU/game median':<22}" + "".join(f"{med[a]:>18.2f}" for a in labels))
        print(f"{'  vs ' + base:<22}" + "".join(f"{(med[a] / med[base] - 1) * 100:>+17.1f}%" for a in labels))
        print(f"{'ms / 1,000 walks':<22}" + "".join(f"{per_walk[a]:>18.3f}" for a in labels))
        print(f"{'  vs ' + base:<22}" + "".join(f"{(per_walk[a] / per_walk[base] - 1) * 100:>+17.1f}%" for a in labels))
        print(f"{'ms / 1,000 queries':<22}" + "".join(f"{per_query[a]:>18.3f}" for a in labels))
        print(f"{'  vs ' + base:<22}" + "".join(f"{(per_query[a] / per_query[base] - 1) * 100:>+17.1f}%" for a in labels))
        print(f"{'µs / decision':<22}" + "".join(
            f"{per_dec[a]:>18.1f}" if per_dec[a] is not None else f"{'?':>18}" for a in labels))
        if per_dec[base] is not None:
            print(f"{'  vs ' + base:<22}" + "".join(
                f"{(per_dec[a] / per_dec[base] - 1) * 100:>+17.1f}%" if per_dec[a] is not None else f"{'?':>18}"
                for a in labels))
        print(f"{'CPU/game p50 median':<22}" + "".join(f"{statistics.median(p50[a]):>18.2f}" for a in labels))
        print(f"{'CPU/game p99 median':<22}" + "".join(f"{statistics.median(p99[a]):>18.2f}" for a in labels))
        print(f"{'CPU/turn p50 median':<22}" + "".join(f"{statistics.median(turn_p50[a]):>18.3f}" for a in labels))
        print(f"{'deterministic':<22}" + "".join(f"{'yes' if det_ok[a] else 'NO':>18}" for a in labels))
        print("\nSpread inside a sitting is ~2-6%; read the counters first, then ms/1,000 walks.")

    print(f"\n{time.time() - t0:.0f}s wall for {len(arms)} arm(s)")


if __name__ == "__main__":
    main()
