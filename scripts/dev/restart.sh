#!/usr/bin/env bash
# Stop, rebuild, re-stash, restart.
set -euo pipefail
cd "$(dirname "$0")/../.."
. scripts/dev/_lib.sh

# Stop the running dev daemon. start.sh launches shipd directly, so we stop
# it by its `pid` handshake file — `ship daemon stop` only targets a
# supervisor process this dev loop never spawns, and would exit 0 without
# stopping anything. kill_daemon waits for shipd's graceful shutdown and
# clears the liveness files, so start.sh below sees a clean slate.
kill_daemon

exec scripts/dev/start.sh
