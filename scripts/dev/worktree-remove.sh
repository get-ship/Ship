#!/usr/bin/env bash
# Reap the daemon for a worktree, then remove it.
# Accepts a path suffix (e.g. "job/some-lane") or exact worktree path.
set -euo pipefail
cd "$(dirname "$0")/../.."
. scripts/dev/_lib.sh

if [ $# -lt 1 ]; then
    echo "usage: just worktree-remove <slug-or-path> [-- <extra git worktree remove flags>]" >&2
    exit 1
fi

slug="$1"
shift
extra_flags=("$@")

home="${SHIP_HOME:-$HOME/.ship}"
machine="$home/machine"

# Resolve the worktree path. git worktree list outputs: /path  sha  [branch]
target=$(git worktree list | awk '{print $1}' | grep -E "(^|/)${slug}$" | head -1 || true)
if [ -z "$target" ]; then
    # Fall back to substring match (useful for full paths).
    target=$(git worktree list | awk '{print $1}' | grep -F "$slug" | head -1 || true)
fi

if [ -z "$target" ]; then
    echo "worktree-remove: no worktree matching '$slug'" >&2
    echo "  known worktrees:" >&2
    git worktree list >&2
    exit 1
fi

target=$(realpath "$target")
echo "worktree-remove: cleaning up $target"

if [ -d "$machine/daemons" ]; then
    for dir in "$machine/daemons"/*/; do
        [ -d "$dir" ] || continue
        dir="${dir%/}"
        ws_file="$dir/workspace"
        [ -f "$ws_file" ] || continue
        ws=$(cat "$ws_file")
        ws_real=$(realpath "$ws" 2>/dev/null || echo "$ws")
        [ "$ws_real" = "$target" ] || continue

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
                echo "worktree-remove: killed daemon $pid"
            fi
        fi

        rm -rf "$dir"
        echo "worktree-remove: removed daemon dir $(basename "$dir")"
    done
fi

git worktree remove "$target" "${extra_flags[@]}"
echo "worktree-remove: done"
