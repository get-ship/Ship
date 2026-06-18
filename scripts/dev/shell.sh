#!/usr/bin/env bash
# The whole desktop loop in one command (`just shell`): rebuild the hub +
# space apps, ensure the dev daemon, build + launch the native window against
# it. The bundler reads `apps.jsonc` for the per-app list; each app is built
# to its own `.deck/` (apps/hub/.deck + apps/spaces/.deck) and shipd serves
# them at `/` and `/space/{id}` respectively.
#
# Why the rebuild is here and not optional: the per-app `.deck/` is a
# pre-built static app the daemon serves at `/`. Unlike view bundles
# (cold-miss, never stale) the per-app build does NOT rebuild with
# `just dev`/`dev-build-all`, so a stale build — dead per-component
# <script> tags from an older index.html — blanks the shell with 404s and
# no error. Rebuilding every launch removes that trap.
set -euo pipefail
cd "$(dirname "$0")/../.."
echo "[1/3] rebuild hub + space apps (the daemon-served / and /space/<id>)"
cargo run -q -p deck-cli -- build \
    --project apps/hub --bundle-framework
cargo run -q -p deck-cli -- build \
    --project apps/spaces
echo "[2/3] ensure dev daemon (idempotent — reuses a live one)"
scripts/dev/start.sh

echo "[3/3] build + launch native shell"
cargo build -q -p ship-host
exec ./target/debug/ship-host "$@"
