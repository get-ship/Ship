#!/usr/bin/env bash
# Kill every daemon + vite tracked under ~/.ship/machine/daemons/.
# Use this to get a clean slate across all worktrees.
set -euo pipefail
cd "$(dirname "$0")/../.."
. scripts/dev/_lib.sh

home="${SHIP_HOME:-$HOME/.ship}"
machine="$home/machine"

if [ ! -d "$machine/daemons" ]; then
    echo "dev-stop-all: no daemon dirs found"
    exit 0
fi

killed=0
for dir in "$machine/daemons"/*/; do
    [ -d "$dir" ] || continue
    dir="${dir%/}"

    pid_file="$dir/pid"
    if [ -f "$pid_file" ]; then
        pid=$(cat "$pid_file")
        if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
            kill "$pid" 2>/dev/null || true
            kd=$(( $(date +%s) + 3 ))
            while kill -0 "$pid" 2>/dev/null && [ "$(date +%s)" -lt "$kd" ]; do
                sleep 0.1
            done
            kill -9 "$pid" 2>/dev/null || true
            echo "dev-stop-all: killed daemon $pid ($(basename "$dir"))"
            killed=$(( killed + 1 ))
        fi
        rm -f "$pid_file"
    fi

    vite_pf="$dir/vite.pid"
    if [ -f "$vite_pf" ]; then
        vpid=$(cat "$vite_pf")
        if [ -n "$vpid" ] && kill -0 "$vpid" 2>/dev/null; then
            kill "$vpid" 2>/dev/null || true
            kd=$(( $(date +%s) + 3 ))
            while kill -0 "$vpid" 2>/dev/null && [ "$(date +%s)" -lt "$kd" ]; do
                sleep 0.1
            done
            kill -9 "$vpid" 2>/dev/null || true
            echo "dev-stop-all: killed vite $vpid ($(basename "$dir"))"
            killed=$(( killed + 1 ))
        fi
        rm -f "$vite_pf"
    fi
done

echo "dev-stop-all: done ($killed process(es) killed)"
