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
- **Fixture rows** for `engineering-practices.md` §3 are the same counters at
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
"registered but not pooled" arm that should reproduce `main` is checked by
construction.

Reads the counters, not the milliseconds: the timing table prints ms per
1,000 walks beside the median, because "the game got longer" and "the walk
got slower" are different findings.
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
    ("P0 / P1", None),
    ("Avg turns", r"^Avg turns/game:\s+([\d.]+)"),
    ("Max turns", r"^Max turns seen:\s+(\d+)"),
    ("Spells cast", r"^\s+Spells cast:\s+([\d.]+)"),
    ("Lands played", r"^\s+Lands played:\s+([\d.]+)"),
    ("Combat w/ atk", r"^\s+Combat w/ atk:\s+([\d.]+)"),
    ("Creatures died", r"^\s+Creatures died:\s+([\d.]+)"),
    ("Damage events", r"^\s+Damage events:\s+([\d.]+)"),
    ("Total damage", r"^\s+Total damage:\s+([\d.]+)"),
    ("Life changes", r"^\s+Life changes:\s+([\d.]+)"),
    ("Layer walks", r"^\s+Layer walks:\s+(\d+)"),
    ("Layer frames", r"^\s+Layer frames:\s+(\d+)"),
    ("Frames/walk", r"^\s+Frames/walk:\s+([\d.]+)"),
    ("Replacement gathers", r"^\s+Replacement gathers:\s+(\d+)"),
    ("Restriction queries", r"^\s+Restriction queries:\s+(\d+)"),
]
THRESHOLDS = ["Errors", "Panics", "Uncast resolved", "Hit turn limit"]
# The §3 table's rows, in its order; the four thresholds stay out of it.
FIXTURE_ROWS = [r for r, _ in ROWS if r not in THRESHOLDS and r != "Max turns"]
BOLD = {"Layer walks", "Layer frames", "Frames/walk", "Replacement gathers", "Restriction queries"}


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


def wins(text):
    p0 = re.search(r"^\s+P0 wins\s+(\d+) \(([\d.]+)%\)", text, re.M)
    p1 = re.search(r"^\s+P1 wins\s+(\d+) \(([\d.]+)%\)", text, re.M)
    if not (p0 and p1):
        return "?"
    return f"{p0.group(1)} ({p0.group(2)}%) / {p1.group(1)} ({p1.group(2)}%)"


def counters(text):
    return {name: (wins(text) if pat is None else grab(text, pat)) for name, pat in ROWS}


def fmt(name, value):
    if name in ("Layer walks", "Layer frames") and value.isdigit():
        return f"{int(value):,}"
    return value


def table(title, arms, rows_by_arm, rows):
    width = max(len(a) for a in arms) + 2
    print(f"\n{title}")
    print(f"{'':<22}" + "".join(f"{a:>{max(width, 18)}}" for a in arms))
    for name in rows:
        print(f"{name:<22}" + "".join(f"{fmt(name, rows_by_arm[a][name]):>{max(width, 18)}}" for a in arms))


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
            same = counted[(a, pool)][1] == base
            print(f"  {a} vs {labels[0]} outside Timing: {'IDENTICAL' if same else 'differ'}")
        for a in labels:
            bad = [t for t in THRESHOLDS if counted[(a, pool)][0][t] not in ("0", "?")]
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
        med = {a: statistics.median(cpu[a]) for a in labels}
        per_walk = {a: med[a] / walks[a] * 1000 for a in labels}
        base = labels[0]
        print(f"\n{'':<22}" + "".join(f"{a:>18}" for a in labels))
        print(f"{'CPU/game median':<22}" + "".join(f"{med[a]:>18.2f}" for a in labels))
        print(f"{'  vs ' + base:<22}" + "".join(f"{(med[a] / med[base] - 1) * 100:>+17.1f}%" for a in labels))
        print(f"{'ms / 1,000 walks':<22}" + "".join(f"{per_walk[a]:>18.3f}" for a in labels))
        print(f"{'  vs ' + base:<22}" + "".join(f"{(per_walk[a] / per_walk[base] - 1) * 100:>+17.1f}%" for a in labels))
        print(f"{'CPU/game p50 median':<22}" + "".join(f"{statistics.median(p50[a]):>18.2f}" for a in labels))
        print(f"{'CPU/game p99 median':<22}" + "".join(f"{statistics.median(p99[a]):>18.2f}" for a in labels))
        print(f"{'CPU/turn p50 median':<22}" + "".join(f"{statistics.median(turn_p50[a]):>18.3f}" for a in labels))
        print(f"{'deterministic':<22}" + "".join(f"{'yes' if det_ok[a] else 'NO':>18}" for a in labels))
        print("\nSpread inside a sitting is ~2-6%; read the counters first, then ms/1,000 walks.")

    print(f"\n{time.time() - t0:.0f}s wall for {len(arms)} arm(s)")


if __name__ == "__main__":
    main()
