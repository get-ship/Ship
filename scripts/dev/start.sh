#!/usr/bin/env bash
# Start the dev daemon for this worktree.
#
# Produces SUI bundles (esbuild + lucide registry), then builds shipd +
# ship, stashes the binaries in the daemon dir, and launches the stashed
# shipd in dev mode. The bundle step runs first because the framework
# crate's build.rs validates every `include_str!`'d asset is non-empty —
# baking in empty bytes silently shipped a daemon with empty island
# bundles in the past, hence the validate-then-fail-loud pattern.
#
# We don't run the bundler from build.rs because the bundler invokes
# `cargo run -p sui-compiler`, and a nested cargo would deadlock waiting
# on the parent's build lock.
#
# The stash is the running binary — rebuilds in this worktree won't
# perturb the live daemon until `just dev-restart`.
set -euo pipefail

cd "$(dirname "$0")/../.."
. scripts/dev/_lib.sh

# Generate the lucide icon registry first (fast — no nested cargo) so
# the bundler can pick up its outputs if anything depends on them.
node packages/ship-set/scripts/generate-lucide-registry.mjs

# Bundle SUI assets. The script's own incremental cache makes this a
# no-op when nothing changed.
node scripts/build-bundles.mjs

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
log="${SHIP_HOME:-$HOME/.ship}/machine/logs/shipd.log"
mkdir -p "$(dirname "$log")"
"$d/.bin/shipd" --dev >>"$log" 2>&1 &
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
