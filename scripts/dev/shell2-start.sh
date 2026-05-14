#!/usr/bin/env bash
# Launch the shell2 Vite dev server for this worktree and track its PID.
# Daemon must already be running. Ctrl-C kills vite and removes the pid file.
set -euo pipefail
cd "$(dirname "$0")/../.."
. scripts/dev/_lib.sh

d=$(daemon_dir)
pf=$(vite_pid_file)

# Reap any already-tracked vite for this worktree before launching a new one.
kill_vite

# Launch vite in the foreground but track its PID so dev-stop can reap it.
bun run --filter shell2 dev &
vite_pid=$!
echo "$vite_pid" > "$pf"

cleanup() {
    kill "$vite_pid" 2>/dev/null || true
    rm -f "$pf"
}
trap cleanup EXIT INT TERM

daemon_port=$(cat "$d/port" 2>/dev/null || echo "9315")
vite_port=$(( 3200 + daemon_port - 9315 ))
echo "shell2 vite: pid $vite_pid, port $vite_port, daemon_dir $d"

wait "$vite_pid" || true
