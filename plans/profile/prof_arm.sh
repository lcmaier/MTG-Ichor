#!/bin/bash
# prof_arm.sh <crate dir under /mnt/c> <arm dir under ~> <fuzz_games board flags...>
#
# One callgrind arm, run inside WSL's Ubuntu-24.04 (engineering-practices.md
# §3's instruction-count instrument). Copies the crate into the arm directory,
# builds fuzz_games with debug info, plays the board natively and then under
# callgrind, and says whether the two agree outside `=== Timing ===`: an
# instruction count is comparable only between identical games, so a run
# where they differ is not a reading.
#
# The board is every flag after the second argument, passed to both runs
# with `--threads 1`. The Commander-scale baseline in fuzz-record.md is
#
#   prof_arm.sh "/mnt/c/.../mtgsim" mtgsim-prof-new \
#       --games 200 --seed 12345 --pool performance --players 4 --deck-size 100 --life 40
#
# An arm outlasts a ten-minute tool call, so start it with launch_prof.sh.
set -uo pipefail
SRC="$1"; DEST="$HOME/$2"; shift 2
if [ "$#" -eq 0 ]; then
    echo "no board: pass fuzz_games flags, e.g. --games 200 --seed 12345 --pool stress --players 4" >&2
    exit 2
fi
mkdir -p "$DEST"
# The sources are replaced rather than overlaid, and extracted with fresh
# mtimes (-m): tar keeps the archive's, and a source older than the arm's
# last build is one cargo would not recompile.
rm -rf "$DEST/src" "$DEST/tests" "$DEST/benches"
(cd "$SRC" && tar --exclude=./target -cf - .) | tar -xmf - -C "$DEST"
cd "$DEST" || exit 1
# A reading derived from an earlier run in this directory must not survive it.
rm -f board.txt native.txt run.txt cg.out cg.log incl.txt
echo "board: $*" > board.txt
CARGO_PROFILE_RELEASE_DEBUG=2 cargo build --release --bin fuzz_games 2>&1 | tail -n 2
./target/release/fuzz_games --threads 1 "$@" > native.txt 2>&1
echo "native exit $?"
valgrind --tool=callgrind --callgrind-out-file=cg.out ./target/release/fuzz_games --threads 1 "$@" > run.txt 2> cg.log
echo "callgrind exit $?"
tail -n 3 cg.log
if diff <(sed '/^=== Timing ===$/,/^$/d' native.txt) <(sed '/^=== Timing ===$/,/^$/d' run.txt); then
    echo "COUNTERS_IDENTICAL_NATIVE_VS_VALGRIND"
fi
