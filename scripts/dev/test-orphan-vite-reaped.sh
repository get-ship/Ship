#!/usr/bin/env bash
# Assert: reap_orphan_vites kills a vite whose cwd (worktree) was deleted.
set -euo pipefail
cd "$(dirname "$0")/../.."
. scripts/dev/_lib.sh

home="${SHIP_HOME:-$HOME/.ship}"
machine="$home/machine"
mkdir -p "$machine/daemons"

# Create a fake daemon dir representing a now-deleted worktree.
fake_dir=$(mktemp -d "$machine/daemons/test-XXXXXX")
fake_workspace=$(mktemp -d)
echo "$fake_workspace" > "$fake_dir/workspace"

# Launch a process from the fake workspace directory (simulates vite's cwd).
(cd "$fake_workspace" && exec sleep 999) &
fake_pid=$!
echo "$fake_pid" > "$fake_dir/vite.pid"
echo "test: fake vite pid $fake_pid running from $fake_workspace"

# Delete the workspace to simulate a removed worktree.
rm -rf "$fake_workspace"
echo "test: deleted $fake_workspace"

# Give /proc a moment to reflect the deletion.
sleep 0.2

reap_orphan_vites

# Cleanup the fake daemon dir regardless.
rm -rf "$fake_dir"

if kill -0 "$fake_pid" 2>/dev/null; then
    kill "$fake_pid" 2>/dev/null || true
    echo "FAIL: orphan vite $fake_pid was not reaped" >&2
    exit 1
fi

echo "PASS"
