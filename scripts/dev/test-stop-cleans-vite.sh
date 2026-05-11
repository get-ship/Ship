#!/usr/bin/env bash
# Assert: kill_vite (called by stop.sh) kills the tracked vite process
# and removes the pid file.
set -euo pipefail
cd "$(dirname "$0")/../.."
. scripts/dev/_lib.sh

d=$(daemon_dir)
pf=$(vite_pid_file)
mkdir -p "$d"

# Register a fake vite process (sleep stands in for pnpm/vite).
sleep 999 &
fake_pid=$!
echo "$fake_pid" > "$pf"
echo "test: registered fake vite pid $fake_pid in $pf"

kill_vite

if kill -0 "$fake_pid" 2>/dev/null; then
    kill "$fake_pid" 2>/dev/null || true
    echo "FAIL: process $fake_pid still alive after kill_vite" >&2
    exit 1
fi

if [ -f "$pf" ]; then
    echo "FAIL: pid file $pf not removed after kill_vite" >&2
    exit 1
fi

echo "PASS"
