#!/usr/bin/env bash
# Proves channels deliver to YOUR active Claude Code thread.
#
# Run this from a separate terminal AFTER you've restarted your Claude
# Code session with `--dangerously-load-development-channels server:ship`.
#
# Pushes a uniquely-marked channel notification to the actor_id that
# represents your thread, then prints exactly what was delivered. If
# the marker shows up in your Claude TUI / next reply context, channels
# work end-to-end. If it doesn't, this script will tell you why.

set -euo pipefail

SHIP_HOME_DIR="${SHIP_HOME:-$HOME/.ship}"
TOKEN=$(cat "$SHIP_HOME_DIR/machine/daemon/token")
BASE=${SHIPD_URL:-http://localhost:9315}
MARKER="PROOF-$(date +%s)-$RANDOM"

api() {
  curl -s -H "Authorization: Bearer $TOKEN" "$@"
}

echo "=== Step 1: which actors are alive on the mesh? ==="
LIVE_JSON=$(api "$BASE/api/actors?limit=20")
echo "$LIVE_JSON" | python3 -c "
import json, sys
d = json.load(sys.stdin)
live = [a for a in d['data']['actors'] if a['live']]
if not live:
    print('  NONE live. Your Claude Code session has not registered with this daemon.')
    print('  Did you restart Claude Code AFTER the daemon was restarted?')
    print('  If so, check that ship-bridge is connecting:')
    print('    pgrep -af \"ship mcp serve\"')
    sys.exit(2)
for a in live:
    print(f'  {a[\"id\"]:48}  agent={a[\"agent_id\"] or \"-\"}')
" || exit $?

ACTOR=$(echo "$LIVE_JSON" | python3 -c "
import json, sys
d = json.load(sys.stdin)
for a in d['data']['actors']:
    if a['live'] and a['kind'] == 'agent':
        print(a['id']); break
")

if [ -z "$ACTOR" ]; then
  echo "  No live agent actor found. Restart your Claude Code session."
  exit 1
fi

echo ""
echo "=== Step 2: push channel notification to $ACTOR ==="
echo "    marker: $MARKER"

PAYLOAD=$(printf '{"text":"PROOF channel delivery test. Echo this marker exactly: %s"}' "$MARKER")
RESP=$(api -H "Content-Type: application/json" \
  -X POST "$BASE/api/actors/$ACTOR/input" \
  -w "HTTP=%{http_code}" \
  -d "$PAYLOAD")
echo "    response: $RESP"

echo ""
echo "=== Step 3: did the kernel route + relay push? ==="
sleep 2
RECENT_LOG=$(grep -aE "$ACTOR|claude channel push: sent" "$SHIP_HOME_DIR/machine/logs/ship.log" 2>/dev/null | tail -10)
echo "$RECENT_LOG" | sed 's/^/    /'

if echo "$RECENT_LOG" | grep -q "claude channel push: sent event_type=studio.chat_turn"; then
  echo ""
  echo "  ✓ Channel push: SENT to your Claude over MCP."
  echo ""
  echo "  Now check your Claude Code TUI. The channel notification will"
  echo "  appear in your context as <channel source=\"ship\" ...>$MARKER..."
  echo "  In your next message to me here, you should see whether the"
  echo "  marker arrived in the context Claude sees."
else
  echo ""
  echo "  ✗ No 'claude channel push: sent' for studio.chat_turn."
  echo "    Either the kernel didn't route or the relay rejected."
  echo "    See the full log: tail -f $SHIP_HOME_DIR/machine/logs/ship.log"
fi
