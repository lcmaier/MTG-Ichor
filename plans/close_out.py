#!/usr/bin/env python3
"""
close_out - the measuring half of the close-out (engineering-practices.md §3.1), one command.

    python plans/close_out.py --arm main=origin/main --arm engine=<sha> [--arm shipped=HEAD] \
                              [--require "Card A,Card B"] [--fixtures] [--cpu LABEL | --no-cpu]

Builds each arm in its own worktree and target dir, refusing two identical binaries (a stale
or shared build reads IDENTICAL having measured nothing); runs `fuzz_ab.py --rounds 0` on both
pools, audited, at two seats and four; reads callgrind instructions per decision, the first arm
against `--cpu`; runs `--require` on the last arm; prints the `fuzz-record.md` block's tables.
"""

import argparse
import hashlib
import os
import re
import subprocess
import sys
import tempfile
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import fuzz_ab  # noqa: E402  (its rows, its stripping and its compare, so the two cannot drift)

#: §3's cost model and the dispatcher's work: rows a change may move without
#: the game moving. Every other row is gameplay.
COST = fuzz_ab.BOLD | {"Windows past gate", "Candidate visits"}
#: Commander scale, TR-1b's callgrind board, cut to under a minute an arm.
CPU_GAMES = 20
CPU_BOARD = f"--games {CPU_GAMES} --seed 12345 --pool performance --players 4 --deck-size 100 --life 40"
WSL = ["wsl", "-d", "Ubuntu-24.04", "-e", "bash", "-lc"] if os.name == "nt" else ["bash", "-lc"]


def sh(cmd, **kw):
    r = subprocess.run(cmd, stdin=subprocess.DEVNULL, capture_output=True, text=True,
                       encoding="utf-8", errors="replace", **kw)
    if r.returncode:
        sys.exit(f"{' '.join(map(str, cmd[:4]))} exited {r.returncode}:\n{(r.stdout + r.stderr)[-2000:]}")
    return r.stdout


def unix(path):
    p = os.path.abspath(path).replace("\\", "/")
    return f"/mnt/{p[0].lower()}{p[2:]}" if os.name == "nt" else p


def build(label, rev, arms_dir):
    sha = sh(["git", "rev-parse", rev + "^{commit}"], cwd=HERE).strip()
    wt = os.path.join(arms_dir, label)
    if os.path.isdir(wt):
        sh(["git", "-C", wt, "checkout", "--quiet", "--force", "--detach", sha])
    else:
        sh(["git", "worktree", "add", "--quiet", "--detach", wt, sha], cwd=HERE)
    target = os.path.join(wt, "mtgsim", "target")
    sh(["cargo", "build", "--release", "--bin", "fuzz_games"], cwd=os.path.join(wt, "mtgsim"),
       env=dict(os.environ, CARGO_TARGET_DIR=target))
    exe = os.path.join(target, "release", "fuzz_games" + (".exe" if os.name == "nt" else ""))
    pool = re.search(r"^Card pool: .*$", sh([exe, "--games", "0"]), re.M)
    print(f"  {label:<10} {sha[:9]}  {pool.group(0) if pool else '(no pool line)'}")
    return label, exe, wt


def cpu(pair):
    """Instructions per decision for each arm, both under callgrind at once, the hasher's seed pinned."""
    runs = {label: subprocess.Popen(
        WSL + [f'MTGSIM_HASH_SEED=1 bash "{unix(HERE)}/profile/prof_arm.sh" "{unix(wt)}/mtgsim" close-out-{label} {CPU_BOARD}'],
        stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
        encoding="utf-8", errors="replace") for label, _, wt in pair}
    out = []
    for label, p in runs.items():
        log = p.communicate()[0]
        if "COUNTERS_IDENTICAL_NATIVE_VS_VALGRIND" not in log:
            sys.exit(f"cpu, {label}: the native and valgrind runs disagree, so there is no reading:\n{log}")
        ir = int(re.search(r"I\s+refs:\s+([\d,]+)", log).group(1).replace(",", ""))
        run = sh(WSL + [f"cat ~/close-out-{label}/run.txt"])
        out.append(ir / (int(re.search(r"^\s+Decisions:\s+(\d+)", run, re.M).group(1)) * CPU_GAMES))
    return out


