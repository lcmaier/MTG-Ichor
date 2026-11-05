#!/usr/bin/env python3
"""
fails_before_fix - show a bug fix's test failing without the fix, and passing with it.

    python plans/fails_before_fix.py -- --test phase_12_integration_test some_test_name
    python plans/fails_before_fix.py --pre HEAD~1 -- --lib engine::turns
    python plans/fails_before_fix.py --held mtgsim/src/engine/turns.rs -- --lib engine::turns
    python plans/fails_before_fix.py --crate devgui --held devgui/src -- --test tools

Everything after `--` goes to `cargo test`, run in `--crate`, `mtgsim` by default. `--pre` names
the tree before the fix, HEAD by default, for a fix still in the working copy. `--held` names
what the fix is, the paths held back at `--pre`, the crate's `src` by default. A unit test lives
in `src/`, so a fix with one holds back only the fixed file, or the test goes with it.

Runs in a detached worktree of its own, so the working copy is never touched. The same worktree
then takes the fix and reruns, because a test failing for some other reason proves nothing. Exit
0 only when the run fails before and every test passes after. A panic, an abort such as a stack
overflow, and a run past `--timeout` all count as failing. A tree that does not compile, or a
filter matching no test, is refused on either side: a filter matching nothing exits 0.
"""

import argparse
import os
import re
import shutil
import signal
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
RAN_RE = re.compile(r"^running (\d+) tests?$", re.M)
FAILED_RE = re.compile(r"^test (\S+)(?: - .*)? \.\.\. FAILED$", re.M)


def git(*args, cwd=ROOT):
    r = subprocess.run(["git", *args], cwd=cwd, stdin=subprocess.DEVNULL, capture_output=True,
                       text=True, encoding="utf-8", errors="replace")
    if r.returncode:
        sys.exit(f"git {' '.join(args[:3])} exited {r.returncode}:\n{r.stderr[-1000:]}")
    return r.stdout


def default_dir():
    # Deep paths break deletion on this machine, so a Windows worktree sits at a short one.
    return r"C:\w\fix-proof" if os.name == "nt" and os.path.isdir(r"C:\w") else os.path.join(
        tempfile.gettempdir(), "ichor-fix-proof")


def is_held(path, held):
    return any(path == h or path.startswith(h.rstrip("/") + "/") for h in held)


def changed_paths(pre):
    """Every path where the working copy differs from `pre`: tracked changes and new files."""
    tracked = git("diff", "--name-only", pre).splitlines()
    new = git("ls-files", "--others", "--exclude-standard").splitlines()
    return sorted(set(tracked) | set(new))


def overlay(paths, wt):
    """Copy `paths` from the working copy into `wt`, deleting one the working copy deleted.
    `copy`, not `copy2`: a fresh mtime is what tells cargo the file changed."""
    for path in paths:
        source, target = os.path.join(ROOT, path), os.path.join(wt, path)
        if os.path.exists(source):
            os.makedirs(os.path.dirname(target), exist_ok=True)
            shutil.copy(source, target)
        elif os.path.exists(target):
            os.remove(target)


def refuse(label, output=""):
    print(output[-3000:])
    sys.exit(f"REFUSED: {label}. This proves nothing.")


def prepare(worktree, pre):
    """Check out `pre` in `worktree`. A path that is not one of this repository's own worktrees
    is refused, since this forces a checkout and cleans whatever it is pointed at."""
    wt = Path(worktree).resolve()
    if wt.is_relative_to(Path(ROOT).resolve()):
        refuse(f"--dir {wt} is inside the working copy")
    if not wt.exists():
        git("worktree", "add", "--quiet", "--detach", str(wt), pre)
        return
    listed = re.findall(r"^worktree (.+)$", git("worktree", "list", "--porcelain"), re.M)
    if wt not in {Path(path).resolve() for path in listed}:
        refuse(f"--dir {wt} exists and is not a worktree of this repository")
    git("checkout", "--quiet", "--force", "--detach", pre, cwd=str(wt))
    git("clean", "--quiet", "-fd", cwd=str(wt))


