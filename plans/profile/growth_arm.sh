#!/bin/bash
# growth_arm.sh <arm dir under ~>
#
# The growth gate's reading (engineering-practices.md#the-growth-gate), run
# inside WSL's Ubuntu-24.04 on an arm prof_arm.sh built. Each ladder shape at
# each rung, one callgrind run apiece and all at once, counting only what
# `ladder_step` executes, so a board's setup is off the count and the number
# repeats exactly. Prints one `GROWTH|shape|rung|instructions` line a read, for
# close_out.py to judge. A read past a minute prints no instructions, which
# close_out.py refuses: a read that slow is the bug this exists to catch, and a
# killed callgrind still prints the partial count it had, which reads as healthy.
set -uo pipefail
cd "$HOME/$1" || exit 1
EXE=./target/release/fuzz_games
RUNGS="1600 6400"
STEPS=2
OUT=$(mktemp -d)

read_one() {
    local shape="$1" rung="$2" log="$3"
    if MTGSIM_HASH_SEED=1 timeout 60 valgrind --tool=callgrind --callgrind-out-file=/dev/null --toggle-collect='*ladder_step*' \
        "$EXE" --ladder --ladder-shape "$shape" --ladder-rung "$rung" --ladder-steps "$STEPS" --threads 1 --seed 1 \
        > /dev/null 2> "$log"; then
        echo "GROWTH|$shape|$rung|$(sed -n 's/.*Collected : //p' "$log")"
    else
        echo "GROWTH|$shape|$rung|"
    fi
}

n=0
while IFS= read -r shape; do
    for rung in $RUNGS; do
        n=$((n + 1))
        read_one "$shape" "$rung" "$OUT/$n.log" &
    done
done < <("$EXE" --ladder --ladder-rung 100 --ladder-steps 1 --threads 1 --seed 1 | sed -n '4,$p' | cut -c1-22 | sed 's/ *$//')
wait
rm -rf "$OUT"
