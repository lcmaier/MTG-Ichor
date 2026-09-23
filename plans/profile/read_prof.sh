#!/bin/bash
# read_prof.sh <arm dir under ~> [rows, default 40]
#
# The reading of one arm: its board, the program's total instruction count,
# and the top inclusive rows of `callgrind_annotate --inclusive=yes`,
# demangled, one row per function (the first, which is the largest; a
# function inlined into several files is listed once per file).
set -euo pipefail
cd "$HOME/$1"
# Annotated once per run, and again whenever cg.out is newer than the annotation.
[ incl.txt -nt cg.out ] || callgrind_annotate --inclusive=yes cg.out 2>/dev/null | c++filt > incl.txt
cat board.txt
grep -m1 "PROGRAM TOTALS" incl.txt
grep -E '^\s*[0-9,]+ \([ 0-9.]+%\)' incl.txt \
    | sed -E 's/ \[[^]]*\]$//' \
    | awk -F: '{ key = $NF; if (!seen[key]++) print }' \
    | head -n "${2:-40}"
