#!/usr/bin/env python3
"""
fails_before_fix - show a bug fix's test failing without the fix, and passing with it.

    python plans/fails_before_fix.py -- --test phase_12_integration_test some_test_name
    python plans/fails_before_fix.py --pre HEAD~1 -- --lib engine::turns
    python plans/fails_before_fix.py --held mtgsim/src/engine/turns.rs -- --lib engine::turns

Everything after `--` goes to `cargo test`. `--pre` names the tree before the fix, HEAD by
default, for a fix still in the working copy. `--held` names what the fix is, the paths held
back at `--pre`, `mtgsim/src` by default. A unit test lives in `src/`, so a fix with one holds
back only the fixed file, or the test goes with it.

Runs in a detached worktree of its own, so the working copy is never touched and no restore
can go wrong: `git stash` resolved its pathspec against the cwd, took a unit test with the fix,
and `mv` left cargo running the probe. The same worktree then takes the fix and runs the same
tests, because a test failing for some other reason proves nothing. Exit 0 only when at least
one test fails before and every one passes after. A run where nothing compiled, or no test
ran, is refused: a filter matching no test exits 0.
"""

import argparse
import os
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CRATE = "mtgsim"
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


def run_tests(wt, cargo_args):
    """`(exit code, tests run, failed names, output)`, read from one `cargo test`."""
    r = subprocess.run(["cargo", "test", *cargo_args], cwd=os.path.join(wt, CRATE),
                       stdin=subprocess.DEVNULL, capture_output=True, text=True,
                       encoding="utf-8", errors="replace")
    output = r.stdout + r.stderr
    ran = sum(int(n) for n in RAN_RE.findall(output))
    return r.returncode, ran, sorted(set(FAILED_RE.findall(output))), output


def refuse(label, output):
    print(output[-3000:])
    sys.exit(f"REFUSED: {label}. This proves nothing.")


def main():
    parser = argparse.ArgumentParser(description="Show a fix's test failing before and passing after.")
    parser.add_argument("--pre", default="HEAD", help="the tree before the fix")
    parser.add_argument("--held", action="append", help="what the fix is (default: mtgsim/src)")
    parser.add_argument("--dir", default=default_dir(), help="the worktree, kept for a warm build")
    parser.add_argument("cargo_args", nargs="+", help="after --, what `cargo test` runs")
    args = parser.parse_args()
    held = [h.replace("\\", "/") for h in args.held or ["mtgsim/src"]]

    pre = git("rev-parse", args.pre + "^{commit}").strip()
    changed = changed_paths(pre)
    fix = [p for p in changed if is_held(p, held)]
    if not fix:
        sys.exit(f"REFUSED: nothing under {held} differs from {pre[:9]}, so there is no fix to hold back.")

    if os.path.isdir(args.dir):
        git("checkout", "--quiet", "--force", "--detach", pre, cwd=args.dir)
        git("clean", "--quiet", "-fd", cwd=args.dir)
    else:
        git("worktree", "add", "--quiet", "--detach", args.dir, pre)
    overlay([p for p in changed if not is_held(p, held)], args.dir)

    print(f"Without the fix: {pre[:9]}, holding back {len(fix)} file(s) under {held}")
    code, ran, failed, output = run_tests(args.dir, args.cargo_args)
    if code and not failed:
        refuse("the tree without the fix did not compile, or cargo failed before a test ran", output)
    if not ran:
        refuse("no test ran, so the filter matched nothing", output)
    if not failed:
        print(f"NOT SHOWN: all {ran} test(s) pass without the fix.")
        return 1
    print(f"  {len(failed)} of {ran} fail: {', '.join(failed)}")

    overlay(fix, args.dir)
    print("With the fix:")
    code, ran_after, failed_after, output = run_tests(args.dir, args.cargo_args)
    if code and not failed_after:
        refuse("the tree with the fix did not compile, or cargo failed before a test ran", output)
    if failed_after:
        print(f"NOT SHOWN: still failing with the fix: {', '.join(failed_after)}")
        return 1
    print(f"  all {ran_after} pass. Shown.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
