#!/bin/bash
# launch_prof.sh <log under ~> <crate dir under /mnt/c> <arm dir under ~> <board flags...>
#
# Starts prof_arm.sh detached from the calling shell, so an arm can outlast
# the ten-minute tool call that launched it, and appends `PROF_DONE exit=N`
# to the log when it ends. Two arms may run at once; neither count moves.
# Wait on the log, never on a process name: `pkill -f` from a `wsl -e bash
# -lc` call matches that call's own command line.
set -euo pipefail
LOG="$HOME/$1"; shift
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
setsid nohup bash -c 'bash "$1" "${@:3}" > "$2" 2>&1; echo "PROF_DONE exit=$?" >> "$2"' \
    _ "$HERE/prof_arm.sh" "$LOG" "$@" > /dev/null 2>&1 < /dev/null &
disown
echo "launched: $LOG"
