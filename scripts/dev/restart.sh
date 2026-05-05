#!/usr/bin/env bash
# Stop, rebuild, re-stash, restart.
#
# After stopping the daemon at the resolved current daemon dir, walk
# ~/.ship/machine/daemons/*/ and prune orphans whose recorded `workspace`
# matches THIS worktree but whose dir != the dir we just resolved. Catches
# stale daemons left over from before per-worktree-isolation landed (when
# linked worktrees redirected to the main checkout's daemon dir).
#
# Also reaps any vite processes running from a deleted worktree path.
set -euo pipefail
cd "$(dirname "$0")/../.."
. scripts/dev/_lib.sh

# Best-effort stop of daemon + vite for this worktree. Ignore failures.
"$(ship_bin)" daemon stop || true
kill_vite || true

# Wait for the pid file to drop so we don't race the next start.
d=$(daemon_dir)
deadline=$(( $(date +%s) + 5 ))
while [ -f "$d/pid" ] && [ "$(date +%s)" -lt "$deadline" ]; do sleep 0.1; done

# Orphan cleanup: any daemon dir under machine/daemons/ whose recorded
# workspace is THIS worktree but whose dir != $d is a leftover from before
# the per-worktree-isolation fix. Kill its pid (if alive), kill its vite,
# and remove the dir.
home="${SHIP_HOME:-$HOME/.ship}"
machine="$home/machine"
this_root=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
this_root=$(realpath "$this_root")
current=$(realpath "$d" 2>/dev/null || echo "$d")

if [ -d "$machine/daemons" ]; then
    for orphan in "$machine/daemons"/*/; do
        [ -d "$orphan" ] || continue
        orphan="${orphan%/}"
        [ "$(realpath "$orphan")" = "$current" ] && continue

        ws_file="$orphan/workspace"
        [ -f "$ws_file" ] || continue
        ws=$(cat "$ws_file")
        [ -n "$ws" ] || continue
        ws_real=$(realpath "$ws" 2>/dev/null || echo "$ws")
        [ "$ws_real" = "$this_root" ] || continue

        # Stale daemon claiming this worktree from a different dir → prune.
        pid_file="$orphan/pid"
        if [ -f "$pid_file" ]; then
            pid=$(cat "$pid_file")
            if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
                kill "$pid" 2>/dev/null || true
                kdeadline=$(( $(date +%s) + 3 ))
                while kill -0 "$pid" 2>/dev/null && [ "$(date +%s)" -lt "$kdeadline" ]; do
                    sleep 0.1
                done
                kill -9 "$pid" 2>/dev/null || true
            fi
        fi
        # Reap any tracked vite before removing the dir.
        vite_pf="$orphan/vite.pid"
        if [ -f "$vite_pf" ]; then
            vpid=$(cat "$vite_pf")
            if [ -n "$vpid" ] && kill -0 "$vpid" 2>/dev/null; then
                kill "$vpid" 2>/dev/null || true
                kvd=$(( $(date +%s) + 2 ))
                while kill -0 "$vpid" 2>/dev/null && [ "$(date +%s)" -lt "$kvd" ]; do
                    sleep 0.1
                done
                kill -9 "$vpid" 2>/dev/null || true
            fi
        fi
        rm -rf "$orphan"
        echo "dev-restart: pruned orphan daemon $(basename "$orphan") (workspace=$ws)"
    done
fi

# Global pass: reap any vite running from a path that no longer exists.
# This catches vites from deleted worktrees that don't share this workspace.
reap_orphan_vites

exec scripts/dev/start.sh
