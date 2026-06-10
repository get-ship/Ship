#!/usr/bin/env bash
# Full local smoke: build everything, restart the daemon, curl every
# baselined view, run the Playwright visual suite. Exits 0 only if every
# step passes — the "would this pass CI?" gate before opening a PR.
set -euo pipefail
cd "$(dirname "$0")/../.."
. scripts/dev/_lib.sh

fail() { printf '\033[31m✗ %s\033[0m\n' "$*"; exit 1; }
pass() { printf '\033[32m✓ %s\033[0m\n' "$*"; }
step() { printf '\n\033[1m== %s ==\033[0m\n' "$*"; }

step "cargo build"
cargo build || fail "cargo build"
pass "cargo build"

# Specta-driven type bindings: ship-set/cloud-types/sui-runtime each
# import a sibling `generated.ts` that is .gitignored and produced by
# the xtask. Without it the bun build fails to resolve.
step "gen-types"
cargo run -p xtask --quiet -- gen-types || fail "gen-types"
pass "gen-types"

step "bun install"
bun install --frozen-lockfile || fail "bun install"
pass "bun install"

step "daemon restart"
just dev-restart || fail "daemon restart"
d=$(daemon_dir)
deadline=$(( $(date +%s) + 15 ))
while [ "$(date +%s)" -lt "$deadline" ]; do
    [ -f "$d/port" ] && [ -s "$d/port" ] && break
    sleep 0.2
done
[ -s "$d/port" ] || fail "daemon did not write $d/port within 15s"
port=$(cat "$d/port")
pass "daemon on port $port"

step "view smoke (HTTP 200 per views.json)"
bun scripts/dev/smoke-views.mjs --port "$port" || fail "view smoke"
pass "view smoke"

step "playwright visual"
bun run test:e2e:visual || fail "visual"
pass "visual"

printf '\n\033[32m✓ smoke-deploy passed\033[0m\n'
