#!/usr/bin/env bash
# Remove stash binaries for any daemons that aren't currently running.
# Leaves the singleton machine/daemon/ alone (production-installed shipd).
# Per devx-05, do NOT touch ~/.ship/library/ — that's portable user state.
set -euo pipefail

home="${SHIP_HOME:-$HOME/.ship}"
daemons="$home/machine/daemons"

if [ ! -d "$daemons" ]; then
    echo "no per-worktree daemons to clean"
    exit 0
fi

cleaned=0
for d in "$daemons"/*/; do
    [ -d "$d" ] || continue
    pid=$( [ -f "$d/pid" ] && cat "$d/pid" || echo "" )
    if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
        continue   # still running
    fi
    if [ -d "$d/.bin" ]; then
        rm -rf "$d/.bin"
        cleaned=$((cleaned + 1))
        echo "cleaned stash: ${d}.bin"
    fi
done

echo "cleaned $cleaned stash dir(s)"
