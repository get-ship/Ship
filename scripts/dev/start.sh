#!/usr/bin/env bash
# Start the dev daemon for this worktree.
#
# Builds shipd + ship, stashes the binaries in the daemon dir, and launches
# the stashed shipd in dev mode. The stash is the running binary — rebuilds
# in this worktree won't perturb the live daemon until `just dev-restart`.
set -euo pipefail

cd "$(dirname "$0")/../.."
. scripts/dev/_lib.sh

cargo build -p shipd -p ship-cli

stash_binary shipd
stash_binary ship

d=$(daemon_dir)

# If a daemon is already alive in this dir, don't double-spawn.
if [ -f "$d/pid" ] && kill -0 "$(cat "$d/pid")" 2>/dev/null; then
    port=$(cat "$d/port" 2>/dev/null || echo "?")
    echo "shipd already running for this worktree (pid $(cat "$d/pid"), port $port)"
    echo "  daemon_dir: $d"
    echo "  use 'just dev-restart' to cycle"
    exit 0
fi

# Launch the stashed shipd. No --port: it auto-derives via path_hash.
# Pass --workspace explicitly so the daemon scopes itself to *this*
# worktree even if its own resolver disagreed: belt-and-suspenders for
# the per-worktree daemon dir (machine/daemons/<hash>/).
# Set SHIP_APPS_BASE_DIR to the ship workspace so the AppSource resolver
# finds apps under `<ship>/node_modules/<package>` (j-12: hub/spaces live
# in ship/apps; root package.json self-declares them as workspace deps).
# The env var is the typed accessor; --apps-base-dir is the same value.
log="${SHIP_HOME:-$HOME/.ship}/machine/logs/shipd.log"
mkdir -p "$(dirname "$log")"
workspace=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
apps_base_dir="${SHIP_APPS_BASE_DIR:-$workspace}"
SHIP_APPS_BASE_DIR="$apps_base_dir" "$d/.bin/shipd" --dev --workspace "$workspace" --apps-base-dir "$apps_base_dir" >>"$log" 2>&1 &
spawned=$!
disown $spawned

# Wait up to 10s for the port file to appear (signals HTTP bind succeeded).
deadline=$(( $(date +%s) + 10 ))
while [ "$(date +%s)" -lt "$deadline" ]; do
    if [ -f "$d/port" ] && [ -s "$d/port" ]; then
        port=$(cat "$d/port")
        echo "shipd dev: pid $spawned, port $port, daemon_dir $d"
        echo "  log: $log"
        exit 0
    fi
    sleep 0.1
done

echo "shipd did not write $d/port within 10s — check $log" >&2
exit 1
