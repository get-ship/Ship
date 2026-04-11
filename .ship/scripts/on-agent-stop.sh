#!/bin/bash
# Called by provider hook on agent exit. Emits job.completed if this
# session belongs to a dispatched job.
if [ -n "$SHIP_MESH_ID" ] && [ -f "$HOME/.ship/daemon.port" ]; then
  PORT=$(cat "$HOME/.ship/daemon.port")
  TOKEN=$(cat "$HOME/.ship/daemon.token" 2>/dev/null)
  curl -s -X POST "http://127.0.0.1:$PORT/api/runtime/jobs/end" \
    -H 'Content-Type: application/json' \
    -H "Authorization: Bearer $TOKEN" \
    -d "{\"slug\": \"$SHIP_MESH_ID\"}" > /dev/null 2>&1
fi