def table(arms, out):
    base = arms[0][0]

    def gameplay(text, stripped_base):
        """The run outside `=== Timing ===`, less rows the baseline never prints and the cost rows."""
        kept = fuzz_ab.comparable(fuzz_ab.strip_timing(text), stripped_base).split("\n")
        return [l for l in kept if not ((m := fuzz_ab.ROW_LABEL.match(l)) and m.group(1) in COST)]
    def read(seats, label, pool):
        with open(os.path.join(out, f"{seats}p", f"counters_{label}_{pool}.txt"), encoding="utf-8", errors="replace") as f:
            return f.read()
    def cells(seats, label, fn):
        return " / ".join(fn(read(seats, base, p), read(seats, label, p)) for p in fuzz_ab.POOLS)
    def verdict(b_text, a_text):
        b, a, stripped = fuzz_ab.counters(b_text), fuzz_ab.counters(a_text), fuzz_ab.strip_timing(b_text)
        if gameplay(a_text, stripped) == gameplay(b_text, stripped):
            return "**IDENTICAL**"
        return "differ: " + (", ".join(r for r, _ in fuzz_ab.ROWS if r not in COST and "?" != b[r] != a[r]) or "a line outside the rows")
    def audit(_, a_text):
        m = re.search(r"^Audit:\s+(\d+) dispatches", a_text, re.M)
        return f"{int(m.group(1)):,}" if m else "not audited"

    print("\n| | 2 seats | 4 seats |\n|---|---|---|")
    for label, _, _ in arms[1:]:
        print(f"| gameplay rows, {label} vs {base}, performance / stress | {cells(2, label, verdict)} | {cells(4, label, verdict)} |")
        for row in (r for r, _ in fuzz_ab.ROWS if r in COST):
            move = lambda b, a: f"{fuzz_ab.fmt(row, fuzz_ab.counters(b)[row])} → {fuzz_ab.fmt(row, fuzz_ab.counters(a)[row])}"
            two, four = cells(2, label, move), cells(4, label, move)
            if any(x.split(" → ")[0] != x.split(" → ")[1] for x in (two + " / " + four).split(" / ")):
                print(f"| `{row}`, {base} → {label}, performance / stress | {two} | {four} |")
        print(f"| audit, {label}, performance / stress, dispatches agreed | {cells(2, label, audit)} | {cells(4, label, audit)} |")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--arm", action="append", required=True, metavar="LABEL=REV", help="repeatable; the first is the baseline")
    ap.add_argument("--require", help="the reachability read's cards, as fuzz_games takes them")
    ap.add_argument("--fixtures", action="store_true", help="also the 50-game §3 rows, which a pool change re-records")
    ap.add_argument("--cpu", metavar="LABEL", help="the arm the budget reads (default: the second)")
    ap.add_argument("--no-cpu", action="store_true", help="skip callgrind")
    # A short path: cargo's target tree overruns Windows' path limit under the scratchpad.
    ap.add_argument("--arms-dir", default="C:/w/arms" if os.name == "nt" else "/tmp/arms")
    ap.add_argument("--out", help="the raw outputs (default: a temp dir)")
    args = ap.parse_args()
    t0 = time.time()
    specs = [s.partition("=") for s in args.arm]
    if len(specs) < 2 or any(not rev for _, _, rev in specs):
        sys.exit("--arm LABEL=REV, at least two")
    out = args.out or tempfile.mkdtemp(prefix="close_out_")
    print(f"outputs: {out}\nbuilding {len(specs)} arms under {args.arms_dir}")
    arms = [build(label, rev, args.arms_dir) for label, _, rev in specs]
    digests = [hashlib.sha256(open(exe, "rb").read()).hexdigest() for _, exe, _ in arms]
    if len(set(digests)) < len(digests):
        sys.exit("two arms built the same binary: check the revs, and that no target dir is shared")
    for seats in (2, 4):
        cmd = [sys.executable, os.path.join(HERE, "fuzz_ab.py"), "--rounds", "0", "--out", os.path.join(out, f"{seats}p")]
        cmd += [f"--arm={label}={exe}" for label, exe, _ in arms]
        cmd += ["--players", "4", "--no-fixtures"] if seats == 4 else ([] if args.fixtures else ["--no-fixtures"])
        text = sh(cmd)
        if args.fixtures and seats == 2:
            print(text[text.find(f"=== §3 fixture rows, {arms[-1][0]}"):].split("\n\n")[0])
    table(arms, out)
    if not args.no_cpu:
        target = args.cpu or arms[1][0]
        b, a = cpu([arms[0]] + [arm for arm in arms if arm[0] == target])
        print(f"| instructions / decision, {target} vs {arms[0][0]}, callgrind, `{CPU_BOARD}` | | "
              f"{b / 1e6:.4f} M → {a / 1e6:.4f} M, **{(a / b - 1) * 100:+.2f}%** |")
    for seats in (2, 4) if args.require else ():
        flags = [arms[-1][1], "--games", "200", "--seed", "12345", "--require", args.require] + (["--players", "4"] if seats == 4 else [])
        first = subprocess.run(flags, capture_output=True, text=True, encoding="utf-8", errors="replace")
        # fuzz_games refuses a registered card the pool does not hold, and says which pool does.
        text = sh(flags + ["--pool", "stress"]) if "Use --pool stress" in first.stdout + first.stderr else first.stdout
        print(f"\nreachability, {arms[-1][0]}, {seats} seats, 200 games:\n" + text[text.find("=== Reachability"):].split("\n\n")[0].rstrip())
    print(f"\n{time.time() - t0:.0f}s wall")


if __name__ == "__main__":
    main()