def kill_tree(process):
    """A hung test is cargo's child, and killing cargo alone leaves it holding the pipes."""
    if os.name == "nt":
        subprocess.run(["taskkill", "/T", "/F", "/PID", str(process.pid)], capture_output=True)
    else:
        os.killpg(process.pid, signal.SIGKILL)


def run_tests(wt, crate, cargo_args, timeout):
    """`(tests run, what failed, output)`. Built first, so the timeout covers the run alone and a
    tree that does not compile is told apart from a test that aborted."""
    cwd = os.path.join(wt, crate)
    built = subprocess.run(["cargo", "test", "--no-run", *cargo_args], cwd=cwd, stdin=subprocess.DEVNULL,
                           capture_output=True, text=True, encoding="utf-8", errors="replace")
    if built.returncode:
        refuse("the tree did not compile", built.stdout + built.stderr)

    process = subprocess.Popen(["cargo", "test", *cargo_args], cwd=cwd, stdin=subprocess.DEVNULL,
                               stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
                               encoding="utf-8", errors="replace", start_new_session=os.name != "nt")
    try:
        output = process.communicate(timeout=timeout)[0]
    except subprocess.TimeoutExpired:
        kill_tree(process)
        output = process.communicate()[0]
        return max(1, ran_in(output)), [f"(still running after {timeout} s)"], output

    ran, failed = ran_in(output), sorted(set(FAILED_RE.findall(output)))
    if process.returncode and not ran:
        refuse("cargo failed before a test ran", output)
    if process.returncode and not failed:
        failed = ["(the test binary aborted)"]
    return ran, failed, output


def ran_in(output):
    return sum(int(n) for n in RAN_RE.findall(output))


def main():
    parser = argparse.ArgumentParser(description="Show a fix's test failing before and passing after.")
    parser.add_argument("--pre", default="HEAD", help="the tree before the fix")
    parser.add_argument("--crate", default="mtgsim", help="where `cargo test` runs")
    parser.add_argument("--held", action="append", help="what the fix is (default: the crate's src)")
    parser.add_argument("--dir", default=default_dir(), help="the worktree, kept for a warm build")
    parser.add_argument("--timeout", type=int, default=600, help="seconds a test run may take")
    parser.add_argument("cargo_args", nargs="+", help="after --, what `cargo test` runs")
    args = parser.parse_args()
    held = [h.replace("\\", "/") for h in args.held or [f"{args.crate}/src"]]

    pre = git("rev-parse", args.pre + "^{commit}").strip()
    changed = changed_paths(pre)
    fix = [p for p in changed if is_held(p, held)]
    if not fix:
        refuse(f"nothing under {held} differs from {pre[:9]}, so there is no fix to hold back")

    prepare(args.dir, pre)
    overlay([p for p in changed if not is_held(p, held)], args.dir)

    print(f"Without the fix: {pre[:9]}, holding back {len(fix)} file(s) under {held}")
    ran, failed, output = run_tests(args.dir, args.crate, args.cargo_args, args.timeout)
    if not ran:
        refuse("no test ran, so the filter matched nothing", output)
    if not failed:
        print(f"NOT SHOWN: all {ran} test(s) pass without the fix.")
        return 1
    print(f"  {len(failed)} of {ran} fail: {', '.join(failed)}")

    overlay(fix, args.dir)
    print("With the fix:")
    ran, failed, output = run_tests(args.dir, args.crate, args.cargo_args, args.timeout)
    if not ran:
        refuse("no test ran with the fix, so the test the fix proves is gone", output)
    if failed:
        print(f"NOT SHOWN: still failing with the fix: {', '.join(failed)}")
        return 1
    print(f"  all {ran} pass. Shown.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
