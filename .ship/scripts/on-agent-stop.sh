#!/bin/bash
# Called by provider hook on agent exit. Emits job.completed if this
# session belongs to a dispatched job.
DAEMON_DIR="${SHIP_HOME:-$HOME/.ship}/machine/daemon"
if [ -n "$SHIP_MESH_ID" ] && [ -f "$DAEMON_DIR/port" ]; then
  PORT=$(cat "$DAEMON_DIR/port")
  TOKEN=$(cat "$DAEMON_DIR/token" 2>/dev/null)
  curl -s -X POST "http://127.0.0.1:$PORT/api/v1/jobs/end" \
    -H 'Content-Type: application/json' \
    -H "Authorization: Bearer $TOKEN" \
    -d "{\"slug\": \"$SHIP_MESH_ID\"}" > /dev/null 2>&1
fi
