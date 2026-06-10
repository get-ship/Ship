#!/usr/bin/env bash
# Slice 6 e2e — config-driven space scaffold, end-to-end, curl-only.
#
# Boots the ship-free deck-demo server and verifies:
#   1. GET /space/demo-1 → 200 HTML
#   2. Body contains window.DECK_APP_ID="demo-1" (global injected from config)
#   3. Body contains /space/_assets/ (asset hrefs re-rooted)
#
# No browser. No ~/.ship. curl-only.
# Usage: examples/deck-demo/e2e/space.sh   (needs `cargo build -p deck-demo`)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
BIN="$ROOT/target/debug/deck-demo"
ADDR="127.0.0.1:7896"
BASE="http://$ADDR"

SRV_PID=""
cleanup() { [ -n "$SRV_PID" ] && kill "$SRV_PID" 2>/dev/null || true; }
trap cleanup EXIT
fail() { echo "E2E FAIL: $*" >&2; exit 1; }

[ -x "$BIN" ] || fail "binary missing — run: cargo build -p deck-demo"
pkill -x deck-demo 2>/dev/null || true

# 1. Boot the server.
DECK_DEMO_ADDR="$ADDR" "$BIN" >/dev/null 2>&1 &
SRV_PID=$!
ready=""
for _ in $(seq 1 50); do
  if curl -sf "$BASE/_deck/runtime.js" >/dev/null 2>&1; then ready=1; break; fi
  sleep 0.2
done
[ -n "$ready" ] || fail "server did not come up"

# 2. Space scaffold serves with the route id substituted.
BODY=$(curl -sf "$BASE/space/demo-1") \
  || fail "GET /space/demo-1 returned non-2xx"
echo "ok: /space/demo-1 → 200 HTML"

# 3. DECK_APP_ID must be injected.
echo "$BODY" | grep -q 'window\.DECK_APP_ID="demo-1"' \
  || fail "body does not contain window.DECK_APP_ID=\"demo-1\""
echo "ok: window.DECK_APP_ID=\"demo-1\" injected"

# 4. Asset hrefs re-rooted under /space/_assets/.
echo "$BODY" | grep -q '/space/_assets/' \
  || fail "body does not contain re-rooted /space/_assets/ hrefs"
echo "ok: asset hrefs re-rooted under /space/_assets/"

echo "E2E PASS: space scaffold → DECK_APP_ID injection + asset re-root verified (ship-free)"
