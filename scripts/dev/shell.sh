#!/usr/bin/env bash
# The whole desktop loop in one command (`just shell`): rebuild the hub
# scaffold, ensure the dev daemon, build + launch the native window against it.
#
# Why the scaffold rebuild is here and not optional: `.deck/hub-scaffold` is a
# pre-built static app the daemon serves at `/`. Unlike view bundles (cold-miss,
# never stale) it does NOT rebuild with `just dev`/`dev-build-all`, so a stale
# scaffold — dead per-component <script> tags from an older index.html — blanks
# the shell with 404s and no error. Rebuilding it every launch removes that trap.
set -euo pipefail
cd "$(dirname "$0")/../.."

echo "[1/3] rebuild hub scaffold (the daemon-served / )"
cargo run -q -p deck-cli -- build \
    --shell-root apps/shell --out .deck/hub-scaffold --bundle-framework

echo "[2/3] ensure dev daemon (idempotent — reuses a live one)"
scripts/dev/start.sh

echo "[3/3] build + launch native shell"
cargo build -q -p ship-shell
exec ./target/debug/ship-shell "$@"
