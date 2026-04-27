#!/usr/bin/env bash
# Stop, rebuild, re-stash, restart.
set -euo pipefail
cd "$(dirname "$0")/../.."
. scripts/dev/_lib.sh

# Best-effort stop. Ignore failures (daemon may not be running).
"$(ship_bin)" daemon stop || true

# Wait for the pid file to drop so we don't race the next start.
d=$(daemon_dir)
deadline=$(( $(date +%s) + 5 ))
while [ -f "$d/pid" ] && [ "$(date +%s)" -lt "$deadline" ]; do sleep 0.1; done

exec scripts/dev/start.sh
